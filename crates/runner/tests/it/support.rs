//! Test harness: the service on an ephemeral port, on the stub engine unless a
//! test points it at a wiremock playing the sandbox daemon.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::net::SocketAddr;

use tbd_proto::runner::v1::runner_service_client::RunnerServiceClient;
use tbd_runner::{Config, Runtime};
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
    start_on(runtime, |_| {}).await
}

/// The stub config, adjusted.
pub async fn start_on(runtime: Runtime, adjust: impl FnOnce(&mut Config)) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let mut config = Config::stub(addr);
    adjust(&mut config);
    let (stop, stopped) = oneshot::channel();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_runner::serve_with(listener, config, rt, async {
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

    pub async fn client(&self) -> RunnerServiceClient<Channel> {
        RunnerServiceClient::new(self.channel().await)
    }

    pub async fn health(&self) -> HealthClient<Channel> {
        HealthClient::new(self.channel().await)
    }
}

/// A request as a verified caller with this role (what Envoy forwards).
pub fn as_role<T>(sub: &str, role: &str, message: T) -> tonic::Request<T> {
    use base64::Engine as _;
    let claims = serde_json::json!({ "sub": sub, "role": role }).to_string();
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
