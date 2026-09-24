//! Test harness: engine and protocol, each on an ephemeral port.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::net::SocketAddr;

use clap::Parser;
use tokio::{net::TcpListener, sync::oneshot};

pub struct Stack {
    pub engine_addr: SocketAddr,
    pub ledger_addr: SocketAddr,
    pub protocol_addr: SocketAddr,
    /// The engine's runtime: `engine.fault.set(..)` injects faults.
    pub engine: tbd_engine::Runtime,
    _stop_engine: oneshot::Sender<()>,
    _stop_ledger: oneshot::Sender<()>,
    _stop_finance: oneshot::Sender<()>,
    _stop_protocol: oneshot::Sender<()>,
}

pub async fn start() -> Stack {
    let engine_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let engine_addr = engine_listener.local_addr().unwrap();
    let engine_config = tbd_engine::Config::parse_from(["engine", "--heartbeat-ms", "20"]);
    let runtime = tbd_engine::Runtime::default();
    let engine_runtime = runtime.clone();
    let (stop_engine, engine_stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_engine::serve_with(engine_listener, engine_config, engine_runtime, async {
            let _ = engine_stopped.await;
        })
        .await
        .unwrap();
    });

    // An in-process ledger on the memory store: the first backend the
    // transcoder exposes.
    let ledger_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let ledger_addr = ledger_listener.local_addr().unwrap();
    let ledger_config = tbd_ledger::Config::in_memory(ledger_addr);
    let (stop_ledger, ledger_stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_ledger::serve_on(ledger_listener, ledger_config, async {
            let _ = ledger_stopped.await;
        })
        .await
        .unwrap();
    });

    // An in-process finance with no store: its data RPCs check the caller
    // before anything else, which is what makes it the backend that proves
    // the verified identity travels through the gateway.
    let finance_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let finance_addr = finance_listener.local_addr().unwrap();
    let finance_dir =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/finance");
    let (mut finance_config, _) = tbd_finance::Config::load(&finance_dir, "local").unwrap();
    finance_config.server.listen = finance_addr;
    finance_config.metrics.listen = None;
    finance_config.store.url = String::new();
    let (stop_finance, finance_stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_finance::serve_on(finance_listener, finance_config, async {
            let _ = finance_stopped.await;
        })
        .await
        .unwrap();
    });

    let protocol_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let protocol_addr = protocol_listener.local_addr().unwrap();
    let mut protocol_config = tbd_protocol::Config::embedded(
        protocol_addr,
        [
            ("engine".to_owned(), format!("http://{engine_addr}")),
            ("ledger".to_owned(), format!("http://{ledger_addr}")),
            // Registered and never up, like a scaffolded service before its
            // first deploy: its routes exist and answer `unavailable`.
            ("humans".to_owned(), "http://127.0.0.1:1".to_owned()),
            ("radar".to_owned(), "http://127.0.0.1:1".to_owned()),
            ("arena".to_owned(), "http://127.0.0.1:1".to_owned()),
            ("llm".to_owned(), "http://127.0.0.1:1".to_owned()),
            ("cv".to_owned(), "http://127.0.0.1:1".to_owned()),
            ("playground".to_owned(), "http://127.0.0.1:1".to_owned()),
            ("finance".to_owned(), format!("http://{finance_addr}")),
        ],
    );
    // Everything but the two backends this harness actually starts is
    // registered and never up. Derived from the registry rather than listed, so
    // `tbd new service` only has to add its line above.
    for (name, service) in &mut protocol_config.services {
        if name != "engine" && name != "ledger" {
            service.required = false;
        }
    }
    // A subject the tests can present as one of our own services.
    protocol_config.principals.services = vec!["svc-ledger".to_owned()];
    // The engine's subscribe stream never ends on its own: a small cap lets the
    // MCP tests see a stream collected and cut.
    protocol_config.mcp.max_stream_items = 3;
    // The tests call these; production's allowlist is narrower (base.toml).
    protocol_config.mcp.tools = [
        "ledger_ping",
        "engine_subscribe",
        "llm_generate",
        "llm_list_models",
    ]
    .map(str::to_owned)
    .to_vec();
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
        ledger_addr,
        protocol_addr,
        engine: runtime,
        _stop_engine: stop_engine,
        _stop_ledger: stop_ledger,
        _stop_finance: stop_finance,
        _stop_protocol: stop_protocol,
    }
}

impl Stack {
    #[allow(clippy::unused_self)]
    pub fn client(&self) -> reqwest::Client {
        reqwest::Client::new()
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{path}", self.protocol_addr)
    }

    pub fn ws_url(&self, path: &str) -> String {
        format!("ws://{}{path}", self.protocol_addr)
    }
}
