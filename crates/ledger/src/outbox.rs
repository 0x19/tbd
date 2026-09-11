//! The outbox drain: lease a batch of unpublished events from the store, hand
//! them to a [`Publisher`], ack what was shipped. At least once: a drainer
//! that dies between publish and ack leaves the lease to expire and the batch
//! is shipped again; consumers dedupe on `event_id`. Two replicas never hold
//! the same rows (the store claims with `SKIP LOCKED`).

use std::{sync::Mutex, time::Duration};

use tbd_common::metrics::names;
use tokio_util::sync::CancellationToken;

use crate::store::{EventKind, OutboxEvent, Store, StoreError};

/// Why a batch could not be shipped. The batch stays claimed until its lease
/// expires and is then retried.
#[derive(Debug, thiserror::Error)]
pub enum PublishError {
    /// The sink could not be reached or refused the batch.
    #[error("publish: {0}")]
    Sink(String),
}

/// Where events go.
#[async_trait::async_trait]
pub trait Publisher: Send + Sync + 'static {
    /// A short label for logs and metrics.
    fn name(&self) -> &'static str;

    /// Ship a batch. All or nothing: on `Err` nothing is acked.
    async fn publish(&self, events: &[OutboxEvent]) -> Result<(), PublishError>;
}

/// Counts and keeps the last events instead of shipping them: what tests and
/// chaos stacks use, and what runs when no sink is configured (analytics off).
#[derive(Debug, Default)]
pub struct RecordingPublisher {
    seen: Mutex<Vec<OutboxEvent>>,
    total: std::sync::atomic::AtomicU64,
}

impl RecordingPublisher {
    /// How many events were handed over in total.
    pub fn total(&self) -> u64 {
        self.total.load(std::sync::atomic::Ordering::Relaxed)
    }

    /// The most recent events (at most [`RecordingPublisher::KEEP`]).
    pub fn events(&self) -> Vec<OutboxEvent> {
        self.seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// How many events are kept for inspection.
    pub const KEEP: usize = 10_000;
}

#[async_trait::async_trait]
impl Publisher for RecordingPublisher {
    fn name(&self) -> &'static str {
        "recording"
    }

    async fn publish(&self, events: &[OutboxEvent]) -> Result<(), PublishError> {
        self.total
            .fetch_add(events.len() as u64, std::sync::atomic::Ordering::Relaxed);
        let mut seen = self
            .seen
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        seen.extend(events.iter().cloned());
        if seen.len() > Self::KEEP {
            let drop = seen.len() - Self::KEEP;
            seen.drain(..drop);
        }
        Ok(())
    }
}

/// The drain loop.
pub struct Drainer<S, P> {
    store: S,
    publisher: P,
    batch: u32,
    period: Duration,
    lease: Duration,
}

impl<S: Store, P: Publisher> Drainer<S, P> {
    /// A drainer over `store` into `publisher`.
    pub fn new(store: S, publisher: P, batch: u32, period: Duration, lease: Duration) -> Self {
        Self {
            store,
            publisher,
            batch: batch.max(1),
            period,
            lease,
        }
    }

    /// One pass: claim, publish, ack. Returns how many events were acked.
    ///
    /// # Errors
    /// The store could not be reached or the publisher refused the batch; the
    /// claimed rows stay leased and come back after the lease.
    pub async fn tick(&self) -> Result<usize, DrainError> {
        let events = self
            .store
            .claim_events(self.batch, self.lease)
            .await
            .map_err(DrainError::Store)?;
        if events.is_empty() {
            return Ok(0);
        }
        let started = std::time::Instant::now();
        let published = self.publisher.publish(&events).await;
        metrics::histogram!(names::LEDGER_OUTBOX_PUBLISH_DURATION)
            .record(started.elapsed().as_secs_f64());
        match published {
            Ok(()) => {
                metrics::counter!(names::LEDGER_OUTBOX_BATCHES_TOTAL, "status" => "ok")
                    .increment(1);
            }
            Err(e) => {
                metrics::counter!(names::LEDGER_OUTBOX_BATCHES_TOTAL, "status" => "error")
                    .increment(1);
                return Err(DrainError::Publish(e));
            }
        }
        for kind in [
            EventKind::FactRecorded,
            EventKind::FactRetracted,
            EventKind::SubjectErased,
        ] {
            let n = events.iter().filter(|e| e.kind == kind).count();
            if n > 0 {
                metrics::counter!(names::LEDGER_OUTBOX_EVENTS_TOTAL, "kind" => kind.as_str())
                    .increment(n as u64);
            }
        }
        let ids: Vec<uuid::Uuid> = events.iter().map(|e| e.event_id).collect();
        self.store
            .ack_events(&ids)
            .await
            .map_err(DrainError::Store)?;
        // The batch's lag is its oldest event's age at the moment it is acked.
        if let Some(oldest) = events.iter().map(|e| e.recorded_at).min() {
            let lag = (chrono::Utc::now() - oldest).to_std().unwrap_or_default();
            metrics::histogram!(names::LEDGER_OUTBOX_LAG_SECONDS).record(lag.as_secs_f64());
        }
        Ok(events.len())
    }

    /// Drain until cancelled: a full batch is followed by another pass at
    /// once, an empty one by a sleep of `period`, an error by a sleep too.
    pub async fn run(self, cancel: CancellationToken) {
        tracing::info!(
            sink = self.publisher.name(),
            batch = self.batch,
            "outbox drainer started"
        );
        loop {
            let wait = match self.tick().await {
                Ok(n) if n >= self.batch as usize => Duration::ZERO,
                Ok(_) => self.period,
                Err(error) => {
                    tracing::warn!(%error, sink = self.publisher.name(), "outbox batch not shipped; will retry");
                    self.period.max(Duration::from_secs(1))
                }
            };
            tokio::select! {
                () = cancel.cancelled() => break,
                () = tokio::time::sleep(wait) => {}
            }
        }
        tracing::info!("outbox drainer stopped");
    }
}

/// Why a pass failed.
#[derive(Debug, thiserror::Error)]
pub enum DrainError {
    /// The store.
    #[error(transparent)]
    Store(StoreError),
    /// The sink.
    #[error(transparent)]
    Publish(PublishError),
}

#[async_trait::async_trait]
impl<P: Publisher + ?Sized> Publisher for std::sync::Arc<P> {
    fn name(&self) -> &'static str {
        (**self).name()
    }
    async fn publish(&self, events: &[OutboxEvent]) -> Result<(), PublishError> {
        (**self).publish(events).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Envelope, NewFact, ScopeId, Source, memory::MemoryStore};

    fn fact() -> NewFact {
        NewFact {
            path: "profile.name".into(),
            source: Source::Declared,
            value: Envelope::plaintext(&serde_json::json!("x")),
            origin: Envelope::plaintext(&serde_json::json!({})),
            confidence: Some(1.0),
            counterparty_id: None,
            observed_at: chrono::Utc::now(),
            expires_at: None,
            consent: vec![ScopeId::new("self")],
            stub: false,
            idempotency_key: None,
        }
    }

    struct Refusing;

    #[async_trait::async_trait]
    impl Publisher for Refusing {
        fn name(&self) -> &'static str {
            "refusing"
        }
        async fn publish(&self, _: &[OutboxEvent]) -> Result<(), PublishError> {
            Err(PublishError::Sink("down".into()))
        }
    }

    #[tokio::test]
    async fn a_pass_ships_and_acks_and_a_refused_batch_is_retried_after_the_lease() {
        let store = std::sync::Arc::new(MemoryStore::new());
        let s = uuid::Uuid::now_v7();
        store
            .append(s, fact())
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        store
            .append(s, fact())
            .await
            .unwrap_or_else(|e| panic!("{e}"));
        let refusing = Drainer::new(
            store.clone(),
            Refusing,
            10,
            Duration::from_millis(1),
            Duration::ZERO,
        );
        assert!(refusing.tick().await.is_err());
        let recording = std::sync::Arc::new(RecordingPublisher::default());
        let drainer = Drainer::new(
            store.clone(),
            recording.clone(),
            10,
            Duration::from_millis(1),
            Duration::from_secs(30),
        );
        assert_eq!(
            drainer.tick().await.unwrap_or(0),
            2,
            "the zero lease let the refused batch come back"
        );
        assert_eq!(drainer.tick().await.unwrap_or(9), 0);
        assert_eq!(recording.total(), 2);
        assert_eq!(recording.events().len(), 2);
    }
}
