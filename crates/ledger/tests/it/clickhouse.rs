//! The `ClickHouse` sink against a real server: rows land, the erased subject's
//! rows are gone (verified below the deletion mask), and the drainer ships
//! what the store produced.
//!
//! The server comes from `LEDGER_TEST_CLICKHOUSE_URL` (CI's services block)
//! or a container this test starts through Docker. With neither the test
//! fails and says so.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{sync::Arc, time::Duration};

use tbd_ledger::{
    MemoryStore,
    clickhouse::{ClickHousePublisher, EventRow},
    outbox::{Drainer, Publisher},
    store::{Envelope, EventKind, NewFact, OutboxEvent, ScopeId, Source, Store},
};
use testcontainers::{
    ContainerAsync, GenericImage, ImageExt,
    core::{IntoContainerPort, WaitFor},
    runners::AsyncRunner,
};

const IMAGE: (&str, &str) = ("clickhouse/clickhouse-server", "26.8.2.7");

struct ChTest {
    publisher: ClickHousePublisher,
    _container: Option<ContainerAsync<GenericImage>>,
}

async fn server() -> ChTest {
    let (url, container) = if let Ok(url) = std::env::var("LEDGER_TEST_CLICKHOUSE_URL")
        && !url.trim().is_empty()
    {
        (url, None)
    } else {
        let container = GenericImage::new(IMAGE.0, IMAGE.1)
            .with_exposed_port(8123.tcp())
            // ClickHouse logs to files inside the container, not to its
            // streams, so readiness is the schema call below succeeding.
            .with_wait_for(WaitFor::Nothing)
            .with_env_var("CLICKHOUSE_USER", "test")
            .with_env_var("CLICKHOUSE_PASSWORD", "test")
            .with_env_var("CLICKHOUSE_DB", "ledger")
            .start()
            .await
            .unwrap_or_else(|e| {
                panic!(
                    "the ClickHouse sink tests need Docker (to start {}:{}) or LEDGER_TEST_CLICKHOUSE_URL: {e}",
                    IMAGE.0, IMAGE.1
                )
            });
        let port = container.get_host_port_ipv4(8123).await.unwrap();
        let host = container.get_host().await.unwrap();
        (format!("http://test:test@{host}:{port}"), Some(container))
    };
    let publisher = ClickHousePublisher::new(&url).unwrap();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(90);
    loop {
        match publisher.ensure_schema().await {
            Ok(()) => break,
            Err(e) if tokio::time::Instant::now() < deadline => {
                tracing::debug!(error = %e, "clickhouse not ready yet");
                tokio::time::sleep(Duration::from_millis(500)).await;
            }
            Err(e) => panic!("clickhouse schema: {e}"),
        }
    }
    ChTest {
        publisher,
        _container: container,
    }
}

fn fact(path: &str) -> NewFact {
    NewFact {
        path: path.into(),
        source: Source::Declared,
        value: Envelope::plaintext(&serde_json::json!("the secret")),
        origin: Envelope::plaintext(&serde_json::json!({"by": "me"})),
        confidence: Some(1.0),
        counterparty_id: None,
        observed_at: chrono::Utc::now(),
        expires_at: None,
        consent: vec![ScopeId::new("self")],
        stub: false,
        idempotency_key: None,
    }
}

async fn count(t: &ChTest, subject: uuid::Uuid, apply_mask: bool) -> u64 {
    t.publisher
        .client()
        .query("select count() from ledger.facts_events where subject_id = ?")
        .bind(subject)
        .with_setting("apply_deleted_mask", if apply_mask { "1" } else { "0" })
        .fetch_one::<u64>()
        .await
        .unwrap()
}

#[tokio::test]
async fn events_land_and_carry_no_value() {
    let t = server().await;
    let store = Arc::new(MemoryStore::new());
    let s = uuid::Uuid::now_v7();
    store.append(s, fact("journal.entry")).await.unwrap();
    store
        .retract(
            s,
            "journal.entry",
            Source::Declared,
            Envelope::plaintext(&serde_json::json!({})),
        )
        .await
        .unwrap();
    let drainer = Drainer::new(
        store.clone(),
        t.publisher.clone(),
        100,
        Duration::from_millis(10),
        Duration::from_secs(30),
    );
    assert_eq!(drainer.tick().await.unwrap(), 2);
    assert_eq!(
        drainer.tick().await.unwrap(),
        0,
        "acked: nothing left to ship"
    );
    assert_eq!(count(&t, s, true).await, 2);
    let rows: Vec<(String, bool, String)> = t
        .publisher
        .client()
        .query("select kind, tombstone, path from ledger.facts_events where subject_id = ? order by recorded_at, event_id")
        .bind(s)
        .fetch_all()
        .await
        .unwrap();
    assert_eq!(
        rows,
        [
            (
                "fact.recorded".to_owned(),
                false,
                "journal.entry".to_owned()
            ),
            (
                "fact.retracted".to_owned(),
                true,
                "journal.entry".to_owned()
            ),
        ]
    );
    // Nothing in the table can hold the value: there is no column for it, and
    // the row builder drops everything but the clear columns.
    let columns: Vec<String> = t
        .publisher
        .client()
        .query(
            "select name from system.columns where database = 'ledger' and table = 'facts_events'",
        )
        .fetch_all()
        .await
        .unwrap();
    assert!(!columns.iter().any(|c| c == "value" || c == "origin"));
}

#[tokio::test]
async fn an_erased_subject_is_deleted_below_the_mask_and_its_event_remains() {
    let t = server().await;
    let s = uuid::Uuid::now_v7();
    // Direct rows, then the erased event through the publisher.
    let fact_rows: Vec<OutboxEvent> = (0..3)
        .map(|i| OutboxEvent {
            event_id: uuid::Uuid::now_v7(),
            subject_id: s,
            kind: EventKind::FactRecorded,
            fact_id: Some(tbd_ledger::store::FactId(i)),
            payload: serde_json::json!({"path": "profile.name", "source": "declared", "consent": ["self"], "stub": false}),
            recorded_at: chrono::Utc::now(),
        })
        .collect();
    t.publisher.publish(&fact_rows).await.unwrap();
    assert_eq!(count(&t, s, true).await, 3);
    let erased = OutboxEvent {
        event_id: uuid::Uuid::now_v7(),
        subject_id: s,
        kind: EventKind::SubjectErased,
        fact_id: None,
        payload: serde_json::json!({"requested_at": chrono::Utc::now(), "executed_at": chrono::Utc::now()}),
        recorded_at: chrono::Utc::now(),
    };
    t.publisher
        .publish(std::slice::from_ref(&erased))
        .await
        .unwrap();
    // Logically: only the erased event.
    assert_eq!(count(&t, s, true).await, 1);
    // Physically: force the merge, then look below the deletion mask.
    t.publisher
        .client()
        .query("optimize table ledger.facts_events final")
        .execute()
        .await
        .unwrap();
    assert_eq!(
        count(&t, s, false).await,
        1,
        "the deleted rows are gone from disk"
    );
    let row: EventRow = EventRow::of(&erased);
    assert_eq!(row.kind, "subject.erased");
}
