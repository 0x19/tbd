//! The `ClickHouse` sink for the outbox: `ledger.facts_events`, one row per
//! event, clear columns only. A `subject.erased` event first deletes every
//! row of that subject (a lightweight `DELETE`, waited on), then inserts the
//! erased event itself, so the deletion is verifiable afterwards.

use chrono::{DateTime, Utc};
use clickhouse::{Client, Row};
use serde::Serialize;

use crate::{
    outbox::{PublishError, Publisher},
    store::{EventKind, OutboxEvent},
};

/// The table, created on start if missing.
pub const TABLE: &str = "ledger.facts_events";
/// The database the client is bound to; inserts name the table unqualified.
const DATABASE: &str = "ledger";
const INSERT_TABLE: &str = "facts_events";

const DDL: &[&str] = &[
    "create database if not exists ledger",
    "create table if not exists ledger.facts_events (
        event_id UUID,
        subject_id UUID,
        kind LowCardinality(String),
        fact_id Nullable(Int64),
        path String,
        source LowCardinality(String),
        tombstone Bool,
        confidence Nullable(Float32),
        counterparty_id Nullable(UUID),
        observed_at Nullable(DateTime64(3, 'UTC')),
        recorded_at DateTime64(3, 'UTC'),
        expires_at Nullable(DateTime64(3, 'UTC')),
        consent Array(String),
        stub Bool
    ) engine = MergeTree
      order by (subject_id, recorded_at, event_id)
      partition by toYYYYMM(recorded_at)",
];

/// One row of `facts_events`.
#[derive(Debug, Clone, Row, Serialize)]
pub struct EventRow {
    /// The outbox event id; the dedupe key.
    #[serde(with = "clickhouse::serde::uuid")]
    pub event_id: uuid::Uuid,
    /// The subject.
    #[serde(with = "clickhouse::serde::uuid")]
    pub subject_id: uuid::Uuid,
    /// `fact.recorded`, `fact.retracted` or `subject.erased`.
    pub kind: String,
    /// The fact, for fact events.
    pub fact_id: Option<i64>,
    /// The fact path; empty for `subject.erased`.
    pub path: String,
    /// The source; empty for `subject.erased`.
    pub source: String,
    /// The event carries a tombstone.
    pub tombstone: bool,
    /// The producer confidence.
    pub confidence: Option<f32>,
    /// The other subject, for relations.
    #[serde(with = "clickhouse::serde::uuid::option")]
    pub counterparty_id: Option<uuid::Uuid>,
    /// When the fact was true.
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub observed_at: Option<DateTime<Utc>>,
    /// When the store recorded the event.
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis")]
    pub recorded_at: DateTime<Utc>,
    /// When the fact stops being current.
    #[serde(with = "clickhouse::serde::chrono::datetime64::millis::option")]
    pub expires_at: Option<DateTime<Utc>>,
    /// Scope ids the fact may be read under.
    pub consent: Vec<String>,
    /// The producer could not do the real computation.
    pub stub: bool,
}

impl EventRow {
    /// The row for an event: clear columns from the payload, nothing else.
    #[must_use]
    pub fn of(event: &OutboxEvent) -> Self {
        let p = &event.payload;
        let text = |key: &str| p.get(key).and_then(|v| v.as_str()).map(str::to_owned);
        let time = |key: &str| {
            p.get(key)
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<DateTime<Utc>>().ok())
        };
        Self {
            event_id: event.event_id,
            subject_id: event.subject_id,
            kind: event.kind.as_str().to_owned(),
            fact_id: event.fact_id.map(|f| f.0),
            path: text("path").unwrap_or_default(),
            source: text("source").unwrap_or_default(),
            tombstone: p
                .get("tombstone")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
            // A confidence is `0..=1`; the column is Float32 like the store's.
            #[allow(clippy::cast_possible_truncation)]
            confidence: p
                .get("confidence")
                .and_then(serde_json::Value::as_f64)
                .map(|c| c as f32),
            counterparty_id: p
                .get("counterparty_id")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse().ok()),
            observed_at: time("observed_at"),
            recorded_at: event.recorded_at,
            expires_at: time("expires_at"),
            consent: p
                .get("consent")
                .and_then(|v| v.as_array())
                .map(|a| {
                    a.iter()
                        .filter_map(|s| s.as_str().map(str::to_owned))
                        .collect()
                })
                .unwrap_or_default(),
            stub: p
                .get("stub")
                .and_then(serde_json::Value::as_bool)
                .unwrap_or(false),
        }
    }
}

/// The publisher.
#[derive(Clone)]
pub struct ClickHousePublisher {
    client: Client,
}

impl std::fmt::Debug for ClickHousePublisher {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClickHousePublisher")
            .finish_non_exhaustive()
    }
}

impl ClickHousePublisher {
    /// A client for `http://user:password@host:8123`. The user info is taken
    /// off the URL and sent as headers; the database is always `ledger`.
    ///
    /// # Errors
    /// The URL does not parse.
    pub fn new(url: &str) -> Result<Self, PublishError> {
        let parsed =
            url::Url::parse(url).map_err(|e| PublishError::Sink(format!("clickhouse url: {e}")))?;
        let mut base = parsed.clone();
        let _ = base.set_username("");
        let _ = base.set_password(None);
        base.set_path("");
        base.set_query(None);
        let mut client = Client::default()
            .with_url(base.as_str().trim_end_matches('/'))
            .with_database(DATABASE);
        if !parsed.username().is_empty() {
            client = client.with_user(parsed.username());
        }
        if let Some(p) = parsed.password() {
            client = client.with_password(p);
        }
        Ok(Self { client })
    }

    /// The client, for tests that verify what was shipped.
    #[must_use]
    pub fn client(&self) -> &Client {
        &self.client
    }

    /// Create the database and table if missing.
    ///
    /// # Errors
    /// `ClickHouse` could not be reached or refused the DDL.
    pub async fn ensure_schema(&self) -> Result<(), PublishError> {
        for ddl in DDL {
            self.client
                .query(ddl)
                .execute()
                .await
                .map_err(|e| PublishError::Sink(format!("ddl: {e}")))?;
        }
        Ok(())
    }

    async fn delete_subject(&self, subject: uuid::Uuid) -> Result<(), PublishError> {
        self.client
            .query("delete from ledger.facts_events where subject_id = ?")
            .bind(subject)
            .with_setting("lightweight_deletes_sync", "2")
            .execute()
            .await
            .map_err(|e| PublishError::Sink(format!("delete: {e}")))?;
        metrics::counter!(tbd_common::metrics::names::LEDGER_ANALYTICS_DELETES_TOTAL).increment(1);
        Ok(())
    }
}

#[async_trait::async_trait]
impl Publisher for ClickHousePublisher {
    fn name(&self) -> &'static str {
        "clickhouse"
    }

    async fn publish(&self, events: &[OutboxEvent]) -> Result<(), PublishError> {
        // Erasures first: every earlier row of the subject goes, then the
        // batch (including the erased event row) is inserted.
        for e in events.iter().filter(|e| e.kind == EventKind::SubjectErased) {
            self.delete_subject(e.subject_id).await?;
        }
        let mut insert = self
            .client
            .insert::<EventRow>(INSERT_TABLE)
            .await
            .map_err(|e| PublishError::Sink(format!("insert: {e}")))?;
        for e in events {
            insert
                .write(&EventRow::of(e))
                .await
                .map_err(|e| PublishError::Sink(format!("write: {e}")))?;
        }
        insert
            .end()
            .await
            .map_err(|e| PublishError::Sink(format!("end: {e}")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Envelope, Fact, FactId, ScopeId, Source};

    /// The row never carries a value or an origin, whatever the event held.
    #[test]
    fn rows_carry_clear_columns_only() {
        let fact = Fact {
            subject_id: uuid::Uuid::now_v7(),
            id: FactId(7),
            path: "journal.entry".into(),
            source: Source::Declared,
            value: Some(Envelope::plaintext(&serde_json::json!("the secret"))),
            origin: Envelope::plaintext(&serde_json::json!({"by": "me"})),
            confidence: Some(0.5),
            counterparty_id: None,
            observed_at: Utc::now(),
            recorded_at: Utc::now(),
            expires_at: None,
            consent: vec![ScopeId::new("self")],
            stub: false,
        };
        let event = OutboxEvent {
            event_id: uuid::Uuid::now_v7(),
            subject_id: fact.subject_id,
            kind: EventKind::FactRecorded,
            fact_id: Some(fact.id),
            payload: OutboxEvent::fact_payload(&fact),
            recorded_at: fact.recorded_at,
        };
        let row = EventRow::of(&event);
        let text = format!("{row:?}");
        assert!(!text.contains("secret") && !text.contains("\"by\""));
        assert_eq!(row.path, "journal.entry");
        assert_eq!(row.consent, ["self"]);
        assert_eq!(row.fact_id, Some(7));
    }

    #[test]
    fn urls_with_credentials_are_split() {
        assert!(ClickHousePublisher::new("http://ledger:pw@clickhouse:8123").is_ok());
        assert!(ClickHousePublisher::new("not a url").is_err());
    }
}
