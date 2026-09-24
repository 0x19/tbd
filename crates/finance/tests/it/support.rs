//! Test harness: the service on an ephemeral port with the shipped `local` config.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::{net::SocketAddr, path::Path};

use sqlx::PgPool;
use tbd_finance::{Config, Runtime};
use tbd_proto::finance::v1::finance_service_client::FinanceServiceClient;
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
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/finance");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    let (stop, stopped) = oneshot::channel();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_finance::serve_with(listener, config, rt, async {
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

/// The service on a fresh, migrated database.
///
/// The database is on the shared test Postgres (`tbd_db::testing`:
/// `TBD_TEST_DATABASE_URL`, or the one reusable container). With neither the
/// test fails and says so: an access-control test that silently skips is worse
/// than none.
pub async fn start_with_store() -> (Server, PgPool) {
    let url = tbd_db::testing::fresh_database(tbd_db::testing::ENV, "finance_test").await;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    tbd_db::migrate(&pool).await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/finance");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    config.store.url = url;
    let runtime = Runtime::default();
    let rt = runtime.clone();
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_finance::serve_with(listener, config, rt, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });

    (
        Server {
            addr,
            runtime,
            _stop: stop,
        },
        pool,
    )
}

/// Like `start_with_store`, with a sealing key and the given connector kinds.
pub async fn start_with_kinds(kinds: tbd_finance::connectors::KindsFactory) -> (Server, PgPool) {
    let url = tbd_db::testing::fresh_database(tbd_db::testing::ENV, "finance_test").await;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    tbd_db::migrate(&pool).await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/finance");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    config.store.url = url;
    config.connectors.key =
        base64::Engine::encode(&base64::engine::general_purpose::STANDARD, [9u8; 32]);
    config.connectors.redirect_url = "https://finance.test/connectors/callback".into();
    let runtime = Runtime::default();
    let rt = runtime.clone();
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_finance::serve_with_kinds(listener, config, rt, kinds, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });

    (
        Server {
            addr,
            runtime,
            _stop: stop,
        },
        pool,
    )
}

/// Like `start_with_store`, with a bank attached (the `Mock`, in tests) so the
/// connection and refresh RPCs can be driven end to end.
pub async fn start_with_bank(
    provider: std::sync::Arc<dyn tbd_finance::banking::Provider>,
) -> (Server, PgPool) {
    let url = tbd_db::testing::fresh_database(tbd_db::testing::ENV, "finance_test").await;
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    tbd_db::migrate(&pool).await.unwrap();

    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/finance");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.server.listen = addr;
    config.metrics.listen = None;
    config.store.url = url;
    let runtime = Runtime::default();
    let rt = runtime.clone();
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_finance::serve_with_bank(listener, config, rt, provider, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });

    (
        Server {
            addr,
            runtime,
            _stop: stop,
        },
        pool,
    )
}

impl Server {
    pub async fn channel(&self) -> Channel {
        Channel::from_shared(format!("http://{}", self.addr))
            .unwrap()
            .connect()
            .await
            .unwrap()
    }

    pub async fn client(&self) -> FinanceServiceClient<Channel> {
        FinanceServiceClient::new(self.channel().await)
    }

    pub async fn health(&self) -> HealthClient<Channel> {
        HealthClient::new(self.channel().await)
    }
}
