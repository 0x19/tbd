//! The gapless counter.
//!
//! One row per issuer, year and series (premises and device), incremented
//! under its row lock inside the transaction that approves the invoice. A
//! Postgres sequence is explicitly the wrong tool: `nextval` is
//! non-transactional, so an approval that rolls back after taking a number
//! leaves a permanent hole -- and Croatian numbering is gapless per year by
//! law.

use sqlx::{Postgres, Transaction};
use tbd_db::{DbError, map_err};
use uuid::Uuid;

/// One numbering stream: an issuer's year on one premises and device.
#[derive(Debug, Clone, Copy)]
pub struct Series<'a> {
    /// The issuing party.
    pub party: Uuid,
    /// The year the number belongs to: the year of approval.
    pub year: i32,
    /// *Oznaka poslovnog prostora.*
    pub premises: &'a str,
    /// *Oznaka naplatnog uređaja.*
    pub device: &'a str,
}

/// The number the next approval would take. A peek: no lock, no allocation.
///
/// # Errors
/// The database.
pub async fn peek(pool: &sqlx::PgPool, series: &Series<'_>) -> Result<i32, DbError> {
    let row: Option<(i32,)> = sqlx::query_as(
        "select next_ordinal from finance.invoice_numbers
          where party_id = $1 and year = $2 and premises = $3 and device = $4",
    )
    .bind(series.party)
    .bind(series.year)
    .bind(series.premises)
    .bind(series.device)
    .fetch_optional(pool)
    .await
    .map_err(map_err)?;
    Ok(row.map_or(1, |(n,)| n))
}

/// Take the next number, inside `tx`. The row lock serialises concurrent
/// approvals of the same year; the caller's commit is what makes it final.
///
/// # Errors
/// The database.
pub async fn allocate(
    tx: &mut Transaction<'_, Postgres>,
    series: &Series<'_>,
) -> Result<i32, DbError> {
    sqlx::query(
        "insert into finance.invoice_numbers (party_id, year, premises, device) values ($1, $2, $3, $4)
         on conflict (party_id, year, premises, device) do nothing",
    )
    .bind(series.party)
    .bind(series.year)
    .bind(series.premises)
    .bind(series.device)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    let (ordinal,): (i32,) = sqlx::query_as(
        "update finance.invoice_numbers
            set next_ordinal = next_ordinal + 1
          where party_id = $1 and year = $2 and premises = $3 and device = $4
          returning next_ordinal - 1",
    )
    .bind(series.party)
    .bind(series.year)
    .bind(series.premises)
    .bind(series.device)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_err)?;
    Ok(ordinal)
}
