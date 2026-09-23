//! The one Postgres for integration tests, and a fresh database on it.
//!
//! Every test used to start its own container and drop it at the end. A run
//! that ends on time cleans up after itself; a run that is killed halfway
//! (a tool's timeout, nextest's slow-test kill, a Ctrl-C) never runs a `Drop`,
//! and each such run left its containers behind until the Docker bridge was
//! full and no test could start at all (2026-09-23: a thousand of them).
//!
//! So the container is one, named, and reused: `tbd-test-postgres`, started
//! by whichever test gets there first, found by every one after it, on this
//! run and the next, and never removed by a test. There is nothing to leak.
//! Each test still gets its own database, named after a v7 UUID, so the name
//! carries its birth time; databases older than an hour are dropped by the
//! next test that comes along, which keeps the container from growing. A
//! killed test's database lives at most an hour longer than the test did.
//!
//! `TBD_TEST_DATABASE_URL` (or the variable a caller names) is an admin URL
//! on a Postgres somebody else runs, the CI job's `services:` block; with it
//! set, Docker is never touched. `mise run test:db:reset` removes the shared
//! container when it has to go (an image bump, a corrupted volume).
#![allow(clippy::unwrap_used, clippy::expect_used, clippy::missing_panics_doc)]

use std::time::Duration;

use sqlx::PgPool;
use testcontainers::{
    GenericImage, ImageExt, ReuseDirective,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

/// The image, the same one the ledger's local database runs on.
pub const IMAGE: (&str, &str) = ("pgvector/pgvector", "0.8.6-pg17-trixie");
/// The container's name: what `docker ps` shows and `mise run test:db:reset` removes.
pub const CONTAINER: &str = "tbd-test-postgres";
/// The variable that names a Postgres to use instead of the container.
pub const ENV: &str = "TBD_TEST_DATABASE_URL";
/// A test database older than this is somebody's leftover.
const STALE_AFTER: Duration = Duration::from_hours(1);

/// An admin URL on the shared server: `env` if it is set, else the container,
/// started if it is not running yet. Panics, saying so, when there is neither.
pub async fn admin_url(env: &str) -> String {
    if let Ok(url) = std::env::var(env)
        && !url.trim().is_empty()
    {
        return url;
    }
    let mut last = None;
    // Two processes can race to create the one container; the loser sees a
    // name conflict and finds the winner's on the next try.
    for attempt in 0..6u32 {
        let started = GenericImage::new(IMAGE.0, IMAGE.1)
            .with_exposed_port(5432.tcp())
            .with_wait_for(WaitFor::message_on_stderr(
                "database system is ready to accept connections",
            ))
            .with_env_var("POSTGRES_USER", "test")
            .with_env_var("POSTGRES_PASSWORD", "test")
            .with_env_var("POSTGRES_DB", "postgres")
            // Many test processes share it, each with a small pool; and
            // durability buys a test nothing.
            .with_cmd([
                "postgres",
                "-c",
                "max_connections=1000",
                "-c",
                "fsync=off",
                "-c",
                "synchronous_commit=off",
                "-c",
                "full_page_writes=off",
            ])
            .with_container_name(CONTAINER)
            .with_reuse(ReuseDirective::Always)
            .start()
            .await;
        match started {
            Ok(container) => {
                let port = container.get_host_port_ipv4(5432).await.unwrap();
                let host = container.get_host().await.unwrap();
                let url = format!("postgres://test:test@{host}:{port}/postgres?sslmode=disable");
                // A reused container is handed back without the readiness
                // wait, and one that was stopped is started again first.
                ready(&url).await;
                return url;
            }
            Err(e) => {
                last = Some(e);
                tokio::time::sleep(Duration::from_millis(300 * u64::from(attempt + 1))).await;
            }
        }
    }
    panic!(
        "these tests need Docker (to run {}:{} as `{CONTAINER}`) or {env}: {}",
        IMAGE.0,
        IMAGE.1,
        last.unwrap()
    );
}

/// A fresh database on the shared server, migrated by nobody: its URL.
///
/// `prefix` names the caller (`finance_test`), and the rest of the name is a
/// v7 UUID, which is how stale ones are told apart and dropped here first.
pub async fn fresh_database(env: &str, prefix: &str) -> String {
    let admin_url = admin_url(env).await;
    let admin = connect(&admin_url).await;
    prune(&admin).await;
    let database = format!("{prefix}_{}", uuid::Uuid::now_v7().simple());
    sqlx::query(sqlx::AssertSqlSafe(format!("create database {database}")))
        .execute(&admin)
        .await
        .unwrap();
    admin.close().await;
    // A URL, not a pool: services take one from config, and a pool is one
    // parse away.
    let mut url = url::Url::parse(&admin_url).unwrap();
    url.set_path(&database);
    url.to_string()
}

/// Wait for the server behind `url` to answer, up to a minute.
async fn ready(url: &str) {
    connect(url).await.close().await;
}

async fn connect(url: &str) -> PgPool {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        match sqlx::postgres::PgPoolOptions::new()
            .max_connections(2)
            .connect(url)
            .await
        {
            Ok(pool) => return pool,
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(e) => panic!("connect {url}: {e}"),
        }
    }
}

/// Drop every `<prefix>_<v7 uuid>` database older than [`STALE_AFTER`].
/// Best effort: two tests pruning at once both succeed, and a database in use
/// is left alone (it is younger than an hour anyway).
async fn prune(admin: &PgPool) {
    let names: Vec<String> = sqlx::query_scalar(
        "select datname from pg_database where datname ~ '^[a-z]+_test_[0-9a-f]{32}$'",
    )
    .fetch_all(admin)
    .await
    .unwrap_or_default();
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    for name in names {
        let Some(hex) = name.rsplit('_').next() else {
            continue;
        };
        let Some(born) = uuid::Uuid::parse_str(hex)
            .ok()
            .and_then(|u| u.get_timestamp())
            .map(|t| t.to_unix().0)
        else {
            continue;
        };
        if now.saturating_sub(born) < STALE_AFTER.as_secs() {
            continue;
        }
        let _ = sqlx::query(sqlx::AssertSqlSafe(format!(
            "drop database if exists {name} with (force)"
        )))
        .execute(admin)
        .await;
    }
}
