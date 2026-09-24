//! A migrated database per test, on the shared test Postgres
//! (`tbd_db::testing`: `TBD_TEST_DATABASE_URL`, or the one reusable container).
//! With neither, the test **fails and says so**: an access-control test that
//! silently passes is worse than no test.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::time::Duration;

use sqlx::PgPool;

/// A pool on its own database.
pub struct Db {
    pub pool: PgPool,
}

impl std::ops::Deref for Db {
    type Target = PgPool;

    fn deref(&self) -> &PgPool {
        &self.pool
    }
}

/// A fresh, migrated database.
pub async fn db() -> Db {
    let url = tbd_db::testing::fresh_database(tbd_db::testing::ENV, "tbd_test").await;
    let options: sqlx::postgres::PgConnectOptions = url.parse().unwrap();
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(8)
        .acquire_timeout(Duration::from_secs(10))
        .connect_lazy_with(options);
    tbd_db::migrate(&pool).await.unwrap();
    Db { pool }
}
