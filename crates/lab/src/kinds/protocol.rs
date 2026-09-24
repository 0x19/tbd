//! The protocol kind: an in-process `tbd-protocol` bound to an engine, named
//! either as an instance in this stack or as a bare URL.

use std::net::SocketAddr;

use async_trait::async_trait;
use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tbd_proto::protocol::v1::{PingRequest, protocol_service_client::ProtocolServiceClient};
use tbd_protocol::Config;
use tokio_tungstenite::tungstenite::Message;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use crate::kind::{Field, FieldKind, Kind, Target};
use crate::{
    check::{Check, Endpoint as Ep},
    service::{Instance, InstanceHandle, Peers, Service, TaskHandle},
};

/// The registry entry.
pub static KIND: Kind = Kind {
    name: "protocol",
    label: "Protocol",
    plural: "protocols",
    surface: "http/ws/graphql/grpc",
    target: Some(Target {
        help: "Protocol base URL",
        default_url: "http://127.0.0.1:8080",
    }),
    fields: &[
        // `engine` is what a stack file and the admin form give. `engine_url` is
        // the code-built alternative — a balancer, or an engine the stack does
        // not own — and a table carrying it may leave `engine` out; the parser
        // holds the one-of-two rule, since a field cannot say "required unless".
        Field {
            name: "engine",
            label: "Engine",
            kind: FieldKind::InstanceOf("engine"),
            required: true,
            default: None,
        },
        Field {
            name: "engine_url",
            label: "Engine URL (instead of an engine)",
            kind: FieldKind::Text,
            required: false,
            default: None,
        },
    ],
    fault: false,
    store_fault: false,
    counters: false,
    load_target: true,
    addable: true,
    parse: crate::kind::parse::<Protocol>,
    checks: &[
        Check {
            name: "http_healthz",
            surface: "http",
            doc: "`GET /healthz` is 2xx",
            run: |e| Box::pin(http_healthz(e)),
        },
        Check {
            name: "http_readyz",
            surface: "http",
            doc: "`GET /readyz` is 2xx (every required backend is SERVING)",
            run: |e| Box::pin(http_readyz(e)),
        },
        Check {
            name: "rest_evaluate",
            surface: "rest",
            doc: "`POST /v1/evaluate` echoes the subject and is labelled a stub",
            run: |e| Box::pin(rest_evaluate(e)),
        },
        Check {
            name: "sse_events",
            surface: "sse",
            doc: "`GET /v1/subjects/{id}/events` delivers two events",
            run: |e| Box::pin(sse_events(e)),
        },
        Check {
            name: "graphql_evaluate",
            surface: "graphql",
            doc: "`version`, `engineReady` and `evaluate` resolve without errors",
            run: |e| Box::pin(graphql_evaluate(e)),
        },
        Check {
            name: "ws_echo",
            surface: "ws",
            doc: "`/ws` echoes a text frame as a `data` message",
            run: |e| Box::pin(ws_echo(e)),
        },
        Check {
            name: "ws_mux",
            surface: "ws",
            doc: "`/v1/ws` calls a public RPC by name and ends it on cancel",
            run: |e| Box::pin(ws_mux(e)),
        },
        Check {
            name: "http_mcp_tools",
            surface: "http",
            doc: "`POST /mcp` `tools/list` offers the public RPCs as tools, each with an object input schema",
            run: |e| Box::pin(http_mcp_tools(e)),
        },
        Check {
            name: "grpc_protocol_health",
            surface: "grpc",
            doc: "the overall health check answers",
            run: |e| Box::pin(grpc_protocol_health(e)),
        },
        Check {
            name: "grpc_protocol_ping",
            surface: "grpc",
            doc: "`Ping` echoes the message",
            run: |e| Box::pin(grpc_protocol_ping(e)),
        },
    ],
};

/// `[stack.protocols.<name>]` minus `listen`: a protocol forwarding to an
/// engine, named either as an instance in the same stack or as a bare URL.
///
/// Exactly one of the two. `engine` is the ordinary case and the stack resolves
/// it; `engine_url` points at something the stack does not own — a balancer in
/// front of several engines, or an engine already running elsewhere.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, try_from = "RawProtocol")]
pub struct Protocol {
    /// Name of the engine instance this protocol forwards to.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    /// A literal `http://host:port` to forward to instead.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine_url: Option<String>,
}

/// The table as written, before the one-of-two rule is checked. Checking it in
/// `TryFrom` means a table with neither or both is rejected where an unknown
/// field is — at parse time, as a spec error — rather than when the instance
/// fails to start.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawProtocol {
    #[serde(default)]
    engine: Option<String>,
    #[serde(default)]
    engine_url: Option<String>,
}

impl TryFrom<RawProtocol> for Protocol {
    type Error = String;

    fn try_from(raw: RawProtocol) -> Result<Self, Self::Error> {
        match (&raw.engine, &raw.engine_url) {
            (None, None) => Err("a protocol needs an engine or an engine_url".to_owned()),
            (Some(_), Some(_)) => Err("set engine or engine_url, not both".to_owned()),
            _ => Ok(Self {
                engine: raw.engine,
                engine_url: raw.engine_url,
            }),
        }
    }
}

impl Protocol {
    /// The engine address. The one-of-two rule already held at parse time, so
    /// the only thing left to go wrong here is a named engine that is not up.
    fn upstream(&self, name: &str, peers: &Peers<'_>) -> anyhow::Result<String> {
        match (self.engine.as_deref(), self.engine_url.as_deref()) {
            (Some(instance), _) => peers
                .addr(instance)
                .map(|addr| format!("http://{addr}"))
                .ok_or_else(|| {
                    anyhow::anyhow!("protocol {name}: engine {instance:?} is not running")
                }),
            (None, Some(url)) => Ok(url.to_owned()),
            (None, None) => Err(anyhow::anyhow!(
                "protocol {name}: needs an engine or an engine_url"
            )),
        }
    }
}

struct ProtocolHandle {
    addr: SocketAddr,
    client: reqwest::Client,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for ProtocolHandle {
    async fn ready(&self) -> bool {
        self.client
            .get(format!("http://{}/readyz", self.addr))
            .send()
            .await
            .is_ok_and(|r| r.status().is_success())
    }

    async fn stop(self: Box<Self>) {
        self.task.stop().await;
    }
}

#[async_trait]
impl Service for Protocol {
    fn kind(&self) -> &'static str {
        KIND.name
    }

    fn depends_on(&self) -> Vec<String> {
        // A bare URL is not this stack's to start or order.
        self.engine.clone().into_iter().collect()
    }

    async fn start(
        &self,
        name: &str,
        listen: SocketAddr,
        peers: &Peers<'_>,
    ) -> anyhow::Result<Instance> {
        let engine_url = self.upstream(name, peers)?;
        let listener = tokio::net::TcpListener::bind(listen).await?;
        let addr = listener.local_addr()?;
        let config = Config::embedded(addr, [("engine".to_owned(), engine_url)]);
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let instance_name = name.to_owned();
        let task = tokio::spawn(async move {
            if let Err(error) = tbd_protocol::serve_on(listener, config, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "protocol exited with error");
            }
        });
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(2))
            .build()?;
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            ProtocolHandle {
                addr,
                client,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}

async fn get_2xx(e: &Ep, path: &str) -> Result<String, String> {
    let r = e
        .http()
        .get(format!("{}{path}", e.url))
        .send()
        .await
        .map_err(|e| e.to_string())?;
    let status = r.status();
    let body = r.text().await.unwrap_or_default();
    if status.is_success() {
        Ok(format!("{status} {body}"))
    } else {
        Err(format!("{status} {body}"))
    }
}

async fn http_healthz(e: Ep) -> Result<String, String> {
    get_2xx(&e, "/healthz").await
}

async fn http_readyz(e: Ep) -> Result<String, String> {
    get_2xx(&e, "/readyz").await
}

/// The MCP transport lists tools the way an agent asks for them: JSON-RPC
/// over POST, JSON or one server-sent event back.
async fn http_mcp_tools(e: Ep) -> Result<String, String> {
    let text = e
        .http()
        .post(format!("{}/mcp", e.url))
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", "2025-11-25")
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": "tools/list", "params": {}}))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .text()
        .await
        .map_err(|e| e.to_string())?;
    let body = text
        .lines()
        .filter_map(|l| l.strip_prefix("data:"))
        .next_back()
        .map_or(text.as_str(), str::trim);
    let v: Value = serde_json::from_str(body).map_err(|e| format!("{e}: {text}"))?;
    let tools = v["result"]["tools"]
        .as_array()
        .ok_or_else(|| format!("no tools in {v}"))?;
    if tools.is_empty() {
        return Err("no tools listed".into());
    }
    if let Some(bad) = tools.iter().find(|t| t["inputSchema"]["type"] != "object") {
        return Err(format!("a tool without an object schema: {bad}"));
    }
    Ok(format!("tools={}", tools.len()))
}

async fn rest_evaluate(e: Ep) -> Result<String, String> {
    let v: Value = e
        .http()
        .post(format!("{}/v1/evaluate", e.url))
        .json(&json!({ "subject_id": "validate", "payload": "hi" }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if v["stub"].is_boolean() && v["subject_id"] == "validate" {
        Ok(format!("stub={} model={}", v["stub"], v["model_version"]))
    } else {
        Err(format!("unexpected body {v}"))
    }
}

async fn sse_events(e: Ep) -> Result<String, String> {
    let r = e
        .http()
        .get(format!("{}/v1/subjects/validate/events", e.url))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .error_for_status()
        .map_err(|e| e.to_string())?;
    let mut body = r.bytes_stream();
    let mut buf = String::new();
    let mut events = 0;
    while events < 2 {
        let chunk = body
            .next()
            .await
            .ok_or("stream ended")?
            .map_err(|e| e.to_string())?;
        buf.push_str(&String::from_utf8_lossy(&chunk));
        events = buf.matches("data:").count();
    }
    Ok(format!("{events} events"))
}

async fn graphql_evaluate(e: Ep) -> Result<String, String> {
    let v: Value = e.http()
        .post(format!("{}/graphql", e.url))
        .json(&json!({ "query": "{ version engineReady evaluate(subjectId:\"validate\"){ stub modelVersion } }" }))
        .send()
        .await
        .map_err(|e| e.to_string())?
        .json()
        .await
        .map_err(|e| e.to_string())?;
    if !v["errors"].is_null() {
        return Err(format!("errors: {}", v["errors"]));
    }
    if v["data"]["engineReady"] != true {
        return Err(format!("engineReady={}", v["data"]["engineReady"]));
    }
    Ok(format!(
        "version={} stub={}",
        v["data"]["version"], v["data"]["evaluate"]["stub"]
    ))
}

async fn ws_echo(e: Ep) -> Result<String, String> {
    let mut ws = e.connect_ws("/ws").await?;
    ws.send(Message::Text("validate".into()))
        .await
        .map_err(|e| e.to_string())?;
    loop {
        let msg = ws
            .next()
            .await
            .ok_or("closed before echo")?
            .map_err(|e| e.to_string())?;
        let text = msg.into_text().map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        if v["type"] == "data" {
            let _ = ws.close(None).await;
            return if v["data"] == "validate" {
                Ok("echo ok".into())
            } else {
                Err(format!("wrong echo {v}"))
            };
        }
    }
}

/// The multiplexed socket: one call, one answer, then a cancel that ends it.
/// `Subscribe` is the engine's public RPC, so this needs no other backend.
async fn ws_mux(e: Ep) -> Result<String, String> {
    let mut ws = e.connect_ws("/v1/ws").await?;
    let call = serde_json::json!({
        "type": "call",
        "id": "validate",
        "method": "tbd.engine.v1.EngineService/Subscribe",
        "body": {"subject_id": "validate"},
    });
    ws.send(Message::Text(call.to_string().into()))
        .await
        .map_err(|e| e.to_string())?;
    let mut answered = false;
    loop {
        let msg = ws
            .next()
            .await
            .ok_or("closed before the call ended")?
            .map_err(|e| e.to_string())?;
        let text = msg.into_text().map_err(|e| e.to_string())?;
        let v: Value = serde_json::from_str(&text).map_err(|e| e.to_string())?;
        match v["type"].as_str() {
            Some("data") if !answered => {
                answered = true;
                let cancel = serde_json::json!({"type": "cancel", "id": "validate"});
                ws.send(Message::Text(cancel.to_string().into()))
                    .await
                    .map_err(|e| e.to_string())?;
            }
            Some("data") => {}
            Some("end") => {
                let _ = ws.close(None).await;
                return Ok("call, data, cancel, end ok".into());
            }
            _ => {
                let _ = ws.close(None).await;
                return Err(format!("unexpected frame {v}"));
            }
        }
    }
}

async fn grpc_protocol_health(e: Ep) -> Result<String, String> {
    let mut h = HealthClient::new(e.grpc()?);
    let resp = h
        .check(HealthCheckRequest {
            service: String::new(),
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("status {}", resp.into_inner().status))
}

async fn grpc_protocol_ping(e: Ep) -> Result<String, String> {
    let mut c = ProtocolServiceClient::new(e.grpc()?);
    let r = c
        .ping(PingRequest {
            message: "validate".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if r.message == "validate" {
        Ok(format!("version={}", r.protocol_version))
    } else {
        Err(format!("wrong echo {r:?}"))
    }
}
