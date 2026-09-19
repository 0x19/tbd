//! Postgres.

use async_trait::async_trait;
use sqlx::PgPool;
use tbd_db::{Access, DbError, map_err};
use uuid::Uuid;

use super::{Store, Transaction, TransactionFilter, page_size, view};

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
        filter: &TransactionFilter,
        limit: u32,
    ) -> Result<Vec<Transaction>, DbError> {
        let view = view(access, narrow_to);
        // A caller with no grants reads as an empty page, never as an error and
        // never as an unfiltered one.
        if view.is_empty() {
            return Ok(Vec::new());
        }
        // Every optional filter is a `$n is null or ...` clause, so one
        // statement serves every combination and the planner sees one shape.
        // `category`: $4 says whether to filter at all, $5 the id (null for
        // "uncategorised").
        let (filter_category, category_id) = match filter.category {
            None => (false, None),
            Some(id) => (true, id),
        };
        sqlx::query_as::<_, Transaction>(
            "select t.id, t.account_id, t.party_id, t.status, t.amount_minor, t.currency,
                    t.scale, t.booking_date, t.counterparty_name, t.remittance,
                    t.value_date, t.counterparty_iban, t.category_id,
                    c.name as category, t.category_source, t.internal,
                    t.reference_number, t.entry_reference, t.category_rule_id,
                    t.categorised_at, a.name as account_name,
                    case when $9::uuid is null then null else t.raw::text end as raw
               from finance.transactions_enriched t
               join finance.accounts a on a.id = t.account_id
               left join finance.categories c on c.id = t.category_id
              where t.party_id = any($1)
                and ($3::text is null or to_char(t.booking_date, 'YYYY-MM') = $3)
                and (not $4 or t.category_id is not distinct from $5)
                and ($6::uuid is null or t.account_id = $6)
                and ($7::text is null
                     or coalesce(t.counterparty_name, '') || ' ' || coalesce(t.remittance, '')
                        ilike '%' || $7 || '%')
                and ($9::uuid is null or t.id = $9)
              order by t.booking_date desc nulls last, t.id desc
              limit $2 offset $8",
        )
        .bind(view.party_ids())
        .bind(i64::from(page_size(limit)))
        .bind(filter.month.as_deref())
        .bind(filter_category)
        .bind(category_id)
        .bind(filter.account_id)
        .bind(filter.search.as_deref())
        .bind(i64::from(filter.offset))
        .bind(filter.id)
        .fetch_all(&self.pool)
        .await
        .map_err(map_err)
    }
}
