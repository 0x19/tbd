//! The protocol service as a chaos [`Service`].

use std::net::SocketAddr;

use async_trait::async_trait;
use tbd_common::telemetry::{LogArgs, LogFormat};
use tbd_protocol::Config;

use super::{Instance, InstanceHandle, Peers, Service, TaskHandle};

/// Start a protocol bound to a named engine instance.
#[derive(Debug, Clone)]
pub struct Protocol {
    /// Name of the engine instance this protocol forwards to.
    pub engine: String,
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
        "protocol"
    }

    fn depends_on(&self) -> Vec<String> {
        vec![self.engine.clone()]
    }

    async fn start(
        &self,
        name: &str,
        listen: SocketAddr,
        peers: &Peers<'_>,
    ) -> anyhow::Result<Instance> {
        let engine_addr = peers.addr(&self.engine).ok_or_else(|| {
            anyhow::anyhow!("protocol {name}: engine {:?} is not running", self.engine)
        })?;
        let listener = tokio::net::TcpListener::bind(listen).await?;
        let addr = listener.local_addr()?;
        let config = Config {
            listen_addr: addr,
            engine_url: format!("http://{engine_addr}"),
            log: LogArgs {
                format: LogFormat::Text,
                filter: "info".into(),
            },
        };
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
