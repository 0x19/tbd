//! [`Faulty`]: the store decorator behind chaos's `set_store_behavior`. Where
//! the request adapter's fault handle refuses a call before anything runs,
//! this one sits between the service and the store and fails like a database
//! does: a **read** fails before it runs; a **write** runs, commits, and then
//! fails, so the caller loses the acknowledgement of something that stands.
//! That lost-ack case is the one an adapter-level fault cannot produce, and the
//! one idempotency keys exist for.
//!
//! Background work (the sweeper's erasures and purge, the drainer's claim and
//! ack) is passed through unfaulted: those loops retry on their own clocks and a
//! fault there proves nothing about the contract a client sees.

use std::sync::Arc;

use async_trait::async_trait;
use metrics::counter;
use tbd_common::{
    fault::{ErrorKind, Fault, FaultHandle},
    metrics::names,
};

use super::{
    Appended, Envelope, Erasure, ErasureExecuted, Fact, NewFact, OutboxEvent, Page, Query, Source,
    Store, StoreError, StoreKind, Subject, SubjectId,
};

/// A store whose request-path calls may fail on command.
pub struct Faulty<S> {
    inner: S,
    fault: FaultHandle,
}

impl<S> Faulty<S> {
    /// Wrap `inner`; `fault` is the handle chaos flips.
    pub fn new(inner: S, fault: FaultHandle) -> Self {
        Self { inner, fault }
    }

    /// The wrapped store.
    pub fn inner(&self) -> &S {
        &self.inner
    }

    /// The handle.
    pub fn fault(&self) -> &FaultHandle {
        &self.fault
    }
}

/// An injected fault as the store error a real outage would produce.
fn injected(fault: Fault) -> StoreError {
    counter!(names::LEDGER_STORE_FAULTS_INJECTED_TOTAL, "kind" => format!("{:?}", fault.kind).to_lowercase())
        .increment(1);
    match fault.kind {
        ErrorKind::Internal => StoreError::Internal(fault.message),
        ErrorKind::Unavailable | ErrorKind::Overloaded | ErrorKind::Timeout => {
            StoreError::Unavailable {
                source: Box::new(fault),
            }
        }
    }
}

impl<S: Store> Faulty<S> {
    /// A read: the fault first, then the call.
    async fn before<T>(
        &self,
        f: impl Future<Output = Result<T, StoreError>>,
    ) -> Result<T, StoreError> {
        self.fault.apply().await.map_err(injected)?;
        f.await
    }

    /// A write: the call first; when it succeeded, the fault may still eat the
    /// acknowledgement. A failed write is reported as it failed.
    async fn after<T>(
        &self,
        f: impl Future<Output = Result<T, StoreError>>,
    ) -> Result<T, StoreError> {
        let out = f.await?;
        self.fault.apply().await.map_err(injected)?;
        Ok(out)
    }
}

#[async_trait]
impl<S: Store> Store for Faulty<S> {
    fn kind(&self) -> StoreKind {
        self.inner.kind()
    }

    fn as_any(&self) -> &dyn std::any::Any {
        self.inner.as_any()
    }

    async fn ping(&self) -> Result<(), StoreError> {
        self.before(self.inner.ping()).await
    }

    async fn subject(&self, id: SubjectId) -> Result<Option<Subject>, StoreError> {
        self.before(self.inner.subject(id)).await
    }

    async fn append(&self, subject: SubjectId, fact: NewFact) -> Result<Appended, StoreError> {
        self.after(self.inner.append(subject, fact)).await
    }

    async fn current(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        self.before(self.inner.current(subject, query)).await
    }

    async fn history(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        self.before(self.inner.history(subject, query)).await
    }

    async fn retract(
        &self,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: Envelope,
    ) -> Result<Fact, StoreError> {
        self.after(self.inner.retract(subject, path, source, origin))
            .await
    }

    async fn request_erasure(&self, subject: SubjectId) -> Result<Erasure, StoreError> {
        self.after(self.inner.request_erasure(subject)).await
    }

    async fn restore(&self, subject: SubjectId) -> Result<(), StoreError> {
        self.after(self.inner.restore(subject)).await
    }

    async fn execute_due_erasures(
        &self,
        grace: std::time::Duration,
        limit: u32,
    ) -> Result<Vec<ErasureExecuted>, StoreError> {
        self.inner.execute_due_erasures(grace, limit).await
    }

    async fn claim_events(
        &self,
        limit: u32,
        lease: std::time::Duration,
    ) -> Result<Vec<OutboxEvent>, StoreError> {
        self.inner.claim_events(limit, lease).await
    }

    async fn ack_events(&self, event_ids: &[uuid::Uuid]) -> Result<u64, StoreError> {
        self.inner.ack_events(event_ids).await
    }

    async fn purge_idempotency(&self, ttl: std::time::Duration) -> Result<u64, StoreError> {
        self.inner.purge_idempotency(ttl).await
    }
}

/// Wrap a shared store, so `serve_store` can decorate whatever it was given.
pub fn wrap(store: Arc<dyn Store>, fault: FaultHandle) -> Arc<dyn Store> {
    Arc::new(Faulty::new(store, fault))
}

#[cfg(test)]
mod tests {
    use tbd_common::fault::Behavior;

    use super::*;
    use crate::store::{Envelope, NewFact, ScopeId, memory::MemoryStore};

    fn fact() -> NewFact {
        NewFact {
            path: "profile.name".into(),
            source: Source::Declared,
            value: Envelope::plaintext(&serde_json::json!("Ada")),
            origin: Envelope::plaintext(&serde_json::json!({})),
            confidence: Some(0.9),
            counterparty_id: None,
            observed_at: chrono::Utc::now(),
            expires_at: None,
            consent: vec![ScopeId::new("self")],
            stub: false,
            idempotency_key: Some("k1".into()),
        }
    }

    fn query() -> Query {
        Query::under(vec![ScopeId::new("self")])
    }

    #[tokio::test]
    async fn a_write_under_a_fault_commits_and_loses_its_acknowledgement() {
        let store = Faulty::new(MemoryStore::new(), FaultHandle::default());
        let s = SubjectId::now_v7();
        store.fault.set(Behavior::Error {
            kind: ErrorKind::Unavailable,
            rate: 1.0,
            message: "db down".into(),
        });
        // The same bytes both times: the key must find the same content.
        let f = fact();
        // The write fails for the caller...
        assert!(matches!(
            store.append(s, f.clone()).await,
            Err(StoreError::Unavailable { .. })
        ));
        // ...and reads fail before they run.
        assert!(matches!(
            store.history(s, &query()).await,
            Err(StoreError::Unavailable { .. })
        ));
        store.fault.set(Behavior::Healthy);
        // ...but the commit stands: history holds it, and the key replays it.
        let page = store.history(s, &query()).await.unwrap();
        assert_eq!(page.items.len(), 1);
        let again = store.append(s, f).await.unwrap();
        assert!(again.replayed);
        assert_eq!(again.fact.id, page.items[0].id);
    }

    #[tokio::test]
    async fn internal_faults_are_internal_and_the_sweeper_is_never_faulted() {
        let store = Faulty::new(MemoryStore::new(), FaultHandle::default());
        store.fault.set(Behavior::Error {
            kind: ErrorKind::Internal,
            rate: 1.0,
            message: "bug".into(),
        });
        assert!(matches!(
            store.ping().await,
            Err(StoreError::Internal(m)) if m == "bug"
        ));
        assert!(
            store
                .execute_due_erasures(std::time::Duration::ZERO, 10)
                .await
                .is_ok()
        );
    }
}
