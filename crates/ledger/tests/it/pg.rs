//! The Postgres store against a real server: the conformance suite, plus
//! what only Postgres can prove (the cascade leaves zero rows).
//!
//! Each test gets its own database. The server comes from
//! `LEDGER_TEST_DATABASE_URL` (an admin URL on a running Postgres, the CI
//! `services:` block) or, when unset, from a container this test starts
//! through Docker. With neither, the test fails and says so: a store test
//! never silently passes.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use sqlx::PgPool;
use tbd_ledger::store::{
    Source, Store, SubjectId,
    pg::{PgOptions, PgStore},
};

use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

use crate::conformance;

const IMAGE: (&str, &str) = ("pgvector/pgvector", "0.8.6-pg17-trixie");

/// A store on its own database, and the container that serves it when one
/// was started (dropped with the test, removing the container).
pub struct PgTest {
    pub store: PgStore,
    _container: Option<ContainerAsync<GenericImage>>,
}

impl std::ops::Deref for PgTest {
    type Target = PgStore;

    fn deref(&self) -> &PgStore {
        &self.store
    }
}

/// An admin URL: the environment's, or a fresh container's.
async fn admin_url() -> (String, Option<ContainerAsync<GenericImage>>) {
    if let Ok(url) = std::env::var("LEDGER_TEST_DATABASE_URL")
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
                "the Postgres store tests need Docker (to start {}:{}) or LEDGER_TEST_DATABASE_URL: {e}",
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

/// Connect to `url`, retrying while the server finishes starting.
async fn connect(url: &str) -> PgPool {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        match PgPool::connect(url).await {
            Ok(pool) => return pool,
            Err(e) if tokio::time::Instant::now() < deadline => {
                tracing::debug!(error = %e, "postgres not ready yet");
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(e) => panic!("connect {url}: {e}"),
        }
    }
}

/// A migrated store on a fresh database.
pub async fn store() -> PgTest {
    let (admin_url, container) = admin_url().await;
    let admin = connect(&admin_url).await;
    let database = format!("ledger_test_{}", uuid::Uuid::now_v7().simple());
    sqlx::query(sqlx::AssertSqlSafe(format!("create database {database}")))
        .execute(&admin)
        .await
        .unwrap();
    let options: sqlx::postgres::PgConnectOptions = admin_url.parse().unwrap();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(10))
        .connect_lazy_with(options.database(&database));
    let store = PgStore::from_pool(pool);
    store.migrate().await.unwrap();
    drop(admin);
    PgTest {
        store,
        _container: container,
    }
}

crate::conformance_suite!(pg, async { crate::pg::store().await });

/// The one thing only Postgres can prove: after the cascade, no table holds a
/// row for the subject except `erasures`.
#[tokio::test]
async fn cascade_leaves_zero_rows_except_the_erasure() {
    let t = store().await;
    let a: SubjectId = uuid::Uuid::now_v7();
    let b: SubjectId = uuid::Uuid::now_v7();
    t.append(
        a,
        conformance::fact("profile.name", Source::Declared, &serde_json::json!("A")),
    )
    .await
    .unwrap();
    t.append(
        b,
        conformance::fact("profile.name", Source::Declared, &serde_json::json!("B")),
    )
    .await
    .unwrap();
    let mut rel = conformance::fact(
        "relations.match.m",
        Source::Declared,
        &serde_json::json!("m"),
    );
    rel.counterparty_id = Some(a);
    t.append(b, rel).await.unwrap();
    let mut keyed = conformance::fact("profile.bio", Source::Declared, &serde_json::json!("x"));
    keyed.idempotency_key = Some("k".into());
    t.append(a, keyed).await.unwrap();
    t.request_erasure(a).await.unwrap();
    let executed = t.execute_due_erasures(Duration::ZERO, 10).await.unwrap();
    assert_eq!(executed.len(), 1);

    for table in [
        "subjects",
        "facts",
        "facts_current",
        "outbox",
        "idempotency",
    ] {
        let col = if table == "subjects" {
            "id"
        } else {
            "subject_id"
        };
        let (n,): (i64,) = sqlx::query_as(sqlx::AssertSqlSafe(format!(
            "select count(*) from {table} where {col} = $1"
        )))
        .bind(a)
        .fetch_one(t.store.pool())
        .await
        .unwrap();
        assert_eq!(n, 0, "{table} still holds rows for the erased subject");
    }
    let (n,): (i64,) = sqlx::query_as("select count(*) from facts where counterparty_id = $1")
        .bind(a)
        .fetch_one(t.store.pool())
        .await
        .unwrap();
    assert_eq!(n, 0, "no fact anywhere names the erased subject");
    let (n,): (i64,) = sqlx::query_as(
        "select count(*) from erasures where subject_id = $1 and executed_at is not null",
    )
    .bind(a)
    .fetch_one(t.store.pool())
    .await
    .unwrap();
    assert_eq!(n, 1, "the erasure record survives");
}

/// Migrations are idempotent and checksummed: running them twice is a no-op.
#[tokio::test]
async fn migrations_run_twice_are_a_no_op() {
    let t = store().await;
    t.migrate().await.unwrap();
    let (n,): (i64,) = sqlx::query_as("select count(*) from _sqlx_migrations")
        .fetch_one(t.store.pool())
        .await
        .unwrap();
    assert_eq!(n, 1);
}

/// Readiness: `ping` succeeds on a live pool and fails once the database is
/// unreachable.
#[tokio::test]
async fn stats_count_the_backlog_and_the_tables() {
    let t = store().await;
    let s = uuid::Uuid::now_v7();
    t.append(s, conformance::declared("profile.name", "x"))
        .await
        .unwrap();
    t.request_erasure(s).await.unwrap();
    let stats = t.stats(Duration::from_hours(168)).await.unwrap();
    assert_eq!(stats.outbox_pending, 1, "{stats:?}");
    assert!(stats.outbox_oldest_secs >= 0.0);
    assert_eq!((stats.erasures_pending, stats.erasures_due), (1, 0));
    let names: Vec<&str> = stats.tables.iter().map(|t| t.name.as_str()).collect();
    let mut expected = tbd_ledger::store::pg::LEDGER_TABLES.to_vec();
    expected.sort_unstable();
    assert_eq!(names, expected);
    assert!(stats.tables.iter().all(|t| t.bytes > 0));
    // Past the window the same erasure is due.
    let stats = t.stats(Duration::ZERO).await.unwrap();
    assert_eq!((stats.erasures_pending, stats.erasures_due), (1, 1));
}

#[tokio::test]
async fn ping_reports_the_database() {
    let t = store().await;
    t.ping().await.unwrap();
    let dead = PgStore::connect_lazy(&PgOptions {
        url: "postgres://nobody:nothing@127.0.0.1:1/none?sslmode=disable".into(),
        max_connections: 1,
        acquire_timeout: Duration::from_millis(500),
    })
    .unwrap();
    assert!(dead.ping().await.is_err());
}
