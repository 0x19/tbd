//! The gapless counter.
//!
//! One row per issuer and year, incremented under its row lock inside the
//! transaction that approves the invoice. A Postgres sequence is explicitly
//! the wrong tool: `nextval` is non-transactional, so an approval that rolls
//! back after taking a number leaves a permanent hole -- and Croatian
//! numbering is gapless per year by law.

use sqlx::{Postgres, Transaction};
use tbd_db::{DbError, map_err};
use uuid::Uuid;

/// The number the next approval would take. A peek: no lock, no allocation.
///
/// # Errors
/// The database.
pub async fn peek(pool: &sqlx::PgPool, party: Uuid, year: i32) -> Result<i32, DbError> {
    let row: Option<(i32,)> = sqlx::query_as(
        "select next_ordinal from finance.invoice_numbers where party_id = $1 and year = $2",
    )
    .bind(party)
    .bind(year)
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
    party: Uuid,
    year: i32,
) -> Result<i32, DbError> {
    sqlx::query(
        "insert into finance.invoice_numbers (party_id, year) values ($1, $2)
         on conflict (party_id, year) do nothing",
    )
    .bind(party)
    .bind(year)
    .execute(&mut **tx)
    .await
    .map_err(map_err)?;
    let (ordinal,): (i32,) = sqlx::query_as(
        "update finance.invoice_numbers
            set next_ordinal = next_ordinal + 1
          where party_id = $1 and year = $2
          returning next_ordinal - 1",
    )
    .bind(party)
    .bind(year)
    .fetch_one(&mut **tx)
    .await
    .map_err(map_err)?;
    Ok(ordinal)
}
