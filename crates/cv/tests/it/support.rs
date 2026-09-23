//! Test harness: the service on an ephemeral port with the shipped `local`
//! config, with or without a fresh migrated database.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::{net::SocketAddr, path::Path};

use base64::Engine as _;
use sqlx::PgPool;
use tbd_cv::{Config, Runtime};
use tbd_proto::cv::v1::cv_service_client::CvServiceClient;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt as _, core::IntoContainerPort as _, core::WaitFor,
    runners::AsyncRunner as _,
};
use tokio::{net::TcpListener, sync::oneshot};
use tonic::{Request, transport::Channel};
use tonic_health::pb::health_client::HealthClient;

pub struct Server {
    pub addr: SocketAddr,
    pub runtime: Runtime,
    _stop: oneshot::Sender<()>,
    _container: Option<ContainerAsync<GenericImage>>,
}

pub async fn start() -> Server {
    start_with(Runtime::default()).await
}

pub async fn start_with(runtime: Runtime) -> Server {
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (mut config, _) = load();
    config.server.listen = addr;
    let (stop, stopped) = oneshot::channel();
    let rt = runtime.clone();
    tokio::spawn(async move {
        tbd_cv::serve_with(listener, config, rt, async {
            let _ = stopped.await;
        })
        .await
        .unwrap();
    });
    Server {
        addr,
        runtime,
        _stop: stop,
        _container: None,
    }
}

fn load() -> (Config, tbd_cv::Source) {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/cv");
    let (mut config, source) = Config::load(&dir, "local").unwrap();
    config.metrics.listen = None;
    (config, source)
}

const IMAGE: (&str, &str) = ("pgvector/pgvector", "0.8.6-pg17-trixie");

/// A fresh, migrated database: its URL and the pool the test reads with.
///
/// The server comes from `TBD_TEST_DATABASE_URL` or, when unset, a container
/// this test starts. With neither the test fails and says so.
pub async fn database() -> (String, PgPool, Option<ContainerAsync<GenericImage>>) {
    let (admin_url, container) = admin_url().await;
    let admin = PgPool::connect(&admin_url).await.unwrap();
    let database = format!("cv_test_{}", uuid::Uuid::now_v7().simple());
    sqlx::query(sqlx::AssertSqlSafe(format!("create database {database}")))
        .execute(&admin)
        .await
        .unwrap();
    drop(admin);
    let url = {
        let mut u = url::Url::parse(&admin_url).unwrap();
        u.set_path(&database);
        u.to_string()
    };
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .connect(&url)
        .await
        .unwrap();
    tbd_db::migrate(&pool).await.unwrap();
    (url, pool, container)
}

/// The service on a fresh, migrated database, with `adjust` applied to the
/// configuration first (a finance URL, the notify addresses, ...).
pub async fn start_with_store(adjust: impl FnOnce(&mut Config)) -> (Server, PgPool) {
    let (url, pool, container) = database().await;
    let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let (mut config, _) = load();
    config.server.listen = addr;
    config.store.url = url;
    adjust(&mut config);
    let runtime = Runtime::default();
    let rt = runtime.clone();
    let (stop, stopped) = oneshot::channel();
    tokio::spawn(async move {
        tbd_cv::serve_with(listener, config, rt, async {
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
            _container: container,
        },
        pool,
    )
}

async fn admin_url() -> (String, Option<ContainerAsync<GenericImage>>) {
    if let Ok(url) = std::env::var("TBD_TEST_DATABASE_URL")
        && !url.trim().is_empty()
    {
        return (url, None);
    }
    let container = GenericImage::new(IMAGE.0, IMAGE.1)
        .with_exposed_port(5432.tcp())
        .with_wait_for(WaitFor::message_on_stderr(
            "database system is ready to accept connections",
        ))
        .with_env_var("POSTGRES_USER", "test")
        .with_env_var("POSTGRES_PASSWORD", "test")
        .with_env_var("POSTGRES_DB", "postgres")
        .start()
        .await
        .unwrap_or_else(|e| {
            panic!(
                "these tests need Docker (to start {}:{}) or TBD_TEST_DATABASE_URL: {e}",
                IMAGE.0, IMAGE.1
            )
        });
    let port = container.get_host_port_ipv4(5432).await.unwrap();
    let host = container.get_host().await.unwrap();
    (
        format!("postgres://test:test@{host}:{port}/postgres?sslmode=disable"),
        Some(container),
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

    pub async fn client(&self) -> CvServiceClient<Channel> {
        CvServiceClient::new(self.channel().await)
    }

    pub async fn health(&self) -> HealthClient<Channel> {
        HealthClient::new(self.channel().await)
    }
}

/// A person, as Envoy's verified payload describes them.
#[derive(Debug, Clone)]
pub struct Person {
    pub subject: &'static str,
    pub email: Option<&'static str>,
    pub name: Option<&'static str>,
    pub role: Option<&'static str>,
}

pub const OWNER: Person = Person {
    subject: "owner-1",
    email: Some("nevio@inorbit.hr"),
    name: Some("Nevio Vesic"),
    role: Some("admin"),
};

pub const VISITOR: Person = Person {
    subject: "visitor-1",
    email: Some("rita@example.org"),
    name: Some("Rita Reader"),
    role: Some("viewer"),
};

pub const NAMELESS: Person = Person {
    subject: "machine-1",
    email: None,
    name: None,
    role: None,
};

/// A request as `who`, with the payload Envoy would have forwarded.
pub fn as_caller<T>(who: &Person, message: T) -> Request<T> {
    let mut claims = serde_json::json!({ "sub": who.subject, "aud": ["tbd-ui"] });
    if let Some(e) = who.email {
        claims["email"] = e.into();
    }
    if let Some(n) = who.name {
        claims["name"] = n.into();
    }
    if let Some(r) = who.role {
        claims["role"] = r.into();
    }
    let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
    let mut request = Request::new(message);
    request
        .metadata_mut()
        .insert("x-jwt-payload", payload.parse().unwrap());
    request
        .metadata_mut()
        .insert("user-agent", "test-agent/1".parse().unwrap());
    request
        .metadata_mut()
        .insert("x-forwarded-for", "203.0.113.7, 10.0.0.1".parse().unwrap());
    request
}
