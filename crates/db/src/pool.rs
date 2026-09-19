//! The Postgres pool and the project's migrations.

use std::time::Duration;

use sqlx::{PgPool, postgres::PgPoolOptions};

use crate::error::{DbError, map_err};

/// Every migration in `/migrations`, embedded at compile time.
///
/// Per project, not per service: foreign keys cross service boundaries, so one
/// ordered set and one database. Services own schemas, not migration
/// directories.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!("../../migrations");

/// How many migrations this binary was built with. A service compares it with
/// what the database reports and refuses to start if the database is behind,
/// so a half-migrated cluster fails loudly instead of serving wrong answers.
#[must_use]
pub fn expected_migrations() -> usize {
    MIGRATOR.iter().count()
}

/// Pool settings.
#[derive(Debug, Clone)]
pub struct PgOptions {
    /// Connection URL. A credential: never logged, never serialised.
    pub url: String,
    /// Pool size.
    pub max_connections: u32,
    /// How long a request waits for a connection.
    pub acquire_timeout: Duration,
}

impl Default for PgOptions {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 10,
            acquire_timeout: Duration::from_secs(5),
        }
    }
}

/// Open a lazy pool: the first query connects.
///
/// Lazy on purpose. A service starts even while the database is down and
/// reports it through readiness, rather than crash-looping before it can say
/// why.
///
/// # Errors
/// The URL does not parse.
pub fn connect_lazy(options: &PgOptions) -> Result<PgPool, DbError> {
    if options.url.is_empty() {
        return Err(DbError::Invalid {
            field: "database_url",
            reason: "empty".into(),
        });
    }
    PgPoolOptions::new()
        .max_connections(options.max_connections)
        .acquire_timeout(options.acquire_timeout)
        .connect_lazy(&options.url)
        .map_err(map_err)
}

/// Run every migration under sqlx's advisory lock.
///
/// Safe to call concurrently -- the lock serialises them -- but it is not the
/// normal path. Migrations run once, from `tbd migrate`, not on service start:
/// `migrate_on_start` across several services racing on a shared schema is how
/// a cluster ends up half-migrated.
///
/// # Errors
/// The database cannot be reached, a migration fails, or one was edited after
/// it had been applied.
pub async fn migrate(pool: &PgPool) -> Result<(), DbError> {
    // sqlx creates `_sqlx_migrations` unqualified, so it lands wherever
    // search_path points. On a database whose role name matches a schema --
    // `ledger` here -- `"$user", public` puts it in that schema, the applied
    // check looks in `public`, finds nothing, and every migration runs a second
    // time into the wrong schema.
    //
    // The pin has to be on the connection that does the migrating. Setting it
    // through the pool takes *a* connection and hands back another, so acquire
    // one explicitly and keep it for both.
    let mut conn = pool.acquire().await.map_err(map_err)?;
    sqlx::query("set search_path to public")
        .execute(&mut *conn)
        .await
        .map_err(map_err)?;
    MIGRATOR
        .run(&mut *conn)
        .await
        .map_err(|e| DbError::Internal(format!("migrate: {e}")))
}

/// How many migrations the database reports as applied.
///
/// A database that has never been migrated has no tracking table; that is zero,
/// not an error, so a fresh database gives a clear "run `tbd migrate`" rather
/// than a driver error about a missing relation.
///
/// # Errors
/// The database cannot be reached.
pub async fn applied_migrations(pool: &PgPool) -> Result<usize, DbError> {
    // Explicitly `public`, matching where `migrate` pins it. Asking
    // `to_regclass('_sqlx_migrations')` would resolve through search_path and
    // could find a stray copy in another schema, which is the failure this
    // whole pair exists to avoid.
    let (exists,): (bool,) =
        sqlx::query_as("select to_regclass('public._sqlx_migrations') is not null")
            .fetch_one(pool)
            .await
            .map_err(map_err)?;
    if !exists {
        return Ok(0);
    }
    let (count,): (i64,) =
        sqlx::query_as("select count(*) from public._sqlx_migrations where success")
            .fetch_one(pool)
            .await
            .map_err(map_err)?;
    usize::try_from(count).map_err(|_| DbError::Internal("negative migration count".into()))
}

/// Refuse to run against a database that is behind this binary.
///
/// # Errors
/// The database is behind, or its state cannot be read.
pub async fn require_current_schema(pool: &PgPool) -> Result<(), DbError> {
    let applied = applied_migrations(pool).await?;
    let expected = expected_migrations();
    if applied < expected {
        return Err(DbError::Internal(format!(
            "database has {applied} migrations applied, this binary expects {expected}; \
             run `tbd migrate` before starting"
        )));
    }
    Ok(())
}

/// Idle and in-use connection counts, for the pool gauges.
#[must_use]
pub fn pool_counts(pool: &PgPool) -> (u32, u32) {
    let size = pool.size();
    let idle = u32::try_from(pool.num_idle()).unwrap_or(u32::MAX);
    (idle, size.saturating_sub(idle))
}

#[cfg(test)]
mod tests {
    use super::{PgOptions, connect_lazy, expected_migrations};

    #[test]
    fn an_empty_url_is_refused_before_it_reaches_the_driver() {
        let e = connect_lazy(&PgOptions::default()).unwrap_err();
        assert!(e.to_string().contains("database_url"), "{e}");
    }

    #[test]
    fn a_malformed_url_is_refused() {
        let options = PgOptions {
            url: "not a url".into(),
            ..PgOptions::default()
        };
        assert!(connect_lazy(&options).is_err());
    }

    #[test]
    fn the_migrations_are_embedded() {
        assert!(
            expected_migrations() > 0,
            "no migrations embedded; is /migrations empty?"
        );
    }
}
