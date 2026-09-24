//! Test harness: the service on an ephemeral port with the shipped `local`
//! config; with a store on the shared test Postgres (`tbd_db::testing`), the
//! sources served by wiremock (they are external feeds, not our services), and
//! the real llm service on its in-process stub engine when a test needs one.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::{net::SocketAddr, path::Path};

use base64::Engine as _;
use sqlx::PgPool;
use tbd_proto::radar::v1::radar_service_client::RadarServiceClient;
use tbd_radar::{
    Config, Runtime,
    config::{SourceKind, SourceSpec},
};
use tokio::{net::TcpListener, sync::oneshot};
use tonic::{Request, transport::Channel};
use tonic_health::pb::health_client::HealthClient;

pub struct Server {
    pub addr: SocketAddr,
    pub runtime: Runtime,
    _stop: oneshot::Sender<()>,
}

fn load() -> Config {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/radar");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.metrics.listen = None;
    // No timers in tests: every read and write is asked for.
    config.fetch.interval_secs = 0;
    config.digest.check_secs = 0;
    config.llm.url = String::new();
    config
}

pub async fn start() -> Server {
    start_with(Runtime::default()).await
}

pub async fn start_with(runtime: Runtime) -> Server {
    spawn(load(), runtime).await
}

async fn spawn(mut config: Config, runtime: Runtime) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    config.server.listen = addr;
    let (stop, stopped) = oneshot::channel();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_radar::serve_with(listener, config, rt, async {
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

/// A fresh, migrated database: its URL and the pool the test reads with.
pub async fn database() -> (String, PgPool) {
    let url = tbd_db::testing::fresh_database(tbd_db::testing::ENV, "radar_test").await;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(4)
        .connect(&url)
        .await
        .unwrap();
    tbd_db::migrate(&pool).await.unwrap();
    (url, pool)
}

/// The service on a fresh database, `adjust` applied to the config first.
pub async fn start_with_store(adjust: impl FnOnce(&mut Config)) -> (Server, PgPool) {
    let (url, pool) = database().await;
    let mut config = load();
    config.store.url = url;
    adjust(&mut config);
    (spawn(config, Runtime::default()).await, pool)
}

/// A source of `kind` at `url`.
pub fn source(name: &str, kind: SourceKind, language: &str, url: &str) -> SourceSpec {
    SourceSpec {
        name: name.to_owned(),
        kind,
        language: language.to_owned(),
        url: url.to_owned(),
        query: if kind == SourceKind::Github {
            "repo:x/y updated:>={since}".to_owned()
        } else {
            String::new()
        },
        lookback_days: 14,
    }
}

/// The llm service on its stub engine, on an ephemeral port: its URL.
pub async fn llm_stub() -> (String, oneshot::Sender<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let config = tbd_llm::Config::stub(addr);
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_llm::serve_with(listener, config, Runtime::default(), async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });
    (format!("http://{addr}"), stop)
}

/// A request carrying the claims Envoy would forward for `subject` with `role`.
pub fn as_caller<T>(subject: &str, role: Option<&str>, message: T) -> Request<T> {
    let mut claims = serde_json::json!({ "sub": subject, "aud": ["tbd-ui"] });
    if let Some(r) = role {
        claims["role"] = r.into();
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request
        .metadata_mut()
        .insert("x-jwt-payload", payload.parse().unwrap());
    request
}

impl Server {
    pub async fn channel(&self) -> Channel {
        Channel::from_shared(format!("http://{}", self.addr))
            .unwrap()
            .connect()
            .await
            .unwrap()
    }

    pub async fn client(&self) -> RadarServiceClient<Channel> {
        RadarServiceClient::new(self.channel().await)
    }

    pub async fn health(&self) -> HealthClient<Channel> {
        HealthClient::new(self.channel().await)
    }
}
