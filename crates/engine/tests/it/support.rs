//! Test harness: engine on an ephemeral port with a fast heartbeat.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::net::SocketAddr;

use clap::Parser;
use tbd_engine::{Config, Runtime};
use tbd_proto::engine::v1::engine_service_client::EngineServiceClient;
use tokio::{net::TcpListener, sync::oneshot};
use tonic::transport::Channel;

pub struct Server {
    pub addr: SocketAddr,
    pub runtime: Runtime,
    _stop: oneshot::Sender<()>,
}

pub async fn start() -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = Config::parse_from(["engine", "--heartbeat-ms", "20"]);
    let (stop, stopped) = oneshot::channel();
    let runtime = Runtime::default();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_engine::serve_with(listener, config, rt, async {
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
    pub async fn client(&self) -> EngineServiceClient<Channel> {
        EngineServiceClient::connect(format!("http://{}", self.addr))
            .await
            .unwrap()
    }
}
