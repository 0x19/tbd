//! Postgres.

use async_trait::async_trait;
use sqlx::PgPool;
use tbd_db::{Access, DbError, map_err};
use uuid::Uuid;

use super::{Store, Transaction, page_size, view};

/// The real store.
#[derive(Debug, Clone)]
pub struct PgStore {
    pool: PgPool,
}

impl PgStore {
    /// Wrap a pool.
    #[must_use]
    pub const fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[async_trait]
impl Store for PgStore {
    fn kind(&self) -> &'static str {
        "postgres"
    }

    async fn transactions(
        &self,
        access: &Access,
        narrow_to: &[Uuid],
        limit: u32,
    ) -> Result<Vec<Transaction>, DbError> {
        let view = view(access, narrow_to);
        // A caller with no grants reads as an empty page, never as an error and
        // never as an unfiltered one.
        if view.is_empty() {
            return Ok(Vec::new());
        }
        sqlx::query_as::<_, Transaction>(
            "select id, account_id, party_id, status, amount_minor, currency, scale,
                    booking_date, counterparty_name, remittance
               from finance.bank_transactions
              where party_id = any($1)
              order by booking_date desc nulls last, id desc
              limit $2",
        )
        .bind(view.party_ids())
        .bind(i64::from(page_size(limit)))
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)
    }
}
