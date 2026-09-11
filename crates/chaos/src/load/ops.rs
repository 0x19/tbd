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

fn uuid_like() -> String {
    format!(
        "{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}
