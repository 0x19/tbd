//! The Postgres store: the source of truth. Every write is one transaction
//! that also maintains `facts_current` and the outbox; every erasure is one
//! transaction whose commit proves zero rows.
//!
//! Locking: a subject's writers serialise on `SELECT ... FOR NO KEY UPDATE`
//! of its `subjects` row, and every `recorded_at` is `clock_timestamp()`
//! taken after that lock, so `(recorded_at, id)` strictly increases per
//! subject. Lock order everywhere: the `erasures` row, then survivors'
//! `subjects` rows in id order, then the subject's own row. `FOR NO KEY
//! UPDATE` does not conflict with the `KEY SHARE` an insert referencing the
//! subject takes, so two appends naming each other cannot deadlock.

use std::time::Duration;

use chrono::{DateTime, Utc};
use sqlx::{PgPool, Postgres, Transaction, postgres::PgPoolOptions};
use uuid::Uuid;

/// The ledger's tables, for [`PgStore::stats`].
pub const LEDGER_TABLES: &[&str] = &[
    "subjects",
    "facts",
    "facts_current",
    "erasures",
    "outbox",
    "idempotency",
];

/// What [`PgStore::stats`] samples.
#[derive(Debug, Clone, PartialEq)]
pub struct PgStats {
    /// Outbox events not yet published.
    pub outbox_pending: u64,
    /// Age in seconds of the oldest unpublished event (0 when none).
    pub outbox_oldest_secs: f64,
    /// Erasures inside their grace window.
    pub erasures_pending: u64,
    /// Erasures past it, waiting for the sweeper.
    pub erasures_due: u64,
    /// Per table.
    pub tables: Vec<TableStats>,
}

/// One table's estimate and size.
#[derive(Debug, Clone, PartialEq)]
pub struct TableStats {
    /// Table name.
    pub name: String,
    /// The planner's row estimate (exact after `analyze`).
    pub rows: f64,
    /// Bytes on disk, indexes and TOAST included.
    pub bytes: u64,
}

/// A count as a gauge value; counts here never reach 2^53.
#[must_use]
#[allow(clippy::cast_precision_loss)]
pub fn gauge_value(n: u64) -> f64 {
    n as f64
}

fn count(n: i64) -> u64 {
    u64::try_from(n).unwrap_or(0)
}

use super::{
    Appended, Envelope, Erasure, ErasureExecuted, EventKind, Fact, FactId, NewFact, OutboxEvent,
    Page, Query, ScopeId, Source, Store, StoreError, StoreKind, Subject, SubjectId, fingerprint,
    registry::{Registry, ShapeOnly},
    sql::PathParams,
    validate,
};

/// The embedded migrations (`crates/ledger/migrations`).
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();

/// Pool settings.
#[derive(Debug, Clone)]
pub struct PgOptions {
    /// Connection URL.
    pub url: String,
    /// Pool size.
    pub max_connections: u32,
    /// How long a request waits for a connection.
    pub acquire_timeout: Duration,
}

/// The store.
#[derive(Clone)]
pub struct PgStore {
    pool: PgPool,
    registry: std::sync::Arc<dyn Registry>,
}

impl std::fmt::Debug for PgStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PgStore").finish_non_exhaustive()
    }
}

impl PgStore {
    /// Open a lazy pool: the first query connects. The store starts even if
    /// the database is down; readiness reports it.
    ///
    /// # Errors
    /// The URL does not parse.
    pub fn connect_lazy(options: &PgOptions) -> Result<Self, StoreError> {
        Self::connect_lazy_with(options, std::sync::Arc::new(ShapeOnly))
    }

    /// Open a lazy pool with an explicit registry.
    ///
    /// # Errors
    /// The URL does not parse.
    pub fn connect_lazy_with(
        options: &PgOptions,
        registry: std::sync::Arc<dyn Registry>,
    ) -> Result<Self, StoreError> {
        let pool = PgPoolOptions::new()
            .max_connections(options.max_connections)
            .acquire_timeout(options.acquire_timeout)
            .connect_lazy(&options.url)
            .map_err(map_err)?;
        Ok(Self { pool, registry })
    }

    /// Wrap an existing pool.
    #[must_use]
    pub fn from_pool(pool: PgPool) -> Self {
        Self {
            pool,
            registry: std::sync::Arc::new(ShapeOnly),
        }
    }

    /// The pool, for embedders that share it.
    #[must_use]
    pub fn pool(&self) -> &PgPool {
        &self.pool
    }

    /// Run the embedded migrations under sqlx's advisory lock.
    ///
    /// # Errors
    /// The database cannot be reached, or a migration fails or was edited
    /// after it was applied.
    pub async fn migrate(&self) -> Result<(), StoreError> {
        MIGRATOR
            .run(&self.pool)
            .await
            .map_err(|e| StoreError::Internal(format!("migrate: {e}")))
    }

    /// Idle and in-use connection counts, for the pool gauges.
    #[must_use]
    pub fn pool_counts(&self) -> (u32, u32) {
        let size = self.pool.size();
        let idle = u32::try_from(self.pool.num_idle()).unwrap_or(u32::MAX);
        (idle, size.saturating_sub(idle))
    }

    /// The pool's configured maximum.
    #[must_use]
    pub fn max_connections(&self) -> u32 {
        self.pool.options().get_max_connections()
    }

    /// The database's health counters for the gauges: outbox backlog and the
    /// age of its oldest event, erasures inside and past `grace`, and the
    /// planner's row estimate and on-disk size per table. Three cheap queries;
    /// none touches a fact.
    ///
    /// # Errors
    /// The database could not be reached.
    pub async fn stats(&self, grace: Duration) -> Result<PgStats, StoreError> {
        #[derive(sqlx::FromRow)]
        struct Outbox {
            pending: i64,
            oldest: f64,
        }
        #[derive(sqlx::FromRow)]
        struct Erasures {
            pending: i64,
            due: i64,
        }
        #[derive(sqlx::FromRow)]
        struct Table {
            name: String,
            rows: f64,
            bytes: i64,
        }
        let outbox: Outbox = sqlx::query_as(
            "select count(*)::bigint as pending, \
             coalesce(extract(epoch from (clock_timestamp() - min(recorded_at))), 0)::float8 as oldest \
             from outbox where published_at is null",
        )
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;
        let erasures: Erasures = sqlx::query_as(
            "select count(*) filter (where executed_at is null and cancelled_at is null)::bigint as pending, \
             count(*) filter (where executed_at is null and cancelled_at is null \
                              and requested_at + make_interval(secs => $1) <= clock_timestamp())::bigint as due \
             from erasures",
        )
        .bind(grace.as_secs_f64())
        .fetch_one(&self.pool)
        .await
        .map_err(map_err)?;
        let tables: Vec<Table> = sqlx::query_as(
            "select relname::text as name, greatest(reltuples, 0)::float8 as rows, \
             pg_total_relation_size(oid)::bigint as bytes \
             from pg_class where relkind = 'r' and relname = any($1) order by relname",
        )
        .bind(
            LEDGER_TABLES
                .iter()
                .map(|s| (*s).to_owned())
                .collect::<Vec<String>>(),
        )
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)?;
        Ok(PgStats {
            outbox_pending: count(outbox.pending),
            outbox_oldest_secs: outbox.oldest,
            erasures_pending: count(erasures.pending),
            erasures_due: count(erasures.due),
            tables: tables
                .into_iter()
                .map(|t| TableStats {
                    name: t.name,
                    rows: t.rows,
                    bytes: count(t.bytes),
                })
                .collect(),
        })
    }

    async fn begin(&self) -> Result<Transaction<'static, Postgres>, StoreError> {
        self.pool.begin().await.map_err(map_err)
    }

    /// Lock the subject row for writing and check it is open.
    async fn lock_open(
        tx: &mut Transaction<'static, Postgres>,
        subject: SubjectId,
    ) -> Result<(), StoreError> {
        let row: Option<(Option<DateTime<Utc>>,)> =
            sqlx::query_as("select erased_at from subjects where id = $1 for no key update")
                .bind(subject)
                .fetch_optional(&mut **tx)
                .await
                .map_err(map_err)?;
        match row {
            None => {
                if executed(tx, subject).await? {
                    Err(StoreError::Erased)
                } else {
                    Err(StoreError::NotFound { what: "subject" })
                }
            }
            Some((Some(_),)) => Err(StoreError::Erased),
            Some((None,)) => Ok(()),
        }
    }

    /// Reads: the subject must exist and be open (no lock).
    async fn check_open(&self, subject: SubjectId) -> Result<(), StoreError> {
        let row: Option<(Option<DateTime<Utc>>,)> =
            sqlx::query_as("select erased_at from subjects where id = $1")
                .bind(subject)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_err)?;
        match row {
            None => {
                let executed: Option<(i64,)> = sqlx::query_as(
                    "select 1::bigint from erasures where subject_id = $1 and executed_at is not null limit 1",
                )
                .bind(subject)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_err)?;
                if executed.is_some() {
                    Err(StoreError::Erased)
                } else {
                    Err(StoreError::NotFound { what: "subject" })
                }
            }
            Some((Some(_),)) => Err(StoreError::Erased),
            Some((None,)) => Ok(()),
        }
    }

    async fn insert_tombstone(
        tx: &mut Transaction<'static, Postgres>,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: &Envelope,
        consent: &[ScopeId],
        kind: EventKind,
    ) -> Result<Fact, StoreError> {
        let row: FactRow = sqlx::query_as(
            "insert into facts (subject_id, path, source, value, origin, envelope_version, observed_at, recorded_at, consent)
             values ($1, $2, $3, null, $4, $5, clock_timestamp(), clock_timestamp(), $6)
             returning subject_id, id, path, source, value, origin, envelope_version, confidence, counterparty_id,
                       observed_at, recorded_at, expires_at, consent, stub",
        )
        .bind(subject)
        .bind(path)
        .bind(source.as_str())
        .bind(&origin.bytes)
        .bind(i16::try_from(origin.version).unwrap_or(i16::MAX))
        .bind(scope_strings(consent))
        .fetch_one(&mut **tx)
        .await
        .map_err(map_err)?;
        let fact = row.into_fact()?;
        upsert_current(tx, &fact).await?;
        insert_event(tx, kind, &fact).await?;
        Ok(fact)
    }
}

async fn executed(
    tx: &mut Transaction<'static, Postgres>,
    subject: SubjectId,
) -> Result<bool, StoreError> {
    let row: Option<(i64,)> = sqlx::query_as(
        "select 1::bigint from erasures where subject_id = $1 and executed_at is not null limit 1",
    )
    .bind(subject)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_err)?;
    Ok(row.is_some())
}

/// The counterparty must exist and be open.
async fn check_counterparty(
    tx: &mut Transaction<'static, Postgres>,
    cp: SubjectId,
) -> Result<(), StoreError> {
    let row: Option<(Option<DateTime<Utc>>,)> =
        sqlx::query_as("select erased_at from subjects where id = $1")
            .bind(cp)
            .fetch_optional(&mut **tx)
            .await
            .map_err(map_err)?;
    match row {
        None => {
            if executed(tx, cp).await? {
                Err(StoreError::Erased)
            } else {
                Err(StoreError::NotFound {
                    what: "counterparty",
                })
            }
        }
        Some((Some(_),)) => Err(StoreError::Erased),
        Some((None,)) => Ok(()),
    }
}

/// The fact an idempotency key replays, if the key was seen: `Conflict` when
/// the key was used for different content or its fact was retracted since.
async fn replay(
    tx: &mut Transaction<'static, Postgres>,
    subject: SubjectId,
    key: &str,
    print: &[u8],
) -> Result<Option<Fact>, StoreError> {
    let hit: Option<(i64, Vec<u8>)> = sqlx::query_as(
        "select fact_id, fingerprint from idempotency where subject_id = $1 and key = $2",
    )
    .bind(subject)
    .bind(key)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_err)?;
    let Some((fact_id, stored)) = hit else {
        return Ok(None);
    };
    if stored != print {
        return Err(StoreError::Conflict {
            reason: "idempotency key reused with a different fact".into(),
        });
    }
    let existing: Option<FactRow> = sqlx::query_as(sqlx::AssertSqlSafe(format!(
        "select {FACT_COLUMNS} from facts f where f.subject_id = $1 and f.id = $2"
    )))
    .bind(subject)
    .bind(fact_id)
    .fetch_optional(&mut **tx)
    .await
    .map_err(map_err)?;
    match existing {
        Some(row) => Ok(Some(row.into_fact()?)),
        None => Err(StoreError::Conflict {
            reason: "the fact recorded under this key was retracted".into(),
        }),
    }
}

/// `(subject, path, source)` of a fact the cascade removed.
type SurvivorKey = (SubjectId, String, String);

/// A fact deleted because it named the erased subject as counterparty.
#[derive(sqlx::FromRow)]
struct RemovedRow {
    subject_id: SubjectId,
    path: String,
    source: String,
    consent: Vec<String>,
    recorded_at: DateTime<Utc>,
    id: i64,
}

/// Delete every fact anywhere that names `sid` as counterparty and append one
/// tombstone per surviving `(subject, path, source)`, with the latest deleted
/// row's consent and no counterparty (the one place a tombstone has none).
async fn tombstone_survivors(
    tx: &mut Transaction<'static, Postgres>,
    sid: SubjectId,
) -> Result<u32, StoreError> {
    let removed: Vec<RemovedRow> = sqlx::query_as(
        "delete from facts where counterparty_id = $1
         returning subject_id, path, source, consent, recorded_at, id",
    )
    .bind(sid)
    .fetch_all(&mut **tx)
    .await
    .map_err(map_err)?;
    // The latest deleted row per (subject, path, source) lends the tombstone its consent.
    let mut latest: std::collections::BTreeMap<SurvivorKey, RemovedRow> =
        std::collections::BTreeMap::new();
    for row in removed {
        let key = (row.subject_id, row.path.clone(), row.source.clone());
        let newer = latest
            .get(&key)
            .is_none_or(|have| (row.recorded_at, row.id) > (have.recorded_at, have.id));
        if newer {
            latest.insert(key, row);
        }
    }
    let origin = Envelope::plaintext(&serde_json::json!({"cause": "counterparty_erased"}));
    let mut tombstoned = 0u32;
    for ((other, path, source), row) in latest {
        let source: Source = source.parse()?;
        PgStore::insert_tombstone(
            tx,
            other,
            &path,
            source,
            &origin,
            &scopes_of(row.consent),
            EventKind::FactRetracted,
        )
        .await?;
        tombstoned += 1;
    }
    Ok(tombstoned)
}

async fn upsert_current(
    tx: &mut Transaction<'static, Postgres>,
    fact: &Fact,
) -> Result<(), StoreError> {
    sqlx::query(
        "insert into facts_current (subject_id, path, source, fact_id) values ($1, $2, $3, $4)
         on conflict (subject_id, path, source) do update set fact_id = excluded.fact_id",
    )
    .bind(fact.subject_id)
    .bind(&fact.path)
    .bind(fact.source.as_str())
    .bind(fact.id.0)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    Ok(())
}

async fn insert_event(
    tx: &mut Transaction<'static, Postgres>,
    kind: EventKind,
    fact: &Fact,
) -> Result<(), StoreError> {
    sqlx::query(
        "insert into outbox (event_id, subject_id, kind, fact_id, payload, recorded_at) values ($1, $2, $3, $4, $5, $6)",
    )
    .bind(Uuid::now_v7())
    .bind(fact.subject_id)
    .bind(kind.as_str())
    .bind(fact.id.0)
    .bind(OutboxEvent::fact_payload(fact))
    .bind(fact.recorded_at)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    Ok(())
}

fn scope_strings(consent: &[ScopeId]) -> Vec<String> {
    consent.iter().map(|s| s.0.clone()).collect()
}

fn scopes_of(strings: Vec<String>) -> Vec<ScopeId> {
    strings.into_iter().map(ScopeId).collect()
}

/// Map a driver error onto the store's classes.
fn map_err(e: sqlx::Error) -> StoreError {
    match e {
        sqlx::Error::RowNotFound => StoreError::NotFound { what: "row" },
        sqlx::Error::Database(db) => {
            if db.is_unique_violation() {
                StoreError::Conflict {
                    reason: db.message().to_owned(),
                }
            } else {
                StoreError::Internal(format!("database: {}", db.message()))
            }
        }
        sqlx::Error::PoolTimedOut
        | sqlx::Error::PoolClosed
        | sqlx::Error::WorkerCrashed
        | sqlx::Error::Io(_)
        | sqlx::Error::Tls(_)
        | sqlx::Error::Protocol(_)
        | sqlx::Error::Configuration(_) => StoreError::unavailable(e),
        other => StoreError::Internal(other.to_string()),
    }
}

/// One `facts` row.
#[derive(sqlx::FromRow)]
struct FactRow {
    subject_id: Uuid,
    id: i64,
    path: String,
    source: String,
    value: Option<Vec<u8>>,
    origin: Vec<u8>,
    envelope_version: i16,
    confidence: Option<f32>,
    counterparty_id: Option<Uuid>,
    observed_at: DateTime<Utc>,
    recorded_at: DateTime<Utc>,
    expires_at: Option<DateTime<Utc>>,
    consent: Vec<String>,
    stub: bool,
}

impl FactRow {
    fn into_fact(self) -> Result<Fact, StoreError> {
        let version = u16::try_from(self.envelope_version)
            .map_err(|_| StoreError::Internal("negative envelope version".into()))?;
        Ok(Fact {
            subject_id: self.subject_id,
            id: FactId(self.id),
            path: self.path,
            source: self.source.parse()?,
            value: self.value.map(|bytes| Envelope { version, bytes }),
            origin: Envelope {
                version,
                bytes: self.origin,
            },
            confidence: self.confidence,
            counterparty_id: self.counterparty_id,
            observed_at: self.observed_at,
            recorded_at: self.recorded_at,
            expires_at: self.expires_at,
            consent: scopes_of(self.consent),
            stub: self.stub,
        })
    }
}

const FACT_COLUMNS: &str = "f.subject_id, f.id, f.path, f.source, f.value, f.origin, f.envelope_version, f.confidence, \
     f.counterparty_id, f.observed_at, f.recorded_at, f.expires_at, f.consent, f.stub";

/// The shared filter tail of `current` and `history`: `$2` sources, `$3`
/// scopes, `$4` exact paths, `$5` prefixes, `$6`/`$7` cursor, `$8` limit.
const FILTERS: &str = "and ($2::text[] is null or f.source = any($2))
       and ($3::text[] is null or f.consent && $3)
       and ($4::text[] is null or f.path = any($4) or f.path like any($5))
       and ($6::timestamptz is null or (f.recorded_at, f.id) > ($6, $7))";

struct Filters {
    sources: Option<Vec<String>>,
    scopes: Option<Vec<String>>,
    paths: PathParams,
    cursor_at: Option<DateTime<Utc>>,
    cursor_id: i64,
    limit: i64,
}

impl Filters {
    fn of(query: &Query) -> Result<Self, StoreError> {
        let (cursor_at, cursor_id) = match &query.cursor {
            Some(c) => {
                let (at, id) = c.decode()?;
                (Some(at), id.0)
            }
            None => (None, 0),
        };
        Ok(Self {
            sources: (!query.sources.is_empty()).then(|| {
                query
                    .sources
                    .iter()
                    .map(|s| s.as_str().to_owned())
                    .collect()
            }),
            scopes: query.scopes.as_ref().map(|s| scope_strings(s)),
            paths: PathParams::of(&query.paths),
            cursor_at,
            cursor_id,
            limit: i64::try_from(query.page_size() + 1).unwrap_or(i64::MAX),
        })
    }
}

#[async_trait::async_trait]
impl Store for PgStore {
    fn kind(&self) -> StoreKind {
        StoreKind::Postgres
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self
    }

    async fn ping(&self) -> Result<(), StoreError> {
        let mut conn = self.pool.acquire().await.map_err(map_err)?;
        sqlx::Connection::ping(&mut *conn).await.map_err(map_err)
    }

    async fn subject(&self, id: SubjectId) -> Result<Option<Subject>, StoreError> {
        let row: Option<(DateTime<Utc>, Option<DateTime<Utc>>)> =
            sqlx::query_as("select enrolled_at, erased_at from subjects where id = $1")
                .bind(id)
                .fetch_optional(&self.pool)
                .await
                .map_err(map_err)?;
        Ok(row.map(|(enrolled_at, erased_at)| Subject {
            id,
            enrolled_at,
            erased_at,
        }))
    }

    async fn append(&self, subject: SubjectId, fact: NewFact) -> Result<Appended, StoreError> {
        validate::new_fact(&fact, &*self.registry)?;
        let mut tx = self.begin().await?;
        // A subject that was erased and executed keeps its id out of use.
        if executed(&mut tx, subject).await? {
            return Err(StoreError::Erased);
        }
        sqlx::query("insert into subjects (id) values ($1) on conflict (id) do nothing")
            .bind(subject)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        Self::lock_open(&mut tx, subject).await?;
        if let Some(cp) = fact.counterparty_id {
            check_counterparty(&mut tx, cp).await?;
        }
        let print = fact.idempotency_key.as_ref().map(|_| fingerprint(&fact));
        if let (Some(key), Some(print)) = (&fact.idempotency_key, &print)
            && let Some(replayed) = replay(&mut tx, subject, key, print).await?
        {
            tx.commit().await.map_err(map_err)?;
            return Ok(Appended {
                fact: replayed,
                replayed: true,
            });
        }
        let row: FactRow = sqlx::query_as(
            "insert into facts (subject_id, path, source, value, origin, envelope_version, confidence,
                                counterparty_id, observed_at, recorded_at, expires_at, consent, stub)
             values ($1, $2, $3, $4, $5, $6, $7, $8, $9, clock_timestamp(), $10, $11, $12)
             returning subject_id, id, path, source, value, origin, envelope_version, confidence, counterparty_id,
                       observed_at, recorded_at, expires_at, consent, stub",
        )
        .bind(subject)
        .bind(&fact.path)
        .bind(fact.source.as_str())
        .bind(&fact.value.bytes)
        .bind(&fact.origin.bytes)
        .bind(i16::try_from(fact.value.version).unwrap_or(i16::MAX))
        .bind(fact.confidence)
        .bind(fact.counterparty_id)
        .bind(fact.observed_at)
        .bind(fact.expires_at)
        .bind(scope_strings(&fact.consent))
        .bind(fact.stub)
        .fetch_one(&mut *tx)
        .await
        .map_err(map_err)?;
        let stored = row.into_fact()?;
        upsert_current(&mut tx, &stored).await?;
        insert_event(&mut tx, EventKind::FactRecorded, &stored).await?;
        if let (Some(key), Some(print)) = (&fact.idempotency_key, &print) {
            sqlx::query(
                "insert into idempotency (subject_id, key, fact_id, fingerprint) values ($1, $2, $3, $4)",
            )
            .bind(subject)
            .bind(key)
            .bind(stored.id.0)
            .bind(print)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        }
        tx.commit().await.map_err(map_err)?;
        Ok(Appended {
            fact: stored,
            replayed: false,
        })
    }

    async fn current(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        validate::query(query, false)?;
        self.check_open(subject).await?;
        let f = Filters::of(query)?;
        let sql = format!(
            "select {FACT_COLUMNS} from facts_current c
               join facts f on (f.subject_id, f.id) = (c.subject_id, c.fact_id)
              where c.subject_id = $1
                and f.value is not null
                and (f.expires_at is null or f.expires_at > now())
                {FILTERS}
              order by f.recorded_at, f.id
              limit $8"
        );
        let rows: Vec<FactRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
            .bind(subject)
            .bind(&f.sources)
            .bind(&f.scopes)
            .bind(&f.paths.exact)
            .bind(&f.paths.prefixes)
            .bind(f.cursor_at)
            .bind(f.cursor_id)
            .bind(f.limit)
            .fetch_all(&self.pool)
            .await
            .map_err(map_err)?;
        let facts = rows
            .into_iter()
            .map(FactRow::into_fact)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page::cut(facts, query.page_size(), |f| {
            (f.recorded_at, f.id)
        }))
    }

    async fn history(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        validate::query(query, true)?;
        self.check_open(subject).await?;
        let f = Filters::of(query)?;
        let sql = format!(
            "select {FACT_COLUMNS} from facts f
              where f.subject_id = $1
                and ($9::timestamptz is null or f.recorded_at <= $9)
                {FILTERS}
              order by f.recorded_at, f.id
              limit $8"
        );
        let rows: Vec<FactRow> = sqlx::query_as(sqlx::AssertSqlSafe(sql))
            .bind(subject)
            .bind(&f.sources)
            .bind(&f.scopes)
            .bind(&f.paths.exact)
            .bind(&f.paths.prefixes)
            .bind(f.cursor_at)
            .bind(f.cursor_id)
            .bind(f.limit)
            .bind(query.at)
            .fetch_all(&self.pool)
            .await
            .map_err(map_err)?;
        let facts = rows
            .into_iter()
            .map(FactRow::into_fact)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Page::cut(facts, query.page_size(), |f| {
            (f.recorded_at, f.id)
        }))
    }

    async fn retract(
        &self,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: Envelope,
    ) -> Result<Fact, StoreError> {
        validate::path_shape(path, 2).map_err(|r| StoreError::invalid("path", r))?;
        validate::envelope("origin", &origin)?;
        let mut tx = self.begin().await?;
        Self::lock_open(&mut tx, subject).await?;
        let latest: Option<(Vec<String>,)> = sqlx::query_as(
            "select consent from facts
              where subject_id = $1 and path = $2 and source = $3 and value is not null
              order by recorded_at desc, id desc limit 1",
        )
        .bind(subject)
        .bind(path)
        .bind(source.as_str())
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_err)?;
        let Some((consent,)) = latest else {
            return Err(StoreError::NotFound { what: "fact" });
        };
        sqlx::query(
            "delete from facts where subject_id = $1 and path = $2 and source = $3 and value is not null",
        )
        .bind(subject)
        .bind(path)
        .bind(source.as_str())
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        let consent = scopes_of(consent);
        let tomb = Self::insert_tombstone(
            &mut tx,
            subject,
            path,
            source,
            &origin,
            &consent,
            EventKind::FactRetracted,
        )
        .await?;
        tx.commit().await.map_err(map_err)?;
        Ok(tomb)
    }

    async fn request_erasure(&self, subject: SubjectId) -> Result<Erasure, StoreError> {
        let mut tx = self.begin().await?;
        // Lock order: the pending erasure row first, then the subject.
        let pending: Option<ErasureRow> = sqlx::query_as(
            "select subject_id, event_id, requested_at, executed_at, cancelled_at, published_at
               from erasures where subject_id = $1 and executed_at is null and cancelled_at is null
                for update",
        )
        .bind(subject)
        .fetch_optional(&mut *tx)
        .await
        .map_err(map_err)?;
        if let Some(row) = pending {
            return Ok(row.into_erasure());
        }
        let exists: Option<(Option<DateTime<Utc>>,)> =
            sqlx::query_as("select erased_at from subjects where id = $1 for no key update")
                .bind(subject)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_err)?;
        if exists.is_none() {
            return Err(StoreError::NotFound { what: "subject" });
        }
        let row: ErasureRow = sqlx::query_as(
            "with marked as (
                 update subjects set erased_at = coalesce(erased_at, clock_timestamp()) where id = $1
                 returning erased_at
             )
             insert into erasures (subject_id, event_id, requested_at)
             select $1, $2, marked.erased_at from marked
             returning subject_id, event_id, requested_at, executed_at, cancelled_at, published_at",
        )
        .bind(subject)
        .bind(Uuid::now_v7())
        .fetch_one(&mut *tx)
        .await
        .map_err(map_err)?;
        tx.commit().await.map_err(map_err)?;
        Ok(row.into_erasure())
    }

    async fn restore(&self, subject: SubjectId) -> Result<(), StoreError> {
        let mut tx = self.begin().await?;
        sqlx::query(
            "select 1 from erasures where subject_id = $1 and executed_at is null and cancelled_at is null for update",
        )
        .bind(subject)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        let exists: Option<(Option<DateTime<Utc>>,)> =
            sqlx::query_as("select erased_at from subjects where id = $1 for no key update")
                .bind(subject)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_err)?;
        if exists.is_none() {
            return Err(StoreError::NotFound { what: "subject" });
        }
        sqlx::query(
            "update erasures set cancelled_at = clock_timestamp()
              where subject_id = $1 and executed_at is null and cancelled_at is null",
        )
        .bind(subject)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        sqlx::query("update subjects set erased_at = null where id = $1")
            .bind(subject)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
        tx.commit().await.map_err(map_err)?;
        Ok(())
    }

    async fn execute_due_erasures(
        &self,
        grace: Duration,
        limit: u32,
    ) -> Result<Vec<ErasureExecuted>, StoreError> {
        let mut out = Vec::new();
        for _ in 0..limit {
            let mut tx = self.begin().await?;
            let due: Option<(SubjectId, DateTime<Utc>)> = sqlx::query_as(
                "select subject_id, requested_at from erasures
                  where executed_at is null and cancelled_at is null
                    and requested_at + $1 <= now()
                  order by requested_at limit 1 for update skip locked",
            )
            .bind(grace)
            .fetch_optional(&mut *tx)
            .await
            .map_err(map_err)?;
            let Some((sid, requested_at)) = due else {
                break;
            };
            // Lock the survivors in id order, then the subject.
            sqlx::query(
                "select 1 from subjects where id in (select distinct subject_id from facts where counterparty_id = $1)
                  order by id for no key update",
            )
            .bind(sid)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
            let marked: Option<(Option<DateTime<Utc>>,)> =
                sqlx::query_as("select erased_at from subjects where id = $1 for no key update")
                    .bind(sid)
                    .fetch_optional(&mut *tx)
                    .await
                    .map_err(map_err)?;
            if !matches!(marked, Some((Some(_),))) {
                // Restored meanwhile, or already gone: leave the row alone.
                tx.rollback().await.map_err(map_err)?;
                continue;
            }
            let tombstoned = tombstone_survivors(&mut tx, sid).await?;
            let deleted =
                sqlx::query("delete from subjects where id = $1 and erased_at is not null")
                    .bind(sid)
                    .execute(&mut *tx)
                    .await
                    .map_err(map_err)?
                    .rows_affected();
            if deleted == 0 {
                tx.rollback().await.map_err(map_err)?;
                continue;
            }
            let (executed_at,): (DateTime<Utc>,) = sqlx::query_as(
                "update erasures set executed_at = clock_timestamp()
                  where subject_id = $1 and requested_at = $2
                  returning executed_at",
            )
            .bind(sid)
            .bind(requested_at)
            .fetch_one(&mut *tx)
            .await
            .map_err(map_err)?;
            tx.commit().await.map_err(map_err)?;
            out.push(ErasureExecuted {
                subject_id: sid,
                executed_at,
                tombstoned,
            });
        }
        Ok(out)
    }

    async fn claim_events(
        &self,
        limit: u32,
        lease: Duration,
    ) -> Result<Vec<OutboxEvent>, StoreError> {
        let limit_i = i64::from(limit);
        let mut tx = self.begin().await?;
        let rows: Vec<(Uuid, SubjectId, String, Option<i64>, serde_json::Value, DateTime<Utc>)> =
            sqlx::query_as(
                "with c as (
                     select seq from outbox
                      where published_at is null and (claimed_until is null or claimed_until < now())
                      order by seq limit $1 for update skip locked)
                 update outbox o set claimed_until = now() + $2 from c where o.seq = c.seq
                 returning o.event_id, o.subject_id, o.kind, o.fact_id, o.payload, o.recorded_at",
            )
            .bind(limit_i)
            .bind(lease)
            .fetch_all(&mut *tx)
            .await
            .map_err(map_err)?;
        let mut out = Vec::with_capacity(rows.len());
        for (event_id, subject_id, kind, fact_id, payload, recorded_at) in rows {
            out.push(OutboxEvent {
                event_id,
                subject_id,
                kind: kind.parse()?,
                fact_id: fact_id.map(FactId),
                payload,
                recorded_at,
            });
        }
        out.sort_by_key(|e| e.recorded_at);
        let remaining = limit_i.saturating_sub(i64::try_from(out.len()).unwrap_or(i64::MAX));
        if remaining > 0 {
            let erased: Vec<(Uuid, SubjectId, DateTime<Utc>, DateTime<Utc>)> = sqlx::query_as(
                "with c as (
                     select subject_id, requested_at from erasures
                      where executed_at is not null and published_at is null
                        and (claimed_until is null or claimed_until < now())
                      order by executed_at limit $1 for update skip locked)
                 update erasures e set claimed_until = now() + $2 from c
                  where (e.subject_id, e.requested_at) = (c.subject_id, c.requested_at)
                 returning e.event_id, e.subject_id, e.requested_at, e.executed_at",
            )
            .bind(remaining)
            .bind(lease)
            .fetch_all(&mut *tx)
            .await
            .map_err(map_err)?;
            for (event_id, subject_id, requested_at, executed_at) in erased {
                out.push(OutboxEvent {
                    event_id,
                    subject_id,
                    kind: EventKind::SubjectErased,
                    fact_id: None,
                    payload: serde_json::json!({
                        "requested_at": requested_at,
                        "executed_at": executed_at,
                    }),
                    recorded_at: executed_at,
                });
            }
        }
        tx.commit().await.map_err(map_err)?;
        Ok(out)
    }

    async fn ack_events(&self, event_ids: &[Uuid]) -> Result<u64, StoreError> {
        let mut tx = self.begin().await?;
        let a = sqlx::query(
            "update outbox set published_at = now() where event_id = any($1) and published_at is null",
        )
        .bind(event_ids)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?
        .rows_affected();
        let b = sqlx::query(
            "update erasures set published_at = now() where event_id = any($1) and published_at is null",
        )
        .bind(event_ids)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?
        .rows_affected();
        tx.commit().await.map_err(map_err)?;
        Ok(a + b)
    }

    async fn purge_idempotency(&self, ttl: Duration) -> Result<u64, StoreError> {
        let n = sqlx::query("delete from idempotency where created_at < now() - $1")
            .bind(ttl)
            .execute(&self.pool)
            .await
            .map_err(map_err)?
            .rows_affected();
        Ok(n)
    }
}

#[derive(sqlx::FromRow)]
struct ErasureRow {
    subject_id: Uuid,
    event_id: Uuid,
    requested_at: DateTime<Utc>,
    executed_at: Option<DateTime<Utc>>,
    cancelled_at: Option<DateTime<Utc>>,
    published_at: Option<DateTime<Utc>>,
}

impl ErasureRow {
    fn into_erasure(self) -> Erasure {
        Erasure {
            subject_id: self.subject_id,
            event_id: self.event_id,
            requested_at: self.requested_at,
            executed_at: self.executed_at,
            cancelled_at: self.cancelled_at,
            published_at: self.published_at,
        }
    }
}
