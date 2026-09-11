//! Integration tests: start the real server on an ephemeral port and drive it
//! with the generated client.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod clickhouse;
mod conformance;
mod pg;
mod support;

use std::sync::Arc;

use tbd_ledger::MemoryStore;

conformance_suite!(memory, async { Arc::new(MemoryStore::new()) });
// The metrics decorator is transparent: the same contract through it.
conformance_suite!(instrumented, async {
    Arc::new(tbd_ledger::Instrumented(MemoryStore::new()))
});

use tbd_ledger::{Behavior, FaultHandle, Runtime};
use tbd_proto::ledger::v1::PingRequest;
use tonic_health::pb::{HealthCheckRequest, health_check_response::ServingStatus};

#[tokio::test]
async fn ping_reports_the_store_and_is_not_a_stub() {
    let server = support::start().await;
    let mut client = server.client().await;

    let resp = client
        .ping(PingRequest {
            message: "hello".into(),
        })
        .await
        .unwrap()
        .into_inner();

    assert_eq!(resp.message, "hello");
    assert!(
        !resp.stub,
        "the facts RPCs are real; nothing is a placeholder"
    );
    assert_eq!(resp.store, "memory");
    assert_eq!(resp.version, tbd_common::VERSION);
}

#[tokio::test]
async fn ping_rejects_over_long_message() {
    let server = support::start().await;
    let mut client = server.client().await;

    let err = client
        .ping(PingRequest {
            message: "x".repeat(1025),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::InvalidArgument);
}

#[tokio::test]
async fn health_reports_serving() {
    let server = support::start().await;
    let mut health = server.health().await;

    let resp = health
        .check(HealthCheckRequest {
            service: "tbd.ledger.v1.LedgerService".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.status(), ServingStatus::Serving);
}

#[tokio::test]
async fn injected_fault_surfaces_and_is_counted() {
    let runtime = Runtime {
        fault: FaultHandle::new(Behavior::Error {
            kind: tbd_common::fault::ErrorKind::Unavailable,
            rate: 1.0,
            message: "injected".into(),
        }),
        ..Default::default()
    };
    let server = support::start_with(runtime).await;
    let mut client = server.client().await;

    let err = client
        .ping(PingRequest {
            message: "x".into(),
        })
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::Unavailable);

    server.runtime.fault.set(Behavior::Healthy);
    client
        .ping(PingRequest {
            message: "x".into(),
        })
        .await
        .unwrap();

    let stats = server.runtime.stats.snapshot();
    assert_eq!(stats.requests_total, 2);
    assert_eq!(stats.requests_failed, 1);
}
// ---- the facts RPCs over the memory store (appended to tests/it/main.rs) ----

use tbd_proto::ledger::v1::{
    AppendRequest, CurrentRequest, Envelope, EraseRequest, HistoryRequest, RestoreRequest,
    RetractRequest, Source,
};

fn now() -> prost_types::Timestamp {
    prost_types::Timestamp::from(std::time::SystemTime::now())
}

fn envelope(v: &serde_json::Value) -> Envelope {
    Envelope {
        version: 0,
        bytes: serde_json::to_vec(v).unwrap(),
    }
}

fn append(subject: &str, path: &str, value: &str) -> AppendRequest {
    AppendRequest {
        subject_id: subject.into(),
        path: path.into(),
        source: Source::Declared as i32,
        value: Some(envelope(&serde_json::json!(value))),
        origin: Some(envelope(&serde_json::json!({"by": "test"}))),
        confidence: Some(1.0),
        counterparty_id: None,
        observed_at: Some(now()),
        expires_at: None,
        consent: vec!["self".into()],
        stub: false,
        idempotency_key: String::new(),
    }
}

/// The status code of a failed call; panics on success.
fn code<T: std::fmt::Debug>(r: Result<tonic::Response<T>, tonic::Status>) -> tonic::Code {
    r.unwrap_err().code()
}

fn current(subject: &str) -> CurrentRequest {
    CurrentRequest {
        subject_id: subject.into(),
        paths: vec![],
        sources: vec![],
        scopes: vec!["self".into()],
        cursor: String::new(),
        limit: 0,
    }
}

#[tokio::test]
async fn facts_round_trip_over_grpc() {
    let server = support::start().await;
    let mut client = server.client().await;
    let s = uuid::Uuid::now_v7().to_string();

    let first = client
        .append(append(&s, "profile.name", "Ada"))
        .await
        .unwrap()
        .into_inner();
    assert!(!first.replayed);
    let fact = first.fact.unwrap();
    assert_eq!(fact.subject_id, s);
    assert!(fact.id > 0);
    assert_eq!(fact.source, Source::Declared as i32);
    assert!(fact.recorded_at.is_some());
    let before = fact.recorded_at;

    client
        .append(append(&s, "profile.name", "Ada L."))
        .await
        .unwrap();
    let page = client.current(current(&s)).await.unwrap().into_inner();
    assert_eq!(page.facts.len(), 1);
    assert_eq!(
        page.facts[0].value.as_ref().unwrap().bytes,
        serde_json::to_vec(&serde_json::json!("Ada L.")).unwrap()
    );
    assert!(page.next.is_empty());

    let history = client
        .history(HistoryRequest {
            subject_id: s.clone(),
            paths: vec!["profile.*".into()],
            sources: vec![],
            scopes: vec!["self".into()],
            cursor: String::new(),
            limit: 0,
            at: None,
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(history.facts.len(), 2);

    let tomb = client
        .retract(RetractRequest {
            subject_id: s.clone(),
            path: "profile.name".into(),
            source: Source::Declared as i32,
            origin: Some(envelope(&serde_json::json!({"by": "test"}))),
        })
        .await
        .unwrap()
        .into_inner()
        .tombstone
        .unwrap();
    assert!(tomb.value.is_none(), "a tombstone has no value");
    let page = client.current(current(&s)).await.unwrap().into_inner();
    assert!(page.facts.is_empty());
    // The value is absent from every cut of history.
    let cut = client
        .history(HistoryRequest {
            subject_id: s.clone(),
            paths: vec![],
            sources: vec![],
            scopes: vec!["self".into()],
            cursor: String::new(),
            limit: 0,
            at: before,
        })
        .await
        .unwrap()
        .into_inner();
    assert!(cut.facts.is_empty());
}

#[tokio::test]
async fn erase_denies_reads_and_restore_reopens() {
    let server = support::start().await;
    let mut client = server.client().await;
    let s = uuid::Uuid::now_v7().to_string();
    client
        .append(append(&s, "profile.name", "x"))
        .await
        .unwrap();
    let erasure = client
        .erase(EraseRequest {
            subject_id: s.clone(),
        })
        .await
        .unwrap()
        .into_inner();
    let requested = erasure.requested_at.unwrap();
    let after = erasure.executes_after.unwrap();
    assert!(
        after.seconds - requested.seconds >= 7 * 24 * 3600 - 1,
        "the grace window"
    );
    let err = client.current(current(&s)).await.unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    let err = client
        .append(append(&s, "profile.name", "y"))
        .await
        .unwrap_err();
    assert_eq!(err.code(), tonic::Code::FailedPrecondition);
    client
        .restore(RestoreRequest {
            subject_id: s.clone(),
        })
        .await
        .unwrap();
    let page = client.current(current(&s)).await.unwrap().into_inner();
    assert_eq!(page.facts.len(), 1);
}

#[tokio::test]
async fn bad_requests_get_the_right_codes() {
    let server = support::start().await;
    let mut client = server.client().await;
    let s = uuid::Uuid::now_v7().to_string();

    let mut req = append("not-a-uuid", "profile.name", "x");
    assert_eq!(
        code(client.append(req.clone()).await),
        tonic::Code::InvalidArgument
    );
    req = append(&s, "nope", "x");
    assert_eq!(
        code(client.append(req.clone()).await),
        tonic::Code::InvalidArgument
    );
    req = append(&s, "profile.name", "x");
    req.source = Source::Unspecified as i32;
    assert_eq!(
        code(client.append(req.clone()).await),
        tonic::Code::InvalidArgument
    );
    req = append(&s, "profile.name", "x");
    req.consent.clear();
    assert_eq!(code(client.append(req).await), tonic::Code::InvalidArgument);

    assert_eq!(
        code(client.current(current(&s)).await),
        tonic::Code::NotFound
    );
    client
        .append(append(&s, "profile.name", "x"))
        .await
        .unwrap();
    let mut no_scopes = current(&s);
    no_scopes.scopes.clear();
    assert_eq!(
        code(client.current(no_scopes).await),
        tonic::Code::InvalidArgument
    );
    let mut bad_cursor = current(&s);
    bad_cursor.cursor = "garbage".into();
    assert_eq!(
        code(client.current(bad_cursor).await),
        tonic::Code::InvalidArgument
    );

    let mut keyed = append(&s, "profile.bio", "a");
    keyed.idempotency_key = "k1".into();
    let first = client.append(keyed.clone()).await.unwrap().into_inner();
    let again = client.append(keyed.clone()).await.unwrap().into_inner();
    assert!(again.replayed);
    assert_eq!(again.fact.unwrap().id, first.fact.unwrap().id);
    keyed.value = Some(envelope(&serde_json::json!("b")));
    assert_eq!(code(client.append(keyed).await), tonic::Code::Aborted);

    // A value over the envelope cap is refused by validation; a request over
    // the message cap is refused by the codec before it is buffered.
    let mut big = append(&s, "profile.bio", "x");
    big.value = Some(envelope(&serde_json::json!(
        "x".repeat(tbd_ledger::store::validate::MAX_ENVELOPE_LEN)
    )));
    assert_eq!(code(client.append(big).await), tonic::Code::InvalidArgument);
    let mut huge = append(&s, "profile.bio", "x");
    huge.value = Some(envelope(&serde_json::json!(
        "x".repeat(tbd_ledger::store::validate::MAX_MESSAGE_LEN)
    )));
    assert_eq!(code(client.append(huge).await), tonic::Code::OutOfRange);

    assert_eq!(
        code(
            client
                .retract(RetractRequest {
                    subject_id: s.clone(),
                    path: "profile.missing".into(),
                    source: Source::Declared as i32,
                    origin: Some(envelope(&serde_json::json!({}))),
                })
                .await
        ),
        tonic::Code::NotFound
    );
    assert_eq!(
        code(
            client
                .restore(RestoreRequest {
                    subject_id: uuid::Uuid::now_v7().to_string(),
                })
                .await
        ),
        tonic::Code::NotFound
    );
}
