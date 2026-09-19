//! In memory, for stacks that have no database.
//!
//! This exists so access control can be proven *continuously* -- by a chaos
//! scenario driving a running service under load and fault -- rather than only
//! by a test that constructs a Postgres. It applies the same rule through the
//! same [`view`] helper, and `tests/it/conformance.rs` runs one suite against
//! this and Postgres so the two cannot drift.

use std::sync::{Arc, RwLock};

use async_trait::async_trait;
use tbd_db::{Access, DbError};
use uuid::Uuid;

use super::{Store, Transaction, TransactionFilter, page_size, view};

/// Transactions held in memory.
#[derive(Debug, Clone, Default)]
pub struct MemoryStore {
    rows: Arc<RwLock<Vec<Transaction>>>,
}

impl MemoryStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Seed it.
    ///
    /// A poisoned lock is recovered rather than propagated: the data behind it
    /// is a plain `Vec` that cannot be left half-written, and a test store must
    /// not be able to take a service down.
    pub fn insert(&self, rows: impl IntoIterator<Item = Transaction>) {
        let mut guard = self
            .rows
            .write()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        guard.extend(rows);
        // Newest first, the order the query promises.
        guard.sort_by(|a, b| b.booking_date.cmp(&a.booking_date).then(b.id.cmp(&a.id)));
    }

    /// How many rows it holds, ignoring every grant. Tests only.
    #[must_use]
    pub fn len(&self) -> usize {
        self.rows
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .len()
    }

    /// Whether it holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

#[async_trait]
impl Store for MemoryStore {
    fn kind(&self) -> &'static str {
        "memory"
    }

    async fn transactions(
        &self,
        access: &Access,
        narrow_to: &[Uuid],
        filter: &TransactionFilter,
        limit: u32,
    ) -> Result<Vec<Transaction>, DbError> {
        let view = view(access, narrow_to);
        if view.is_empty() {
            return Ok(Vec::new());
        }
        let allowed = view.party_ids();
        let guard = self
            .rows
            .read()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        Ok(guard
            .iter()
            .filter(|t| allowed.contains(&t.party_id) && filter.matches(t))
            .skip(filter.offset as usize)
            .take(page_size(limit) as usize)
            .cloned()
            .collect())
    }
}
