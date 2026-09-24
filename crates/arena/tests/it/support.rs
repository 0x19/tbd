//! Test harness: the service on an ephemeral port with the shipped `local` config.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::{net::SocketAddr, path::Path};

use tbd_arena::{Config, Runtime};
use tbd_proto::arena::v1::arena_service_client::ArenaServiceClient;
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
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/arena");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    let (stop, stopped) = oneshot::channel();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_arena::serve_with(listener, config, rt, async {
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

    pub async fn client(&self) -> ArenaServiceClient<Channel> {
        ArenaServiceClient::new(self.channel().await)
    }

    pub async fn health(&self) -> HealthClient<Channel> {
        HealthClient::new(self.channel().await)
    }
}

/// The arena with its collectors running against a real llm (both tiers on
/// the stub engine) and whatever `adjust` points the other sources at, every
/// interval shortened so a test sees frames in milliseconds.
pub async fn start_live(adjust: impl FnOnce(&mut Config)) -> Server {
    let llm_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let llm_addr = llm_listener.local_addr().unwrap();
    let (llm_stop, llm_stopped) = oneshot::channel::<()>();
    tokio::spawn(async move {
        tbd_llm::serve_on(llm_listener, tbd_llm::Config::stub(llm_addr), async {
            let _ = llm_stopped.await;
        })
        .await
        .unwrap();
    });

    // A real runner on its stub engine, the same way.
    let runner_listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let runner_addr = runner_listener.local_addr().unwrap();
    let (runner_stop, runner_stopped) = oneshot::channel::<()>();
    tokio::spawn(async move {
        tbd_runner::serve_on(
            runner_listener,
            tbd_runner::Config::stub(runner_addr),
            async {
                let _ = runner_stopped.await;
            },
        )
        .await
        .unwrap();
    });

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/arena");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    config.sources.llm_url = format!("http://{llm_addr}");
    config.sources.runner_url = format!("http://{runner_addr}");
    let fast = std::time::Duration::from_millis(50);
    config.collect.llm_every = fast;
    config.collect.runner_every = fast;
    config.collect.metrics_every = fast;
    config.collect.chaos_every = fast;
    config.watch.tick = fast;
    adjust(&mut config);
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_arena::serve_on(listener, config, async {
            let _ = stopped.await;
            drop(llm_stop);
            drop(runner_stop);
        })
        .await
        .unwrap();
    });
    Server {
        addr,
        runtime: Runtime::default(),
        _stop: stop,
    }
}

/// A request as a verified caller with this role (what Envoy forwards).
pub fn as_role<T>(role: &str, message: T) -> tonic::Request<T> {
    use base64::Engine as _;
    let claims = serde_json::json!({ "sub": "person-1", "role": role }).to_string();
    let mut request = tonic::Request::new(message);
    request.metadata_mut().insert(
        tbd_common::principal::PAYLOAD_HEADER,
        base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(claims)
            .parse()
            .unwrap(),
    );
    request
}
