//! The Postgres schema `llm` (`migrations/0029_llm.sql`): sessions and
//! generations. The budget is a sum over generations per subject per UTC day.
//! Nothing here reads a prompt or a completion; the record is what happened.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

use crate::{config::Tier, engine::Usage};

/// One generation as recorded.
#[derive(Debug, Clone, FromRow)]
pub struct GenerationRow {
    /// The generation.
    pub id: Uuid,
    /// The session it belongs to, if the session still exists.
    pub session_id: Option<Uuid>,
    /// The caller.
    pub subject: String,
    /// `fast` or `deep`.
    pub tier: String,
    /// The engine kind that answered.
    pub engine: String,
    /// The model the tier was configured with.
    pub model: String,
    /// The engine was the test stub.
    pub stub: bool,
    /// The engine's build when the generation started; empty before the first probe.
    pub engine_version: String,
    /// The weights' revision when the generation started; empty before the first probe.
    pub model_revision: String,
    /// The agent it spoke as, or empty.
    pub agent: String,
    /// `running`, `ok`, `failed` or `cancelled`.
    pub status: String,
    /// Why it failed, or empty.
    pub error: String,
    /// Tokens in the prompt, as the engine reported.
    pub prompt_tokens: i32,
    /// Tokens generated, as the engine reported.
    pub completion_tokens: i32,
    /// Milliseconds to the first text chunk, if one came.
    pub first_token_ms: Option<i32>,
    /// When the engine was asked.
    pub started_at: DateTime<Utc>,
    /// When it ended, however it ended.
    pub finished_at: Option<DateTime<Utc>>,
}

/// Why the store did not do it.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// No such session for this caller.
    #[error("no such session")]
    NotFound,
    /// The database.
    #[error("store: {0}")]
    Db(#[from] sqlx::Error),
}

/// The session to record under: `wanted` if it is the caller's, a new one if
/// none was named. A session that exists but belongs to someone else is
/// `NotFound`, the same answer as one that does not exist.
///
/// # Errors
/// The database, or a foreign session.
pub async fn touch_session(
    pool: &PgPool,
    subject: &str,
    wanted: Option<Uuid>,
) -> Result<Uuid, StoreError> {
    let Some(id) = wanted else {
        let (id,): (Uuid,) =
            sqlx::query_as("insert into llm.sessions (subject) values ($1) returning id")
                .bind(subject)
                .fetch_one(pool)
                .await?;
        return Ok(id);
    };
    let changed =
        sqlx::query("update llm.sessions set last_used_at = now() where id = $1 and subject = $2")
            .bind(id)
            .bind(subject)
            .execute(pool)
            .await?
            .rows_affected();
    if changed == 0 {
        return Err(StoreError::NotFound);
    }
    Ok(id)
}

/// What a generation starts as.
#[derive(Debug, Clone)]
pub struct Start<'a> {
    /// The generation id the service minted.
    pub id: Uuid,
    /// Its session.
    pub session_id: Uuid,
    /// The caller.
    pub subject: &'a str,
    /// The tier.
    pub tier: Tier,
    /// The engine kind.
    pub engine: &'a str,
    /// The model.
    pub model: &'a str,
    /// The engine is the stub.
    pub stub: bool,
    /// The engine's build, as the probe last read it (empty until it did).
    pub engine_version: &'a str,
    /// The weights' revision, as the probe last read it (empty until it did).
    pub model_revision: &'a str,
    /// The agent it spoke as, or empty (RFC 0011).
    pub agent: &'a str,
}

/// Record that the engine was asked.
///
/// # Errors
/// The database.
pub async fn start_generation(pool: &PgPool, start: Start<'_>) -> Result<(), StoreError> {
    sqlx::query(
        "insert into llm.generations (id, session_id, subject, tier, engine, model, stub, status,
                                      engine_version, model_revision, agent)
         values ($1, $2, $3, $4, $5, $6, $7, 'running', $8, $9, $10)",
    )
    .bind(start.id)
    .bind(start.session_id)
    .bind(start.subject)
    .bind(start.tier.as_str())
    .bind(start.engine)
    .bind(start.model)
    .bind(start.stub)
    .bind(start.engine_version)
    .bind(start.model_revision)
    .bind(start.agent)
    .execute(pool)
    .await?;
    Ok(())
}

/// How a generation ended.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    /// The done chunk arrived.
    Ok,
    /// The engine failed or the deadline passed.
    Failed,
    /// The caller went away first.
    Cancelled,
}

impl Outcome {
    fn as_str(self) -> &'static str {
        match self {
            Outcome::Ok => "ok",
            Outcome::Failed => "failed",
            Outcome::Cancelled => "cancelled",
        }
    }
}

/// Record how it ended. Idempotent on a row that already ended: the first
/// outcome stands.
///
/// # Errors
/// The database.
pub async fn finish_generation(
    pool: &PgPool,
    id: Uuid,
    outcome: Outcome,
    error: &str,
    usage: Option<Usage>,
    first_token_ms: Option<i32>,
) -> Result<(), StoreError> {
    let usage = usage.unwrap_or_default();
    sqlx::query(
        "update llm.generations
         set status = $2, error = $3, prompt_tokens = $4, completion_tokens = $5,
             first_token_ms = coalesce($6, first_token_ms), finished_at = now()
         where id = $1 and status = 'running'",
    )
    .bind(id)
    .bind(outcome.as_str())
    .bind(error)
    .bind(i32::try_from(usage.prompt_tokens).unwrap_or(i32::MAX))
    .bind(i32::try_from(usage.completion_tokens).unwrap_or(i32::MAX))
    .bind(first_token_ms)
    .execute(pool)
    .await?;
    Ok(())
}

/// Tokens (prompt plus completion) this subject spent on generations started
/// at or after `since`.
///
/// # Errors
/// The database.
pub async fn used_since(
    pool: &PgPool,
    subject: &str,
    since: DateTime<Utc>,
) -> Result<u64, StoreError> {
    let (sum,): (i64,) = sqlx::query_as(
        "select coalesce(sum(prompt_tokens + completion_tokens), 0)::bigint
         from llm.generations where subject = $1 and started_at >= $2",
    )
    .bind(subject)
    .bind(since)
    .fetch_one(pool)
    .await?;
    Ok(u64::try_from(sum).unwrap_or(0))
}

/// One generation, for the record's readers.
///
/// # Errors
/// The database.
pub async fn generation(pool: &PgPool, id: Uuid) -> Result<Option<GenerationRow>, StoreError> {
    let row = sqlx::query_as::<_, GenerationRow>(
        "select id, session_id, subject, tier, engine, model, stub, engine_version, model_revision, agent,
                status, error, prompt_tokens, completion_tokens, first_token_ms, started_at, finished_at
         from llm.generations where id = $1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(row)
}

/// A session's generations, newest first.
///
/// # Errors
/// The database.
pub async fn generations_of(
    pool: &PgPool,
    session_id: Uuid,
) -> Result<Vec<GenerationRow>, StoreError> {
    let rows = sqlx::query_as::<_, GenerationRow>(
        "select id, session_id, subject, tier, engine, model, stub, engine_version, model_revision, agent,
                status, error, prompt_tokens, completion_tokens, first_token_ms, started_at, finished_at
         from llm.generations where session_id = $1 order by started_at desc",
    )
    .bind(session_id)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// The start of the current UTC day: what "today" means for the budget.
#[must_use]
pub fn day_start(now: DateTime<Utc>) -> DateTime<Utc> {
    now.date_naive()
        .and_hms_opt(0, 0, 0)
        .map_or(now, |d| d.and_utc())
}
