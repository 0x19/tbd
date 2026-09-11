//! What a worker did, in a form that replays on another subject: requests are
//! symbolic (`Own`, `Peer(1)`, "the `recorded_at` of step 3", "now minus a
//! second"), responses are plain JSON, so a finding is a file a person can
//! read and `replay` can run.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tbd_proto::ledger::v1::{
    AppendRequest, AppendResponse, CurrentRequest, CurrentResponse, Envelope, EraseRequest,
    EraseResponse, Fact, HistoryRequest, HistoryResponse, RestoreRequest, RetractRequest,
    RetractResponse,
};
use uuid::Uuid;

use crate::client::CallError;

/// Which subject a request names: the worker's own, or one of its peers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SubjectRef {
    /// The subject the trace belongs to.
    Own,
    /// A peer, by index in the binding.
    Peer(u8),
}

/// An instant, resolved when the request is built.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TimeRef {
    /// A fixed instant (`observed_at` of appends: replaying keeps the same bytes,
    /// so idempotency keys keep matching).
    At(DateTime<Utc>),
    /// The `recorded_at` the ledger answered at that step.
    RecordedAtOf(usize),
    /// Now, shifted; negative is the past.
    OffsetFromNow {
        /// Milliseconds.
        ms: i64,
    },
}

/// A request, symbolic.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum Request {
    /// Append.
    Append {
        /// Subject.
        subject: SubjectRef,
        /// Path.
        path: String,
        /// `Source` as the proto number.
        source: i32,
        /// Value as JSON (envelope version 0).
        value: serde_json::Value,
        /// Origin as JSON.
        origin: serde_json::Value,
        /// Confidence.
        confidence: Option<f32>,
        /// Counterparty.
        counterparty: Option<SubjectRef>,
        /// Observed.
        observed_at: TimeRef,
        /// Expiry.
        expires_at: Option<TimeRef>,
        /// Consent scope ids.
        consent: Vec<String>,
        /// Stub flag.
        stub: bool,
        /// Idempotency key.
        idempotency_key: Option<String>,
    },
    /// Current.
    Current {
        /// Subject.
        subject: SubjectRef,
        /// Path filters.
        paths: Vec<String>,
        /// Source filters.
        sources: Vec<i32>,
        /// Scopes.
        scopes: Vec<String>,
        /// Page size (0 = the ledger's default).
        limit: u32,
        /// The `next` cursor of that step's response.
        cursor_from: Option<usize>,
    },
    /// History.
    History {
        /// Subject.
        subject: SubjectRef,
        /// Path filters.
        paths: Vec<String>,
        /// Source filters.
        sources: Vec<i32>,
        /// Scopes.
        scopes: Vec<String>,
        /// Page size.
        limit: u32,
        /// The `next` cursor of that step's response.
        cursor_from: Option<usize>,
        /// The cut.
        at: Option<TimeRef>,
    },
    /// Retract.
    Retract {
        /// Subject.
        subject: SubjectRef,
        /// Path.
        path: String,
        /// Source.
        source: i32,
        /// Origin as JSON.
        origin: serde_json::Value,
    },
    /// Erase.
    Erase {
        /// Subject.
        subject: SubjectRef,
    },
    /// Restore.
    Restore {
        /// Subject.
        subject: SubjectRef,
    },
    /// Wait (for a cascade).
    Sleep {
        /// Milliseconds.
        ms: u64,
    },
    /// A hostile request on the worker's own subject ([`crate::fuzz`]).
    Fuzz {
        /// The case.
        case: crate::fuzz::FuzzCase,
    },
}

impl Request {
    /// The operation name, for metrics.
    #[must_use]
    pub fn op(&self) -> &'static str {
        match self {
            Self::Append { .. } => "append",
            Self::Current { .. } => "current",
            Self::History { .. } => "history",
            Self::Retract { .. } => "retract",
            Self::Erase { .. } => "erase",
            Self::Restore { .. } => "restore",
            Self::Sleep { .. } => "sleep",
            Self::Fuzz { .. } => "fuzz",
        }
    }

    /// The subject the request names, if any.
    #[must_use]
    pub fn subject(&self) -> Option<SubjectRef> {
        match self {
            Self::Append { subject, .. }
            | Self::Current { subject, .. }
            | Self::History { subject, .. }
            | Self::Retract { subject, .. }
            | Self::Erase { subject }
            | Self::Restore { subject } => Some(*subject),
            Self::Sleep { .. } | Self::Fuzz { .. } => None,
        }
    }

    /// Earlier steps this request depends on (`cursor_from`, `RecordedAtOf`).
    #[must_use]
    pub fn depends_on(&self) -> Vec<usize> {
        let mut out = Vec::new();
        match self {
            Self::Append {
                observed_at,
                expires_at,
                ..
            } => {
                if let TimeRef::RecordedAtOf(i) = observed_at {
                    out.push(*i);
                }
                if let Some(TimeRef::RecordedAtOf(i)) = expires_at {
                    out.push(*i);
                }
            }
            Self::Current { cursor_from, .. } => out.extend(*cursor_from),
            Self::History {
                cursor_from, at, ..
            } => {
                out.extend(*cursor_from);
                if let Some(TimeRef::RecordedAtOf(i)) = at {
                    out.push(*i);
                }
            }
            Self::Retract { .. }
            | Self::Erase { .. }
            | Self::Restore { .. }
            | Self::Sleep { .. }
            | Self::Fuzz { .. } => {}
        }
        out
    }
}

/// One request and what came back.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Step {
    /// Position in the trace.
    pub index: usize,
    /// The request.
    pub request: Request,
    /// The response as JSON, when the call succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub response: Option<serde_json::Value>,
    /// The error, when it did not.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<CallError>,
    /// Milliseconds since the worker started.
    pub at_ms: f64,
    /// Whether the outcome was tolerated as a fault and re-driven.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tolerated: bool,
}

/// One broken rule.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Violation {
    /// The invariant, from [`crate::model::invariants::ALL`].
    pub invariant: &'static str,
    /// One sentence naming what was wrong.
    pub message: String,
    /// What the model expected.
    pub expected: serde_json::Value,
    /// What the ledger answered.
    pub actual: serde_json::Value,
}

/// Subject ids a trace is bound to.
#[derive(Debug, Clone)]
pub struct Binding {
    /// `SubjectRef::Own`.
    pub own: Uuid,
    /// `SubjectRef::Peer(i)`.
    pub peers: Vec<Uuid>,
}

impl Binding {
    /// Resolve a reference.
    #[must_use]
    pub fn resolve(&self, s: SubjectRef) -> Uuid {
        match s {
            SubjectRef::Own => self.own,
            SubjectRef::Peer(i) => self.peers.get(usize::from(i)).copied().unwrap_or(self.own),
        }
    }
}

/// Why a symbolic request could not be built.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum MaterialiseError {
    /// A `RecordedAtOf` or `cursor_from` names a step with no such value.
    #[error("step {0} has no usable response")]
    MissingStep(usize),
}

/// The concrete requests, one per RPC.
#[derive(Debug, Clone)]
pub enum Concrete {
    /// Append.
    Append(AppendRequest),
    /// Current.
    Current(CurrentRequest),
    /// History.
    History(HistoryRequest),
    /// Retract.
    Retract(RetractRequest),
    /// Erase.
    Erase(EraseRequest),
    /// Restore.
    Restore(RestoreRequest),
    /// Wait.
    Sleep(std::time::Duration),
    /// A hostile request: the RPC it became, and what a clean answer is.
    Fuzz {
        /// The case's name.
        case: &'static str,
        /// The concrete RPC.
        inner: Box<Concrete>,
        /// The classes the ledger may answer with (`ok` included).
        expect: &'static [&'static str],
    },
}

/// Plaintext JSON envelope (version 0).
#[must_use]
pub fn envelope(v: &serde_json::Value) -> Envelope {
    Envelope {
        version: 0,
        bytes: serde_json::to_vec(v).unwrap_or_default(),
    }
}

/// Envelope bytes back to JSON, or the raw bytes as a string when not JSON.
#[must_use]
pub fn envelope_json(e: Option<&Envelope>) -> serde_json::Value {
    match e {
        None => serde_json::Value::Null,
        Some(e) => serde_json::from_slice(&e.bytes)
            .unwrap_or_else(|_| serde_json::Value::String(format!("<{} bytes>", e.bytes.len()))),
    }
}

/// `prost_types::Timestamp` from a `DateTime`.
#[must_use]
pub fn stamp(t: DateTime<Utc>) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: t.timestamp(),
        nanos: i32::try_from(t.timestamp_subsec_nanos()).unwrap_or(0),
    }
}

/// `DateTime` from a `prost_types::Timestamp`.
#[must_use]
pub fn datetime(t: &prost_types::Timestamp) -> Option<DateTime<Utc>> {
    DateTime::from_timestamp(t.seconds, u32::try_from(t.nanos).unwrap_or(0))
}

/// The `recorded_at` a step's response carries: an appended or tombstone fact.
#[must_use]
pub fn recorded_at_of(step: &Step) -> Option<DateTime<Utc>> {
    let r = step.response.as_ref()?;
    let s = r
        .pointer("/fact/recorded_at")
        .or_else(|| r.pointer("/tombstone/recorded_at"))?
        .as_str()?;
    s.parse::<DateTime<Utc>>().ok()
}

/// The `next` cursor a step's response carries.
#[must_use]
pub fn next_of(step: &Step) -> Option<String> {
    step.response
        .as_ref()?
        .get("next")?
        .as_str()
        .map(str::to_owned)
}

impl TimeRef {
    /// Resolve against the trace so far.
    fn resolve(self, steps: &[Step]) -> Result<DateTime<Utc>, MaterialiseError> {
        match self {
            Self::At(t) => Ok(t),
            Self::RecordedAtOf(i) => steps
                .iter()
                .find(|s| s.index == i)
                .and_then(recorded_at_of)
                .ok_or(MaterialiseError::MissingStep(i)),
            Self::OffsetFromNow { ms } => Ok(Utc::now() + chrono::Duration::milliseconds(ms)),
        }
    }
}

impl Request {
    /// Build the concrete request against `binding`, resolving times and
    /// cursors from the trace so far.
    ///
    /// # Errors
    /// [`MaterialiseError`] when a referenced step has no usable response.
    pub fn materialise(
        &self,
        binding: &Binding,
        steps: &[Step],
    ) -> Result<Concrete, MaterialiseError> {
        let cursor = |from: Option<usize>| -> Result<String, MaterialiseError> {
            match from {
                None => Ok(String::new()),
                Some(i) => steps
                    .iter()
                    .find(|s| s.index == i)
                    .and_then(next_of)
                    .ok_or(MaterialiseError::MissingStep(i)),
            }
        };
        Ok(match self {
            Self::Append {
                subject,
                path,
                source,
                value,
                origin,
                confidence,
                counterparty,
                observed_at,
                expires_at,
                consent,
                stub,
                idempotency_key,
            } => Concrete::Append(AppendRequest {
                subject_id: binding.resolve(*subject).to_string(),
                path: path.clone(),
                source: *source,
                value: Some(envelope(value)),
                origin: Some(envelope(origin)),
                confidence: *confidence,
                counterparty_id: counterparty.map(|c| binding.resolve(c).to_string()),
                observed_at: Some(stamp(observed_at.resolve(steps)?)),
                expires_at: expires_at
                    .map(|e| e.resolve(steps).map(stamp))
                    .transpose()?,
                consent: consent.clone(),
                stub: *stub,
                idempotency_key: idempotency_key.clone().unwrap_or_default(),
            }),
            Self::Current {
                subject,
                paths,
                sources,
                scopes,
                limit,
                cursor_from,
            } => Concrete::Current(CurrentRequest {
                subject_id: binding.resolve(*subject).to_string(),
                paths: paths.clone(),
                sources: sources.clone(),
                scopes: scopes.clone(),
                cursor: cursor(*cursor_from)?,
                limit: *limit,
            }),
            Self::History {
                subject,
                paths,
                sources,
                scopes,
                limit,
                cursor_from,
                at,
            } => Concrete::History(HistoryRequest {
                subject_id: binding.resolve(*subject).to_string(),
                paths: paths.clone(),
                sources: sources.clone(),
                scopes: scopes.clone(),
                cursor: cursor(*cursor_from)?,
                limit: *limit,
                at: at.map(|a| a.resolve(steps).map(stamp)).transpose()?,
            }),
            Self::Retract {
                subject,
                path,
                source,
                origin,
            } => Concrete::Retract(RetractRequest {
                subject_id: binding.resolve(*subject).to_string(),
                path: path.clone(),
                source: *source,
                origin: Some(envelope(origin)),
            }),
            Self::Erase { subject } => Concrete::Erase(EraseRequest {
                subject_id: binding.resolve(*subject).to_string(),
            }),
            Self::Restore { subject } => Concrete::Restore(RestoreRequest {
                subject_id: binding.resolve(*subject).to_string(),
            }),
            Self::Sleep { ms } => Concrete::Sleep(std::time::Duration::from_millis(*ms)),
            Self::Fuzz { case } => Concrete::Fuzz {
                case: case.name(),
                inner: Box::new(case.build(binding)),
                expect: case.expect(),
            },
        })
    }
}

/// A fact as JSON, the shape findings and the UI show.
#[must_use]
pub fn fact_json(f: &Fact) -> serde_json::Value {
    serde_json::json!({
        "subject_id": f.subject_id,
        "id": f.id,
        "path": f.path,
        "source": f.source,
        "value": f.value.as_ref().map(|v| envelope_json(Some(v))),
        "origin": envelope_json(f.origin.as_ref()),
        "confidence": f.confidence,
        "counterparty_id": f.counterparty_id,
        "observed_at": f.observed_at.as_ref().and_then(datetime).map(|t| t.to_rfc3339()),
        "recorded_at": f.recorded_at.as_ref().and_then(datetime).map(|t| t.to_rfc3339()),
        "expires_at": f.expires_at.as_ref().and_then(datetime).map(|t| t.to_rfc3339()),
        "consent": f.consent,
        "stub": f.stub,
    })
}

/// Response JSON per RPC.
#[must_use]
pub fn append_json(r: &AppendResponse) -> serde_json::Value {
    serde_json::json!({ "fact": r.fact.as_ref().map(fact_json), "replayed": r.replayed })
}

/// Response JSON per RPC.
#[must_use]
pub fn current_json(r: &CurrentResponse) -> serde_json::Value {
    serde_json::json!({ "facts": r.facts.iter().map(fact_json).collect::<Vec<_>>(), "next": r.next })
}

/// Response JSON per RPC.
#[must_use]
pub fn history_json(r: &HistoryResponse) -> serde_json::Value {
    serde_json::json!({ "facts": r.facts.iter().map(fact_json).collect::<Vec<_>>(), "next": r.next })
}

/// Response JSON per RPC.
#[must_use]
pub fn retract_json(r: &RetractResponse) -> serde_json::Value {
    serde_json::json!({ "tombstone": r.tombstone.as_ref().map(fact_json) })
}

/// Response JSON per RPC.
#[must_use]
pub fn erase_json(r: &EraseResponse) -> serde_json::Value {
    serde_json::json!({
        "requested_at": r.requested_at.as_ref().and_then(datetime).map(|t| t.to_rfc3339()),
        "executes_after": r.executes_after.as_ref().and_then(datetime).map(|t| t.to_rfc3339()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requests_materialise_against_a_binding_and_earlier_steps() {
        let b = Binding {
            own: Uuid::now_v7(),
            peers: vec![Uuid::now_v7()],
        };
        let t = Utc::now();
        let append = Request::Append {
            subject: SubjectRef::Own,
            path: "profile.name".into(),
            source: 2,
            value: serde_json::json!("Ada"),
            origin: serde_json::json!({"by": "stress"}),
            confidence: Some(1.0),
            counterparty: Some(SubjectRef::Peer(0)),
            observed_at: TimeRef::At(t),
            expires_at: Some(TimeRef::OffsetFromNow { ms: 60_000 }),
            consent: vec!["self".into()],
            stub: false,
            idempotency_key: Some("k".into()),
        };
        let Concrete::Append(a) = append.materialise(&b, &[]).unwrap() else {
            panic!("append");
        };
        assert_eq!(a.subject_id, b.own.to_string());
        assert_eq!(
            a.counterparty_id.as_deref(),
            Some(b.peers[0].to_string().as_str())
        );
        assert_eq!(
            datetime(a.observed_at.as_ref().unwrap())
                .unwrap()
                .timestamp(),
            t.timestamp()
        );
        assert!(a.expires_at.is_some());

        let step = Step {
            index: 0,
            request: append,
            response: Some(
                serde_json::json!({"fact": {"recorded_at": t.to_rfc3339()}, "next": ""}),
            ),
            error: None,
            at_ms: 0.0,
            tolerated: false,
        };
        let cut = Request::History {
            subject: SubjectRef::Own,
            paths: vec![],
            sources: vec![],
            scopes: vec!["self".into()],
            limit: 10,
            cursor_from: None,
            at: Some(TimeRef::RecordedAtOf(0)),
        };
        let Concrete::History(h) = cut.materialise(&b, &[step]).unwrap() else {
            panic!("history");
        };
        assert_eq!(h.at.unwrap().seconds, t.timestamp());
        assert_eq!(cut.depends_on(), vec![0]);
        assert_eq!(
            cut.materialise(&b, &[]).unwrap_err(),
            MaterialiseError::MissingStep(0)
        );
    }

    #[test]
    fn envelopes_round_trip_and_bad_bytes_are_labelled() {
        let v = serde_json::json!({"a": [1, 2]});
        assert_eq!(envelope_json(Some(&envelope(&v))), v);
        let raw = Envelope {
            version: 0,
            bytes: vec![0xff, 0x00],
        };
        assert_eq!(envelope_json(Some(&raw)), serde_json::json!("<2 bytes>"));
    }
}
