//! The operation extension point: one unit of load against one target.
//!
//! Add an operation by implementing [`Operation`], adding an [`OpKind`]
//! variant and mapping it in [`OpKind::build`]. Errors are classified into a
//! short string so the report can group them.

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::Deserialize;
use serde_json::{Value, json};
use tbd_proto::protocol::v1::{PingRequest, protocol_service_client::ProtocolServiceClient};
use tokio::sync::Mutex;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};
use tonic::transport::Channel;

/// Where load goes: one protocol instance.
#[derive(Debug, Clone)]
pub struct Target {
    /// Instance name from the topology.
    pub name: String,
    /// `http://host:port`.
    pub http_url: String,
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

type Ws = WebSocketStream<MaybeTlsStream<tokio::net::TcpStream>>;

/// Connections shared by every operation. One per load run.
pub struct Clients {
    http: reqwest::Client,
    ws: Mutex<HashMap<String, Vec<Ws>>>,
    grpc: Mutex<HashMap<String, Channel>>,
    timeout: Duration,
}

impl Clients {
    /// Build with a per-request timeout.
    pub fn new(timeout: Duration) -> Self {
        Self {
            http: reqwest::Client::builder()
                .timeout(timeout)
                .build()
                .unwrap_or_default(),
            ws: Mutex::new(HashMap::new()),
            grpc: Mutex::new(HashMap::new()),
            timeout,
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
        let (ws, _) = tokio::time::timeout(
            self.timeout,
            tokio_tungstenite::connect_async(target.ws_url()),
        )
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

    async fn grpc(&self, target: &Target) -> Result<Channel, OpError> {
        let mut map = self.grpc.lock().await;
        if let Some(ch) = map.get(&target.name) {
            return Ok(ch.clone());
        }
        let ch = tonic::transport::Endpoint::from_shared(target.http_url.clone())
            .map_err(|_| OpError::Transport)?
            .timeout(self.timeout)
            .connect_lazy();
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
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
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
}

impl OpKind {
    /// Instantiate.
    pub fn build(self) -> Arc<dyn Operation> {
        match self {
            Self::RestEvaluate => Arc::new(RestEvaluate),
            Self::GraphqlEvaluate => Arc::new(GraphqlEvaluate),
            Self::WsEcho => Arc::new(WsEcho),
            Self::GrpcPing => Arc::new(GrpcPing),
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

/// Open a WebSocket with `TCP_NODELAY` set. `connect_async` leaves Nagle on,
/// which stalls small frames by ~40 ms against a server with delayed ACKs.
pub async fn connect_ws(url: &str) -> anyhow::Result<Ws> {
    let parsed = url::Url::parse(url)?;
    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("no host in {url}"))?;
    let port = parsed
        .port_or_known_default()
        .ok_or_else(|| anyhow::anyhow!("no port in {url}"))?;
    let stream = tokio::net::TcpStream::connect((host, port)).await?;
    stream.set_nodelay(true)?;
    let (ws, _) = tokio_tungstenite::client_async(url, MaybeTlsStream::Plain(stream)).await?;
    Ok(ws)
}

fn uuid_like() -> String {
    format!(
        "{:016x}{:016x}",
        rand::random::<u64>(),
        rand::random::<u64>()
    )
}
