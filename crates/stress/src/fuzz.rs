//! Hostile requests and what a clean refusal of each looks like. A
//! [`FuzzCase`] is symbolic like any other request (the subject is bound at
//! send time) so a case sits in a trace and replays; `build` turns it into the
//! concrete RPC and names the classes the ledger may answer with. `ok` is a
//! class too: the last case is a valid read that must still work after
//! everything else.

use rand::{Rng, SeedableRng, rngs::StdRng, seq::IndexedRandom};
use serde::{Deserialize, Serialize};
use tbd_proto::ledger::v1::{
    AppendRequest, CurrentRequest, Envelope, EraseRequest, HistoryRequest, RestoreRequest,
    RetractRequest,
};
use uuid::Uuid;

use crate::trace::{Binding, Concrete, SubjectRef, envelope, stamp};

/// One hostile request. Every variant carries what made it hostile, so two
/// runs with the same seed send the same bytes.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "case", rename_all = "snake_case")]
pub enum FuzzCase {
    /// A subject id that is not a UUID.
    BadSubjectId {
        /// The text sent.
        text: String,
    },
    /// `Current` on a subject that never existed.
    UnknownSubjectCurrent {
        /// The id.
        id: Uuid,
    },
    /// `Retract` on a subject that never existed.
    UnknownSubjectRetract {
        /// The id.
        id: Uuid,
    },
    /// `Restore` on a subject that never existed.
    UnknownSubjectRestore {
        /// The id.
        id: Uuid,
    },
    /// `Erase` on a subject that never existed.
    UnknownSubjectErase {
        /// The id.
        id: Uuid,
    },
    /// A read with no scopes.
    EmptyScopes,
    /// A page size over the cap.
    LimitOverCap {
        /// The limit sent.
        limit: u32,
    },
    /// A cursor that never came from the ledger.
    BadCursor {
        /// The text sent.
        text: String,
    },
    /// A path that is not `a.b`.
    BadPath {
        /// The path sent.
        path: String,
    },
    /// A source number outside the enum.
    UnknownSource {
        /// The number sent.
        source: i32,
    },
    /// A confidence outside `0..=1`.
    ConfidenceOutOfRange {
        /// The value sent.
        confidence: f32,
    },
    /// A source that may carry a confidence, sent without one: accepted, the
    /// field is optional.
    MissingConfidence,
    /// A source that carries no confidence, sent with one.
    ConfidenceOnVerified,
    /// A fact with no consent.
    EmptyConsent,
    /// A value envelope over the cap.
    HugeValue {
        /// Bytes of the string sent.
        bytes: usize,
    },
    /// An envelope version the ledger does not know.
    BadEnvelopeVersion {
        /// The version sent.
        version: u32,
    },
    /// A version-0 envelope whose bytes are not JSON.
    NonJsonValue,
    /// An idempotency key longer than the ledger keeps.
    LongIdempotencyKey {
        /// Its length.
        len: usize,
    },
    /// An expiry before the observation.
    ExpiresBeforeObserved,
    /// A counterparty id that is not a UUID.
    BadCounterparty {
        /// The text sent.
        text: String,
    },
    /// A retraction of a key never written on the fuzz subject.
    RetractUnknownKey {
        /// The path.
        path: String,
    },
    /// A valid `Current` on the fuzz subject: must still work.
    ValidRead,
}

/// The classes `[faults] tolerate` and reports use, plus `ok`.
const INVALID: &[&str] = &["invalid_argument"];
const NOT_FOUND: &[&str] = &["not_found"];
const OK: &[&str] = &["ok"];

impl FuzzCase {
    /// The case's name in reports and messages.
    #[must_use]
    pub fn name(&self) -> &'static str {
        match self {
            Self::BadSubjectId { .. } => "bad_subject_id",
            Self::UnknownSubjectCurrent { .. } => "unknown_subject_current",
            Self::UnknownSubjectRetract { .. } => "unknown_subject_retract",
            Self::UnknownSubjectRestore { .. } => "unknown_subject_restore",
            Self::UnknownSubjectErase { .. } => "unknown_subject_erase",
            Self::EmptyScopes => "empty_scopes",
            Self::LimitOverCap { .. } => "limit_over_cap",
            Self::BadCursor { .. } => "bad_cursor",
            Self::BadPath { .. } => "bad_path",
            Self::UnknownSource { .. } => "unknown_source",
            Self::ConfidenceOutOfRange { .. } => "confidence_out_of_range",
            Self::MissingConfidence => "missing_confidence",
            Self::ConfidenceOnVerified => "confidence_on_verified",
            Self::EmptyConsent => "empty_consent",
            Self::HugeValue { .. } => "huge_value",
            Self::BadEnvelopeVersion { .. } => "bad_envelope_version",
            Self::NonJsonValue => "non_json_value",
            Self::LongIdempotencyKey { .. } => "long_idempotency_key",
            Self::ExpiresBeforeObserved => "expires_before_observed",
            Self::BadCounterparty { .. } => "bad_counterparty",
            Self::RetractUnknownKey { .. } => "retract_unknown_key",
            Self::ValidRead => "valid_read",
        }
    }

    /// The classes the ledger may answer with.
    #[must_use]
    pub fn expect(&self) -> &'static [&'static str] {
        match self {
            Self::UnknownSubjectCurrent { .. }
            | Self::UnknownSubjectRetract { .. }
            | Self::UnknownSubjectRestore { .. }
            | Self::UnknownSubjectErase { .. }
            | Self::RetractUnknownKey { .. } => NOT_FOUND,
            Self::ValidRead | Self::MissingConfidence => OK,
            _ => INVALID,
        }
    }

    /// Every case, drawn with `rng` for the parts that vary.
    #[must_use]
    pub fn draw(rng: &mut StdRng, paths: &[String]) -> Self {
        let path = paths
            .choose(rng)
            .cloned()
            .unwrap_or_else(|| "profile.name".into());
        let n = rng.random_range(0..22u8);
        match n {
            0 => Self::BadSubjectId {
                text: ["", "nope", "0x19", "01a0-not-a-uuid"]
                    .choose(rng)
                    .map_or_else(String::new, |s| (*s).to_owned()),
            },
            1 => Self::UnknownSubjectCurrent {
                id: seeded_uuid(rng),
            },
            2 => Self::UnknownSubjectRetract {
                id: seeded_uuid(rng),
            },
            3 => Self::UnknownSubjectRestore {
                id: seeded_uuid(rng),
            },
            4 => Self::UnknownSubjectErase {
                id: seeded_uuid(rng),
            },
            5 => Self::EmptyScopes,
            6 => Self::LimitOverCap {
                limit: *[1001u32, 5000, u32::MAX].choose(rng).unwrap_or(&1001),
            },
            7 => Self::BadCursor {
                text: ["garbage", "AAAA", "1:2:3"]
                    .choose(rng)
                    .map_or_else(String::new, |s| (*s).to_owned()),
            },
            8 => Self::BadPath {
                path: ["nope", "Profile.Name", "a..b", "", "a.b c"]
                    .choose(rng)
                    .map_or_else(String::new, |s| (*s).to_owned()),
            },
            9 => Self::UnknownSource {
                source: *[0, 99, -1].choose(rng).unwrap_or(&99),
            },
            10 => Self::ConfidenceOutOfRange {
                confidence: *[2.0f32, -0.5, 1.5].choose(rng).unwrap_or(&2.0),
            },
            11 => Self::MissingConfidence,
            12 => Self::ConfidenceOnVerified,
            13 => Self::EmptyConsent,
            14 => Self::HugeValue {
                bytes: 64 * 1024 + rng.random_range(1..4096usize),
            },
            15 => Self::BadEnvelopeVersion {
                version: *[1u32, 7, 65_536].choose(rng).unwrap_or(&7),
            },
            16 => Self::NonJsonValue,
            17 => Self::LongIdempotencyKey {
                len: 256 + rng.random_range(1..1024usize),
            },
            18 => Self::ExpiresBeforeObserved,
            19 => Self::BadCounterparty {
                text: "not-a-uuid".into(),
            },
            20 => Self::RetractUnknownKey {
                path: format!("{path}.never"),
            },
            _ => Self::ValidRead,
        }
    }

    /// The concrete request, bound to `binding.own` (the fuzz worker's subject).
    #[must_use]
    pub fn build(&self, binding: &Binding) -> Concrete {
        let own = binding.resolve(SubjectRef::Own).to_string();
        let now = chrono::Utc::now();
        if let Some(append) = self.build_append(&own, now) {
            return Concrete::Append(append);
        }
        let read = |subject: String| CurrentRequest {
            subject_id: subject,
            paths: vec![],
            sources: vec![],
            scopes: vec!["self".into()],
            cursor: String::new(),
            limit: 0,
        };
        let origin = || Some(envelope(&serde_json::json!({"by": "fuzz"})));
        match self {
            Self::UnknownSubjectCurrent { id } => Concrete::Current(read(id.to_string())),
            Self::UnknownSubjectRetract { id } => Concrete::Retract(RetractRequest {
                subject_id: id.to_string(),
                path: "profile.name".into(),
                source: 2,
                origin: origin(),
            }),
            Self::UnknownSubjectRestore { id } => Concrete::Restore(RestoreRequest {
                subject_id: id.to_string(),
            }),
            Self::UnknownSubjectErase { id } => Concrete::Erase(EraseRequest {
                subject_id: id.to_string(),
            }),
            Self::EmptyScopes => Concrete::Current(CurrentRequest {
                scopes: vec![],
                ..read(own)
            }),
            Self::LimitOverCap { limit } => Concrete::History(HistoryRequest {
                subject_id: own,
                paths: vec![],
                sources: vec![],
                scopes: vec!["self".into()],
                cursor: String::new(),
                limit: *limit,
                at: None,
            }),
            Self::BadCursor { text } => Concrete::Current(CurrentRequest {
                cursor: text.clone(),
                ..read(own)
            }),
            Self::RetractUnknownKey { path } => Concrete::Retract(RetractRequest {
                subject_id: own,
                path: path.clone(),
                source: 2,
                origin: origin(),
            }),
            // Every append case was built above; `ValidRead` is the fallthrough.
            _ => Concrete::Current(read(own)),
        }
    }

    /// The append cases: a valid fact with one field made hostile.
    fn build_append(&self, own: &str, now: chrono::DateTime<chrono::Utc>) -> Option<AppendRequest> {
        let valid = |subject: &str| AppendRequest {
            subject_id: subject.to_owned(),
            path: "profile.name".into(),
            source: 2,
            value: Some(envelope(&serde_json::json!("fuzz"))),
            origin: Some(envelope(&serde_json::json!({"by": "fuzz"}))),
            confidence: Some(0.5),
            counterparty_id: None,
            observed_at: Some(stamp(now)),
            expires_at: None,
            consent: vec!["self".into()],
            stub: false,
            idempotency_key: String::new(),
        };
        Some(match self {
            Self::BadSubjectId { text } => valid(text),
            Self::BadPath { path } => AppendRequest {
                path: path.clone(),
                ..valid(own)
            },
            Self::UnknownSource { source } => AppendRequest {
                source: *source,
                ..valid(own)
            },
            Self::ConfidenceOutOfRange { confidence } => AppendRequest {
                confidence: Some(*confidence),
                ..valid(own)
            },
            Self::MissingConfidence => AppendRequest {
                confidence: None,
                ..valid(own)
            },
            Self::ConfidenceOnVerified => AppendRequest {
                source: 1,
                confidence: Some(0.5),
                ..valid(own)
            },
            Self::EmptyConsent => AppendRequest {
                consent: vec![],
                ..valid(own)
            },
            Self::HugeValue { bytes } => AppendRequest {
                value: Some(envelope(&serde_json::json!("x".repeat(*bytes)))),
                ..valid(own)
            },
            Self::BadEnvelopeVersion { version } => AppendRequest {
                value: Some(Envelope {
                    version: *version,
                    bytes: b"\"x\"".to_vec(),
                }),
                ..valid(own)
            },
            Self::NonJsonValue => AppendRequest {
                value: Some(Envelope {
                    version: 0,
                    bytes: b"nope".to_vec(),
                }),
                ..valid(own)
            },
            Self::LongIdempotencyKey { len } => AppendRequest {
                idempotency_key: "k".repeat(*len),
                ..valid(own)
            },
            Self::ExpiresBeforeObserved => AppendRequest {
                expires_at: Some(stamp(now - chrono::Duration::seconds(60))),
                ..valid(own)
            },
            Self::BadCounterparty { text } => AppendRequest {
                counterparty_id: Some(text.clone()),
                ..valid(own)
            },
            _ => return None,
        })
    }
}

/// A UUID drawn from the seeded generator, so unknown subjects are the same
/// across runs with one seed.
fn seeded_uuid(rng: &mut StdRng) -> Uuid {
    let mut bytes = [0u8; 16];
    rng.fill(&mut bytes);
    Uuid::from_bytes(bytes)
}

/// A generator for one worker.
#[must_use]
pub fn rng(seed: u64) -> StdRng {
    StdRng::seed_from_u64(seed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_case_builds_and_names_its_classes() {
        let binding = Binding {
            own: Uuid::now_v7(),
            peers: vec![],
        };
        let mut rng = rng(7);
        let mut seen = std::collections::BTreeSet::new();
        for _ in 0..500 {
            let c = FuzzCase::draw(&mut rng, &["profile.name".to_owned()]);
            let _ = c.build(&binding);
            assert!(!c.expect().is_empty());
            seen.insert(c.name());
            // Round-trips as a request in a trace.
            let json = serde_json::to_string(&c).unwrap();
            let back: FuzzCase = serde_json::from_str(&json).unwrap();
            assert_eq!(back.name(), c.name());
        }
        assert_eq!(seen.len(), 22, "{seen:?}");
    }

    #[test]
    fn the_same_seed_draws_the_same_cases() {
        let a: Vec<_> = (0..20)
            .scan(rng(3), |r, _| Some(FuzzCase::draw(r, &[]).name()))
            .collect();
        let b: Vec<_> = (0..20)
            .scan(rng(3), |r, _| Some(FuzzCase::draw(r, &[]).name()))
            .collect();
        assert_eq!(a, b);
    }
}
