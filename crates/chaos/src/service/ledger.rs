//! The ledger service as a chaos [`Service`].

use std::net::SocketAddr;

use async_trait::async_trait;
use tbd_common::fault::Behavior;
use tbd_ledger::{
    Config, Runtime,
    config::{Metrics, Ping, Server},
};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle};

/// Start a ledger with an initial behaviour.
#[derive(Debug, Clone)]
pub struct Ledger {
    /// Initial fault behaviour.
    pub behavior: Behavior,
}

impl Default for Ledger {
    fn default() -> Self {
        Self {
            behavior: Behavior::Healthy,
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
        "ledger"
    }

    async fn start(
        &self,
        name: &str,
        listen: SocketAddr,
        _peers: &Peers<'_>,
    ) -> anyhow::Result<Instance> {
        let listener = tokio::net::TcpListener::bind(listen).await?;
        let addr = listener.local_addr()?;
        let config = Config {
            server: Server { listen: addr },
            metrics: Metrics { listen: None },
            ping: Ping::default(),
        };
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
