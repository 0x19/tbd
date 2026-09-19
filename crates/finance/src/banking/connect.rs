//! Linking a bank: the consent flow, from "start" to "these accounts are ours".
//!
//! Two functions, one per leg of the redirect. Between them a person logs in
//! at the bank, and the only thing that ties the two legs together is
//! `state`: ours, random, single-use, stored on the connection row before the
//! person is sent anywhere. The callback presents `state` and `code`; the code
//! is exchanged only if the state names a pending connection **owned by the
//! same party**. A code cannot be redeemed into somebody else's connection,
//! and a state cannot be replayed, because completing it consumes it.
//!
//! The code is never stored, never logged, and never part of a URL on our
//! side. It appears in exactly one place: the body of one POST to the bank.

use base64::{Engine, engine::general_purpose::URL_SAFE_NO_PAD};
use chrono::{Days, Utc};
use sqlx::PgPool;
use tbd_db::{DbError, map_err};
use uuid::Uuid;

use super::{AuthorizationRequest, Provider, ProviderError};
use crate::import::upsert_account;

/// Erste's `maximum_consent_validity` is 15552000s, 180 days. A day inside
/// it, so a slow authorization cannot land past the limit and be refused.
const CONSENT_DAYS: u64 = 179;

/// What `start` hands back: the URL to visit, and the row to complete later.
#[derive(Debug, Clone)]
pub struct Started {
    /// Our connection row.
    pub connection_id: Uuid,
    /// Where the person logs in.
    pub url: String,
    /// The state the callback must present. Not secret from the person, but
    /// unguessable by anyone else.
    pub state: String,
}

/// What `complete` did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Completed {
    /// The connection, now `authorized`.
    pub connection_id: Uuid,
    /// Accounts the consent covers, created or matched, in provider order.
    pub accounts: Vec<Uuid>,
}

/// Why a completion was refused. Each is deliberately indistinguishable from
/// the outside except to the party that owns the connection.
#[derive(Debug, thiserror::Error)]
pub enum ConnectError {
    /// No pending connection for that state and party. Also the answer for
    /// a state that belongs to a different party, on purpose: a
    /// "forbidden" would confirm the state exists.
    #[error("no pending connection for that state")]
    NotFound,
    /// The state was already completed. Replayed callbacks land here.
    #[error("that authorization was already completed")]
    AlreadyCompleted,
    /// The bank refused the code. The connection is marked `failed`.
    #[error("bank refused: {0}")]
    Provider(#[from] ProviderError),
    /// The database.
    #[error(transparent)]
    Db(#[from] DbError),
}

/// Begin a consent for a party at a bank.
///
/// Writes the `pending` row first, then asks the bank for a URL, so a crash
/// between the two leaves a pending row and no orphaned authorization at the
/// bank that we could never match.
///
/// # Errors
/// The database, or the bank refuses to start an authorization.
pub async fn start<P: Provider + ?Sized>(
    pool: &PgPool,
    provider: &P,
    party_id: Uuid,
    psu_type: &str,
    aspsp_name: &str,
    aspsp_country: &str,
    redirect_url: &str,
) -> Result<Started, ConnectError> {
    let connection_id = Uuid::new_v4();
    let state = fresh_state();
    let valid_until = Utc::now()
        .checked_add_days(Days::new(CONSENT_DAYS))
        .unwrap_or_else(Utc::now);
    sqlx::query(
        "insert into finance.connections
            (id, party_id, provider, psu_type, aspsp_name, aspsp_country, status, state, valid_until)
         values ($1, $2, $3, $4, $5, $6, 'pending', $7, $8)",
    )
    .bind(connection_id)
    .bind(party_id)
    .bind(provider.name())
    .bind(psu_type)
    .bind(aspsp_name)
    .bind(aspsp_country)
    .bind(&state)
    .bind(valid_until)
    .execute(pool)
    .await
    .map_err(map_err)?;

    let request = AuthorizationRequest {
        aspsp_name: aspsp_name.to_owned(),
        aspsp_country: aspsp_country.to_owned(),
        psu_type: psu_type.to_owned(),
        redirect_url: redirect_url.to_owned(),
        state: state.clone(),
        valid_until,
    };
    let auth = match provider.start_authorization(&request).await {
        Ok(a) => a,
        Err(e) => {
            mark_failed(pool, connection_id, "authorization refused").await?;
            return Err(e.into());
        }
    };
    Ok(Started {
        connection_id,
        url: auth.url,
        state,
    })
}

/// Finish a consent with what the bank's redirect carried.
///
/// `party_id` is the caller's, from the verified principal -- never from the
/// callback. A state that does not belong to this party is a `NotFound`.
///
/// # Errors
/// See [`ConnectError`].
pub async fn complete<P: Provider + ?Sized>(
    pool: &PgPool,
    provider: &P,
    party_id: Uuid,
    state: &str,
    code: &str,
) -> Result<Completed, ConnectError> {
    // Lock the row so two callbacks with the same state cannot both exchange
    // the code. The second sees `authorized` and is told so.
    let mut tx = pool.begin().await.map_err(map_err)?;
    let row: Option<(Uuid, String)> = sqlx::query_as(
        "select id, status from finance.connections
          where state = $1 and party_id = $2
            for update",
    )
    .bind(state)
    .bind(party_id)
    .fetch_optional(&mut *tx)
    .await
    .map_err(map_err)?;
    // `failed` and `expired` are not-found too: a state is redeemable once,
    // while it is pending, and never again.
    let connection_id = match row.as_ref().map(|(id, s)| (id, s.as_str())) {
        Some((id, "pending")) => *id,
        Some((_, "authorized")) => return Err(ConnectError::AlreadyCompleted),
        _ => return Err(ConnectError::NotFound),
    };

    // The exchange happens under the lock: it is one POST, it is never
    // retried, and holding the row for its duration is what makes a replayed
    // callback wait and then see `authorized`.
    let session = match provider.create_session(code).await {
        Ok(s) => s,
        Err(e) => {
            sqlx::query(
                "update finance.connections
                    set status = 'failed', failure = $2, updated_at = now()
                  where id = $1",
            )
            .bind(connection_id)
            .bind("code exchange refused")
            .execute(&mut *tx)
            .await
            .map_err(map_err)?;
            tx.commit().await.map_err(map_err)?;
            return Err(e.into());
        }
    };

    sqlx::query(
        "update finance.connections
            set status = 'authorized', session_id = $2, valid_until = coalesce($3, valid_until),
                authorized_at = now(), updated_at = now()
          where id = $1",
    )
    .bind(connection_id)
    .bind(&session.session_id)
    .bind(session.valid_until)
    .execute(&mut *tx)
    .await
    .map_err(map_err)?;
    tx.commit().await.map_err(map_err)?;

    // Accounts are matched on (party, iban, currency), so re-linking the same
    // bank after a consent expires finds the existing rows -- and their
    // history -- rather than starting a second copy of each.
    let mut accounts = Vec::with_capacity(session.accounts.len());
    for seen in &session.accounts {
        let id = upsert_account(pool, party_id, seen).await?;
        sqlx::query(
            "update finance.accounts
                set connection_id = $2, provider_uid = $3, sync_enabled = true
              where id = $1",
        )
        .bind(id)
        .bind(connection_id)
        .bind(&seen.uid)
        .execute(pool)
        .await
        .map_err(map_err)?;
        accounts.push(id);
    }
    Ok(Completed {
        connection_id,
        accounts,
    })
}

async fn mark_failed(pool: &PgPool, id: Uuid, why: &str) -> Result<(), DbError> {
    sqlx::query(
        "update finance.connections set status = 'failed', failure = $2, updated_at = now()
          where id = $1",
    )
    .bind(id)
    .bind(why)
    .execute(pool)
    .await
    .map_err(map_err)?;
    Ok(())
}

/// 32 random bytes, URL-safe. Unguessable is the whole requirement.
fn fresh_state() -> String {
    let bytes: [u8; 32] = rand::random();
    URL_SAFE_NO_PAD.encode(bytes)
}
