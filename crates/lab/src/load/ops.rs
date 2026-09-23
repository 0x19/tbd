//! The operation extension point: one unit of load against one target.
//!
//! Add an operation by implementing [`Operation`], adding an [`OpKind`]
//! variant and mapping it in [`OpKind::build`]. Errors are classified into a
//! short string so the report can group them.

use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
    time::{Duration, Instant},
};

use super::Meter;
use crate::tls::{Grpc, Trust, Ws};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tbd_proto::finance::v1::{
    CreateIssuerRequest, ImportOpeningBalancesRequest, ListPartiesRequest,
    ListTransactionsRequest as FinanceListRequest, OpeningBalance,
    PingRequest as FinancePingRequest, TrialBalanceRequest,
    finance_service_client::FinanceServiceClient,
};
use tbd_proto::llm::v1::{
    GenerateRequest, Message as LlmMessage, Tier as LlmTier, llm_service_client::LlmServiceClient,
};
use tbd_proto::protocol::v1::{PingRequest, protocol_service_client::ProtocolServiceClient};
use tokio::sync::Mutex;
use tokio_tungstenite::tungstenite::Message;

/// Where load goes: one instance of a kind that takes load.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Target {
    /// Instance name from the topology.
    pub name: String,
    /// `http://host:port`.
    pub http_url: String,
    /// The kind of the instance; operations run against their own kind.
    #[serde(default = "default_target_kind")]
    pub kind: String,
}

fn default_target_kind() -> String {
    "protocol".to_owned()
}

impl Target {
    /// `ws://host:port/ws`.
    pub fn ws_url(&self) -> String {
        self.http_url.replacen("http", "ws", 1) + "/ws"
    }
}

/// Why a request failed, as a stable class for grouping.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum OpError {
    /// Could not connect or the connection broke.
    #[error("transport")]
    Transport,
    /// HTTP status outside 2xx.
    #[error("http {0}")]
    Http(u16),
    /// gRPC status.
    #[error("grpc {0}")]
    Grpc(String),
    /// Well-formed reply that violates the contract.
    #[error("contract: {0}")]
    Contract(String),
}

impl OpError {
    /// Group key for reports.
    pub fn class(&self) -> String {
        self.to_string()
    }
}

/// Connections shared by every operation, and the meter an operation may
/// report counters and timings into. One per load run.
pub struct Clients {
    http: reqwest::Client,
    ws: Mutex<HashMap<String, Vec<Ws>>>,
    grpc: Mutex<HashMap<String, Grpc>>,
    timeout: Duration,
    trust: Trust,
    /// What an operation meters beside its latency (`tokens`, `ttft`, ...);
    /// records nothing when no run is behind it.
    pub meter: Meter,
}

impl Clients {
    /// Build with a per-request timeout.
    pub fn new(timeout: Duration) -> Self {
        Self::with_trust(timeout, Trust::default())
    }

    /// Build with a per-request timeout and explicit TLS trust for `https`/`wss`
    /// targets. Pass a [`Trust::snapshot`] so the clients carry the bearer token.
    pub fn with_trust(timeout: Duration, trust: Trust) -> Self {
        Self {
            http: trust.http(Some(timeout)),
            ws: Mutex::new(HashMap::new()),
            grpc: Mutex::new(HashMap::new()),
            timeout,
            trust,
            meter: Meter::none(),
        }
    }

    /// Record what operations meter into the run's metrics.
    #[must_use]
    pub fn with_meter(mut self, meter: Meter) -> Self {
        self.meter = meter;
        self
    }

    async fn take_ws(&self, target: &Target) -> Result<Ws, OpError> {
        if let Some(ws) = self
            .ws
            .lock()
            .await
            .get_mut(&target.name)
            .and_then(Vec::pop)
        {
            return Ok(ws);
        }
        let ws = tokio::time::timeout(self.timeout, self.trust.connect_ws(&target.ws_url()))
            .await
            .map_err(|_| OpError::Transport)?
            .map_err(|_| OpError::Transport)?;
        Ok(ws)
    }

    async fn give_ws(&self, target: &Target, ws: Ws) {
        self.ws
            .lock()
            .await
            .entry(target.name.clone())
            .or_default()
            .push(ws);
    }

    /// A gRPC channel to the target, one per target for the run.
    pub async fn grpc(&self, target: &Target) -> Result<Grpc, OpError> {
        let mut map = self.grpc.lock().await;
        if let Some(ch) = map.get(&target.name) {
            return Ok(ch.clone());
        }
        let ch = self
            .trust
            .grpc(&target.http_url, Some(self.timeout))
            .map_err(|_| OpError::Transport)?;
        map.insert(target.name.clone(), ch.clone());
        Ok(ch)
    }
}

/// One unit of load.
#[async_trait]
pub trait Operation: Send + Sync {
    /// Short name used in reports.
    fn name(&self) -> &'static str;
    /// Perform one request against `target`.
    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError>;
}

/// Operations known to this project. The TOML value is the `snake_case` name.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpKind {
    /// `POST /v1/evaluate`.
    RestEvaluate,
    /// GraphQL `evaluate` query.
    GraphqlEvaluate,
    /// One text frame over `/ws`, wait for its echo.
    WsEcho,
    /// `Protocol/Ping` over gRPC on the protocol port.
    GrpcPing,
    /// `LedgerService/Append` on a pooled subject.
    LedgerAppend,
    /// `LedgerService/Current` on a pooled subject.
    LedgerCurrent,
    /// `LedgerService/History` with a random cut and page size.
    LedgerHistory,
    /// `LedgerService/Retract` of a pooled path; nothing to retract is fine.
    LedgerRetract,
    /// Append, current, retract, history cut: the retraction rule per request.
    LedgerLifecycle,
    /// Append, erase, restore, erase, wait, gone: the erasure rule per request.
    LedgerEraseCycle,
    /// Hostile ledger requests that must draw a clean refusal, never an internal error.
    LedgerFuzz,
    /// `FinanceService/Ping` over gRPC on the finance port.
    FinancePing,
    /// Two callers against one finance instance: the owner, who may read a
    /// personal party and a company, and a reader granted the company only.
    /// Fails the run if the reader is ever handed a personal row.
    FinanceAccess,
    /// Money over the wire: the amounts that come back must be the exact minor
    /// units that went in, with the sign the direction implies.
    FinanceMoney,
    /// `TrialBalance` of the owner's company for the current year: it balances
    /// and its rows add up to its totals. Needs a database on the instance.
    FinanceTrialBalance,
    /// `ImportOpeningBalances` of a random balanced set, then the trial balance
    /// equals it; every fourth set is off by one cent and must be refused.
    /// Needs a database on the instance.
    FinanceImportOpening,
    /// `LlmService/Generate` of a short prompt on the default tier, streamed
    /// to its done chunk. Meters `prompt_tokens`, `completion_tokens`,
    /// `generations`, `answered` and the time to the first chunk as `ttft`.
    /// Runs against llms; the generator's timeout covers the whole stream.
    LlmGenerate,
    /// The same on the deep tier, the large model that runs from memory, so
    /// both tiers are measured with one instrument and told apart by name.
    LlmGenerateDeep,
}

/// What every operation of a run shares: the ledger subject pool and the seed.
#[derive(Debug, Clone)]
pub struct OpContext {
    /// Subjects the ledger operations spread over.
    pub subjects: u32,
    /// Seed for generated data.
    pub seed: u64,
}

impl OpKind {
    /// The kind of instance this operation targets.
    #[must_use]
    pub fn target_kind(self) -> &'static str {
        match self {
            Self::RestEvaluate | Self::GraphqlEvaluate | Self::WsEcho | Self::GrpcPing => {
                "protocol"
            }
            Self::LedgerAppend
            | Self::LedgerCurrent
            | Self::LedgerHistory
            | Self::LedgerRetract
            | Self::LedgerLifecycle
            | Self::LedgerEraseCycle
            | Self::LedgerFuzz => "ledger",
            Self::FinancePing
            | Self::FinanceAccess
            | Self::FinanceMoney
            | Self::FinanceTrialBalance
            | Self::FinanceImportOpening => "finance",
            Self::LlmGenerate | Self::LlmGenerateDeep => "llm",
        }
    }

    /// Instantiate for one run; the ledger operations share one subject pool.
    pub fn build(self, ctx: &OpContext, pool: &Arc<super::ledger_ops::Pool>) -> Arc<dyn Operation> {
        let _ = ctx;
        match self {
            Self::RestEvaluate => Arc::new(RestEvaluate),
            Self::GraphqlEvaluate => Arc::new(GraphqlEvaluate),
            Self::WsEcho => Arc::new(WsEcho),
            Self::GrpcPing => Arc::new(GrpcPing),
            Self::FinancePing => Arc::new(FinancePing),
            Self::FinanceAccess => Arc::new(FinanceAccess),
            Self::FinanceMoney => Arc::new(FinanceMoney),
            Self::FinanceTrialBalance => Arc::new(FinanceTrialBalance::default()),
            Self::FinanceImportOpening => Arc::new(FinanceImportOpening::default()),
            Self::LlmGenerate => Arc::new(LlmGenerate::default()),
            Self::LlmGenerateDeep => Arc::new(LlmGenerate::deep()),
            Self::LedgerAppend => Arc::new(super::ledger_ops::Append(Arc::clone(pool))),
            Self::LedgerCurrent => Arc::new(super::ledger_ops::Current(Arc::clone(pool))),
            Self::LedgerHistory => Arc::new(super::ledger_ops::History(Arc::clone(pool))),
            Self::LedgerRetract => Arc::new(super::ledger_ops::Retract(Arc::clone(pool))),
            Self::LedgerLifecycle => Arc::new(super::ledger_ops::Lifecycle),
            Self::LedgerEraseCycle => Arc::new(super::ledger_ops::EraseCycle {
                settle: Duration::from_millis(1500),
            }),
            Self::LedgerFuzz => Arc::new(super::ledger_ops::Fuzz(Arc::clone(pool))),
        }
    }
}

struct RestEvaluate;

#[async_trait]
impl Operation for RestEvaluate {
    fn name(&self) -> &'static str {
        "rest_evaluate"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let resp = clients
            .http
            .post(format!("{}/v1/evaluate", target.http_url))
            .json(&json!({ "subject_id": "load", "payload": "x" }))
            .send()
            .await
            .map_err(|_| OpError::Transport)?;
        let status = resp.status();
        if !status.is_success() {
            return Err(OpError::Http(status.as_u16()));
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|_| OpError::Contract("not json".into()))?;
        if v["stub"].is_boolean() {
            Ok(())
        } else {
            Err(OpError::Contract("missing stub flag".into()))
        }
    }
}

struct GraphqlEvaluate;

#[async_trait]
impl Operation for GraphqlEvaluate {
    fn name(&self) -> &'static str {
        "graphql_evaluate"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let resp = clients
            .http
            .post(format!("{}/graphql", target.http_url))
            .json(&json!({ "query": "{ evaluate(subjectId:\"load\"){ stub } }" }))
            .send()
            .await
            .map_err(|_| OpError::Transport)?;
        let status = resp.status();
        if !status.is_success() {
            return Err(OpError::Http(status.as_u16()));
        }
        let v: Value = resp
            .json()
            .await
            .map_err(|_| OpError::Contract("not json".into()))?;
        if !v["errors"].is_null() {
            return Err(OpError::Contract(format!(
                "graphql errors: {}",
                v["errors"]
            )));
        }
        Ok(())
    }
}

struct WsEcho;

#[async_trait]
impl Operation for WsEcho {
    fn name(&self) -> &'static str {
        "ws_echo"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut ws = clients.take_ws(target).await?;
        let token = uuid_like();
        if ws.send(Message::Text(token.clone().into())).await.is_err() {
            return Err(OpError::Transport);
        }
        let result = tokio::time::timeout(clients.timeout, async {
            loop {
                match ws.next().await {
                    Some(Ok(Message::Text(text))) => {
                        let v: Value = serde_json::from_str(&text).unwrap_or(Value::Null);
                        if v["type"] == "data" && v["data"] == token.as_str() {
                            return Ok(());
                        }
                        if v["type"] == "error" {
                            return Err(OpError::Contract(v["message"].to_string()));
                        }
                    }
                    Some(Ok(Message::Close(_)) | Err(_)) | None => return Err(OpError::Transport),
                    Some(Ok(_)) => {}
                }
            }
        })
        .await
        .unwrap_or(Err(OpError::Transport));
        if result.is_ok() {
            clients.give_ws(target, ws).await;
        }
        result
    }
}

struct GrpcPing;

#[async_trait]
impl Operation for GrpcPing {
    fn name(&self) -> &'static str {
        "grpc_ping"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut client = ProtocolServiceClient::new(clients.grpc(target).await?);
        let resp = client
            .ping(PingRequest {
                message: "load".into(),
            })
            .await
            .map_err(|s| match s.code() {
                tonic::Code::Unavailable | tonic::Code::Unknown => OpError::Transport,
                code => OpError::Grpc(format!("{code:?}")),
            })?
            .into_inner();
        if resp.message == "load" {
            Ok(())
        } else {
            Err(OpError::Contract("wrong echo".into()))
        }
    }
}

/// `FinanceService/Ping`.
///
/// The service is a scaffold: `Ping` is all it serves, and it answers
/// `stub: true`. That is deliberate — this operation exists so the finance
/// instance is drivable under load and fault injection from the first commit,
/// rather than arriving untested once it has real RPCs. It asserts the echo
/// **and** that the stub flag is still honest.
struct FinancePing;

#[async_trait]
impl Operation for FinancePing {
    fn name(&self) -> &'static str {
        "finance_ping"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut client = FinanceServiceClient::new(clients.grpc(target).await?);
        let resp = client
            .ping(FinancePingRequest {
                message: "load".into(),
            })
            .await
            .map_err(|s| match s.code() {
                tonic::Code::Unavailable | tonic::Code::Unknown => OpError::Transport,
                code => OpError::Grpc(format!("{code:?}")),
            })?
            .into_inner();
        if resp.message != "load" {
            return Err(OpError::Contract("wrong echo".into()));
        }
        if !resp.stub {
            return Err(OpError::Contract(
                "finance reported stub: false while it is still a scaffold".into(),
            ));
        }
        Ok(())
    }
}

/// The party ids the seeded world uses. Fixed so an assertion can name them.
const PERSONAL_PARTY: &str = "11111111-1111-4111-8111-111111111111";
const COMPANY_PARTY: &str = "22222222-2222-4222-8222-222222222222";

/// Envoy's verified-claims header, as the service reads it.
fn caller(subject: &str) -> tonic::metadata::MetadataValue<tonic::metadata::Ascii> {
    use base64::Engine as _;
    let claims = format!(r#"{{"sub":"{subject}","scp":["tbd.finance"]}}"#);
    let encoded = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims);
    encoded
        .parse()
        .unwrap_or_else(|_| "".parse().unwrap_or_else(|_| unreachable!()))
}

/// Access control, driven continuously.
///
/// Each request is the pair that matters: the owner reads everything they were
/// granted, and the reader reads the company and *only* the company. A leak is
/// silent -- nothing errors, a query just returns one row too many -- so this
/// asserts on absence, and a violation is an `OpError::Contract`, which no
/// scenario's `max_error_rate` forgives.
struct FinanceAccess;

#[async_trait]
impl Operation for FinanceAccess {
    fn name(&self) -> &'static str {
        "finance_access"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let channel = clients.grpc(target).await?;
        let mut client = FinanceServiceClient::new(channel);

        let map = |s: tonic::Status| match s.code() {
            tonic::Code::Unavailable | tonic::Code::Unknown => OpError::Transport,
            code => OpError::Grpc(format!("{code:?}")),
        };

        // The owner: both parties, nothing missing.
        let mut request = tonic::Request::new(FinanceListRequest::default());
        request
            .metadata_mut()
            .insert("x-jwt-payload", caller("chaos-owner"));
        let owner = client
            .list_transactions(request)
            .await
            .map_err(map)?
            .into_inner();
        if owner.transactions.len() != 2 {
            return Err(OpError::Contract(format!(
                "owner saw {} transactions, expected 2",
                owner.transactions.len()
            )));
        }

        // The reader: the company, and never the personal party.
        let mut request = tonic::Request::new(FinanceListRequest::default());
        request
            .metadata_mut()
            .insert("x-jwt-payload", caller("chaos-reader"));
        let reader = client
            .list_transactions(request)
            .await
            .map_err(map)?
            .into_inner();
        if reader
            .transactions
            .iter()
            .any(|t| t.party_id == PERSONAL_PARTY)
        {
            return Err(OpError::Contract(
                "LEAK: the reader was handed a personal transaction".into(),
            ));
        }
        if reader.transactions.len() != 1 || reader.transactions[0].party_id != COMPANY_PARTY {
            return Err(OpError::Contract(format!(
                "reader saw {} transactions, expected exactly the company's",
                reader.transactions.len()
            )));
        }

        // And naming the forbidden party directly must not reach it either.
        let mut request = tonic::Request::new(FinanceListRequest {
            party_ids: vec![PERSONAL_PARTY.to_owned()],
            ..Default::default()
        });
        request
            .metadata_mut()
            .insert("x-jwt-payload", caller("chaos-reader"));
        let named = client
            .list_transactions(request)
            .await
            .map_err(map)?
            .into_inner();
        if !named.transactions.is_empty() {
            return Err(OpError::Contract(
                "LEAK: naming a forbidden party directly returned rows".into(),
            ));
        }
        Ok(())
    }
}

/// Money, checked to the cent on every request.
///
/// The seeded world holds two amounts chosen to catch the failures that matter:
/// a debit of -4,250 minor units and a credit of 1,450,082 -- the second being
/// a real Tenderly settlement, 14,500.82 EUR, large enough that a float would
/// start losing it and awkward enough that a naive decimal parse would too.
///
/// What this proves that a unit test does not: the value survives the whole
/// path under load -- the store, the gRPC encoding, the JSON the transcoder
/// renders int64 as -- and keeps its sign. A wrong sign is the quietest bug in
/// accounting software: the totals still look plausible.
struct FinanceMoney;

#[async_trait]
impl Operation for FinanceMoney {
    fn name(&self) -> &'static str {
        "finance_money"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut client = FinanceServiceClient::new(clients.grpc(target).await?);
        let mut request = tonic::Request::new(FinanceListRequest::default());
        request
            .metadata_mut()
            .insert("x-jwt-payload", caller("chaos-owner"));
        let resp = client
            .list_transactions(request)
            .await
            .map_err(|s| match s.code() {
                tonic::Code::Unavailable | tonic::Code::Unknown => OpError::Transport,
                code => OpError::Grpc(format!("{code:?}")),
            })?
            .into_inner();

        let mut debit = None;
        let mut credit = None;
        for t in &resp.transactions {
            if t.amount_minor < 0 {
                debit = Some(t.amount_minor);
            } else {
                credit = Some(t.amount_minor);
            }
            if t.currency != "EUR" {
                return Err(OpError::Contract(format!(
                    "currency came back as {:?}",
                    t.currency
                )));
            }
            if t.scale != 2 {
                return Err(OpError::Contract(format!("scale came back as {}", t.scale)));
            }
        }

        match (debit, credit) {
            (Some(-4_250), Some(1_450_082)) => Ok(()),
            (d, c) => Err(OpError::Contract(format!(
                "amounts changed in flight: debit {d:?} (want -4250), credit {c:?} (want 1450082)"
            ))),
        }
    }
}

fn finance_status(s: &tonic::Status) -> OpError {
    match s.code() {
        tonic::Code::Unavailable | tonic::Code::Unknown => OpError::Transport,
        code => OpError::Grpc(format!("{code:?}")),
    }
}

/// The company the books operations run on: the first the owner owns, made
/// through `CreateIssuer` when there is none, and remembered per target.
/// The same path a person takes on `/issuer/`, so an instance with an empty
/// database is usable from the first request.
#[derive(Default)]
struct BooksCompany {
    per_target: Mutex<HashMap<String, String>>,
}

impl BooksCompany {
    async fn id(&self, clients: &Clients, target: &Target) -> Result<String, OpError> {
        let mut map = self.per_target.lock().await;
        if let Some(id) = map.get(&target.name) {
            return Ok(id.clone());
        }
        let mut client = FinanceServiceClient::new(clients.grpc(target).await?);
        let mut list = tonic::Request::new(ListPartiesRequest::default());
        list.metadata_mut()
            .insert("x-jwt-payload", caller("chaos-owner"));
        let parties = client
            .list_parties(list)
            .await
            .map_err(|s| finance_status(&s))?
            .into_inner()
            .parties;
        let found = parties
            .iter()
            .find(|p| p.kind == "org" && p.capability == "own")
            .map(|p| p.id.clone());
        let id = if let Some(id) = found {
            id
        } else {
            let mut create = tonic::Request::new(CreateIssuerRequest {
                legal_name: "Chaos d.o.o.".into(),
                oib: String::new(),
                vat_id: String::new(),
                country_code: "HR".into(),
            });
            create
                .metadata_mut()
                .insert("x-jwt-payload", caller("chaos-owner"));
            client
                .create_issuer(create)
                .await
                .map_err(|s| finance_status(&s))?
                .into_inner()
                .party
                .map(|p| p.id)
                .ok_or_else(|| OpError::Contract("CreateIssuer returned no party".into()))?
        };
        map.insert(target.name.clone(), id.clone());
        Ok(id)
    }
}

/// The trial balance, driven continuously.
///
/// The one identity a ledger cannot lose: sum(debit) = sum(credit). Every
/// request reads the owner's company for the current year and requires
/// `balanced`, and that the rows add up to the totals the service claims.
/// Needs a database on the instance; without one the call is `UNAVAILABLE`,
/// a transport failure.
#[derive(Default)]
struct FinanceTrialBalance {
    company: BooksCompany,
}

#[async_trait]
impl Operation for FinanceTrialBalance {
    fn name(&self) -> &'static str {
        "finance_trial_balance"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let party_id = self.company.id(clients, target).await?;
        let mut client = FinanceServiceClient::new(clients.grpc(target).await?);
        let mut request = tonic::Request::new(TrialBalanceRequest {
            party_id,
            fiscal_year: 0,
            through_month: 0,
        });
        request
            .metadata_mut()
            .insert("x-jwt-payload", caller("chaos-owner"));
        let tb = client
            .trial_balance(request)
            .await
            .map_err(|s| finance_status(&s))?
            .into_inner();
        check_trial_balance(&tb)
    }
}

/// The identities a trial balance must satisfy, whatever is in it.
fn check_trial_balance(tb: &tbd_proto::finance::v1::TrialBalanceResponse) -> Result<(), OpError> {
    let (mut debit, mut credit) = (0i128, 0i128);
    for r in &tb.rows {
        if r.total_debit_minor != r.opening_debit_minor + r.period_debit_minor
            || r.total_credit_minor != r.opening_credit_minor + r.period_credit_minor
            || r.balance_minor != r.total_debit_minor - r.total_credit_minor
        {
            return Err(OpError::Contract(format!(
                "account {}: its totals are not its opening plus its movement",
                r.account_code
            )));
        }
        debit += i128::from(r.total_debit_minor);
        credit += i128::from(r.total_credit_minor);
    }
    if debit != i128::from(tb.total_debit_minor) || credit != i128::from(tb.total_credit_minor) {
        return Err(OpError::Contract(format!(
            "rows add up to {debit}/{credit}, the service says {}/{}",
            tb.total_debit_minor, tb.total_credit_minor
        )));
    }
    if tb.balanced != (debit == credit) {
        return Err(OpError::Contract(format!(
            "balanced={} with debit {debit} and credit {credit}",
            tb.balanced
        )));
    }
    if !tb.balanced {
        return Err(OpError::Contract(format!(
            "the trial balance is off: debit {debit}, credit {credit}"
        )));
    }
    Ok(())
}

/// The opening import, driven continuously.
///
/// A random balanced set on three accounts of the shipped chart goes in and
/// must come back as the trial balance, to the cent; every fourth set is off
/// by one cent and must draw `INVALID_ARGUMENT`, never land and never be an
/// internal error. Imports of one year serialise on the row, so the operation
/// holds a lock around import-then-read; the rate a scenario asks for is the
/// rate of that pair. Needs a database on the instance.
#[derive(Default)]
struct FinanceImportOpening {
    company: BooksCompany,
    calls: std::sync::atomic::AtomicU64,
    one_at_a_time: Mutex<()>,
}

#[async_trait]
impl Operation for FinanceImportOpening {
    fn name(&self) -> &'static str {
        "finance_import_opening"
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let party_id = self.company.id(clients, target).await?;
        let n = self.calls.fetch_add(1, Ordering::Relaxed);
        let bank = i64::from(rand::random::<u32>() % 1_000_000) + 1;
        let fee = i64::from(rand::random::<u32>() % 100_000) + 1;
        let off = n % 4 == 3;
        let rows = vec![
            OpeningBalance {
                account_code: "1000".into(),
                debit_minor: bank,
                credit_minor: 0,
            },
            OpeningBalance {
                account_code: "4164".into(),
                debit_minor: fee,
                credit_minor: 0,
            },
            OpeningBalance {
                account_code: "2200".into(),
                debit_minor: 0,
                credit_minor: bank + fee + i64::from(off),
            },
        ];
        // A year of its own so nothing a person imported is touched.
        let year = 2001;
        let _serial = self.one_at_a_time.lock().await;
        let mut client = FinanceServiceClient::new(clients.grpc(target).await?);
        let mut import = tonic::Request::new(ImportOpeningBalancesRequest {
            party_id: party_id.clone(),
            fiscal_year: year,
            as_of: format!("{year}-01-01"),
            source: "imported".into(),
            rows,
        });
        import
            .metadata_mut()
            .insert("x-jwt-payload", caller("chaos-owner"));
        match client.import_opening_balances(import).await {
            Ok(_) if off => {
                return Err(OpError::Contract(
                    "an opening off by one cent was accepted".into(),
                ));
            }
            Ok(_) => {}
            Err(s) if off && s.code() == tonic::Code::InvalidArgument => return Ok(()),
            Err(s) if off => {
                return Err(OpError::Contract(format!(
                    "an opening off by one cent drew {:?}, not INVALID_ARGUMENT",
                    s.code()
                )));
            }
            Err(s) => return Err(finance_status(&s)),
        }
        let mut read = tonic::Request::new(TrialBalanceRequest {
            party_id,
            fiscal_year: year,
            through_month: 0,
        });
        read.metadata_mut()
            .insert("x-jwt-payload", caller("chaos-owner"));
        let tb = client
            .trial_balance(read)
            .await
            .map_err(|s| finance_status(&s))?
            .into_inner();
        check_trial_balance(&tb)?;
        let want = bank + fee;
        if tb.total_debit_minor != want || tb.rows.len() != 3 {
            return Err(OpError::Contract(format!(
                "imported {want} on both sides over 3 accounts, read back {} over {}",
                tb.total_debit_minor,
                tb.rows.len()
            )));
        }
        Ok(())
    }
}

fn uuid_like() -> String {
    format!(
        "{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}

/// The prompts `llm_generate` rotates through: short, fixed, and several, so
/// the engine's prompt cache is neither always hot nor always cold. The
/// answers are not checked; the stream's shape is.
const LLM_PROMPTS: [&str; 4] = [
    "In one sentence, what is a load balancer?",
    "Name three properties of a good retry policy.",
    "Explain idempotency to a new engineer in two sentences.",
    "What does p99 latency tell you that the mean does not?",
];

/// Longest completion asked for; the engine may stop earlier.
const LLM_MAX_TOKENS: u32 = 48;

/// `LlmService/Generate` streamed to the end. One request is one generation;
/// its latency is the whole stream, and the meter carries what a latency
/// cannot: tokens per second, the time to the first token (reasoning or
/// answer, whichever comes first) and how many generations produced an
/// answer at all, since a reasoning model can spend a short budget thinking
/// and end with nothing to show.
#[derive(Default)]
struct LlmGenerate {
    next: AtomicUsize,
    /// The tier on the wire; 0 (unspecified) is the service's default.
    tier: i32,
}

impl LlmGenerate {
    fn deep() -> Self {
        Self {
            next: AtomicUsize::new(0),
            tier: LlmTier::Deep as i32,
        }
    }
}

#[async_trait]
impl Operation for LlmGenerate {
    fn name(&self) -> &'static str {
        if self.tier == LlmTier::Deep as i32 {
            "llm_generate_deep"
        } else {
            "llm_generate"
        }
    }

    async fn run(&self, clients: &Clients, target: &Target) -> Result<(), OpError> {
        let mut client = LlmServiceClient::new(clients.grpc(target).await?);
        let i = self.next.fetch_add(1, Ordering::Relaxed) % LLM_PROMPTS.len();
        let mut request = tonic::Request::new(GenerateRequest {
            messages: vec![LlmMessage {
                role: "user".into(),
                content: LLM_PROMPTS[i].into(),
            }],
            max_tokens: Some(LLM_MAX_TOKENS),
            tier: self.tier,
            ..Default::default()
        });
        request
            .metadata_mut()
            .insert("x-jwt-payload", caller("chaos-load"));
        let started = Instant::now();
        let mut stream = client
            .generate(request)
            .await
            .map_err(|s| finance_status(&s))?
            .into_inner();

        let mut index = 0u32;
        let mut first_text: Option<Duration> = None;
        let mut answered = false;
        let mut stub: Option<bool> = None;
        loop {
            let chunk = match stream.message().await {
                Ok(Some(c)) => c,
                Ok(None) => {
                    return Err(OpError::Contract(
                        "the stream ended without a done chunk".into(),
                    ));
                }
                Err(s) => return Err(finance_status(&s)),
            };
            if chunk.index != index {
                return Err(OpError::Contract(format!(
                    "chunk index {} where {index} was due",
                    chunk.index
                )));
            }
            index += 1;
            match stub {
                None => stub = Some(chunk.stub),
                Some(s) if s != chunk.stub => {
                    return Err(OpError::Contract(
                        "the stub flag changed within one stream".into(),
                    ));
                }
                Some(_) => {}
            }
            if first_text.is_none() && !chunk.text.is_empty() {
                first_text = Some(started.elapsed());
            }
            if !chunk.reasoning && !chunk.text.is_empty() {
                answered = true;
            }
            if chunk.done {
                let Some(usage) = chunk.usage else {
                    return Err(OpError::Contract("the done chunk carries no usage".into()));
                };
                let op = self.name();
                clients
                    .meter
                    .count(op, "prompt_tokens", u64::from(usage.prompt_tokens));
                clients
                    .meter
                    .count(op, "completion_tokens", u64::from(usage.completion_tokens));
                clients.meter.count(op, "generations", 1);
                clients.meter.count(op, "answered", u64::from(answered));
                if let Some(t) = first_text {
                    clients.meter.sample(op, "ttft", t);
                }
                return Ok(());
            }
        }
    }
}
