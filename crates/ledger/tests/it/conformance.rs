//! The store contract, as tests generic over the backend. Every backend must
//! pass every case; the privacy claims of the design (retraction deletes,
//! erasure cascades, consent filters) are the assertions here.

#![allow(clippy::unwrap_used, clippy::expect_used, dead_code)]

use std::time::Duration;

use tbd_ledger::store::{
    Envelope, EventKind, Fact, NewFact, PathPattern, Query, ScopeId, Source, Store, StoreError,
    SubjectId,
};

fn sid() -> SubjectId {
    uuid::Uuid::now_v7()
}

pub fn scopes(list: &[&str]) -> Vec<ScopeId> {
    list.iter().map(|s| ScopeId::new(*s)).collect()
}

pub fn fact(path: &str, source: Source, value: &serde_json::Value) -> NewFact {
    NewFact {
        path: path.into(),
        source,
        value: Envelope::plaintext(value),
        origin: Envelope::plaintext(&serde_json::json!({"by": "test"})),
        confidence: source.carries_confidence().then_some(1.0),
        counterparty_id: None,
        observed_at: chrono::Utc::now(),
        expires_at: None,
        consent: scopes(&["self"]),
        stub: false,
        idempotency_key: None,
    }
}

fn declared(path: &str, value: &str) -> NewFact {
    fact(path, Source::Declared, &serde_json::json!(value))
}

fn origin() -> Envelope {
    Envelope::plaintext(&serde_json::json!({"by": "test"}))
}

fn under_self() -> Query {
    Query::under(scopes(&["self"]))
}

fn values(page: &[Fact]) -> Vec<String> {
    page.iter()
        .map(|f| {
            f.value.as_ref().map_or_else(
                || "<tombstone>".into(),
                |v| v.plaintext_json().unwrap().to_string(),
            )
        })
        .collect()
}

async fn all_history<S: Store>(store: &S, subject: SubjectId) -> Vec<Fact> {
    let mut out = Vec::new();
    let mut q = Query {
        limit: 1000,
        ..Query::default()
    };
    loop {
        let page = store.history(subject, &q).await.unwrap();
        out.extend(page.items);
        match page.next {
            Some(c) => q.cursor = Some(c),
            None => return out,
        }
    }
}

pub async fn append_returns_fact_with_store_set_fields<S: Store>(store: &S) {
    let s = sid();
    let appended = store
        .append(s, declared("profile.name", "Ada"))
        .await
        .unwrap();
    assert!(!appended.replayed);
    let f = &appended.fact;
    assert_eq!(f.subject_id, s);
    assert!(f.id.0 > 0);
    assert_eq!(f.path, "profile.name");
    assert_eq!(f.source, Source::Declared);
    assert_eq!(
        f.value.as_ref().unwrap().plaintext_json().unwrap(),
        serde_json::json!("Ada")
    );
    assert_eq!(f.consent, scopes(&["self"]));
    let subject = store.subject(s).await.unwrap().unwrap();
    assert!(subject.erased_at.is_none());
    assert!(subject.enrolled_at <= f.recorded_at);
    assert!(store.subject(sid()).await.unwrap().is_none());
}

/// The field an `Invalid` error names; panics on anything else.
fn field<T: std::fmt::Debug>(r: Result<T, StoreError>) -> &'static str {
    match r {
        Err(StoreError::Invalid { field, .. }) => field,
        other => panic!("expected Invalid, got {other:?}"),
    }
}

pub async fn append_rejects_bad_input<S: Store>(store: &S) {
    let s = sid();
    assert_eq!(field(store.append(s, declared("nope", "x")).await), "path");
    let mut f = declared("profile.name", "x");
    f.consent.clear();
    assert_eq!(field(store.append(s, f).await), "consent");
    let mut f = declared("profile.name", "x");
    f.confidence = Some(2.0);
    assert_eq!(field(store.append(s, f).await), "confidence");
    let mut f = fact("readings.sun", Source::Symbolic, &serde_json::json!("leo"));
    f.confidence = Some(0.5);
    assert_eq!(field(store.append(s, f).await), "confidence");
    let mut f = declared("profile.name", "x");
    f.origin.version = 3;
    assert_eq!(field(store.append(s, f).await), "origin");
    let mut f = declared("profile.name", "x");
    f.value = Envelope {
        version: 0,
        bytes: b"nope".to_vec(),
    };
    assert_eq!(field(store.append(s, f).await), "value");
    let mut f = declared("profile.name", "x");
    f.idempotency_key = Some("k".repeat(300));
    assert_eq!(field(store.append(s, f).await), "idempotency_key");
    // Nothing was written: the subject does not exist.
    assert!(matches!(
        store.history(s, &under_self()).await,
        Err(StoreError::NotFound { what: "subject" })
    ));
    // Queries are bounded too.
    store
        .append(s, declared("profile.name", "x"))
        .await
        .unwrap();
    let q = Query {
        limit: 5000,
        ..under_self()
    };
    assert_eq!(field(store.current(s, &q).await), "limit");
    let q = Query {
        cursor: Some(tbd_ledger::store::Cursor::from_wire("garbage")),
        ..under_self()
    };
    assert_eq!(field(store.history(s, &q).await), "cursor");
    let q = Query {
        at: Some(chrono::Utc::now()),
        ..under_self()
    };
    assert_eq!(field(store.current(s, &q).await), "at");
}

pub async fn unknown_subject_is_not_found_never_empty<S: Store>(store: &S) {
    let s = sid();
    assert!(matches!(
        store.current(s, &under_self()).await,
        Err(StoreError::NotFound { what: "subject" })
    ));
    assert!(matches!(
        store
            .retract(s, "profile.name", Source::Declared, origin())
            .await,
        Err(StoreError::NotFound { what: "subject" })
    ));
    assert!(matches!(
        store.request_erasure(s).await,
        Err(StoreError::NotFound { what: "subject" })
    ));
    let mut f = declared("relations.match.x", "y");
    f.counterparty_id = Some(sid());
    assert!(matches!(
        store.append(s, f).await,
        Err(StoreError::NotFound {
            what: "counterparty"
        })
    ));
}

pub async fn current_is_latest_per_path_and_source<S: Store>(store: &S) {
    let s = sid();
    store
        .append(s, declared("profile.name", "a"))
        .await
        .unwrap();
    store
        .append(s, declared("profile.name", "b"))
        .await
        .unwrap();
    store
        .append(s, declared("profile.name", "c"))
        .await
        .unwrap();
    let mut inferred = fact(
        "profile.name",
        Source::Inferred,
        &serde_json::json!("guess"),
    );
    inferred.confidence = Some(0.4);
    store.append(s, inferred).await.unwrap();
    let page = store.current(s, &under_self()).await.unwrap();
    assert_eq!(values(&page.items), ["\"c\"", "\"guess\""]);
    assert!(page.next.is_none());
    for w in page.items.windows(2) {
        assert!((w[0].recorded_at, w[0].id) < (w[1].recorded_at, w[1].id));
    }
    let history = all_history(store, s).await;
    assert_eq!(history.len(), 4);
}

pub async fn history_is_everything_and_at_cuts<S: Store>(store: &S) {
    let s = sid();
    let a = store
        .append(s, declared("profile.name", "a"))
        .await
        .unwrap()
        .fact;
    let b = store
        .append(s, declared("profile.name", "b"))
        .await
        .unwrap()
        .fact;
    store
        .append(s, declared("profile.name", "c"))
        .await
        .unwrap();
    assert!(a.recorded_at < b.recorded_at);
    let at = |t| Query {
        at: Some(t),
        ..under_self()
    };
    let page = store.history(s, &at(b.recorded_at)).await.unwrap();
    assert_eq!(values(&page.items), ["\"a\"", "\"b\""]);
    let before = a.recorded_at - chrono::TimeDelta::microseconds(1);
    let page = store.history(s, &at(before)).await.unwrap();
    assert!(page.items.is_empty());
    assert_eq!(all_history(store, s).await.len(), 3);
}

pub async fn retract_deletes_values_and_appends_a_tombstone<S: Store>(store: &S) {
    let s = sid();
    let mut f = declared("journal.entry", "private");
    f.consent = scopes(&["self", "engine.base"]);
    let before = store.append(s, f).await.unwrap().fact.recorded_at;
    store
        .append(s, declared("journal.entry", "still private"))
        .await
        .unwrap();
    store
        .append(s, declared("profile.name", "kept"))
        .await
        .unwrap();
    let tomb = store
        .retract(s, "journal.entry", Source::Declared, origin())
        .await
        .unwrap();
    assert!(tomb.is_tombstone());
    assert_eq!(tomb.path, "journal.entry");
    assert_eq!(tomb.counterparty_id, None);
    assert_eq!(
        tomb.consent,
        scopes(&["self"]),
        "the latest value's consent"
    );
    // current: gone; history: only the tombstone for that path.
    let page = store.current(s, &under_self()).await.unwrap();
    assert_eq!(values(&page.items), ["\"kept\""]);
    let history = all_history(store, s).await;
    let journal: Vec<&Fact> = history
        .iter()
        .filter(|f| f.path == "journal.entry")
        .collect();
    assert_eq!(journal.len(), 1);
    assert!(journal[0].is_tombstone());
    // The value is absent from every cut of history: physical deletion.
    let q = Query {
        at: Some(before),
        paths: vec![PathPattern::parse("journal.entry").unwrap()],
        ..under_self()
    };
    assert!(store.history(s, &q).await.unwrap().items.is_empty());
    // Retracting again finds nothing valued.
    assert!(matches!(
        store
            .retract(s, "journal.entry", Source::Declared, origin())
            .await,
        Err(StoreError::NotFound { what: "fact" })
    ));
    // A second value + retract cycle keeps the earlier tombstone in history.
    store
        .append(s, declared("journal.entry", "again"))
        .await
        .unwrap();
    store
        .retract(s, "journal.entry", Source::Declared, origin())
        .await
        .unwrap();
    let tombs = all_history(store, s)
        .await
        .into_iter()
        .filter(|f| f.path == "journal.entry")
        .count();
    assert_eq!(tombs, 2);
}

pub async fn expired_is_excluded_from_current_but_held_in_history<S: Store>(store: &S) {
    let s = sid();
    let mut f = declared("identity.age.over_18", "true");
    f.observed_at = chrono::Utc::now() - chrono::TimeDelta::hours(2);
    f.expires_at = Some(chrono::Utc::now() - chrono::TimeDelta::hours(1));
    store.append(s, f).await.unwrap();
    store
        .append(s, declared("profile.name", "x"))
        .await
        .unwrap();
    let page = store.current(s, &under_self()).await.unwrap();
    assert_eq!(values(&page.items), ["\"x\""]);
    assert_eq!(all_history(store, s).await.len(), 2);
}

pub async fn consent_filters_reads<S: Store>(store: &S) {
    let s = sid();
    let mut shared = declared("profile.name", "shared");
    shared.consent = scopes(&["self", "tier2@persona-a"]);
    store.append(s, shared).await.unwrap();
    store
        .append(s, declared("journal.entry", "mine"))
        .await
        .unwrap();
    let page = store
        .current(s, &Query::under(scopes(&["tier2@persona-a"])))
        .await
        .unwrap();
    assert_eq!(values(&page.items), ["\"shared\""]);
    let page = store.current(s, &Query::under(vec![])).await.unwrap();
    assert!(page.items.is_empty(), "no scopes: nothing readable");
    let page = store.current(s, &Query::default()).await.unwrap();
    assert_eq!(page.items.len(), 2, "unfiltered (in-process only)");
    let page = store
        .history(s, &Query::under(scopes(&["tier2@persona-a"])))
        .await
        .unwrap();
    assert_eq!(page.items.len(), 1);
}

pub async fn path_and_source_filters<S: Store>(store: &S) {
    let s = sid();
    store
        .append(s, declared("traits.warmth", "1"))
        .await
        .unwrap();
    store.append(s, declared("traits.a_b", "2")).await.unwrap();
    store.append(s, declared("traits.axb", "3")).await.unwrap();
    store
        .append(s, declared("profile.name", "4"))
        .await
        .unwrap();
    let mut inferred = fact("traits.warmth", Source::Inferred, &serde_json::json!("5"));
    inferred.confidence = Some(0.9);
    store.append(s, inferred).await.unwrap();
    let q = |paths: &[&str], sources: &[Source]| Query {
        paths: paths
            .iter()
            .map(|p| PathPattern::parse(p).unwrap())
            .collect(),
        sources: sources.to_vec(),
        ..under_self()
    };
    let page = store.current(s, &q(&["traits.*"], &[])).await.unwrap();
    assert_eq!(page.items.len(), 4);
    let page = store.current(s, &q(&["traits.a_b.*"], &[])).await.unwrap();
    assert!(page.items.is_empty(), "the underscore is literal");
    let page = store.current(s, &q(&["traits.warmth"], &[])).await.unwrap();
    assert_eq!(page.items.len(), 2);
    let page = store
        .current(s, &q(&["traits.warmth"], &[Source::Inferred]))
        .await
        .unwrap();
    assert_eq!(values(&page.items), ["\"5\""]);
    let page = store
        .current(s, &q(&["profile.name", "traits.axb"], &[]))
        .await
        .unwrap();
    assert_eq!(page.items.len(), 2);
}

pub async fn pagination_is_stable<S: Store>(store: &S) {
    let s = sid();
    for i in 0..25 {
        store
            .append(s, declared(&format!("profile.field{i}"), "v"))
            .await
            .unwrap();
    }
    let mut seen = Vec::new();
    let mut q = Query {
        limit: 10,
        ..under_self()
    };
    let mut pages = 0;
    loop {
        let page = store.current(s, &q).await.unwrap();
        pages += 1;
        seen.extend(page.items.iter().map(|f| f.id));
        match page.next {
            Some(c) => {
                q.cursor = Some(c);
                // A fact appended between pages lands on a later page.
                if pages == 1 {
                    store
                        .append(s, declared("profile.late", "v"))
                        .await
                        .unwrap();
                }
            }
            None => break,
        }
    }
    assert_eq!(pages, 3);
    assert_eq!(seen.len(), 26);
    let mut dedup = seen.clone();
    dedup.sort();
    dedup.dedup();
    assert_eq!(dedup.len(), 26, "no duplicates across pages");
    // Same for history.
    let mut q = Query {
        limit: 7,
        ..under_self()
    };
    let mut total = 0;
    loop {
        let page = store.history(s, &q).await.unwrap();
        total += page.items.len();
        match page.next {
            Some(c) => q.cursor = Some(c),
            None => break,
        }
    }
    assert_eq!(total, 26);
}

pub async fn idempotency_replays_and_conflicts<S: Store>(store: &S) {
    let s = sid();
    let mut f = declared("profile.name", "a");
    f.idempotency_key = Some("req-1".into());
    let first = store.append(s, f.clone()).await.unwrap();
    let again = store.append(s, f.clone()).await.unwrap();
    assert!(again.replayed);
    assert_eq!(again.fact.id, first.fact.id);
    assert_eq!(all_history(store, s).await.len(), 1);
    // Same key, different content: conflict.
    let mut other = declared("profile.name", "b");
    other.idempotency_key = Some("req-1".into());
    assert!(matches!(
        store.append(s, other).await,
        Err(StoreError::Conflict { .. })
    ));
    // The key is scoped per subject.
    let t = sid();
    let fresh = store.append(t, f.clone()).await.unwrap();
    assert!(!fresh.replayed);
    // Replay after retraction is a conflict, not a resurrection.
    store
        .retract(s, "profile.name", Source::Declared, origin())
        .await
        .unwrap();
    assert!(matches!(
        store.append(s, f.clone()).await,
        Err(StoreError::Conflict { .. })
    ));
    // Purged keys make the same request a fresh fact.
    let purged = store.purge_idempotency(Duration::ZERO).await.unwrap();
    assert!(purged >= 1);
    let fresh = store.append(s, f).await.unwrap();
    assert!(!fresh.replayed);
    // Exactly one outbox event per real write on s: recorded, retracted, recorded.
    let events = store
        .claim_events(100, Duration::from_secs(60))
        .await
        .unwrap();
    let kinds: Vec<EventKind> = events
        .iter()
        .filter(|e| e.subject_id == s)
        .map(|e| e.kind)
        .collect();
    assert_eq!(
        kinds,
        [
            EventKind::FactRecorded,
            EventKind::FactRetracted,
            EventKind::FactRecorded
        ]
    );
}

pub async fn erasure_request_denies_everything_and_is_idempotent<S: Store>(store: &S) {
    let s = sid();
    store
        .append(s, declared("profile.name", "a"))
        .await
        .unwrap();
    let e1 = store.request_erasure(s).await.unwrap();
    assert!(e1.executed_at.is_none() && e1.cancelled_at.is_none());
    let e2 = store.request_erasure(s).await.unwrap();
    assert_eq!(
        e1.requested_at, e2.requested_at,
        "the window does not restart"
    );
    assert_eq!(e1.event_id, e2.event_id);
    assert!(matches!(
        store.current(s, &under_self()).await,
        Err(StoreError::Erased)
    ));
    assert!(matches!(
        store.history(s, &under_self()).await,
        Err(StoreError::Erased)
    ));
    assert!(matches!(
        store.append(s, declared("profile.name", "b")).await,
        Err(StoreError::Erased)
    ));
    assert!(matches!(
        store
            .retract(s, "profile.name", Source::Declared, origin())
            .await,
        Err(StoreError::Erased)
    ));
    let subject = store.subject(s).await.unwrap().unwrap();
    assert_eq!(subject.erased_at, Some(e1.requested_at));
    // Another subject may not point at it either.
    let t = sid();
    store
        .append(t, declared("profile.name", "t"))
        .await
        .unwrap();
    let mut rel = declared("relations.match.x", "y");
    rel.counterparty_id = Some(s);
    assert!(matches!(
        store.append(t, rel).await,
        Err(StoreError::Erased)
    ));
}

pub async fn restore_within_the_window_reopens<S: Store>(store: &S) {
    let s = sid();
    store
        .append(s, declared("profile.name", "a"))
        .await
        .unwrap();
    store.request_erasure(s).await.unwrap();
    store.restore(s).await.unwrap();
    let page = store.current(s, &under_self()).await.unwrap();
    assert_eq!(values(&page.items), ["\"a\""]);
    assert!(store.subject(s).await.unwrap().unwrap().erased_at.is_none());
    // Nothing is due: the cancelled request is not executed even with no grace.
    let executed = store
        .execute_due_erasures(Duration::ZERO, 100)
        .await
        .unwrap();
    assert!(executed.iter().all(|e| e.subject_id != s));
    // Restore with nothing pending is fine; on an unknown subject it is NotFound.
    store.restore(s).await.unwrap();
    assert!(matches!(
        store.restore(sid()).await,
        Err(StoreError::NotFound { what: "subject" })
    ));
}

pub async fn erasure_execute_cascades_and_tombstones_the_counterparty<S: Store>(store: &S) {
    let a = sid();
    let b = sid();
    store
        .append(a, declared("profile.name", "A"))
        .await
        .unwrap();
    store
        .append(b, declared("profile.name", "B"))
        .await
        .unwrap();
    let mut a_to_b = declared("relations.match.m1", "matched");
    a_to_b.counterparty_id = Some(b);
    store.append(a, a_to_b).await.unwrap();
    let mut b_to_a = declared("relations.match.m1", "matched");
    b_to_a.counterparty_id = Some(a);
    b_to_a.consent = scopes(&["self", "engine.base"]);
    store.append(b, b_to_a).await.unwrap();
    let mut keyed = declared("profile.bio", "bio");
    keyed.idempotency_key = Some("k".into());
    store.append(a, keyed).await.unwrap();

    let erasure = store.request_erasure(a).await.unwrap();
    let executed = store
        .execute_due_erasures(Duration::ZERO, 100)
        .await
        .unwrap();
    let ours = executed
        .iter()
        .find(|e| e.subject_id == a)
        .expect("executed");
    assert_eq!(ours.tombstoned, 1);

    // A is gone: unknown to reads, its id never reusable for writes.
    assert!(store.subject(a).await.unwrap().is_none());
    assert!(matches!(
        store.current(a, &under_self()).await,
        Err(StoreError::Erased)
    ));
    assert!(matches!(
        store.append(a, declared("profile.name", "again")).await,
        Err(StoreError::Erased)
    ));
    assert!(matches!(
        store.request_erasure(a).await,
        Err(StoreError::NotFound { what: "subject" })
    ));
    assert!(matches!(
        store.restore(a).await,
        Err(StoreError::NotFound { what: "subject" })
    ));

    // B keeps its own facts; the relation is a tombstone with no counterparty.
    let page = store.current(b, &under_self()).await.unwrap();
    assert_eq!(values(&page.items), ["\"B\""]);
    let history = all_history(store, b).await;
    let rel: Vec<&Fact> = history
        .iter()
        .filter(|f| f.path == "relations.match.m1")
        .collect();
    assert_eq!(rel.len(), 1);
    assert!(rel[0].is_tombstone());
    assert_eq!(rel[0].counterparty_id, None);
    assert_eq!(rel[0].consent, scopes(&["self", "engine.base"]));
    assert_eq!(
        rel[0].origin.plaintext_json().unwrap(),
        serde_json::json!({"cause": "counterparty_erased"})
    );

    // A second sweep executes nothing for A.
    let again = store
        .execute_due_erasures(Duration::ZERO, 100)
        .await
        .unwrap();
    assert!(again.iter().all(|e| e.subject_id != a));

    // The outbox: A's fact events are gone with A; B got a retracted event;
    // exactly one subject.erased with the erasure's own event id.
    let events = store
        .claim_events(1000, Duration::from_secs(60))
        .await
        .unwrap();
    assert!(
        events
            .iter()
            .all(|e| e.subject_id != a || e.kind == EventKind::SubjectErased)
    );
    let erased: Vec<&tbd_ledger::store::OutboxEvent> = events
        .iter()
        .filter(|e| e.kind == EventKind::SubjectErased && e.subject_id == a)
        .collect();
    assert_eq!(erased.len(), 1);
    assert_eq!(erased[0].event_id, erasure.event_id);
    assert_eq!(erased[0].fact_id, None);
    let b_events: Vec<EventKind> = events
        .iter()
        .filter(|e| e.subject_id == b)
        .map(|e| e.kind)
        .collect();
    assert_eq!(
        b_events,
        [
            EventKind::FactRecorded,
            EventKind::FactRecorded,
            EventKind::FactRetracted
        ]
    );
}

pub async fn erasure_respects_the_grace_window<S: Store>(store: &S) {
    let s = sid();
    store
        .append(s, declared("profile.name", "a"))
        .await
        .unwrap();
    store.request_erasure(s).await.unwrap();
    let executed = store
        .execute_due_erasures(Duration::from_hours(7 * 24), 100)
        .await
        .unwrap();
    assert!(executed.iter().all(|e| e.subject_id != s));
    assert!(store.subject(s).await.unwrap().is_some());
    let executed = store
        .execute_due_erasures(Duration::ZERO, 100)
        .await
        .unwrap();
    assert!(executed.iter().any(|e| e.subject_id == s));
    assert!(store.subject(s).await.unwrap().is_none());
}

pub async fn outbox_events_are_ordered_leased_and_acked<S: Store>(store: &S) {
    let s = sid();
    store
        .append(s, declared("profile.name", "a"))
        .await
        .unwrap();
    store
        .append(s, declared("profile.name", "b"))
        .await
        .unwrap();
    store
        .retract(s, "profile.name", Source::Declared, origin())
        .await
        .unwrap();
    let first = store
        .claim_events(1000, Duration::from_secs(60))
        .await
        .unwrap();
    let ours: Vec<&tbd_ledger::store::OutboxEvent> =
        first.iter().filter(|e| e.subject_id == s).collect();
    assert_eq!(
        ours.iter().map(|e| e.kind).collect::<Vec<_>>(),
        [
            EventKind::FactRecorded,
            EventKind::FactRecorded,
            EventKind::FactRetracted
        ]
    );
    for w in ours.windows(2) {
        assert!(w[0].recorded_at <= w[1].recorded_at);
    }
    // Leased: not claimable again until the lease passes.
    let second = store
        .claim_events(1000, Duration::from_secs(60))
        .await
        .unwrap();
    assert!(second.iter().all(|e| e.subject_id != s));
    // Ack, twice is harmless, then nothing is claimable.
    let ids: Vec<uuid::Uuid> = ours.iter().map(|e| e.event_id).collect();
    assert_eq!(store.ack_events(&ids).await.unwrap(), 3);
    assert_eq!(store.ack_events(&ids).await.unwrap(), 0);
    // A zero lease is reclaimable at once, and unacked events come back.
    store
        .append(s, declared("profile.name", "c"))
        .await
        .unwrap();
    let a = store.claim_events(1000, Duration::ZERO).await.unwrap();
    let b = store.claim_events(1000, Duration::ZERO).await.unwrap();
    let ours_a: Vec<_> = a.iter().filter(|e| e.subject_id == s).collect();
    let ours_b: Vec<_> = b.iter().filter(|e| e.subject_id == s).collect();
    assert_eq!(ours_a.len(), 1);
    assert_eq!(ours_a.len(), ours_b.len());
    assert_eq!(ours_a[0].event_id, ours_b[0].event_id);
}

pub async fn outbox_payload_carries_no_value<S: Store>(store: &S) {
    let s = sid();
    store
        .append(s, declared("journal.entry", "the secret text"))
        .await
        .unwrap();
    let events = store
        .claim_events(1000, Duration::from_secs(60))
        .await
        .unwrap();
    let ours = events.iter().find(|e| e.subject_id == s).unwrap();
    let text = ours.payload.to_string();
    assert!(!text.contains("secret"));
    assert!(ours.payload.get("value").is_none());
    assert!(ours.payload.get("origin").is_none());
    for key in [
        "path",
        "source",
        "observed_at",
        "recorded_at",
        "consent",
        "stub",
    ] {
        assert!(ours.payload.get(key).is_some(), "{key}");
    }
}

pub async fn concurrent_appends_keep_order<S: Store>(store: &S) {
    let a = sid();
    let b = sid();
    store
        .append(a, declared("profile.name", "a"))
        .await
        .unwrap();
    store
        .append(b, declared("profile.name", "b"))
        .await
        .unwrap();
    // Fifty appends in flight at once, each naming the other subject.
    let writes = (0..50).map(|i| {
        let (me, other) = if i % 2 == 0 { (a, b) } else { (b, a) };
        async move {
            let mut f = declared(&format!("relations.match.r{}", i / 2), "m");
            f.counterparty_id = Some(other);
            store.append(me, f).await.map(|_| ())
        }
    });
    for r in futures_join_all(writes).await {
        r.unwrap();
    }
    for s in [a, b] {
        let history = all_history(store, s).await;
        assert_eq!(history.len(), 26);
        for w in history.windows(2) {
            assert!((w[0].recorded_at, w[0].id) < (w[1].recorded_at, w[1].id));
        }
        let current = store.current(s, &under_self()).await.unwrap();
        assert_eq!(current.items.len(), 26);
    }
}

/// `futures::future::join_all` without the dependency: poll every future to
/// completion on this task.
async fn futures_join_all<F: Future>(futures: impl IntoIterator<Item = F>) -> Vec<F::Output> {
    let mut set: Vec<std::pin::Pin<Box<F>>> = futures.into_iter().map(Box::pin).collect();
    let mut out: Vec<Option<F::Output>> = (0..set.len()).map(|_| None).collect();
    std::future::poll_fn(|cx| {
        let mut pending = false;
        for (i, f) in set.iter_mut().enumerate() {
            if out[i].is_none() {
                match f.as_mut().poll(cx) {
                    std::task::Poll::Ready(v) => out[i] = Some(v),
                    std::task::Poll::Pending => pending = true,
                }
            }
        }
        if pending {
            std::task::Poll::Pending
        } else {
            std::task::Poll::Ready(())
        }
    })
    .await;
    out.into_iter().map(|o| o.unwrap()).collect()
}

/// Every case, in one call, so a backend runs the whole suite with one line.
#[macro_export]
macro_rules! conformance_suite {
    ($prefix:ident, $make:expr) => {
        mod $prefix {
            use super::*;
            macro_rules! case {
                ($name:ident) => {
                    #[tokio::test]
                    async fn $name() {
                        let store = $make.await;
                        $crate::conformance::$name(&*store).await;
                    }
                };
            }
            case!(append_returns_fact_with_store_set_fields);
            case!(append_rejects_bad_input);
            case!(unknown_subject_is_not_found_never_empty);
            case!(current_is_latest_per_path_and_source);
            case!(history_is_everything_and_at_cuts);
            case!(retract_deletes_values_and_appends_a_tombstone);
            case!(expired_is_excluded_from_current_but_held_in_history);
            case!(consent_filters_reads);
            case!(path_and_source_filters);
            case!(pagination_is_stable);
            case!(idempotency_replays_and_conflicts);
            case!(erasure_request_denies_everything_and_is_idempotent);
            case!(restore_within_the_window_reopens);
            case!(erasure_execute_cascades_and_tombstones_the_counterparty);
            case!(erasure_respects_the_grace_window);
            case!(outbox_events_are_ordered_leased_and_acked);
            case!(outbox_payload_carries_no_value);
            case!(concurrent_appends_keep_order);
        }
    };
}
