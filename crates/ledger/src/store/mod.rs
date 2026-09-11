//! The facts store: the crate's API, as a trait two backends implement.
//!
//! A fact is one assertion about an opaque subject with provenance
//! (`docs/design/humans/000-facts-ledger.md`). The store is append-only
//! except for two operations the design names: **retraction** (the valued
//! rows of a `(subject, path, source)` are physically deleted and a tombstone
//! is appended in the same transaction) and **erasure** (a request denies
//! every read, and after a grace window the subject and everything under it
//! is deleted in one transaction, the `erasures` record surviving to drive
//! publication).
//!
//! Nothing here depends on a database driver: [`Store`] is the contract a
//! gRPC-backed implementation could satisfy later without any caller changing.

pub mod clock;
pub mod memory;
pub mod pg;
pub mod registry;
pub mod sql;
pub mod validate;

use std::{fmt, sync::Arc, time::Duration};

use base64::Engine as _;
use serde::{Deserialize, Serialize};

/// The opaque subject identifier. The ledger never learns what it names.
pub type SubjectId = uuid::Uuid;
/// Every timestamp the store holds or returns, in UTC.
pub type Timestamp = chrono::DateTime<chrono::Utc>;

/// Identity of a fact within the store, minted by the store on append.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct FactId(pub i64);

impl fmt::Display for FactId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Who or what produced a fact (`docs/design/humans/001-sources.md`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Source {
    /// An identity provider vouched for it.
    Verified,
    /// The person said so.
    Declared,
    /// A model produced it from inputs.
    Inferred,
    /// A symbolic reading (astrology, numerology, the lenses).
    Symbolic,
    /// The system saw it happen.
    Observed,
}

impl Source {
    /// Every source, in declaration order.
    pub const ALL: [Source; 5] = [
        Source::Verified,
        Source::Declared,
        Source::Inferred,
        Source::Symbolic,
        Source::Observed,
    ];

    /// The `snake_case` name stored in the `source` column.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Source::Verified => "verified",
            Source::Declared => "declared",
            Source::Inferred => "inferred",
            Source::Symbolic => "symbolic",
            Source::Observed => "observed",
        }
    }

    /// Whether this source carries a `confidence` (verified and symbolic do not).
    #[must_use]
    pub fn carries_confidence(self) -> bool {
        !matches!(self, Source::Verified | Source::Symbolic)
    }
}

impl fmt::Display for Source {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl std::str::FromStr for Source {
    type Err = StoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Source::ALL
            .into_iter()
            .find(|k| k.as_str() == s)
            .ok_or_else(|| StoreError::Invalid {
                field: "source",
                reason: format!("unknown source {s:?}"),
            })
    }
}

/// A registry scope id under which a fact may be read
/// (`docs/design/humans/003-consent-and-erasure.md`): `self`, `engine.base`,
/// `tier2@<persona>`. The ledger treats them as opaque strings.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(transparent)]
pub struct ScopeId(pub String);

impl ScopeId {
    /// Build from a string.
    pub fn new(s: impl Into<String>) -> Self {
        Self(s.into())
    }

    /// The id as text.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// The bytes of a `value` or `origin` and the envelope format they are in.
///
/// Version [`Envelope::PLAINTEXT_JSON`] is labelled plaintext JSON. The
/// encryption phase (`docs/design/humans/005-encryption.md`) adds versions,
/// not columns: the store only ever sees bytes and a version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Envelope {
    /// Format version. `0` is plaintext JSON.
    pub version: u16,
    /// The bytes in that format.
    #[serde(with = "bytes_base64")]
    pub bytes: Vec<u8>,
}

mod bytes_base64 {
    use base64::Engine as _;
    use serde::{Deserialize, Deserializer, Serialize, Serializer};

    pub fn serialize<S: Serializer>(bytes: &[u8], s: S) -> Result<S::Ok, S::Error> {
        base64::engine::general_purpose::STANDARD
            .encode(bytes)
            .serialize(s)
    }

    pub fn deserialize<'de, D: Deserializer<'de>>(d: D) -> Result<Vec<u8>, D::Error> {
        let text = String::deserialize(d)?;
        base64::engine::general_purpose::STANDARD
            .decode(text)
            .map_err(serde::de::Error::custom)
    }
}

impl Envelope {
    /// The labelled plaintext JSON format.
    pub const PLAINTEXT_JSON: u16 = 0;

    /// Wrap a JSON value as plaintext.
    #[must_use]
    pub fn plaintext(value: &serde_json::Value) -> Self {
        Self {
            version: Self::PLAINTEXT_JSON,
            bytes: serde_json::to_vec(value).unwrap_or_default(),
        }
    }

    /// The JSON inside a plaintext envelope.
    ///
    /// # Errors
    /// The envelope is not version 0, or its bytes are not JSON.
    pub fn plaintext_json(&self) -> Result<serde_json::Value, StoreError> {
        if self.version != Self::PLAINTEXT_JSON {
            return Err(StoreError::Invalid {
                field: "envelope",
                reason: format!("version {} is not plaintext", self.version),
            });
        }
        serde_json::from_slice(&self.bytes).map_err(|e| StoreError::Invalid {
            field: "envelope",
            reason: format!("not JSON: {e}"),
        })
    }
}

/// What a caller appends. The store mints `id` and `recorded_at`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct NewFact {
    /// Dotted path from the registry, e.g. `traits.warmth`.
    pub path: String,
    /// Who produced it.
    pub source: Source,
    /// The value. Never a tombstone: only retraction and erasure write those.
    pub value: Envelope,
    /// Who or what produced it, in the same envelope version as `value`.
    pub origin: Envelope,
    /// Producer's confidence in `0..=1`; absent for verified and symbolic.
    pub confidence: Option<f32>,
    /// The other subject, for relations and pair facts.
    pub counterparty_id: Option<SubjectId>,
    /// When it was true.
    pub observed_at: Timestamp,
    /// When it stops being current. The caller mirrors it from `origin`.
    pub expires_at: Option<Timestamp>,
    /// Registry scope ids under which it may be read. Never empty.
    pub consent: Vec<ScopeId>,
    /// The producer could not do the real computation.
    pub stub: bool,
    /// Replay key, scoped per subject, remembered for the idempotency TTL.
    pub idempotency_key: Option<String>,
}

/// A fact as held by the store.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Fact {
    /// The subject.
    pub subject_id: SubjectId,
    /// Store-minted identity.
    pub id: FactId,
    /// Dotted path.
    pub path: String,
    /// Who produced it.
    pub source: Source,
    /// The value; `None` is a tombstone.
    pub value: Option<Envelope>,
    /// Who or what produced it.
    pub origin: Envelope,
    /// Producer's confidence.
    pub confidence: Option<f32>,
    /// The other subject, for relations.
    pub counterparty_id: Option<SubjectId>,
    /// When it was true.
    pub observed_at: Timestamp,
    /// When the store learned it. Set by the store, strictly increasing per subject.
    pub recorded_at: Timestamp,
    /// When it stops being current.
    pub expires_at: Option<Timestamp>,
    /// Scope ids under which it may be read.
    pub consent: Vec<ScopeId>,
    /// The producer could not do the real computation.
    pub stub: bool,
}

impl Fact {
    /// A tombstone: the value was retracted.
    #[must_use]
    pub fn is_tombstone(&self) -> bool {
        self.value.is_none()
    }

    /// `true` when `expires_at` is set and at or before `now`.
    #[must_use]
    pub fn is_expired_at(&self, now: Timestamp) -> bool {
        self.expires_at.is_some_and(|e| e <= now)
    }

    /// The consent list intersects `scopes`.
    #[must_use]
    pub fn readable_under(&self, scopes: &[ScopeId]) -> bool {
        self.consent.iter().any(|c| scopes.contains(c))
    }
}

/// The result of an append.
#[derive(Debug, Clone, PartialEq)]
pub struct Appended {
    /// The fact, freshly written or replayed.
    pub fact: Fact,
    /// The idempotency key matched an earlier append; nothing was written.
    pub replayed: bool,
}

/// A path filter: an exact path, or a prefix written as `traits.*`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PathPattern {
    /// Exactly this path.
    Exact(String),
    /// Every path starting with this prefix (including the trailing dot).
    Prefix(String),
}

impl PathPattern {
    /// Parse `traits.warmth` (exact) or `traits.*` (prefix). Any other glob
    /// form is rejected: a bare `*`, `traits.*.x`, `tra*`, `?`, `[`.
    ///
    /// # Errors
    /// The pattern is not one of the two accepted shapes.
    pub fn parse(s: &str) -> Result<Self, StoreError> {
        let invalid = |reason: &str| StoreError::Invalid {
            field: "paths",
            reason: format!("{s:?}: {reason}"),
        };
        if let Some(prefix) = s.strip_suffix(".*") {
            if prefix.is_empty() {
                return Err(invalid("a prefix needs at least one segment"));
            }
            validate::path_shape(prefix, 1).map_err(|e| invalid(&e))?;
            return Ok(Self::Prefix(format!("{prefix}.")));
        }
        validate::path_shape(s, 1).map_err(|e| invalid(&e))?;
        Ok(Self::Exact(s.to_owned()))
    }

    /// Whether `path` matches.
    #[must_use]
    pub fn matches(&self, path: &str) -> bool {
        match self {
            Self::Exact(p) => p == path,
            Self::Prefix(p) => path.starts_with(p.as_str()),
        }
    }
}

/// An opaque page cursor: `(recorded_at, id)` of the last item of a page.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Cursor(String);

impl Cursor {
    const VERSION: &'static str = "v1";

    /// Encode a position.
    #[must_use]
    pub fn encode(recorded_at: Timestamp, id: FactId) -> Self {
        let raw = format!(
            "{}:{}:{}",
            Self::VERSION,
            recorded_at.timestamp_micros(),
            id.0
        );
        Self(base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(raw))
    }

    /// Decode a cursor into its position.
    ///
    /// # Errors
    /// The text is not a cursor this store wrote.
    pub fn decode(&self) -> Result<(Timestamp, FactId), StoreError> {
        let invalid = || StoreError::Invalid {
            field: "cursor",
            reason: "not a cursor".to_owned(),
        };
        let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(&self.0)
            .map_err(|_| invalid())?;
        let raw = String::from_utf8(raw).map_err(|_| invalid())?;
        let mut parts = raw.split(':');
        if parts.next() != Some(Self::VERSION) {
            return Err(invalid());
        }
        let micros: i64 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(invalid)?;
        let id: i64 = parts
            .next()
            .and_then(|p| p.parse().ok())
            .ok_or_else(invalid)?;
        if parts.next().is_some() {
            return Err(invalid());
        }
        let at = chrono::DateTime::from_timestamp_micros(micros).ok_or_else(invalid)?;
        Ok((at, FactId(id)))
    }

    /// The cursor as text, for the wire.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// A cursor from the wire; validated on use.
    pub fn from_wire(s: impl Into<String>) -> Self {
        Self(s.into())
    }
}

/// Filters for `current` and `history`.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Query {
    /// Path filters, OR-ed. Empty means every path.
    pub paths: Vec<PathPattern>,
    /// Source filter. Empty means every source.
    pub sources: Vec<Source>,
    /// Consent intersection. `Some(scopes)` returns facts whose consent
    /// intersects `scopes`; `None` is unfiltered and reachable in-process only.
    pub scopes: Option<Vec<ScopeId>>,
    /// History only: facts recorded at or before this instant.
    pub at: Option<Timestamp>,
    /// Resume after this position.
    pub cursor: Option<Cursor>,
    /// Page size; `0` means [`Query::DEFAULT_LIMIT`], more than [`Query::MAX_LIMIT`] is invalid.
    pub limit: u32,
}

impl Query {
    /// Page size when `limit` is 0.
    pub const DEFAULT_LIMIT: u32 = 100;
    /// Largest page.
    pub const MAX_LIMIT: u32 = 1000;

    /// The effective page size.
    #[must_use]
    pub fn page_size(&self) -> usize {
        if self.limit == 0 {
            Self::DEFAULT_LIMIT as usize
        } else {
            self.limit as usize
        }
    }

    /// Everything readable under `scopes`.
    #[must_use]
    pub fn under(scopes: Vec<ScopeId>) -> Self {
        Self {
            scopes: Some(scopes),
            ..Self::default()
        }
    }

    /// Whether a fact passes the path, source and scope filters.
    #[must_use]
    pub fn admits(&self, fact: &Fact) -> bool {
        if !self.paths.is_empty() && !self.paths.iter().any(|p| p.matches(&fact.path)) {
            return false;
        }
        if !self.sources.is_empty() && !self.sources.contains(&fact.source) {
            return false;
        }
        if let Some(scopes) = &self.scopes
            && !fact.readable_under(scopes)
        {
            return false;
        }
        true
    }
}

/// One page of results.
#[derive(Debug, Clone, PartialEq)]
pub struct Page<T> {
    /// The items, in `(recorded_at, id)` order.
    pub items: Vec<T>,
    /// Cursor for the next page, when there is one.
    pub next: Option<Cursor>,
}

impl<T> Page<T> {
    /// Cut `items` (fetched as `page_size + 1`) into a page and its cursor.
    pub fn cut(
        mut items: Vec<T>,
        page_size: usize,
        position: impl Fn(&T) -> (Timestamp, FactId),
    ) -> Self {
        let next = if items.len() > page_size {
            items.truncate(page_size);
            items.last().map(|last| {
                let (at, id) = position(last);
                Cursor::encode(at, id)
            })
        } else {
            None
        };
        Self { items, next }
    }
}

/// A subject as the store knows it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Subject {
    /// Its id.
    pub id: SubjectId,
    /// First write.
    pub enrolled_at: Timestamp,
    /// Set while an erasure is pending; every read and write is denied.
    pub erased_at: Option<Timestamp>,
}

/// A pending or executed erasure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Erasure {
    /// The subject.
    pub subject_id: SubjectId,
    /// The id of the `subject.erased` event this erasure publishes.
    pub event_id: uuid::Uuid,
    /// When it was requested; the grace window counts from here.
    pub requested_at: Timestamp,
    /// When the cascade ran.
    pub executed_at: Option<Timestamp>,
    /// When it was cancelled by a restore.
    pub cancelled_at: Option<Timestamp>,
    /// When `subject.erased` was published.
    pub published_at: Option<Timestamp>,
}

/// What an executed erasure did.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ErasureExecuted {
    /// The subject that is gone.
    pub subject_id: SubjectId,
    /// When.
    pub executed_at: Timestamp,
    /// Tombstones appended on the surviving side of relations.
    pub tombstoned: u32,
}

/// What an outbox event announces.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EventKind {
    /// A fact was appended.
    #[serde(rename = "fact.recorded")]
    FactRecorded,
    /// A value was retracted; the event carries the tombstone.
    #[serde(rename = "fact.retracted")]
    FactRetracted,
    /// A subject's erasure was executed.
    #[serde(rename = "subject.erased")]
    SubjectErased,
}

impl EventKind {
    /// The wire name.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::FactRecorded => "fact.recorded",
            Self::FactRetracted => "fact.retracted",
            Self::SubjectErased => "subject.erased",
        }
    }
}

impl std::str::FromStr for EventKind {
    type Err = StoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "fact.recorded" => Ok(Self::FactRecorded),
            "fact.retracted" => Ok(Self::FactRetracted),
            "subject.erased" => Ok(Self::SubjectErased),
            other => Err(StoreError::Internal(format!(
                "unknown event kind {other:?}"
            ))),
        }
    }
}

/// One outbox event. The payload carries clear columns only, never a value or
/// an origin (`docs/design/humans/005-encryption.md`).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct OutboxEvent {
    /// `UUIDv7`; consumers dedupe on it.
    pub event_id: uuid::Uuid,
    /// The subject.
    pub subject_id: SubjectId,
    /// What happened.
    pub kind: EventKind,
    /// The fact, for fact events.
    pub fact_id: Option<FactId>,
    /// Clear columns.
    pub payload: serde_json::Value,
    /// When the store recorded it.
    pub recorded_at: Timestamp,
}

impl OutboxEvent {
    /// The clear-column payload of a fact event.
    #[must_use]
    pub fn fact_payload(fact: &Fact) -> serde_json::Value {
        serde_json::json!({
            "path": fact.path,
            "source": fact.source.as_str(),
            "tombstone": fact.is_tombstone(),
            "confidence": fact.confidence,
            "counterparty_id": fact.counterparty_id,
            "observed_at": fact.observed_at,
            "recorded_at": fact.recorded_at,
            "expires_at": fact.expires_at,
            "consent": fact.consent,
            "stub": fact.stub,
        })
    }
}

/// Which backend a store is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StoreKind {
    /// Postgres, the source of truth.
    Postgres,
    /// In memory: tests, chaos stacks, host runs without a database.
    Memory,
}

impl StoreKind {
    /// The label on the wire and in logs.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Postgres => "postgres",
            Self::Memory => "memory",
        }
    }
}

impl fmt::Display for StoreKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Why a store call failed.
#[derive(Debug, thiserror::Error)]
pub enum StoreError {
    /// The named thing does not exist.
    #[error("{what} not found")]
    NotFound {
        /// `subject`, `fact`, `counterparty` or `erasure`.
        what: &'static str,
    },
    /// The subject has an erasure pending or executed; every read and write is denied.
    #[error("subject erased")]
    Erased,
    /// The request is malformed.
    #[error("invalid {field}: {reason}")]
    Invalid {
        /// Which field.
        field: &'static str,
        /// Why.
        reason: String,
    },
    /// The registry refuses this source on this path.
    #[error("forbidden: {writer} may not write {path}")]
    Forbidden {
        /// The path.
        path: String,
        /// The source that tried to write it.
        writer: Source,
    },
    /// The request contradicts an earlier one (an idempotency key reused differently).
    #[error("conflict: {reason}")]
    Conflict {
        /// Why.
        reason: String,
    },
    /// The backend could not be reached; retryable.
    #[error("store unavailable: {source}")]
    Unavailable {
        /// Cause.
        #[source]
        source: Box<dyn std::error::Error + Send + Sync + 'static>,
    },
    /// A row could not be decoded or an invariant broke; not retryable.
    #[error("store internal: {0}")]
    Internal(String),
}

impl StoreError {
    /// Convenience for [`StoreError::Invalid`].
    pub fn invalid(field: &'static str, reason: impl Into<String>) -> Self {
        Self::Invalid {
            field,
            reason: reason.into(),
        }
    }

    /// Convenience for [`StoreError::Unavailable`].
    pub fn unavailable(source: impl std::error::Error + Send + Sync + 'static) -> Self {
        Self::Unavailable {
            source: Box::new(source),
        }
    }
}

/// The facts store.
#[async_trait::async_trait]
pub trait Store: Send + Sync + 'static {
    /// Which backend this is.
    fn kind(&self) -> StoreKind;

    /// The concrete store, for embedders that need backend-specific handles
    /// (the pool gauges).
    fn as_any(&self) -> &dyn std::any::Any;

    /// A round trip to the backend.
    async fn ping(&self) -> Result<(), StoreError>;

    /// The subject, if the store knows it.
    async fn subject(&self, id: SubjectId) -> Result<Option<Subject>, StoreError>;

    /// Append one fact. The subject is created on its first write. A matching
    /// idempotency key replays the earlier fact without writing.
    async fn append(&self, subject: SubjectId, fact: NewFact) -> Result<Appended, StoreError>;

    /// The latest valued, unexpired fact per `(path, source)`.
    async fn current(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError>;

    /// Everything the store still holds, tombstones included.
    async fn history(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError>;

    /// Delete every valued row of `(subject, path, source)` and append a tombstone.
    async fn retract(
        &self,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: Envelope,
    ) -> Result<Fact, StoreError>;

    /// Request erasure: deny every read and write from now, execute after the grace window.
    async fn request_erasure(&self, subject: SubjectId) -> Result<Erasure, StoreError>;

    /// Cancel a pending erasure and reopen the subject.
    async fn restore(&self, subject: SubjectId) -> Result<(), StoreError>;

    /// Execute up to `limit` erasures whose window has passed.
    async fn execute_due_erasures(
        &self,
        grace: Duration,
        limit: u32,
    ) -> Result<Vec<ErasureExecuted>, StoreError>;

    /// Lease up to `limit` unpublished events for `lease`.
    async fn claim_events(
        &self,
        limit: u32,
        lease: Duration,
    ) -> Result<Vec<OutboxEvent>, StoreError>;

    /// Mark events published. Returns how many changed.
    async fn ack_events(&self, event_ids: &[uuid::Uuid]) -> Result<u64, StoreError>;

    /// Forget idempotency keys older than `ttl`. Returns how many.
    async fn purge_idempotency(&self, ttl: Duration) -> Result<u64, StoreError>;
}

#[async_trait::async_trait]
impl<T: Store + ?Sized> Store for Arc<T> {
    fn kind(&self) -> StoreKind {
        (**self).kind()
    }
    fn as_any(&self) -> &dyn std::any::Any {
        (**self).as_any()
    }
    async fn ping(&self) -> Result<(), StoreError> {
        (**self).ping().await
    }
    async fn subject(&self, id: SubjectId) -> Result<Option<Subject>, StoreError> {
        (**self).subject(id).await
    }
    async fn append(&self, subject: SubjectId, fact: NewFact) -> Result<Appended, StoreError> {
        (**self).append(subject, fact).await
    }
    async fn current(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        (**self).current(subject, query).await
    }
    async fn history(&self, subject: SubjectId, query: &Query) -> Result<Page<Fact>, StoreError> {
        (**self).history(subject, query).await
    }
    async fn retract(
        &self,
        subject: SubjectId,
        path: &str,
        source: Source,
        origin: Envelope,
    ) -> Result<Fact, StoreError> {
        (**self).retract(subject, path, source, origin).await
    }
    async fn request_erasure(&self, subject: SubjectId) -> Result<Erasure, StoreError> {
        (**self).request_erasure(subject).await
    }
    async fn restore(&self, subject: SubjectId) -> Result<(), StoreError> {
        (**self).restore(subject).await
    }
    async fn execute_due_erasures(
        &self,
        grace: Duration,
        limit: u32,
    ) -> Result<Vec<ErasureExecuted>, StoreError> {
        (**self).execute_due_erasures(grace, limit).await
    }
    async fn claim_events(
        &self,
        limit: u32,
        lease: Duration,
    ) -> Result<Vec<OutboxEvent>, StoreError> {
        (**self).claim_events(limit, lease).await
    }
    async fn ack_events(&self, event_ids: &[uuid::Uuid]) -> Result<u64, StoreError> {
        (**self).ack_events(event_ids).await
    }
    async fn purge_idempotency(&self, ttl: Duration) -> Result<u64, StoreError> {
        (**self).purge_idempotency(ttl).await
    }
}

/// The idempotency fingerprint: SHA-256 over the fact minus its key, so a key
/// reused with different content is a conflict, not a replay.
#[must_use]
pub fn fingerprint(fact: &NewFact) -> Vec<u8> {
    use sha2::Digest as _;
    let mut h = sha2::Sha256::new();
    let mut feed = |label: &str, bytes: &[u8]| {
        h.update(label.as_bytes());
        h.update((bytes.len() as u64).to_le_bytes());
        h.update(bytes);
    };
    feed("path", fact.path.as_bytes());
    feed("source", fact.source.as_str().as_bytes());
    feed("value.version", &fact.value.version.to_le_bytes());
    feed("value", &fact.value.bytes);
    feed("origin.version", &fact.origin.version.to_le_bytes());
    feed("origin", &fact.origin.bytes);
    feed(
        "confidence",
        &fact.confidence.map_or([0xff; 4], f32::to_le_bytes),
    );
    feed(
        "counterparty",
        fact.counterparty_id
            .map_or([0u8; 16], |c| *c.as_bytes())
            .as_slice(),
    );
    feed(
        "observed_at",
        &fact.observed_at.timestamp_micros().to_le_bytes(),
    );
    feed(
        "expires_at",
        &fact
            .expires_at
            .map_or(i64::MIN, |e| e.timestamp_micros())
            .to_le_bytes(),
    );
    for scope in &fact.consent {
        feed("consent", scope.as_str().as_bytes());
    }
    feed("stub", &[u8::from(fact.stub)]);
    h.finalize().to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_round_trips() {
        let at = chrono::DateTime::from_timestamp_micros(1_700_000_000_123_456).unwrap_or_default();
        let c = Cursor::encode(at, FactId(42));
        assert_eq!(c.decode().ok(), Some((at, FactId(42))));
        assert!(Cursor::from_wire("nope").decode().is_err());
        assert!(
            Cursor::from_wire("djI6MTox").decode().is_err(),
            "wrong version"
        );
    }

    #[test]
    fn path_patterns_parse_and_match() {
        let exact = PathPattern::parse("traits.warmth").ok();
        assert_eq!(exact, Some(PathPattern::Exact("traits.warmth".into())));
        let prefix = PathPattern::parse("traits.*").ok();
        assert_eq!(prefix, Some(PathPattern::Prefix("traits.".into())));
        assert!(prefix.as_ref().is_some_and(|p| p.matches("traits.warmth")));
        assert!(
            prefix
                .as_ref()
                .is_some_and(|p| !p.matches("traitsx.warmth"))
        );
        for bad in [
            "*",
            "traits.*.x",
            "tra*",
            "?",
            "[a]",
            "",
            ".",
            "Traits.x",
            "a..b",
            "a.",
        ] {
            assert!(PathPattern::parse(bad).is_err(), "{bad:?} must be rejected");
        }
    }

    #[test]
    fn fingerprint_ignores_the_key_and_sees_content() {
        let base = NewFact {
            path: "profile.name".into(),
            source: Source::Declared,
            value: Envelope::plaintext(&serde_json::json!("a")),
            origin: Envelope::plaintext(&serde_json::json!({})),
            confidence: Some(1.0),
            counterparty_id: None,
            observed_at: chrono::Utc::now(),
            expires_at: None,
            consent: vec![ScopeId::new("self")],
            stub: false,
            idempotency_key: Some("k1".into()),
        };
        let other_key = NewFact {
            idempotency_key: Some("k2".into()),
            ..base.clone()
        };
        let other_value = NewFact {
            value: Envelope::plaintext(&serde_json::json!("b")),
            ..base.clone()
        };
        assert_eq!(fingerprint(&base), fingerprint(&other_key));
        assert_ne!(fingerprint(&base), fingerprint(&other_value));
    }

    proptest::proptest! {
        #[test]
        fn cursor_round_trips_for_any_position(micros in -62_135_596_800_000_000i64..253_402_300_799_999_999i64, id in i64::MIN..i64::MAX) {
            let at = chrono::DateTime::from_timestamp_micros(micros).unwrap_or_default();
            let c = Cursor::encode(at, FactId(id));
            proptest::prop_assert_eq!(c.decode().ok(), Some((at, FactId(id))));
        }

        #[test]
        fn prefix_patterns_match_exactly_their_prefix(a in "[a-z0-9_]{1,8}", b in "[a-z0-9_]{1,8}", c in "[a-z0-9_]{1,8}") {
            let p = PathPattern::parse(&format!("{a}.*"));
            proptest::prop_assert!(p.is_ok());
            let p = p.unwrap_or(PathPattern::Exact(String::new()));
            let two = format!("{a}.{b}");
            let three = format!("{a}.{b}.{c}");
            let glued = format!("{a}{b}.{c}");
            proptest::prop_assert!(p.matches(&two));
            proptest::prop_assert!(p.matches(&three));
            proptest::prop_assert!(!p.matches(&a));
            proptest::prop_assert!(!p.matches(&glued));
        }
    }
}
