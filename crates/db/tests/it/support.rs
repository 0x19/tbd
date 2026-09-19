//! A migrated database per test.
//!
//! The server comes from `TBD_TEST_DATABASE_URL` (an admin URL on a running
//! Postgres, the CI `services:` block) or, when unset, from a container this
//! test starts through Docker. With neither, the test **fails and says so**: an
//! access-control test that silently passes is worse than no test.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::time::Duration;

use sqlx::PgPool;
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

const IMAGE: (&str, &str) = ("pgvector/pgvector", "0.8.6-pg17-trixie");

/// A pool on its own database, plus the container serving it when one was
/// started (dropped with the test, removing the container).
pub struct Db {
    pub pool: PgPool,
    _container: Option<ContainerAsync<GenericImage>>,
}

impl std::ops::Deref for Db {
    type Target = PgPool;

    fn deref(&self) -> &PgPool {
        &self.pool
    }
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

async fn connect(url: &str) -> PgPool {
    let deadline = tokio::time::Instant::now() + Duration::from_secs(60);
    loop {
        match PgPool::connect(url).await {
            Ok(pool) => return pool,
            Err(_) if tokio::time::Instant::now() < deadline => {
                tokio::time::sleep(Duration::from_millis(250)).await;
            }
            Err(e) => panic!("connect {url}: {e}"),
        }
    }
}

/// A fresh, migrated database.
pub async fn db() -> Db {
    let (admin_url, container) = admin_url().await;
    let admin = connect(&admin_url).await;
    let database = format!("tbd_test_{}", uuid::Uuid::now_v7().simple());
    sqlx::query(sqlx::AssertSqlSafe(format!("create database {database}")))
        .execute(&admin)
        .await
        .unwrap();
    let options: sqlx::postgres::PgConnectOptions = admin_url.parse().unwrap();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(16)
        .acquire_timeout(Duration::from_secs(10))
        .connect_lazy_with(options.database(&database));
    tbd_db::migrate(&pool).await.unwrap();
    drop(admin);
    Db {
        pool,
        _container: container,
    }
}
