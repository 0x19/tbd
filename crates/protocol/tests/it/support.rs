//! Test harness: engine and gateway, each on an ephemeral port.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::net::SocketAddr;

use clap::Parser;
use tokio::{net::TcpListener, sync::oneshot};

pub struct Stack {
    pub engine_addr: SocketAddr,
    pub gateway_addr: SocketAddr,
    _stop_engine: oneshot::Sender<()>,
    _stop_gateway: oneshot::Sender<()>,
}

pub async fn start() -> Stack {
    let engine_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let engine_addr = engine_listener.local_addr().unwrap();
    let engine_config = tbd_engine::Config::parse_from(["engine", "--heartbeat-ms", "20"]);
    let (stop_engine, engine_stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_engine::serve_on(engine_listener, engine_config, async {
            let _ = engine_stopped.await;
        })
        .await
        .unwrap();
    });

    let gateway_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let gateway_addr = gateway_listener.local_addr().unwrap();
    let gateway_config = tbd_protocol::Config::parse_from([
        "protocol",
        "--engine-url",
        &format!("http://{engine_addr}"),
    ]);
    let (stop_gateway, gateway_stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_protocol::serve_on(gateway_listener, gateway_config, async {
            let _ = gateway_stopped.await;
        })
        .await
        .unwrap();
    });

    Stack {
        engine_addr,
        gateway_addr,
        _stop_engine: stop_engine,
        _stop_gateway: stop_gateway,
    }
}

impl Stack {
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.gateway_addr)
    }

    pub fn ws_url(&self, path: &str) -> String {
        format!("ws://{}{path}", self.gateway_addr)
    }
}
