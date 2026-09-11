//! The sweeper: executes erasures whose grace window has passed and purges
//! stale idempotency keys, every `[erasure] sweep_interval`. Safe on any
//! number of replicas: the store hands out due rows with `SKIP LOCKED`.

use std::time::Duration;

use tbd_common::metrics::names;
use tokio_util::sync::CancellationToken;

use crate::store::{Store, StoreError};

/// The sweep loop.
pub struct Sweeper<S> {
    store: S,
    grace: Duration,
    batch: u32,
    idempotency_ttl: Duration,
    interval: Duration,
}

impl<S: Store> Sweeper<S> {
    /// A sweeper over `store`.
    pub fn new(
        store: S,
        grace: Duration,
        batch: u32,
        idempotency_ttl: Duration,
        interval: Duration,
    ) -> Self {
        Self {
            store,
            grace,
            batch: batch.max(1),
            idempotency_ttl,
            interval,
        }
    }

    /// One pass. Returns `(erasures executed, idempotency keys purged)`.
    ///
    /// # Errors
    /// The store could not be reached.
    pub async fn tick(&self) -> Result<(usize, u64), StoreError> {
        let executed = self
            .store
            .execute_due_erasures(self.grace, self.batch)
            .await?;
        for e in &executed {
            tracing::info!(subject = %e.subject_id, tombstoned = e.tombstoned, "erasure executed");
        }
        if !executed.is_empty() {
            metrics::counter!(names::LEDGER_ERASURES_EXECUTED_TOTAL)
                .increment(executed.len() as u64);
        }
        let purged = self.store.purge_idempotency(self.idempotency_ttl).await?;
        Ok((executed.len(), purged))
    }

    /// Sweep until cancelled.
    pub async fn run(self, cancel: CancellationToken) {
        tracing::info!(grace = ?self.grace, interval = ?self.interval, "erasure sweeper started");
        loop {
            if let Err(error) = self.tick().await {
                tracing::warn!(%error, "sweep failed; will retry");
            }
            tokio::select! {
                () = cancel.cancelled() => break,
                () = tokio::time::sleep(self.interval) => {}
            }
        }
        tracing::info!("erasure sweeper stopped");
    }
}
