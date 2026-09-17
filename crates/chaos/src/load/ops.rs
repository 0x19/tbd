//! The operation extension point: one unit of load against one target.
//!
//! Add an operation by implementing [`Operation`], adding an [`OpKind`]
//! variant and mapping it in [`OpKind::build`]. Errors are classified into a
//! short string so the report can group them.

use std::{collections::HashMap, sync::Arc, time::Duration};

use crate::tls::{Grpc, Trust, Ws};
use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tbd_proto::finance::v1::{
    ListTransactionsRequest as FinanceListRequest, PingRequest as FinancePingRequest,
    finance_service_client::FinanceServiceClient,
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

/// Connections shared by every operation. One per load run.
pub struct Clients {
    http: reqwest::Client,
    ws: Mutex<HashMap<String, Vec<Ws>>>,
    grpc: Mutex<HashMap<String, Grpc>>,
    timeout: Duration,
    trust: Trust,
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
        }
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
            Self::FinancePing | Self::FinanceAccess | Self::FinanceMoney => "finance",
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

fn uuid_like() -> String {
    format!(
        "{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}
