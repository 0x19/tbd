//! Connector rows, and the one place credentials are opened.

use chrono::{DateTime, Days, Utc};
use serde_json::Value;
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use tbd_db::{Access, DbError, PartyId, map_err};
use uuid::Uuid;

use super::{AuthContext, Connector, ConnectorError, Found, Pulled, Reach, crypto::Sealer};

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
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
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
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
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

/// A run that `begin_sync` opened and `run_sync` will finish.
#[derive(Debug, Clone)]
pub struct Started {
    /// The run row, already inserted, `outcome` still null.
    pub run: RunRow,
    /// The connector as it was when the run opened.
    pub connector: ConnectorRow,
}

/// Rounds a run makes before giving up on draining a large mailbox in one
/// go. A kind's per-round cap times this bounds one run's provider quota.
const MAX_ROUNDS: usize = 10;

/// A run older than this that never finished belonged to a process that is
/// gone: it is closed as interrupted, and a new run may open.
const STALE_RUN_MINUTES: i32 = 30;

/// Open a sync run: check the connector is linked and idle, and record the
/// run. The pull itself is `run_sync`, which the caller detaches, because a
/// mailbox takes minutes and a unary call has seconds.
///
/// # Errors
/// Not in the view; not linked; a run already under way (`State("syncing")`);
/// the database.
pub async fn begin_sync(
    pool: &PgPool,
    access: &Access,
    kinds: &[Box<dyn Connector>],
    id: Uuid,
    trigger: &str,
) -> Result<Started, StoreError> {
    let connector = get(pool, access, id).await?;
    if connector.status != "linked" {
        return Err(StoreError::State(connector.status));
    }
    if !kinds.iter().any(|k| k.kind().name == connector.kind) {
        return Err(StoreError::UnknownKind(connector.kind.clone()));
    }
    // A run whose process died mid-pull would otherwise block the
    // connector forever; one that is recent is presumed still pulling.
    sqlx::query(
        "update finance.connector_runs set finished_at = now(), outcome = 'error', error = 'interrupted'
          where connector_id = $1 and finished_at is null
            and started_at < now() - make_interval(mins => $2)",
    )
    .bind(id)
    .bind(STALE_RUN_MINUTES)
    .execute(pool)
    .await
    .map_err(map_err)?;
    let running: Option<(Uuid,)> = sqlx::query_as(
        "select id from finance.connector_runs where connector_id = $1 and finished_at is null limit 1",
    )
    .bind(id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    if running.is_some() {
        return Err(StoreError::State("syncing".into()));
    }
    let run = sqlx::query_as::<_, RunRow>(
        "insert into finance.connector_runs (id, connector_id, trigger) values ($1, $2, $3)
          returning id, connector_id, started_at, finished_at, trigger, outcome, found, stored, skipped, error",
    )
    .bind(Uuid::new_v4())
    .bind(id)
    .bind(trigger)
    .fetch_one(pool)
    .await
    .map_err(map_err)?;
    Ok(Started { run, connector })
}

/// Pull new documents and store them, finishing the run either way.
/// Idempotent: a provider id seen before is skipped, and identical bytes are
/// one document. Runs until the kind says it has everything new, or
/// `MAX_ROUNDS`, after which the run is `partial` and the next one
/// continues where it stopped.
///
/// # Errors
/// The credentials, the provider, or the database. The run row records the
/// same error before it is returned.
pub async fn run_sync(
    pool: &PgPool,
    sealer: &Sealer,
    kinds: &[Box<dyn Connector>],
    started: &Started,
) -> Result<Pulled, StoreError> {
    let row = &started.connector;
    let id = row.id;
    let run_id = started.run.id;
    let kind = kinds
        .iter()
        .find(|k| k.kind().name == row.kind)
        .ok_or_else(|| StoreError::UnknownKind(row.kind.clone()))?;

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
    let creds = match credentials(pool, sealer, id).await {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!(connector = %id, error = %e, "connector sync: credentials");
            finish(pool, run_id, id, Err(&e.to_string())).await?;
            return Err(e);
        }
    };

    let mut pulled = Pulled::default();
    let mut complete = false;
    for round in 0..MAX_ROUNDS {
        let seen_set = seen_refs(pool, id).await?;
        let seen = |external_ref: &str| {
            // Attachments are keyed `<message>:<file>`; the message alone says
            // whether it was pulled at all.
            seen_set.iter().any(|s| {
                s == external_ref || s.split_once(':').is_some_and(|(m, _)| m == external_ref)
            })
        };
        // The kind fetches into one end, the store drains the other, so a
        // document is on disk and counted while the next one is still on
        // the wire. A store failure closes the receiver; the kind sees the
        // closed sink and stops.
        let (tx, mut rx) = tokio::sync::mpsc::channel::<Found>(4);
        let pulling = kind.pull(&creds, &row.config, since, &seen, tx);
        let storing = async {
            let mut counts = Pulled::default();
            let mut failure: Option<StoreError> = None;
            while let Some(f) = rx.recv().await {
                if failure.is_some() {
                    // Drain without storing, so the kind is not blocked on a
                    // full channel while we wait for it to notice.
                    continue;
                }
                match store_one(pool, row.party_id, id, &f, &seen).await {
                    Ok(stored) => {
                        counts.found += 1;
                        counts.stored += usize::from(stored);
                        counts.skipped += usize::from(!stored);
                        if let Err(e) = progress(pool, run_id, stored).await {
                            failure = Some(e);
                            rx.close();
                        }
                    }
                    Err(e) => {
                        failure = Some(e);
                        rx.close();
                    }
                }
            }
            (counts, failure)
        };
        let (reach, (counts, failure)) = tokio::join!(pulling, storing);
        pulled.found += counts.found;
        pulled.stored += counts.stored;
        pulled.skipped += counts.skipped;
        if let Some(e) = failure {
            tracing::warn!(connector = %id, round, error = %e, "connector sync: storing failed");
            finish(pool, run_id, id, Err(&e.to_string())).await?;
            return Err(e);
        }
        let reach = match reach {
            Ok(r) => r,
            Err(e) => {
                tracing::warn!(connector = %id, kind = %row.kind, round, error = %e, "connector sync: pull failed");
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
        tracing::info!(connector = %id, round, found = counts.found, stored = counts.stored, ?reach, "connector sync: round");
        if reach == Reach::Complete {
            complete = true;
            break;
        }
    }
    finish(pool, run_id, id, Ok((&pulled, complete))).await?;
    Ok(pulled)
}

/// Count one stored or skipped document on the run row, so a watcher sees
/// the pull move while it is still going.
async fn progress(pool: &PgPool, run_id: Uuid, stored: bool) -> Result<(), StoreError> {
    sqlx::query(
        "update finance.connector_runs
            set found = found + 1,
                stored = stored + case when $2 then 1 else 0 end,
                skipped = skipped + case when $2 then 0 else 1 end
          where id = $1",
    )
    .bind(run_id)
    .bind(stored)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

/// Close every run that never finished. For start-up: the process that was
/// pulling is gone, and a run it left open would block its connector for
/// half an hour.
///
/// # Errors
/// The database.
pub async fn close_orphans(pool: &PgPool) -> Result<u64, StoreError> {
    let done = sqlx::query(
        "update finance.connector_runs set finished_at = now(), outcome = 'error', error = 'interrupted by restart'
          where finished_at is null",
    )
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(done.rows_affected())
}

/// Every provider id this connector has stored a source for.
async fn seen_refs(
    pool: &PgPool,
    id: Uuid,
) -> Result<std::collections::HashSet<String>, StoreError> {
    let rows: Vec<(String,)> =
        sqlx::query_as("select external_ref from finance.document_sources where connector_id = $1")
            .bind(id)
            .fetch_all(pool)
            .await
            .map_err(map_err)?;
    Ok(rows.into_iter().map(|(r,)| r).collect())
}

/// Store one thing a pull found: new bytes become a document, known bytes
/// gain a source, and a provider id seen before is skipped. `true` when a
/// document was stored.
async fn store_one(
    pool: &PgPool,
    party: Uuid,
    connector: Uuid,
    f: &Found,
    seen: &(dyn for<'a> Fn(&'a str) -> bool + Sync),
) -> Result<bool, StoreError> {
    if seen(&f.external_ref) {
        return Ok(false);
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
    let (document_id, stored) = if let Some((d,)) = existing {
        (d, false)
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
        (d, true)
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
    Ok(stored)
}

/// Record how a run ended, and on the connector what its last sync was.
async fn finish(
    pool: &PgPool,
    run_id: Uuid,
    id: Uuid,
    outcome: Result<(&Pulled, bool), &str>,
) -> Result<(), StoreError> {
    match outcome {
        Ok((p, complete)) => {
            // A partial run stored what it fetched but did not reach the
            // present, so the watermark stays put and the next run resumes
            // from the same window, skipping what this one stored.
            let status = if complete { "ok" } else { "partial" };
            sqlx::query(
                "update finance.connector_runs set finished_at = now(), outcome = $5, found = $2, stored = $3, skipped = $4
                  where id = $1",
            )
            .bind(run_id)
            .bind(i32::try_from(p.found).unwrap_or(i32::MAX))
            .bind(i32::try_from(p.stored).unwrap_or(i32::MAX))
            .bind(i32::try_from(p.skipped).unwrap_or(i32::MAX))
            .bind(status)
            .execute(pool)
            .await
            .map_err(map_err)?;
            sqlx::query(
                "update finance.connectors
                    set last_sync_at = case when $2 then now() else last_sync_at end,
                        last_sync_status = $3, last_sync_error = null, updated_at = now()
                  where id = $1",
            )
            .bind(id)
            .bind(complete)
            .bind(status)
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
    change: &Change,
) -> Result<ConnectorRow, StoreError> {
    let row = get(pool, access, id).await?;
    let mut tx = pool.begin().await.map_err(map_err)?;
    if let Some(config) = &change.config {
        sqlx::query("update finance.connectors set config = $2, updated_at = now() where id = $1")
            .bind(id)
            .bind(config)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    }
    if let Some(label) = &change.label {
        sqlx::query("update finance.connectors set label = $2, updated_at = now() where id = $1")
            .bind(id)
            .bind(label)
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
    }
    if let Some(party) = change.party
        && party != row.party_id
    {
        // A party outside the grant is not found, the same as a row would be.
        access.require(PartyId(party), "party_id")?;
        sqlx::query(
            "update finance.connectors set party_id = $2, updated_at = now() where id = $1",
        )
        .bind(id)
        .bind(party)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
        // Its documents go with it -- those it alone pulled. One that
        // another connector also sourced stays where it is.
        sqlx::query(
            "update finance.documents d set party_id = $2
              where d.id in (select document_id from finance.document_sources where connector_id = $1)
                and not exists (select 1 from finance.document_sources s
                                 where s.document_id = d.id and s.connector_id is distinct from $1)",
        )
        .bind(id)
        .bind(party)
        .execute(&mut *tx)
        .await
        .map_err(map_err)?;
    }
    tx.commit().await.map_err(map_err)?;
    get(pool, access, id).await
}

/// What `configure` may change; `None` leaves a field alone.
#[derive(Debug, Default)]
pub struct Change {
    /// The kind's settings.
    pub config: Option<Value>,
    /// The party the connector and its documents belong to.
    pub party: Option<Uuid>,
    /// The display label.
    pub label: Option<String>,
}

/// Every connector in the view with its latest run, for the watch stream
/// and its diff.
///
/// # Errors
/// The database.
pub async fn snapshot(
    pool: &PgPool,
    view: &Access,
) -> Result<Vec<(ConnectorRow, Option<RunRow>)>, StoreError> {
    let connectors = list(pool, view).await?;
    if connectors.is_empty() {
        return Ok(Vec::new());
    }
    let ids: Vec<Uuid> = connectors.iter().map(|c| c.id).collect();
    let runs: Vec<RunRow> = sqlx::query_as(
        "select distinct on (connector_id)
                id, connector_id, started_at, finished_at, trigger, outcome, found, stored, skipped, error
           from finance.connector_runs where connector_id = any($1)
          order by connector_id, started_at desc",
    )
    .bind(&ids)
    .fetch_all(pool)
    .await
    .map_err(map_err)?;
    Ok(connectors
        .into_iter()
        .map(|c| {
            let run = runs.iter().find(|r| r.connector_id == c.id).cloned();
            (c, run)
        })
        .collect())
}
