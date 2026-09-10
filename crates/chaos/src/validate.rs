//! `chaos validate`: hit every surface of a running stack and report per check.
//!
//! Checks run concurrently, each with its own timeout. A check is a plain
//! async function returning [`CheckResult`]; add one to `all()` to extend.

use std::time::{Duration, Instant};

use futures::{SinkExt, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use tbd_proto::{
    engine::v1::{EvaluateRequest, SubscribeRequest, engine_service_client::EngineServiceClient},
    protocol::v1::{PingRequest, protocol_service_client::ProtocolServiceClient},
};
use tokio_tungstenite::tungstenite::Message;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

/// Where to point the checks.
#[derive(Debug, Clone)]
pub struct Targets {
    /// Protocol base URL, e.g. `http://127.0.0.1:8080`.
    pub protocol: String,
    /// Engine gRPC URL, e.g. `http://127.0.0.1:50051`.
    pub engine: String,
    /// Per-check timeout.
    pub timeout: Duration,
    /// Roots for `https://` / `wss://` targets.
    pub trust: crate::tls::Trust,
}

/// Outcome of one check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name.
    pub name: String,
    /// Surface it exercises.
    pub surface: String,
    /// Passed.
    pub passed: bool,
    /// Wall time.
    pub latency_ms: f64,
    /// What was observed, or the error.
    pub detail: String,
}

/// Whole-run report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Every check.
    pub checks: Vec<CheckResult>,
    /// Count of passed checks.
    pub passed: usize,
    /// Count of failed checks.
    pub failed: usize,
}

impl Report {
    /// True when nothing failed.
    pub fn ok(&self) -> bool {
        self.failed == 0
    }

    /// Human-readable rendering.
    pub fn render(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        for c in &self.checks {
            let mark = if c.passed { "PASS" } else { "FAIL" };
            let _ = writeln!(
                out,
                "{mark}  {:<22} {:<8} {:>8.1} ms  {}",
                c.name, c.surface, c.latency_ms, c.detail
            );
        }
        let _ = writeln!(out, "\n{} passed, {} failed", self.passed, self.failed);
        out
    }
}

type Check = fn(Targets) -> futures::future::BoxFuture<'static, Result<String, String>>;

/// The full list. Order is display order.
fn all() -> Vec<(&'static str, &'static str, Check)> {
    vec![
        ("http_healthz", "http", |t| Box::pin(http_healthz(t))),
        ("http_readyz", "http", |t| Box::pin(http_readyz(t))),
        ("rest_evaluate", "rest", |t| Box::pin(rest_evaluate(t))),
        ("sse_events", "sse", |t| Box::pin(sse_events(t))),
        ("graphql_evaluate", "graphql", |t| {
            Box::pin(graphql_evaluate(t))
        }),
        ("ws_echo", "ws", |t| Box::pin(ws_echo(t))),
        ("grpc_engine_health", "grpc", |t| {
            Box::pin(grpc_engine_health(t))
        }),
        ("grpc_engine_evaluate", "grpc", |t| {
            Box::pin(grpc_engine_evaluate(t))
        }),
        ("grpc_engine_subscribe", "grpc", |t| {
            Box::pin(grpc_engine_subscribe(t))
        }),
        ("grpc_protocol_health", "grpc", |t| {
            Box::pin(grpc_protocol_health(t))
        }),
        ("grpc_protocol_ping", "grpc", |t| {
            Box::pin(grpc_protocol_ping(t))
        }),
    ]
}

/// Run every check concurrently.
pub async fn run(mut targets: Targets) -> Report {
    // One token for the whole run; a failure to get it fails everything at once.
    match targets.trust.snapshot().await {
        Ok(trust) => targets.trust = trust,
        Err(error) => {
            return Report {
                checks: vec![CheckResult {
                    name: "auth_token".into(),
                    surface: "auth".into(),
                    passed: false,
                    latency_ms: 0.0,
                    detail: format!("{error:#}"),
                }],
                passed: 0,
                failed: 1,
            };
        }
    }
    let mut set = tokio::task::JoinSet::new();
    for (idx, (name, surface, check)) in all().into_iter().enumerate() {
        let t = targets.clone();
        set.spawn(async move {
            let started = Instant::now();
            let outcome = tokio::time::timeout(t.timeout, check(t.clone())).await;
            let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
            let (passed, detail) = match outcome {
                Ok(Ok(detail)) => (true, detail),
                Ok(Err(detail)) => (false, detail),
                Err(_) => (false, format!("timed out after {:?}", t.timeout)),
            };
            (
                idx,
                CheckResult {
                    name: name.to_owned(),
                    surface: surface.to_owned(),
                    passed,
                    latency_ms,
                    detail,
                },
            )
        });
    }
    let mut results: Vec<(usize, CheckResult)> = Vec::new();
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(r) => results.push(r),
            Err(error) => tracing::error!(%error, "check task panicked"),
        }
    }
    results.sort_by_key(|(i, _)| *i);
    let checks: Vec<CheckResult> = results.into_iter().map(|(_, c)| c).collect();
    let passed = checks.iter().filter(|c| c.passed).count();
    let failed = checks.len() - passed;
    Report {
        checks,
        passed,
        failed,
    }
}

async fn http_healthz(t: Targets) -> Result<String, String> {
    let r = t
        .trust
        .http(None)
        .get(format!("{}/healthz", t.protocol))
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

async fn http_readyz(t: Targets) -> Result<String, String> {
    let r = t
        .trust
        .http(None)
        .get(format!("{}/readyz", t.protocol))
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

async fn rest_evaluate(t: Targets) -> Result<String, String> {
    let v: Value = t
        .trust
        .http(None)
        .post(format!("{}/v1/evaluate", t.protocol))
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

async fn sse_events(t: Targets) -> Result<String, String> {
    let r = t
        .trust
        .http(None)
        .get(format!("{}/v1/subjects/validate/events", t.protocol))
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

async fn graphql_evaluate(t: Targets) -> Result<String, String> {
    let v: Value = t.trust.http(None)
        .post(format!("{}/graphql", t.protocol))
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

async fn ws_echo(t: Targets) -> Result<String, String> {
    let url = t.protocol.replacen("http", "ws", 1) + "/ws";
    let mut ws = t.trust.connect_ws(&url).await.map_err(|e| e.to_string())?;
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

fn channel(t: &Targets, url: &str) -> Result<crate::tls::Grpc, String> {
    t.trust.grpc(url, None).map_err(|e| e.to_string())
}

async fn grpc_engine_health(t: Targets) -> Result<String, String> {
    let mut h = HealthClient::new(channel(&t, &t.engine)?);
    let resp = h
        .check(HealthCheckRequest {
            service: "tbd.engine.v1.EngineService".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if resp.status == tonic_health::pb::health_check_response::ServingStatus::Serving as i32 {
        Ok("SERVING".into())
    } else {
        Err(format!("status {}", resp.status))
    }
}

async fn grpc_engine_evaluate(t: Targets) -> Result<String, String> {
    let mut c = EngineServiceClient::new(channel(&t, &t.engine)?);
    let r = c
        .evaluate(EvaluateRequest {
            subject_id: "validate".into(),
            payload: b"hi".to_vec(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    Ok(format!("stub={} model={}", r.stub, r.model_version))
}

async fn grpc_engine_subscribe(t: Targets) -> Result<String, String> {
    let mut c = EngineServiceClient::new(channel(&t, &t.engine)?);
    let mut s = c
        .subscribe(SubscribeRequest {
            subject_id: "validate".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    let mut n = 0;
    while n < 2 {
        s.next()
            .await
            .ok_or("stream ended")?
            .map_err(|e| e.to_string())?;
        n += 1;
    }
    Ok(format!("{n} events"))
}

async fn grpc_protocol_health(t: Targets) -> Result<String, String> {
    let mut h = HealthClient::new(channel(&t, &t.protocol)?);
    let resp = h
        .check(HealthCheckRequest {
            service: String::new(),
        })
        .await
        .map_err(|e| e.to_string())?;
    Ok(format!("status {}", resp.into_inner().status))
}

async fn grpc_protocol_ping(t: Targets) -> Result<String, String> {
    let mut c = ProtocolServiceClient::new(channel(&t, &t.protocol)?);
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
