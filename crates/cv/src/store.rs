//! The rows: who asked, what was decided, who downloaded.
//!
//! One row per person, keyed by the verified subject. Asking again renews the
//! same row, so a decision's history is its timestamps. Every download is a
//! row of its own with what the gateway saw of the client.

use chrono::{DateTime, Utc};
use sqlx::{FromRow, PgPool};
use uuid::Uuid;

/// Asked, not yet decided.
pub const REQUESTED: &str = "requested";
/// May download.
pub const APPROVED: &str = "approved";
/// Told no.
pub const REFUSED: &str = "refused";
/// Was approved once; no longer.
pub const REVOKED: &str = "revoked";

/// One person's request, with their downloads counted in.
#[derive(Debug, Clone, FromRow)]
pub struct RequestRow {
    /// The row.
    pub id: Uuid,
    /// The OIDC subject Envoy verified.
    pub subject: String,
    /// From the token, at the last request.
    pub email: String,
    /// From the token, at the last request; may be empty.
    pub name: String,
    /// Why, in their words.
    pub note: String,
    /// One of the four statuses above.
    pub status: String,
    /// When they last asked.
    pub requested_at: DateTime<Utc>,
    /// When the owner last decided; none while requested.
    pub decided_at: Option<DateTime<Utc>>,
    /// Who decided, as the token named them.
    pub decided_by: Option<String>,
    /// When the owner's mail went out; none until it did.
    pub notified_at: Option<DateTime<Utc>>,
    /// How many times the document was handed out.
    pub downloads: i64,
    /// The last time it was.
    pub last_download_at: Option<DateTime<Utc>>,
}

/// `select <the columns of a request> from cv.requests r` followed by the
/// literal `$rest`: sqlx takes only literal SQL, so the column list is a
/// macro rather than a constant.
macro_rules! select_request {
    ($rest:literal) => {
        concat!(
            "select r.id, r.subject, r.email, r.name, r.note, r.status, r.requested_at, ",
            "r.decided_at, r.decided_by, r.notified_at, ",
            "(select count(*) from cv.downloads d where d.request_id = r.id) as downloads, ",
            "(select max(d.at) from cv.downloads d where d.request_id = r.id) as last_download_at ",
            "from cv.requests r ",
            $rest
        )
    };
}

/// What the owner may do to a request.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Decision {
    /// Let them download.
    Approve,
    /// Tell them no.
    Refuse,
    /// Take an approval back.
    Revoke,
}

impl Decision {
    /// The word on the wire.
    ///
    /// # Errors
    /// Not one of `approve`, `refuse`, `revoke`.
    pub fn parse(word: &str) -> Result<Self, StoreError> {
        match word {
            "approve" => Ok(Self::Approve),
            "refuse" => Ok(Self::Refuse),
            "revoke" => Ok(Self::Revoke),
            other => Err(StoreError::Refused(format!(
                "decision must be approve, refuse or revoke, not {other:?}"
            ))),
        }
    }

    /// The statuses a decision applies to, and the one it leads to.
    const fn transition(self) -> (&'static [&'static str], &'static str) {
        match self {
            Self::Approve => (&[REQUESTED, REFUSED, REVOKED], APPROVED),
            Self::Refuse => (&[REQUESTED], REFUSED),
            Self::Revoke => (&[APPROVED], REVOKED),
        }
    }

    /// The verb, for a sentence.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Approve => "approve",
            Self::Refuse => "refuse",
            Self::Revoke => "revoke",
        }
    }
}

/// Why the store did not do it.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// No such request.
    #[error("no such request")]
    NotFound,
    /// The request is in a state the operation does not apply to.
    #[error("{0}")]
    Refused(String),
    /// The database.
    #[error("store: {0}")]
    Db(#[from] sqlx::Error),
}

/// The caller's own request, if they made one.
///
/// # Errors
/// The database.
pub async fn get(pool: &PgPool, subject: &str) -> Result<Option<RequestRow>, StoreError> {
    let row = sqlx::query_as::<_, RequestRow>(select_request!("where r.subject = $1"))
        .bind(subject)
        .fetch_optional(pool)
        .await?;
    Ok(row)
}

/// One request by id, for the owner.
///
/// # Errors
/// Not found; the database.
pub async fn by_id(pool: &PgPool, id: Uuid) -> Result<RequestRow, StoreError> {
    sqlx::query_as::<_, RequestRow>(select_request!("where r.id = $1"))
        .bind(id)
        .fetch_optional(pool)
        .await?
        .ok_or(StoreError::NotFound)
}

/// Ask, or ask again. A refused or revoked request goes back to `requested`
/// with the new note and a fresh clock, so the owner is told again; an
/// approved one is left approved and only its note and names are refreshed.
///
/// # Errors
/// The database.
pub async fn request(
    pool: &PgPool,
    subject: &str,
    email: &str,
    name: &str,
    note: &str,
) -> Result<RequestRow, StoreError> {
    sqlx::query(
        "insert into cv.requests (subject, email, name, note, status)
         values ($1, $2, $3, $4, 'requested')
         on conflict (subject) do update set
             email = excluded.email,
             name = excluded.name,
             note = excluded.note,
             status = case when cv.requests.status = 'approved' then 'approved' else 'requested' end,
             requested_at = case when cv.requests.status = 'approved' then cv.requests.requested_at else now() end,
             decided_at = case when cv.requests.status = 'approved' then cv.requests.decided_at else null end,
             decided_by = case when cv.requests.status = 'approved' then cv.requests.decided_by else null end,
             notified_at = case when cv.requests.status = 'approved' then cv.requests.notified_at else null end,
             updated_at = now()",
    )
    .bind(subject)
    .bind(email)
    .bind(name)
    .bind(note)
    .execute(pool)
    .await?;
    get(pool, subject).await?.ok_or(StoreError::NotFound)
}

/// Every request, newest first; `status` narrows it.
///
/// # Errors
/// The database.
pub async fn list(pool: &PgPool, status: Option<&str>) -> Result<Vec<RequestRow>, StoreError> {
    let rows = sqlx::query_as::<_, RequestRow>(select_request!(
        "where $1::text is null or r.status = $1 order by r.requested_at desc"
    ))
    .bind(status)
    .fetch_all(pool)
    .await?;
    Ok(rows)
}

/// Apply a decision. Only the transitions `Decision::transition` lists are
/// allowed; anything else is refused with a sentence naming the state.
///
/// # Errors
/// Not found; refused; the database.
pub async fn decide(
    pool: &PgPool,
    id: Uuid,
    decision: Decision,
    decided_by: &str,
) -> Result<RequestRow, StoreError> {
    let (from, to) = decision.transition();
    let changed = sqlx::query(
        "update cv.requests
         set status = $2, decided_at = now(), decided_by = $3, updated_at = now()
         where id = $1 and status = any($4)",
    )
    .bind(id)
    .bind(to)
    .bind(decided_by)
    .bind(from)
    .execute(pool)
    .await?
    .rows_affected();
    let row = by_id(pool, id).await?;
    if changed == 0 {
        return Err(StoreError::Refused(format!(
            "cannot {} a request that is {}",
            decision.as_str(),
            row.status
        )));
    }
    Ok(row)
}

/// The owner was told: the mail went out at this moment.
///
/// # Errors
/// The database.
pub async fn mark_notified(pool: &PgPool, id: Uuid) -> Result<(), StoreError> {
    sqlx::query("update cv.requests set notified_at = now(), updated_at = now() where id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// One download, as the gateway saw the client.
///
/// # Errors
/// The database.
pub async fn record_download(
    pool: &PgPool,
    request_id: Uuid,
    user_agent: &str,
    ip: &str,
) -> Result<(), StoreError> {
    sqlx::query("insert into cv.downloads (request_id, user_agent, ip) values ($1, $2, $3)")
        .bind(request_id)
        .bind(user_agent)
        .bind(ip)
        .execute(pool)
        .await?;
    Ok(())
}
