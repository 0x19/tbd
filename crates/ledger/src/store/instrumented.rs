//! [`Instrumented`]: the store decorator that records every operation's
//! duration and result, and the ledger's business counters (facts appended
//! by source, replays, retractions, envelope sizes, page sizes, erasures).
//! `build_store` wraps every backend in it, so the numbers do not depend on
//! which caller (the gRPC adapter, the sweeper, the drainer) made the call.

use std::time::{Duration, Instant};

use async_trait::async_trait;
use metrics::{counter, histogram};
use tbd_common::metrics::names;

use super::{
    Appended, Envelope, Erasure, ErasureExecuted, Fact, NewFact, OutboxEvent, Page, Query, Source,
    Store, StoreError, StoreKind, Subject, SubjectId,
};

/// A store that records metrics for every call and forwards it.
pub struct Instrumented<S>(pub S);

impl<S> Instrumented<S> {
    /// The wrapped store.
    pub fn inner(&self) -> &S {
        &self.0
    }
}

/// The `result` label of an outcome.
fn result_label<T>(r: &Result<T, StoreError>) -> &'static str {
    match r {
        Ok(_) => "ok",
        Err(StoreError::NotFound { .. }) => "not_found",
        Err(StoreError::Erased) => "erased",
        Err(StoreError::Invalid { .. }) => "invalid",
        Err(StoreError::Forbidden { .. }) => "forbidden",
        Err(StoreError::Conflict { .. }) => "conflict",
        Err(StoreError::Unavailable { .. }) => "unavailable",
        Err(StoreError::Internal(_)) => "internal",
    }
}

/// Time `f`, then count it by result.
async fn timed<T>(
    op: &'static str,
    f: impl Future<Output = Result<T, StoreError>>,
) -> Result<T, StoreError> {
    let start = Instant::now();
    let r = f.await;
    histogram!(names::LEDGER_STORE_OP_DURATION, "op" => op).record(start.elapsed().as_secs_f64());
    counter!(names::LEDGER_STORE_OPS_TOTAL, "op" => op, "result" => result_label(&r)).increment(1);
    r
}

fn source_label(s: Source) -> &'static str {
    match s {
        Source::Verified => "verified",
        Source::Declared => "declared",
        Source::Inferred => "inferred",
        Source::Symbolic => "symbolic",
        Source::Observed => "observed",
    }
}

#[async_trait]
impl<S: Store> Store for Instrumented<S> {
    fn kind(&self) -> StoreKind {
        self.0.kind()
    }

    /// The wrapped store's, so a downcast to the backend still works.
    fn as_any(&self) -> &dyn std::any::Any {
        self.0.as_any()
    }

    async fn ping(&self) -> Result<(), StoreError> {
        timed("ping", self.0.ping()).await
    }

    async fn subject(&self, id: SubjectId) -> Result<Option<Subject>, StoreError> {
        timed("subject", self.0.subject(id)).await
    }

    async fn append(&self, subject: SubjectId, fact: NewFact) -> Result<Appended, StoreError> {
        let source = fact.source;
        let (value_len, origin_len) = (fact.value.bytes.len(), fact.origin.bytes.len());
        let r = timed("append", self.0.append(subject, fact)).await;
        if let Ok(appended) = &r {
            if appended.replayed {
                counter!(names::LEDGER_APPENDS_REPLAYED_TOTAL).increment(1);
            } else {
                counter!(names::LEDGER_FACTS_APPENDED_TOTAL, "source" => source_label(source))
                    .increment(1);
                histogram!(names::LEDGER_ENVELOPE_BYTES, "field" => "value")
                    .record(len_f64(value_len));
                histogram!(names::LEDGER_ENVELOPE_BYTES, "field" => "origin")
                    .record(len_f64(origin_len));
            }
        }
        r
    }

    async fn current(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        let r = timed("current", self.0.current(subject, query)).await;
        if let Ok(page) = &r {
            histogram!(names::LEDGER_PAGE_FACTS, "op" => "current")
                .record(len_f64(page.items.len()));
        }
        r
    }

    async fn history(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        let r = timed("history", self.0.history(subject, query)).await;
        if let Ok(page) = &r {
            histogram!(names::LEDGER_PAGE_FACTS, "op" => "history")
                .record(len_f64(page.items.len()));
        }
        r
    }

    async fn retract(
        &self,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: Envelope,
    ) -> Result<Fact, StoreError> {
        let r = timed("retract", self.0.retract(subject, path, source, origin)).await;
        if r.is_ok() {
            counter!(names::LEDGER_FACTS_RETRACTED_TOTAL).increment(1);
        }
        r
    }

    async fn request_erasure(&self, subject: SubjectId) -> Result<Erasure, StoreError> {
        let r = timed("request_erasure", self.0.request_erasure(subject)).await;
        if r.is_ok() {
            counter!(names::LEDGER_ERASURES_REQUESTED_TOTAL).increment(1);
        }
        r
    }

    async fn restore(&self, subject: SubjectId) -> Result<(), StoreError> {
        let r = timed("restore", self.0.restore(subject)).await;
        if r.is_ok() {
            counter!(names::LEDGER_ERASURES_RESTORED_TOTAL).increment(1);
        }
        r
    }

    async fn execute_due_erasures(
        &self,
        grace: Duration,
        limit: u32,
    ) -> Result<Vec<ErasureExecuted>, StoreError> {
        let r = timed(
            "execute_due_erasures",
            self.0.execute_due_erasures(grace, limit),
        )
        .await;
        if let Ok(executed) = &r
            && !executed.is_empty()
        {
            counter!(names::LEDGER_ERASURES_EXECUTED_TOTAL).increment(executed.len() as u64);
            let tombstones: u64 = executed.iter().map(|e| u64::from(e.tombstoned)).sum();
            if tombstones > 0 {
                counter!(names::LEDGER_ERASURE_TOMBSTONES_TOTAL).increment(tombstones);
            }
        }
        r
    }

    async fn claim_events(
        &self,
        limit: u32,
        lease: Duration,
    ) -> Result<Vec<OutboxEvent>, StoreError> {
        timed("claim_events", self.0.claim_events(limit, lease)).await
    }

    async fn ack_events(&self, event_ids: &[uuid::Uuid]) -> Result<u64, StoreError> {
        timed("ack_events", self.0.ack_events(event_ids)).await
    }

    async fn purge_idempotency(&self, ttl: Duration) -> Result<u64, StoreError> {
        let r = timed("purge_idempotency", self.0.purge_idempotency(ttl)).await;
        if let Ok(n) = &r
            && *n > 0
        {
            counter!(names::LEDGER_IDEMPOTENCY_PURGED_TOTAL).increment(*n);
        }
        r
    }
}

#[allow(clippy::cast_precision_loss)]
fn len_f64(n: usize) -> f64 {
    n as f64
}

#[cfg(test)]
mod tests {
    use std::sync::{Arc, Mutex};

    use metrics::{Counter, CounterFn, Gauge, GaugeFn, Histogram, HistogramFn, Key, Recorder};

    use super::Instrumented;
    use crate::store::{Envelope, NewFact, Query, ScopeId, Source, Store, memory::MemoryStore};

    /// A recorder that remembers every counter increment and histogram record
    /// as `name{label=value,…}`.
    #[derive(Default)]
    struct Capture(Arc<Mutex<Vec<String>>>);

    struct Handle(Arc<Mutex<Vec<String>>>, String);

    fn render(key: &Key) -> String {
        let labels: Vec<String> = key
            .labels()
            .map(|l| format!("{}={}", l.key(), l.value()))
            .collect();
        format!("{}{{{}}}", key.name(), labels.join(","))
    }

    impl CounterFn for Handle {
        fn increment(&self, _: u64) {
            self.0.lock().unwrap().push(self.1.clone());
        }
        fn absolute(&self, _: u64) {}
    }
    impl GaugeFn for Handle {
        fn increment(&self, _: f64) {}
        fn decrement(&self, _: f64) {}
        fn set(&self, _: f64) {}
    }
    impl HistogramFn for Handle {
        fn record(&self, _: f64) {
            self.0.lock().unwrap().push(self.1.clone());
        }
    }

    impl Recorder for Capture {
        fn describe_counter(
            &self,
            _: metrics::KeyName,
            _: Option<metrics::Unit>,
            _: metrics::SharedString,
        ) {
        }
        fn describe_gauge(
            &self,
            _: metrics::KeyName,
            _: Option<metrics::Unit>,
            _: metrics::SharedString,
        ) {
        }
        fn describe_histogram(
            &self,
            _: metrics::KeyName,
            _: Option<metrics::Unit>,
            _: metrics::SharedString,
        ) {
        }
        fn register_counter(&self, key: &Key, _: &metrics::Metadata<'_>) -> Counter {
            Counter::from_arc(Arc::new(Handle(Arc::clone(&self.0), render(key))))
        }
        fn register_gauge(&self, key: &Key, _: &metrics::Metadata<'_>) -> Gauge {
            Gauge::from_arc(Arc::new(Handle(Arc::clone(&self.0), render(key))))
        }
        fn register_histogram(&self, key: &Key, _: &metrics::Metadata<'_>) -> Histogram {
            Histogram::from_arc(Arc::new(Handle(Arc::clone(&self.0), render(key))))
        }
    }

    fn fact(path: &str) -> NewFact {
        NewFact {
            path: path.into(),
            source: Source::Declared,
            value: Envelope::plaintext(&serde_json::json!("x")),
            origin: Envelope::plaintext(&serde_json::json!({"by": "test"})),
            confidence: Some(1.0),
            counterparty_id: None,
            // Fixed, so two calls fingerprint the same and the second is a replay.
            observed_at: chrono::DateTime::from_timestamp(1_700_000_000, 0).unwrap(),
            expires_at: None,
            consent: vec![ScopeId("self".into())],
            stub: false,
            idempotency_key: Some("k".into()),
        }
    }

    #[test]
    fn every_operation_is_timed_and_counted() {
        let capture = Capture::default();
        let seen = Arc::clone(&capture.0);
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_time()
            .build()
            .unwrap();
        metrics::with_local_recorder(&capture, || {
            rt.block_on(async {
                let store = Instrumented(MemoryStore::new());
                let s = uuid::Uuid::now_v7();
                store.append(s, fact("profile.name")).await.unwrap();
                store.append(s, fact("profile.name")).await.unwrap(); // replay: same key
                let q = Query {
                    scopes: Some(vec![ScopeId("self".into())]),
                    ..Query::default()
                };
                store.current(s, &q).await.unwrap();
                store.current(uuid::Uuid::now_v7(), &q).await.unwrap_err();
                store
                    .retract(
                        s,
                        "profile.name",
                        Source::Declared,
                        Envelope::plaintext(&serde_json::json!({})),
                    )
                    .await
                    .unwrap();
                store.request_erasure(s).await.unwrap();
                store.restore(s).await.unwrap();
            });
        });
        let seen = seen.lock().unwrap();
        for expected in [
            "tbd_ledger_store_op_duration_seconds{op=append}",
            "tbd_ledger_store_ops_total{op=append,result=ok}",
            "tbd_ledger_facts_appended_total{source=declared}",
            "tbd_ledger_envelope_bytes{field=value}",
            "tbd_ledger_appends_replayed_total{}",
            "tbd_ledger_page_facts{op=current}",
            "tbd_ledger_store_ops_total{op=current,result=not_found}",
            "tbd_ledger_facts_retracted_total{}",
            "tbd_ledger_erasures_requested_total{}",
            "tbd_ledger_erasures_restored_total{}",
        ] {
            assert!(
                seen.iter().any(|s| s == expected),
                "missing {expected} in {seen:?}"
            );
        }
        assert_eq!(
            seen.iter()
                .filter(|s| s.as_str() == "tbd_ledger_facts_appended_total{source=declared}")
                .count(),
            1,
            "a replay is not a fact"
        );
    }
}
