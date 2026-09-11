//! The in-memory store: the same contract as Postgres, under one mutex, for
//! tests, chaos stacks and host runs without a database. Nothing survives
//! the process; it is not a stub and says `memory` wherever the store kind
//! surfaces.

use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::{Arc, Mutex, PoisonError},
    time::Duration,
};

use super::{
    Appended, Envelope, Erasure, ErasureExecuted, EventKind, Fact, FactId, NewFact, OutboxEvent,
    Page, Query, ScopeId, Source, Store, StoreError, StoreKind, Subject, SubjectId, Timestamp,
    clock::{Clock, SystemClock},
    fingerprint,
    registry::{Registry, ShapeOnly},
    validate,
};

/// The store.
pub struct MemoryStore {
    inner: Mutex<Inner>,
    clock: Arc<dyn Clock>,
    registry: Arc<dyn Registry>,
}

impl std::fmt::Debug for MemoryStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MemoryStore").finish_non_exhaustive()
    }
}

#[derive(Default)]
struct Inner {
    subjects: BTreeMap<SubjectId, SubjectRec>,
    /// Subjects whose erasure executed: their id is never reusable.
    executed: BTreeSet<SubjectId>,
    erasures: Vec<ErasureRec>,
    outbox: VecDeque<OutboxRec>,
    next_fact_id: i64,
    next_seq: i64,
    /// Every stamp is strictly greater than the previous one, at microsecond
    /// resolution, so `(recorded_at, id)` is a total order as in Postgres.
    last_stamp: Option<Timestamp>,
}

struct SubjectRec {
    enrolled_at: Timestamp,
    erased_at: Option<Timestamp>,
    /// In `(recorded_at, id)` order by construction.
    facts: Vec<Fact>,
    current: BTreeMap<(String, Source), FactId>,
    idempotency: BTreeMap<String, IdempotencyRec>,
}

struct IdempotencyRec {
    fact_id: FactId,
    fingerprint: Vec<u8>,
    created_at: Timestamp,
}

struct ErasureRec {
    erasure: Erasure,
    claimed_until: Option<Timestamp>,
}

struct OutboxRec {
    seq: i64,
    event: OutboxEvent,
    claimed_until: Option<Timestamp>,
    published_at: Option<Timestamp>,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self::new()
    }
}

impl MemoryStore {
    /// A store on the system clock with the shape-only registry.
    #[must_use]
    pub fn new() -> Self {
        Self::with(Arc::new(SystemClock), Arc::new(ShapeOnly))
    }

    /// A store with an explicit clock and registry.
    #[must_use]
    pub fn with(clock: Arc<dyn Clock>, registry: Arc<dyn Registry>) -> Self {
        Self {
            inner: Mutex::new(Inner::default()),
            clock,
            registry,
        }
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, Inner> {
        self.inner.lock().unwrap_or_else(PoisonError::into_inner)
    }

    fn now(&self) -> Timestamp {
        self.clock.now()
    }
}

impl Inner {
    /// A fresh `recorded_at`: the clock, truncated to microseconds, and at
    /// least one microsecond after the previous stamp.
    fn stamp(&mut self, now: Timestamp) -> Timestamp {
        let micros = now.timestamp_micros();
        let mut candidate = chrono::DateTime::from_timestamp_micros(micros).unwrap_or(now);
        if let Some(last) = self.last_stamp
            && candidate <= last
        {
            candidate = last + chrono::TimeDelta::microseconds(1);
        }
        self.last_stamp = Some(candidate);
        candidate
    }

    fn next_fact_id(&mut self) -> FactId {
        self.next_fact_id += 1;
        FactId(self.next_fact_id)
    }

    fn push_event(&mut self, event: OutboxEvent) {
        self.next_seq += 1;
        self.outbox.push_back(OutboxRec {
            seq: self.next_seq,
            event,
            claimed_until: None,
            published_at: None,
        });
    }

    /// The subject for a read or write: `NotFound` when unknown, `Erased`
    /// when an erasure is pending or executed.
    fn open(&mut self, id: SubjectId) -> Result<&mut SubjectRec, StoreError> {
        if self.executed.contains(&id) {
            return Err(StoreError::Erased);
        }
        let rec = self
            .subjects
            .get_mut(&id)
            .ok_or(StoreError::NotFound { what: "subject" })?;
        if rec.erased_at.is_some() {
            return Err(StoreError::Erased);
        }
        Ok(rec)
    }

    fn fact_event(&mut self, kind: EventKind, fact: &Fact) {
        let event = OutboxEvent {
            event_id: uuid::Uuid::now_v7(),
            subject_id: fact.subject_id,
            kind,
            fact_id: Some(fact.id),
            payload: OutboxEvent::fact_payload(fact),
            recorded_at: fact.recorded_at,
        };
        self.push_event(event);
    }

    /// Append a tombstone for `(path, source)` on `subject` with the given
    /// origin and consent, and point `current` at it.
    fn tombstone(
        &mut self,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: Envelope,
        consent: Vec<ScopeId>,
        kind: EventKind,
        now: Timestamp,
    ) -> Option<Fact> {
        let recorded_at = self.stamp(now);
        let id = self.next_fact_id();
        let fact = Fact {
            subject_id: subject,
            id,
            path: path.to_owned(),
            source,
            value: None,
            origin,
            confidence: None,
            counterparty_id: None,
            observed_at: recorded_at,
            recorded_at,
            expires_at: None,
            consent,
            stub: false,
        };
        let rec = self.subjects.get_mut(&subject)?;
        rec.facts.push(fact.clone());
        rec.current.insert((path.to_owned(), source), id);
        self.fact_event(kind, &fact);
        Some(fact)
    }
}

fn page(mut facts: Vec<Fact>, query: &Query) -> Result<Page<Fact>, StoreError> {
    if let Some(cursor) = &query.cursor {
        let (at, id) = cursor.decode()?;
        facts.retain(|f| (f.recorded_at, f.id) > (at, id));
    }
    facts.sort_by_key(|f| (f.recorded_at, f.id));
    let size = query.page_size();
    facts.truncate(size + 1);
    Ok(Page::cut(facts, size, |f| (f.recorded_at, f.id)))
}

#[async_trait::async_trait]
impl Store for MemoryStore {
    fn kind(&self) -> StoreKind {
        StoreKind::Memory
    }

    async fn ping(&self) -> Result<(), StoreError> {
        Ok(())
    }

    async fn subject(&self, id: SubjectId) -> Result<Option<Subject>, StoreError> {
        let inner = self.lock();
        Ok(inner.subjects.get(&id).map(|s| Subject {
            id,
            enrolled_at: s.enrolled_at,
            erased_at: s.erased_at,
        }))
    }

    async fn append(&self, subject: SubjectId, fact: NewFact) -> Result<Appended, StoreError> {
        validate::new_fact(&fact, &*self.registry)?;
        let now = self.now();
        let mut inner = self.lock();
        if inner.executed.contains(&subject) {
            return Err(StoreError::Erased);
        }
        if let Some(cp) = fact.counterparty_id {
            if inner.executed.contains(&cp) {
                return Err(StoreError::Erased);
            }
            match inner.subjects.get(&cp) {
                None => {
                    return Err(StoreError::NotFound {
                        what: "counterparty",
                    });
                }
                Some(rec) if rec.erased_at.is_some() => return Err(StoreError::Erased),
                Some(_) => {}
            }
        }
        let recorded_at = inner.stamp(now);
        let rec = inner.subjects.entry(subject).or_insert_with(|| SubjectRec {
            enrolled_at: recorded_at,
            erased_at: None,
            facts: Vec::new(),
            current: BTreeMap::new(),
            idempotency: BTreeMap::new(),
        });
        if rec.erased_at.is_some() {
            return Err(StoreError::Erased);
        }
        let print = fact.idempotency_key.as_ref().map(|_| fingerprint(&fact));
        if let (Some(key), Some(print)) = (&fact.idempotency_key, &print)
            && let Some(hit) = rec.idempotency.get(key)
        {
            if &hit.fingerprint != print {
                return Err(StoreError::Conflict {
                    reason: "idempotency key reused with a different fact".into(),
                });
            }
            return match rec.facts.iter().find(|f| f.id == hit.fact_id) {
                Some(existing) => Ok(Appended {
                    fact: existing.clone(),
                    replayed: true,
                }),
                None => Err(StoreError::Conflict {
                    reason: "the fact recorded under this key was retracted".into(),
                }),
            };
        }
        let id = FactId(inner.next_fact_id + 1);
        inner.next_fact_id += 1;
        let rec = inner
            .subjects
            .get_mut(&subject)
            .ok_or(StoreError::Internal("subject vanished".into()))?;
        let stored = Fact {
            subject_id: subject,
            id,
            path: fact.path.clone(),
            source: fact.source,
            value: Some(fact.value.clone()),
            origin: fact.origin.clone(),
            confidence: fact.confidence,
            counterparty_id: fact.counterparty_id,
            observed_at: fact.observed_at,
            recorded_at,
            expires_at: fact.expires_at,
            consent: fact.consent.clone(),
            stub: fact.stub,
        };
        rec.facts.push(stored.clone());
        rec.current.insert((fact.path.clone(), fact.source), id);
        if let (Some(key), Some(print)) = (fact.idempotency_key, print) {
            rec.idempotency.insert(
                key,
                IdempotencyRec {
                    fact_id: id,
                    fingerprint: print,
                    created_at: recorded_at,
                },
            );
        }
        inner.fact_event(EventKind::FactRecorded, &stored);
        Ok(Appended {
            fact: stored,
            replayed: false,
        })
    }

    async fn current(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        validate::query(query, false)?;
        let now = self.now();
        let mut inner = self.lock();
        let rec = inner.open(subject)?;
        let facts: Vec<Fact> = rec
            .current
            .values()
            .filter_map(|id| rec.facts.iter().find(|f| f.id == *id))
            .filter(|f| !f.is_tombstone() && !f.is_expired_at(now) && query.admits(f))
            .cloned()
            .collect();
        page(facts, query)
    }

    async fn history(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        validate::query(query, true)?;
        let mut inner = self.lock();
        let rec = inner.open(subject)?;
        let facts: Vec<Fact> = rec
            .facts
            .iter()
            .filter(|f| query.at.is_none_or(|at| f.recorded_at <= at) && query.admits(f))
            .cloned()
            .collect();
        page(facts, query)
    }

    async fn retract(
        &self,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: Envelope,
    ) -> Result<Fact, StoreError> {
        validate::path_shape(path, 2).map_err(|r| StoreError::invalid("path", r))?;
        validate::envelope("origin", &origin)?;
        let now = self.now();
        let mut inner = self.lock();
        let rec = inner.open(subject)?;
        let consent = rec
            .facts
            .iter()
            .rev()
            .find(|f| f.path == path && f.source == source && !f.is_tombstone())
            .map(|f| f.consent.clone())
            .ok_or(StoreError::NotFound { what: "fact" })?;
        rec.facts
            .retain(|f| !(f.path == path && f.source == source && !f.is_tombstone()));
        inner
            .tombstone(
                subject,
                path,
                source,
                origin,
                consent,
                EventKind::FactRetracted,
                now,
            )
            .ok_or(StoreError::Internal("subject vanished".into()))
    }

    async fn request_erasure(&self, subject: SubjectId) -> Result<Erasure, StoreError> {
        let now = self.now();
        let mut inner = self.lock();
        if inner.executed.contains(&subject) {
            return Err(StoreError::NotFound { what: "subject" });
        }
        if !inner.subjects.contains_key(&subject) {
            return Err(StoreError::NotFound { what: "subject" });
        }
        if let Some(pending) = inner.erasures.iter().find(|e| {
            e.erasure.subject_id == subject
                && e.erasure.executed_at.is_none()
                && e.erasure.cancelled_at.is_none()
        }) {
            return Ok(pending.erasure.clone());
        }
        let requested_at = inner.stamp(now);
        let rec = inner
            .subjects
            .get_mut(&subject)
            .ok_or(StoreError::NotFound { what: "subject" })?;
        rec.erased_at.get_or_insert(requested_at);
        let erasure = Erasure {
            subject_id: subject,
            event_id: uuid::Uuid::now_v7(),
            requested_at,
            executed_at: None,
            cancelled_at: None,
            published_at: None,
        };
        inner.erasures.push(ErasureRec {
            erasure: erasure.clone(),
            claimed_until: None,
        });
        Ok(erasure)
    }

    async fn restore(&self, subject: SubjectId) -> Result<(), StoreError> {
        let now = self.now();
        let mut inner = self.lock();
        if inner.executed.contains(&subject) || !inner.subjects.contains_key(&subject) {
            return Err(StoreError::NotFound { what: "subject" });
        }
        let stamp = inner.stamp(now);
        for e in &mut inner.erasures {
            if e.erasure.subject_id == subject
                && e.erasure.executed_at.is_none()
                && e.erasure.cancelled_at.is_none()
            {
                e.erasure.cancelled_at = Some(stamp);
            }
        }
        if let Some(rec) = inner.subjects.get_mut(&subject) {
            rec.erased_at = None;
        }
        Ok(())
    }

    async fn execute_due_erasures(
        &self,
        grace: Duration,
        limit: u32,
    ) -> Result<Vec<ErasureExecuted>, StoreError> {
        let now = self.now();
        let grace = chrono::TimeDelta::from_std(grace).unwrap_or(chrono::TimeDelta::MAX);
        let mut inner = self.lock();
        let due: Vec<SubjectId> = inner
            .erasures
            .iter()
            .filter(|e| {
                e.erasure.executed_at.is_none()
                    && e.erasure.cancelled_at.is_none()
                    && e.erasure
                        .requested_at
                        .checked_add_signed(grace)
                        .is_some_and(|d| d <= now)
            })
            .map(|e| e.erasure.subject_id)
            .take(limit as usize)
            .collect();
        let mut out = Vec::new();
        for sid in due {
            let Some(target) = inner.subjects.get(&sid) else {
                continue;
            };
            if target.erased_at.is_none() {
                continue; // restored meanwhile
            }
            // Survivors: every fact anywhere naming this subject as counterparty.
            let mut survivors: Vec<(SubjectId, String, Source, Vec<ScopeId>)> = Vec::new();
            for (other, rec) in &mut inner.subjects {
                if *other == sid {
                    continue;
                }
                let mut latest: BTreeMap<(String, Source), Vec<ScopeId>> = BTreeMap::new();
                for f in rec.facts.iter().filter(|f| f.counterparty_id == Some(sid)) {
                    latest.insert((f.path.clone(), f.source), f.consent.clone());
                }
                rec.facts.retain(|f| f.counterparty_id != Some(sid));
                for ((path, source), consent) in latest {
                    survivors.push((*other, path, source, consent));
                }
            }
            survivors.sort();
            let mut tombstoned = 0;
            for (other, path, source, consent) in survivors {
                let origin =
                    Envelope::plaintext(&serde_json::json!({"cause": "counterparty_erased"}));
                if inner
                    .tombstone(
                        other,
                        &path,
                        source,
                        origin,
                        consent,
                        EventKind::FactRetracted,
                        now,
                    )
                    .is_some()
                {
                    tombstoned += 1;
                }
            }
            inner.subjects.remove(&sid);
            inner.outbox.retain(|o| o.event.subject_id != sid);
            inner.executed.insert(sid);
            let executed_at = inner.stamp(now);
            for e in &mut inner.erasures {
                if e.erasure.subject_id == sid
                    && e.erasure.executed_at.is_none()
                    && e.erasure.cancelled_at.is_none()
                {
                    e.erasure.executed_at = Some(executed_at);
                }
            }
            out.push(ErasureExecuted {
                subject_id: sid,
                executed_at,
                tombstoned,
            });
        }
        Ok(out)
    }

    async fn claim_events(
        &self,
        limit: u32,
        lease: Duration,
    ) -> Result<Vec<OutboxEvent>, StoreError> {
        let now = self.now();
        let until = now + chrono::TimeDelta::from_std(lease).unwrap_or(chrono::TimeDelta::MAX);
        let mut inner = self.lock();
        let mut out = Vec::new();
        let mut claimable: Vec<&mut OutboxRec> = inner
            .outbox
            .iter_mut()
            .filter(|o| o.published_at.is_none() && o.claimed_until.is_none_or(|c| c < now))
            .collect();
        claimable.sort_by_key(|o| o.seq);
        for o in claimable.into_iter().take(limit as usize) {
            o.claimed_until = Some(until);
            out.push(o.event.clone());
        }
        let remaining = (limit as usize).saturating_sub(out.len());
        let mut erased: Vec<&mut ErasureRec> = inner
            .erasures
            .iter_mut()
            .filter(|e| {
                e.erasure.executed_at.is_some()
                    && e.erasure.published_at.is_none()
                    && e.claimed_until.is_none_or(|c| c < now)
            })
            .collect();
        erased.sort_by_key(|e| e.erasure.executed_at);
        for e in erased.into_iter().take(remaining) {
            e.claimed_until = Some(until);
            let executed_at = e.erasure.executed_at.unwrap_or(now);
            out.push(OutboxEvent {
                event_id: e.erasure.event_id,
                subject_id: e.erasure.subject_id,
                kind: EventKind::SubjectErased,
                fact_id: None,
                payload: serde_json::json!({
                    "requested_at": e.erasure.requested_at,
                    "executed_at": executed_at,
                }),
                recorded_at: executed_at,
            });
        }
        Ok(out)
    }

    async fn ack_events(&self, event_ids: &[uuid::Uuid]) -> Result<u64, StoreError> {
        let now = self.now();
        let mut inner = self.lock();
        let mut n = 0;
        for o in &mut inner.outbox {
            if o.published_at.is_none() && event_ids.contains(&o.event.event_id) {
                o.published_at = Some(now);
                n += 1;
            }
        }
        for e in &mut inner.erasures {
            if e.erasure.published_at.is_none() && event_ids.contains(&e.erasure.event_id) {
                e.erasure.published_at = Some(now);
                n += 1;
            }
        }
        Ok(n)
    }

    async fn purge_idempotency(&self, ttl: Duration) -> Result<u64, StoreError> {
        let now = self.now();
        let ttl = chrono::TimeDelta::from_std(ttl).unwrap_or(chrono::TimeDelta::MAX);
        let mut inner = self.lock();
        let mut n = 0;
        for rec in inner.subjects.values_mut() {
            let before = rec.idempotency.len();
            rec.idempotency.retain(|_, r| {
                r.created_at
                    .checked_add_signed(ttl)
                    .is_none_or(|exp| exp > now)
            });
            n += before - rec.idempotency.len();
        }
        Ok(n as u64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stamps_are_strictly_increasing_even_when_the_clock_stands_still() {
        let mut inner = Inner::default();
        let t = chrono::Utc::now();
        let a = inner.stamp(t);
        let b = inner.stamp(t);
        let c = inner.stamp(t - chrono::TimeDelta::seconds(10));
        assert!(a < b && b < c);
        assert_eq!(
            a.timestamp_subsec_nanos() % 1000,
            0,
            "microsecond resolution"
        );
    }
}
