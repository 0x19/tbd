//! Connector rows, and the one place credentials are opened.

use chrono::{DateTime, Days, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tbd_db::{Access, DbError, PartyId, map_err};
use uuid::Uuid;

use super::{AuthContext, Connector, ConnectorError, Pulled, crypto::Sealer};

/// Why a store call failed.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The database, or a row outside the grant.
    #[error(transparent)]
    Db(#[from] DbError),
    /// The kind.
    #[error(transparent)]
    Connector(#[from] ConnectorError),
    /// Credentials at rest.
    #[error(transparent)]
    Crypto(#[from] super::crypto::CryptoError),
    /// No such kind.
    #[error("unknown connector kind {0}")]
    UnknownKind(String),
    /// The row is not in a state the call accepts.
    #[error("connector is {0}")]
    State(String),
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct ConnectorRow {
    pub id: Uuid,
    pub party_id: Uuid,
    pub kind: String,
    pub label: String,
    pub status: String,
    pub config: Value,
    pub external_id: Option<String>,
    pub linked_at: Option<DateTime<Utc>>,
    pub last_sync_at: Option<DateTime<Utc>>,
    pub last_sync_status: Option<String>,
    pub last_sync_error: Option<String>,
    pub failure: Option<String>,
    pub created_at: DateTime<Utc>,
}

#[allow(missing_docs)]
#[derive(Debug, Clone, sqlx::FromRow)]
pub struct RunRow {
    pub id: Uuid,
    pub connector_id: Uuid,
    pub started_at: DateTime<Utc>,
    pub finished_at: Option<DateTime<Utc>>,
    pub trigger: String,
    pub outcome: Option<String>,
    pub found: i32,
    pub stored: i32,
    pub skipped: i32,
    pub error: Option<String>,
}

const COLUMNS: &str =
    "id, party_id, kind, label, status, config, external_id, linked_at, last_sync_at,
    last_sync_status, last_sync_error, failure, created_at";

fn sql(s: &str) -> sqlx::AssertSqlSafe<String> {
    sqlx::AssertSqlSafe(s.to_owned())
}

/// Connectors in the view.
///
/// # Errors
/// The database.
pub async fn list(pool: &PgPool, view: &Access) -> Result<Vec<ConnectorRow>, StoreError> {
    if view.is_empty() {
        return Ok(Vec::new());
    }
    Ok(sqlx::query_as::<_, ConnectorRow>(sql(&format!(
        "select {COLUMNS} from finance.connectors where party_id = any($1) order by kind, label, created_at"
    )))
    .bind(view.party_ids())
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// One connector, if the caller may see it.
///
/// # Errors
/// The database; not in the view.
pub async fn get(pool: &PgPool, access: &Access, id: Uuid) -> Result<ConnectorRow, StoreError> {
    let row = sqlx::query_as::<_, ConnectorRow>(sql(&format!(
        "select {COLUMNS} from finance.connectors where id = $1"
    )))
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?
    .ok_or(DbError::NotFound { what: "connector" })?;
    access.require(PartyId(row.party_id), "connector")?;
    Ok(row)
}

/// Recent runs of one connector.
///
/// # Errors
/// The database; not in the view.
pub async fn runs(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
    limit: i64,
) -> Result<Vec<RunRow>, StoreError> {
    get(pool, access, id).await?;
    Ok(sqlx::query_as::<_, RunRow>(
        "select id, connector_id, started_at, finished_at, trigger, outcome, found, stored, skipped, error
           from finance.connector_runs where connector_id = $1 order by started_at desc limit $2",
    )
    .bind(id)
    .bind(limit)
    .fetch_all(pool)
    .await
    .map_err(map_err)?)
}

/// Begin a link: a pending row with a fresh state, and the URL to visit.
///
/// # Errors
/// The party is outside the grant; the kind is unknown or unconfigured.
pub async fn start(
    pool: &PgPool,
    access: &Access,
    kind: &dyn Connector,
    party: Uuid,
    redirect_url: &str,
) -> Result<(Uuid, String), StoreError> {
    access.require(PartyId(party), "party")?;
    let id = Uuid::new_v4();
    let state = {
        use base64::Engine as _;
        let bytes: [u8; 32] = rand::random();
        base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(bytes)
    };
    let ctx = AuthContext {
        state: state.clone(),
        redirect_url: redirect_url.to_owned(),
    };
    let url = kind.start(&ctx).await?;
    sqlx::query(
        "insert into finance.connectors (id, party_id, kind, status, state)
         values ($1, $2, $3, 'pending', $4)",
    )
    .bind(id)
    .bind(party)
    .bind(kind.kind().name)
    .bind(&state)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok((id, url))
}

/// Finish a link with the callback's state and code. The state names the
/// row; the caller must be allowed its party, or it is not-found.
///
/// # Errors
/// Not-found for a foreign or unknown state; the provider's refusal.
pub async fn complete(
    pool: &PgPool,
    access: &Access,
    sealer: &Sealer,
    kinds: &[Box<dyn Connector>],
    redirect_url: &str,
    state: &str,
    code: &str,
) -> Result<ConnectorRow, StoreError> {
    let pending: Option<(Uuid, Uuid, String, String)> = sqlx::query_as(
        "select id, party_id, kind, status from finance.connectors where state = $1",
    )
    .bind(state)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    let (id, party, kind_name, status) = pending.ok_or(DbError::NotFound { what: "connector" })?;
    access.require(PartyId(party), "connector")?;
    if status != "pending" {
        return Err(StoreError::State(status));
    }
    let kind = kinds
        .iter()
        .find(|k| k.kind().name == kind_name)
        .ok_or_else(|| StoreError::UnknownKind(kind_name.clone()))?;
    let ctx = AuthContext {
        state: state.to_owned(),
        redirect_url: redirect_url.to_owned(),
    };
    let linked = match kind.complete(&ctx, code).await {
        Ok(l) => l,
        Err(e) => {
            sqlx::query("update finance.connectors set status = 'failed', failure = $2, updated_at = now() where id = $1")
                .bind(id)
                .bind("link refused")
                .execute(pool)
                .await
                .map_err(map_err)?;
            return Err(e.into());
        }
    };

    // The same account linked before: keep that row, refresh its credential,
    // and drop the pending one -- the mailbox's history stays attached.
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "select id from finance.connectors where party_id = $1 and kind = $2 and external_id = $3 and id <> $4",
    )
    .bind(party)
    .bind(&kind_name)
    .bind(&linked.external_id)
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    let target = existing.map_or(id, |(e,)| e);
    let blob = sealer.seal(linked.credentials.to_string().as_bytes(), target.as_bytes())?;
    sqlx::query(
        "update finance.connectors
            set status = 'linked', credentials = $2, external_id = $3, label = $4, linked_at = now(),
                state = case when id = $5 then state else null end, failure = null, updated_at = now()
          where id = $1",
    )
    .bind(target)
    .bind(&blob)
    .bind(&linked.external_id)
    .bind(&linked.label)
    .bind(target)
    .execute(pool)
    .await
    .map_err(map_err)?;
    if target != id {
        sqlx::query("delete from finance.connectors where id = $1")
            .bind(id)
            .execute(pool)
            .await
            .map_err(map_err)?;
    }
    get(pool, access, target).await
}

/// The opened credential of a linked connector. Private: only this module
/// hands it to a kind, for the duration of one call.
async fn credentials(pool: &PgPool, sealer: &Sealer, id: Uuid) -> Result<Value, StoreError> {
    let (blob,): (Option<Vec<u8>>,) =
        sqlx::query_as("select credentials from finance.connectors where id = $1")
            .bind(id)
            .fetch_one(pool)
            .await
            .map_err(map_err)?;
    let blob = blob.ok_or_else(|| StoreError::State("pending".into()))?;
    let bytes = sealer.open(&blob, id.as_bytes())?;
    serde_json::from_slice(&bytes).map_err(|e| StoreError::State(format!("credentials: {e}")))
}

/// Prove the link still works. A refusal marks the row expired.
///
/// # Errors
/// Not in the view; the provider.
pub async fn test(
    pool: &PgPool,
    access: &Access,
    sealer: &Sealer,
    kinds: &[Box<dyn Connector>],
    id: Uuid,
) -> Result<String, StoreError> {
    let row = get(pool, access, id).await?;
    let kind = kinds
        .iter()
        .find(|k| k.kind().name == row.kind)
        .ok_or_else(|| StoreError::UnknownKind(row.kind.clone()))?;
    let creds = credentials(pool, sealer, id).await?;
    match kind.test(&creds).await {
        Ok(s) => Ok(s),
        Err(e @ ConnectorError::Unlinked(_)) => {
            sqlx::query("update finance.connectors set status = 'expired', updated_at = now() where id = $1")
                .bind(id)
                .execute(pool)
                .await
                .map_err(map_err)?;
            Err(e.into())
        }
        Err(e) => Err(e.into()),
    }
}

/// Pull new documents and store them. Idempotent: a provider id seen before
/// is skipped, and identical bytes are one document.
///
/// # Errors
/// Not in the view; the provider; the database.
pub async fn sync(
    pool: &PgPool,
    access: &Access,
    sealer: &Sealer,
    kinds: &[Box<dyn Connector>],
    id: Uuid,
    trigger: &str,
) -> Result<Pulled, StoreError> {
    let row = get(pool, access, id).await?;
    if row.status != "linked" {
        return Err(StoreError::State(row.status));
    }
    let kind = kinds
        .iter()
        .find(|k| k.kind().name == row.kind)
        .ok_or_else(|| StoreError::UnknownKind(row.kind.clone()))?;
    let run_id = Uuid::new_v4();
    sqlx::query(
        "insert into finance.connector_runs (id, connector_id, trigger) values ($1, $2, $3)",
    )
    .bind(run_id)
    .bind(id)
    .bind(trigger)
    .execute(pool)
    .await
    .map_err(map_err)?;

    // Since the last successful sync, a week back for late arrivals; a first
    // pull reaches back a year.
    let since = row
        .last_sync_at
        .and_then(|t| t.checked_sub_days(Days::new(7)))
        .unwrap_or_else(|| {
            Utc::now()
                .checked_sub_days(Days::new(365))
                .unwrap_or_else(Utc::now)
        });
    let seen_refs: Vec<(String,)> =
        sqlx::query_as("select external_ref from finance.document_sources where connector_id = $1")
            .bind(id)
            .fetch_all(pool)
            .await
            .map_err(map_err)?;
    let seen_set: std::collections::HashSet<String> = seen_refs.into_iter().map(|(r,)| r).collect();
    let seen = |external_ref: &str| {
        // Attachments are keyed `<message>:<file>`; the message alone says
        // whether it was pulled at all.
        seen_set
            .iter()
            .any(|s| s == external_ref || s.split_once(':').is_some_and(|(m, _)| m == external_ref))
    };

    let creds = match credentials(pool, sealer, id).await {
        Ok(c) => c,
        Err(e) => {
            finish(pool, run_id, id, Err(&e.to_string())).await?;
            return Err(e);
        }
    };
    let (found, _) = match kind.pull(&creds, &row.config, since, &seen).await {
        Ok(f) => f,
        Err(e) => {
            let status = if matches!(e, ConnectorError::Unlinked(_)) {
                "expired"
            } else {
                "linked"
            };
            sqlx::query(
                "update finance.connectors set status = $2, updated_at = now() where id = $1",
            )
            .bind(id)
            .bind(status)
            .execute(pool)
            .await
            .map_err(map_err)?;
            finish(pool, run_id, id, Err(&e.to_string())).await?;
            return Err(e.into());
        }
    };

    let pulled = store_found(pool, row.party_id, id, &found, &seen).await?;
    finish(pool, run_id, id, Ok(&pulled)).await?;
    Ok(pulled)
}

/// Store what a pull found: new bytes become a document, known bytes gain a
/// source, and anything seen before is skipped.
async fn store_found(
    pool: &PgPool,
    party: Uuid,
    connector: Uuid,
    found: &[super::Found],
    seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
) -> Result<Pulled, StoreError> {
    let mut pulled = Pulled {
        found: found.len(),
        ..Pulled::default()
    };
    for f in found {
        if seen(&f.external_ref) {
            pulled.skipped += 1;
            continue;
        }
        let mut hasher = Sha256::new();
        hasher.update(&f.bytes);
        let sha = format!("{:x}", hasher.finalize());
        let mut tx = pool.begin().await.map_err(map_err)?;
        // Same bytes seen before (the other mailbox got the same receipt):
        // one document, another source.
        let existing: Option<(Uuid,)> =
            sqlx::query_as("select id from finance.documents where party_id = $1 and sha256 = $2")
                .bind(party)
                .bind(&sha)
                .fetch_optional(&mut *tx)
                .await
                .map_err(map_err)?;
        let document_id = if let Some((d,)) = existing {
            pulled.skipped += 1;
            d
        } else {
            let d = Uuid::new_v4();
            sqlx::query(
                "insert into finance.documents (id, party_id, kind, sha256, content_type, size_bytes, filename)
                 values ($1, $2, 'receipt', $3, $4, $5, $6)",
            )
            .bind(d)
            .bind(party)
            .bind(&sha)
            .bind(&f.content_type)
            .bind(i64::try_from(f.bytes.len()).unwrap_or(i64::MAX))
            .bind(&f.filename)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
            sqlx::query("insert into finance.document_blobs (document_id, bytes) values ($1, $2)")
                .bind(d)
                .bind(&f.bytes)
                .execute(&mut *tx)
                .await
                .map_err(map_err)?;
            pulled.stored += 1;
            d
        };
        sqlx::query(
            "insert into finance.document_sources (document_id, connector_id, external_ref, subject, sender, received_at)
             values ($1, $2, $3, $4, $5, $6) on conflict do nothing",
        )
        .bind(document_id)
        .bind(connector)
        .bind(&f.external_ref)
        .bind(&f.subject)
        .bind(&f.sender)
        .bind(f.received_at)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        tx.commit().await.map_err(map_err)?;
    }
    Ok(pulled)
}

async fn finish(
    pool: &PgPool,
    run_id: Uuid,
    id: Uuid,
    outcome: Result<&Pulled, &str>,
) -> Result<(), StoreError> {
    match outcome {
        Ok(p) => {
            sqlx::query(
                "update finance.connector_runs set finished_at = now(), outcome = 'ok', found = $2, stored = $3, skipped = $4
                  where id = $1",
            )
            .bind(run_id)
            .bind(i32::try_from(p.found).unwrap_or(i32::MAX))
            .bind(i32::try_from(p.stored).unwrap_or(i32::MAX))
            .bind(i32::try_from(p.skipped).unwrap_or(i32::MAX))
            .execute(pool)
            .await
            .map_err(map_err)?;
            sqlx::query(
                "update finance.connectors set last_sync_at = now(), last_sync_status = 'ok', last_sync_error = null,
                    updated_at = now() where id = $1",
            )
            .bind(id)
            .execute(pool)
            .await
            .map_err(map_err)?;
        }
        Err(e) => {
            sqlx::query("update finance.connector_runs set finished_at = now(), outcome = 'error', error = $2 where id = $1")
                .bind(run_id)
                .bind(e)
                .execute(pool)
                .await
                .map_err(map_err)?;
            sqlx::query(
                "update finance.connectors set last_sync_status = 'error', last_sync_error = $2, updated_at = now()
                  where id = $1",
            )
            .bind(id)
            .bind(e)
            .execute(pool)
            .await
            .map_err(map_err)?;
        }
    }
    Ok(())
}

/// Remove a connector. Its documents stay; their sources lose the link.
///
/// # Errors
/// Not in the view.
pub async fn delete(pool: &PgPool, access: &Access, id: Uuid) -> Result<(), StoreError> {
    get(pool, access, id).await?;
    sqlx::query("delete from finance.connectors where id = $1")
        .bind(id)
        .execute(pool)
        .await
        .map_err(map_err)?;
    Ok(())
}

/// Change a connector's non-secret config (the query, say).
///
/// # Errors
/// Not in the view.
pub async fn configure(
    pool: &PgPool,
    access: &Access,
    id: Uuid,
    config: Value,
) -> Result<ConnectorRow, StoreError> {
    get(pool, access, id).await?;
    sqlx::query("update finance.connectors set config = $2, updated_at = now() where id = $1")
        .bind(id)
        .bind(config)
        .execute(pool)
        .await
        .map_err(map_err)?;
    get(pool, access, id).await
}
