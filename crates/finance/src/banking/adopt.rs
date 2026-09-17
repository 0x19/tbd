//! Adopting a consent the prototype established.
//!
//! The two Erste consents were authorized by hand during the prototype and
//! are valid until March 2027. Redoing them through the connect flow would
//! work, but it would spend a bank login to learn what a file already says.
//! This reads the saved session once and records it as an authorized
//! connection. It is a migration of state, not a feature: from here on,
//! consents come through [`super::connect`].

use std::path::Path;

use chrono::{DateTime, Utc};
use serde_json::Value;
use sqlx::PgPool;
use tbd_db::{DbError, map_err};
use uuid::Uuid;

use crate::import::accounts_in;

/// What was adopted.
#[derive(Debug, Clone)]
pub struct Adopted {
    /// The connection row.
    pub connection_id: Uuid,
    /// Its status: `authorized` when the file carries a session id.
    pub status: String,
    /// When the consent lapses, as the file recorded it.
    pub valid_until: Option<DateTime<Utc>>,
    /// Accounts matched to already-imported rows and linked.
    pub linked: usize,
}

/// Record `sessions/<profile>.json` as a connection and link its accounts.
///
/// Idempotent on the session id: adopting the same file twice updates the
/// one connection rather than creating a second.
///
/// # Errors
/// The file is missing or malformed, or the database is unreachable.
pub async fn from_prototype(
    pool: &PgPool,
    dir: &Path,
    profile: &str,
    party_id: Uuid,
) -> Result<Adopted, DbError> {
    let path = dir.join("sessions").join(format!("{profile}.json"));
    let text = std::fs::read_to_string(&path).map_err(|e| DbError::Invalid {
        field: "dir",
        reason: format!("{}: {e}", path.display()),
    })?;
    let session: Value = serde_json::from_str(&text).map_err(|e| DbError::Invalid {
        field: "dir",
        reason: format!("{}: {e}", path.display()),
    })?;
    let session_id = session
        .get("session_id")
        .and_then(Value::as_str)
        .ok_or(DbError::Invalid {
            field: "session_id",
            reason: "missing from the session file".into(),
        })?;
    let valid_until = super::timestamp(session.pointer("/access/valid_until"));
    let aspsp_name = session
        .pointer("/aspsp/name")
        .and_then(Value::as_str)
        .unwrap_or("Erste & Steiermärkische Bank");
    let aspsp_country = session
        .pointer("/aspsp/country")
        .and_then(Value::as_str)
        .unwrap_or("HR");
    let psu_type = session
        .get("psu_type")
        .and_then(Value::as_str)
        .unwrap_or(profile);

    // One connection per session id, whatever it was called before.
    let existing: Option<(Uuid,)> = sqlx::query_as(
        "select id from finance.connections where party_id = $1 and session_id = $2",
    )
    .bind(party_id)
    .bind(session_id)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    let connection_id = if let Some((id,)) = existing {
        sqlx::query(
            "update finance.connections
                set status = 'authorized', valid_until = $2, updated_at = now()
              where id = $1",
        )
        .bind(id)
        .bind(valid_until)
        .execute(pool)
        .await
        .map_err(map_err)?;
        id
    } else {
        let id = Uuid::new_v4();
        sqlx::query(
            "insert into finance.connections
                (id, party_id, provider, psu_type, aspsp_name, aspsp_country, session_id,
                 status, state, valid_until, authorized_at)
             values ($1, $2, 'enablebanking', $3, $4, $5, $6, 'authorized', $7, $8, now())",
        )
        .bind(id)
        .bind(party_id)
        .bind(psu_type)
        .bind(aspsp_name)
        .bind(aspsp_country)
        .bind(session_id)
        // A state is required and unique; an adopted consent never had a
        // callback, so it gets one that no callback can present.
        .bind(format!("adopted:{session_id}"))
        .bind(valid_until)
        .execute(pool)
        .await
        .map_err(map_err)?;
        id
    };

    let mut linked = 0;
    for account in accounts_in(&session) {
        let n = sqlx::query(
            "update finance.accounts
                set connection_id = $3, sync_enabled = true
              where party_id = $1 and provider_uid = $2",
        )
        .bind(party_id)
        .bind(&account.uid)
        .bind(connection_id)
        .execute(pool)
        .await
        .map_err(map_err)?
        .rows_affected();
        linked += usize::try_from(n).unwrap_or(0);
    }
    Ok(Adopted {
        connection_id,
        status: "authorized".into(),
        valid_until,
        linked,
    })
}
