//! Test harness: engine on an ephemeral port with a fast heartbeat.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::net::SocketAddr;

use clap::Parser;
use tbd_engine::Config;
use tbd_proto::engine::v1::engine_client::EngineClient;
use tokio::{net::TcpListener, sync::oneshot};
use tonic::transport::Channel;

pub struct Server {
    pub addr: SocketAddr,
    _stop: oneshot::Sender<()>,
}

pub async fn start() -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = Config::parse_from(["engine", "--heartbeat-ms", "20"]);
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_engine::serve_on(listener, config, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });
    Server { addr, _stop: stop }
}

impl Server {
    pub async fn client(&self) -> EngineClient<Channel> {
        EngineClient::connect(format!("http://{}", self.addr))
            .await
            .unwrap()
    }
}
