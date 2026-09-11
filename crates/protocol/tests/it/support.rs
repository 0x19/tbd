//! Test harness: engine and protocol, each on an ephemeral port.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::net::SocketAddr;

use clap::Parser;
use tokio::{net::TcpListener, sync::oneshot};

pub struct Stack {
    pub engine_addr: SocketAddr,
    pub protocol_addr: SocketAddr,
    _stop_engine: oneshot::Sender<()>,
    _stop_protocol: oneshot::Sender<()>,
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

    let protocol_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let protocol_addr = protocol_listener.local_addr().unwrap();
    let protocol_config = tbd_protocol::Config::embedded(
        protocol_addr,
        [("engine".to_owned(), format!("http://{engine_addr}"))],
    );
    let (stop_protocol, protocol_stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_protocol::serve_on(protocol_listener, protocol_config, async {
            let _ = protocol_stopped.await;
        })
        .await
        .unwrap();
    });

    Stack {
        engine_addr,
        protocol_addr,
        _stop_engine: stop_engine,
        _stop_protocol: stop_protocol,
    }
}

impl Stack {
    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.protocol_addr)
    }

    pub fn ws_url(&self, path: &str) -> String {
        format!("ws://{}{path}", self.protocol_addr)
    }
}
