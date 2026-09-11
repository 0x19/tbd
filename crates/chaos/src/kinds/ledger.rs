//! The ledger kind: an in-process `tbd-ledger` with fault injection.
//! Rendered by `tbd new service`; edit freely, the CLI never rewrites it.

use std::net::SocketAddr;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;
use tbd_ledger::{Config, Runtime};
use tbd_proto::ledger::v1::{
    AppendRequest, CurrentRequest, Envelope, EraseRequest, HistoryRequest, PingRequest,
    RestoreRequest, RetractRequest, Source, ledger_service_client::LedgerServiceClient,
};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Field, FieldKind, Kind, Target};
use crate::{
    service::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle},
    validate::{Check, Endpoint as Ep},
};

/// The registry entry.
pub static KIND: Kind = Kind {
    name: "ledger",
    label: "Ledger",
    plural: "ledgers",
    surface: "grpc",
    target: Some(Target {
        help: "Ledger gRPC URL (through Envoy: the engine LB, matched by service name)",
        default_url: "http://127.0.0.1:50052",
    }),
    fields: &[
        Field {
            name: "grace",
            label: "Erasure grace",
            kind: FieldKind::Duration,
            required: false,
            default: Some("7d"),
        },
        Field {
            name: "database_url",
            label: "Postgres URL (empty: in memory)",
            kind: FieldKind::Text,
            required: false,
            default: None,
        },
    ],
    fault: true,
    counters: true,
    load_target: true,
    addable: true,
    parse: super::parse::<Ledger>,
    checks: &[
        Check {
            name: "grpc_ledger_ping",
            surface: "grpc",
            doc: "`Ping` echoes the message and names the store behind it",
            run: |e| Box::pin(grpc_ledger_ping(e)),
        },
        Check {
            name: "grpc_ledger_facts",
            surface: "grpc",
            doc: "append, current, history, retract, a history cut without the value, erase, restore, on a throwaway subject",
            run: |e| Box::pin(grpc_ledger_facts(e)),
        },
    ],
};

/// `[stack.ledgers.<name>]` minus `listen`: a ledger with an initial
/// behaviour, an erasure grace window, and its store (in memory unless a
/// Postgres URL is given; the sweeper runs every second either way so a
/// scenario can watch an erasure execute).
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Ledger {
    /// Initial fault behaviour.
    pub behavior: Behavior,
    /// The erasure grace window.
    #[serde(with = "humantime_serde")]
    pub grace: std::time::Duration,
    /// A Postgres URL for the real store; empty means in memory.
    pub database_url: String,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            behavior: Behavior::Healthy,
            grace: std::time::Duration::from_hours(7 * 24),
            database_url: String::new(),
        }
    }
}

struct LedgerHandle {
    addr: SocketAddr,
    runtime: Runtime,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for LedgerHandle {
    async fn ready(&self) -> bool {
        let Ok(channel) = Endpoint::from_shared(format!("http://{}", self.addr)) else {
            return false;
        };
        let Ok(channel) = channel.connect().await else {
            return false;
        };
        let mut health = HealthClient::new(channel);
        let req = HealthCheckRequest {
            service: "tbd.ledger.v1.LedgerService".to_owned(),
        };
        health.check(req).await.is_ok()
    }

    async fn stop(self: Box<Self>) {
        self.task.stop().await;
    }

    fn fault(&self) -> Option<tbd_common::fault::FaultHandle> {
        Some(self.runtime.fault.clone())
    }

    fn requests(&self) -> Option<RequestCounts> {
        let s = self.runtime.stats.snapshot();
        Some(RequestCounts {
            total: s.requests_total,
            failed: s.requests_failed,
        })
    }
}

#[async_trait]
impl Service for Ledger {
    fn kind(&self) -> &'static str {
        KIND.name
    }

    async fn start(
        &self,
        name: &str,
        listen: SocketAddr,
        _peers: &Peers<'_>,
    ) -> anyhow::Result<Instance> {
        let listener = tokio::net::TcpListener::bind(listen).await?;
        let addr = listener.local_addr()?;
        let mut config = Config::in_memory(addr);
        config.erasure.grace = self.grace;
        config.erasure.sweep_interval = std::time::Duration::from_secs(1);
        if !self.database_url.is_empty() {
            config.store.kind = tbd_ledger::StoreKind::Postgres;
            config.store.url.clone_from(&self.database_url);
        }
        let runtime = Runtime {
            fault: tbd_common::fault::FaultHandle::new(self.behavior.clone()),
            ..Default::default()
        };
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let rt = runtime.clone();
        let instance_name = name.to_owned();
        let task = tokio::spawn(async move {
            if let Err(error) = tbd_ledger::serve_with(listener, config, rt, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "ledger exited with error");
            }
        });
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            LedgerHandle {
                addr,
                runtime,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}

/// `LedgerService/Ping` echoes and is labelled a stub. Not a named health
/// check: through Envoy's internal listener a health request lands on the
/// engine, and a check whose result depends on the path is worse than none.
async fn grpc_ledger_ping(e: Ep) -> Result<String, String> {
    let mut c = LedgerServiceClient::new(e.grpc()?);
    let r = c
        .ping(PingRequest {
            message: "validate".into(),
        })
        .await
        .map_err(|e| e.to_string())?
        .into_inner();
    if r.message == "validate" {
        Ok(format!(
            "version={} store={} stub={}",
            r.version, r.store, r.stub
        ))
    } else {
        Err(format!("wrong echo {r:?}"))
    }
}

/// The facts contract end to end on a throwaway subject: append, current
/// shows it, history holds it, retract, a history cut at the earlier instant
/// does not show the value, erase denies reads, restore reopens.
async fn grpc_ledger_facts(e: Ep) -> Result<String, String> {
    let mut c = LedgerServiceClient::new(e.grpc()?);
    let subject = uuid::Uuid::now_v7().to_string();
    let envelope = |v: serde_json::Value| {
        Some(Envelope {
            version: 0,
            bytes: serde_json::to_vec(&v).unwrap_or_default(),
        })
    };
    let appended = c
        .append(AppendRequest {
            subject_id: subject.clone(),
            path: "profile.name".into(),
            source: Source::Declared as i32,
            value: envelope(serde_json::json!("validate")),
            origin: envelope(serde_json::json!({ "by": "validate" })),
            confidence: Some(1.0),
            counterparty_id: None,
            observed_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
            expires_at: None,
            consent: vec!["self".into()],
            stub: false,
            idempotency_key: String::new(),
        })
        .await
        .map_err(|s| format!("append: {s}"))?
        .into_inner();
    let fact = appended.fact.ok_or("append: no fact")?;
    let current = |subject: String| CurrentRequest {
        subject_id: subject,
        paths: vec![],
        sources: vec![],
        scopes: vec!["self".into()],
        cursor: String::new(),
        limit: 0,
    };
    let page = c
        .current(current(subject.clone()))
        .await
        .map_err(|s| format!("current: {s}"))?
        .into_inner();
    if page.facts.len() != 1 || page.facts[0].id != fact.id {
        return Err("current does not show the appended fact".into());
    }
    c.retract(RetractRequest {
        subject_id: subject.clone(),
        path: "profile.name".into(),
        source: Source::Declared as i32,
        origin: envelope(serde_json::json!({ "by": "validate" })),
    })
    .await
    .map_err(|s| format!("retract: {s}"))?;
    let cut = c
        .history(HistoryRequest {
            subject_id: subject.clone(),
            paths: vec![],
            sources: vec![],
            scopes: vec!["self".into()],
            cursor: String::new(),
            limit: 0,
            at: fact.recorded_at,
        })
        .await
        .map_err(|s| format!("history: {s}"))?
        .into_inner();
    if !cut.facts.is_empty() {
        return Err("a retracted value is visible in a history cut".into());
    }
    c.erase(EraseRequest {
        subject_id: subject.clone(),
    })
    .await
    .map_err(|s| format!("erase: {s}"))?;
    match c.current(current(subject.clone())).await {
        Err(s) if s.code() == tonic::Code::FailedPrecondition => {}
        Ok(_) => return Err("reads allowed while erased".into()),
        Err(s) => return Err(format!("current while erased: {s}")),
    }
    c.restore(RestoreRequest {
        subject_id: subject.clone(),
    })
    .await
    .map_err(|s| format!("restore: {s}"))?;
    let page = c
        .current(current(subject))
        .await
        .map_err(|s| format!("current after restore: {s}"))?
        .into_inner();
    if !page.facts.is_empty() {
        return Err("restore resurrected a retracted value".into());
    }
    Ok("append, current, retract, cut, erase, restore ok".into())
}
