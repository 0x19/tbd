//! Reading transactions, always on behalf of somebody.
//!
//! There is no "read everything" entry point, by design. Every query takes an
//! [`Access`], and in production the only way to build one is from the subject
//! Envoy verified. A handler cannot accidentally widen its view because no
//! function exists that would let it.
//!
//! Two backends behind one trait:
//!
//! - [`PgStore`] is the real one. The database enforces the same rule
//!   underneath through row-level security (`migrations/0003_finance.sql`), so
//!   a query that somehow lost its filter returns nothing rather than
//!   everything. The filter here makes queries fast; the policy makes them safe.
//! - [`MemoryStore`] is what the chaos tool runs. Access control has to be
//!   provable *continuously*, against a running service, and a scenario cannot
//!   seed a Postgres. Without an in-memory backend the only proof would be a
//!   unit test, which is not the same as watching it hold under load and fault.
//!
//! The two must not drift. `tests/it/conformance.rs` runs one suite against
//! both, so a filter fixed in one and forgotten in the other fails the build.

mod memory;
mod pg;

use async_trait::async_trait;
use tbd_db::{Access, DbError};
use uuid::Uuid;

pub use memory::MemoryStore;
pub use pg::PgStore;

/// One transaction, as the service hands it out.
#[derive(Debug, Clone, Default, PartialEq, Eq, sqlx::FromRow)]
pub struct Transaction {
    /// Our id.
    pub id: Uuid,
    /// The account it belongs to.
    pub account_id: Uuid,
    /// Who owns it. Enforced equal to the account's party by a composite
    /// foreign key, so it cannot drift into somebody else's view.
    pub party_id: Uuid,
    /// `booked` or `pending`.
    pub status: String,
    /// Minor units, signed: negative is money out.
    pub amount_minor: i64,
    /// ISO 4217.
    pub currency: String,
    /// Decimal places in the currency.
    pub scale: i16,
    /// When the bank booked it.
    pub booking_date: Option<chrono::NaiveDate>,
    /// The other side, as the bank named it.
    pub counterparty_name: Option<String>,
    /// Free text. On Erste this is where an invoice number arrives.
    pub remittance: Option<String>,
    /// When the money moved, where that differs from the booking.
    #[sqlx(default)]
    pub value_date: Option<chrono::NaiveDate>,
    /// The other side's account, when the bank gave it.
    #[sqlx(default)]
    pub counterparty_iban: Option<String>,
    /// What it was categorised as, if anything.
    #[sqlx(default)]
    pub category_id: Option<Uuid>,
    /// That category's name.
    #[sqlx(default)]
    pub category: Option<String>,
    /// `declared` (a person) or `inferred` (a rule).
    #[sqlx(default)]
    pub category_source: Option<String>,
    /// A transfer between accounts the same person holds. Derived, never
    /// stored: true exactly when the counterparty is an account we hold.
    #[sqlx(default)]
    pub internal: bool,
    /// The structured reference, when the bank gave one.
    #[sqlx(default)]
    pub reference_number: Option<String>,
    /// The bank's own id for the entry.
    #[sqlx(default)]
    pub entry_reference: Option<String>,
    /// The rule that claimed it, for an inferred category.
    #[sqlx(default)]
    pub category_rule_id: Option<Uuid>,
    /// When it was last categorised.
    #[sqlx(default)]
    pub categorised_at: Option<chrono::DateTime<chrono::Utc>>,
    /// The account's name, for a person reading one row.
    #[sqlx(default)]
    pub account_name: Option<String>,
    /// The bank's record as JSON. Filled only when one row is asked for by
    /// id: a page of a hundred does not carry a hundred records.
    #[sqlx(default)]
    pub raw: Option<String>,
}

/// What a caller narrows a listing to, beyond the parties.
///
/// Every field optional; those set must all hold. None of them widens
/// anything -- the parties come from the grant, and these only cut within it.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TransactionFilter {
    /// `YYYY-MM`.
    pub month: Option<String>,
    /// `Some(None)` is "uncategorised"; `Some(Some(id))` is one category.
    pub category: Option<Option<Uuid>>,
    /// One account.
    pub account_id: Option<Uuid>,
    /// Case-insensitive substring of the counterparty or the remittance.
    pub search: Option<String>,
    /// Rows to skip, for paging.
    pub offset: u32,
    /// Exactly this row. Still subject to the grant: a row the caller may
    /// not read is simply not there.
    pub id: Option<Uuid>,
}

impl TransactionFilter {
    /// Whether a row passes. One implementation, used by the memory store and
    /// as the reference the SQL must agree with.
    #[must_use]
    pub fn matches(&self, t: &Transaction) -> bool {
        if let Some(id) = self.id
            && t.id != id
        {
            return false;
        }
        if let Some(month) = &self.month
            && t.booking_date
                .map(|d| d.format("%Y-%m").to_string())
                .as_deref()
                != Some(month)
        {
            return false;
        }
        if let Some(category) = &self.category
            && t.category_id != *category
        {
            return false;
        }
        if let Some(account) = self.account_id
            && t.account_id != account
        {
            return false;
        }
        if let Some(search) = &self.search {
            let needle = search.to_lowercase();
            let hay = format!(
                "{} {}",
                t.counterparty_name.as_deref().unwrap_or_default(),
                t.remittance.as_deref().unwrap_or_default()
            )
            .to_lowercase();
            if !hay.contains(&needle) {
                return false;
            }
        }
        true
    }
}

/// Largest page a caller may ask for.
pub const MAX_LIMIT: u32 = 500;
/// Page size when the caller does not say.
pub const DEFAULT_LIMIT: u32 = 100;

/// What the service reads through.
#[async_trait]
pub trait Store: Send + Sync + std::fmt::Debug + 'static {
    /// `postgres` or `memory`, for `Ping` and the logs.
    fn kind(&self) -> &'static str;

    /// Transactions visible to `access`, newest first.
    ///
    /// `narrow_to` is the caller's own selection -- a personal/business toggle,
    /// say. It intersects with the grant and can only shrink it: ids the caller
    /// has no grant for are dropped rather than refused, because refusing would
    /// confirm which of them exist.
    ///
    /// # Errors
    /// The backend is unreachable.
    async fn transactions(
        &self,
        access: &Access,
        narrow_to: &[Uuid],
        filter: &TransactionFilter,
        limit: u32,
    ) -> Result<Vec<Transaction>, DbError>;
}

/// The view a query runs under: the grant, narrowed by what the caller asked
/// for. Shared by both backends so the rule cannot be implemented twice.
#[must_use]
pub fn view(access: &Access, narrow_to: &[Uuid]) -> Access {
    if narrow_to.is_empty() {
        access.clone()
    } else {
        access.narrow(narrow_to)
    }
}

/// A page size that is always in range.
#[must_use]
pub fn page_size(asked: u32) -> u32 {
    if asked == 0 { DEFAULT_LIMIT } else { asked }.clamp(1, MAX_LIMIT)
}

#[cfg(test)]
mod tests {
    use super::{DEFAULT_LIMIT, MAX_LIMIT, page_size};

    #[test]
    fn a_page_size_is_always_between_one_and_the_maximum() {
        for asked in [0, 1, 50, MAX_LIMIT, MAX_LIMIT + 1, u32::MAX] {
            let got = page_size(asked);
            assert!((1..=MAX_LIMIT).contains(&got), "{asked} became {got}");
        }
        assert_eq!(page_size(0), DEFAULT_LIMIT);
    }
}
