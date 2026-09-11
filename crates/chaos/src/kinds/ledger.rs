//! The ledger kind: an in-process `tbd-ledger` with fault injection.
//! Rendered by `tbd new service`; edit freely, the CLI never rewrites it.

use std::net::SocketAddr;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;
use tbd_ledger::{Config, Runtime};
use tbd_proto::ledger::v1::{PingRequest, ledger_service_client::LedgerServiceClient};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Kind, Target};
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
    fields: &[],
    fault: true,
    counters: true,
    load_target: false,
    addable: true,
    parse: super::parse::<Ledger>,
    checks: &[Check {
        name: "grpc_ledger_ping",
        surface: "grpc",
        doc: "`Ping` echoes the message and names the store behind it",
        run: |e| Box::pin(grpc_ledger_ping(e)),
    }],
};

/// `[stack.ledgers.<name>]` minus `listen`: a ledger with an initial behaviour.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Ledger {
    /// Initial fault behaviour.
    pub behavior: Behavior,
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
        let config = Config::in_memory(addr);
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
