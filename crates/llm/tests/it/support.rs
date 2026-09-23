//! Test harness: the service on an ephemeral port with the shipped `local` config.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::{net::SocketAddr, path::Path};

use tbd_llm::{Config, Runtime};
use tbd_proto::llm::v1::llm_service_client::LlmServiceClient;
use tokio::{net::TcpListener, sync::oneshot};
use tonic::transport::Channel;
use tonic_health::pb::health_client::HealthClient;

pub struct Server {
    pub addr: SocketAddr,
    pub runtime: Runtime,
    _stop: oneshot::Sender<()>,
}

pub async fn start() -> Server {
    start_with(Runtime::default()).await
}

pub async fn start_with(runtime: Runtime) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/llm");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    let (stop, stopped) = oneshot::channel();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_llm::serve_with(listener, config, rt, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });
    Server {
        addr,
        runtime,
        _stop: stop,
    }
}

impl Server {
    pub async fn channel(&self) -> Channel {
        Channel::from_shared(format!("http://{}", self.addr))
            .unwrap()
            .connect()
            .await
            .unwrap()
    }

    pub async fn client(&self) -> LlmServiceClient<Channel> {
        LlmServiceClient::new(self.channel().await)
    }

    pub async fn health(&self) -> HealthClient<Channel> {
        HealthClient::new(self.channel().await)
    }
}
