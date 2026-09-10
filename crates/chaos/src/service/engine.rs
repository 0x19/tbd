//! The engine as a chaos [`Service`].

use std::{net::SocketAddr, time::Duration};

use async_trait::async_trait;
use tbd_common::{fault::Behavior, telemetry::LogArgs};
use tbd_engine::{Config, Runtime};
use tonic::transport::Endpoint;
use tonic_health::pb::{HealthCheckRequest, health_client::HealthClient};

use super::{Instance, InstanceHandle, Peers, RequestCounts, Service, TaskHandle};

/// Start an engine with an initial behaviour and heartbeat interval.
#[derive(Debug, Clone)]
pub struct Engine {
    /// Initial fault behaviour.
    pub behavior: Behavior,
    /// Heartbeat interval on streams.
    pub heartbeat: Duration,
}

impl Default for Engine {
    fn default() -> Self {
        Self {
            behavior: Behavior::Healthy,
            heartbeat: Duration::from_secs(1),
        }
    }
}

struct EngineHandle {
    addr: SocketAddr,
    runtime: Runtime,
    task: TaskHandle,
}

#[async_trait]
impl InstanceHandle for EngineHandle {
    async fn ready(&self) -> bool {
        let Ok(channel) = Endpoint::from_shared(format!("http://{}", self.addr)) else {
            return false;
        };
        let Ok(channel) = channel.connect().await else {
            return false;
        };
        let mut health = HealthClient::new(channel);
        let req = HealthCheckRequest {
            service: "tbd.engine.v1.EngineService".to_owned(),
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
impl Service for Engine {
    fn kind(&self) -> &'static str {
        "engine"
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
            listen_addr: addr,
            heartbeat_ms: u64::try_from(self.heartbeat.as_millis()).unwrap_or(u64::MAX),
            log: LogArgs {
                format: tbd_common::telemetry::LogFormat::Text,
                filter: "info".into(),
            },
        };
        let runtime = Runtime {
            fault: tbd_common::fault::FaultHandle::new(self.behavior.clone()),
            ..Default::default()
        };
        let (stop, stopped) = tokio::sync::oneshot::channel();
        let rt = runtime.clone();
        let instance_name = name.to_owned();
        let task = tokio::spawn(async move {
            if let Err(error) = tbd_engine::serve_with(listener, config, rt, async {
                let _ = stopped.await;
            })
            .await
            {
                tracing::error!(instance = %instance_name, %error, "engine exited with error");
            }
        });
        Ok(Instance::new(
            name,
            self.kind(),
            addr,
            EngineHandle {
                addr,
                runtime,
                task: TaskHandle::new(stop, task),
            },
        ))
    }
}
