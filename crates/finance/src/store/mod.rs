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
#[derive(Debug, Clone, PartialEq, Eq, sqlx::FromRow)]
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
