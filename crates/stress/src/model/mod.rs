//! The client-side truth for one subject: every fact the worker was
//! acknowledged, every tombstone, the erasure state, the idempotency keys it
//! used. Built only from requests and their responses, never from reading the
//! ledger back, so a read that disagrees with it is a finding and not a
//! self-fulfilling prophecy. The one exception is `learn`, which fills in the
//! ids and stamps of rows the model knows exist but was never told the id of
//! (a tombstone written by a retract whose acknowledgement was lost).

pub mod generate;
pub mod invariants;

use std::collections::{BTreeMap, BTreeSet};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use tbd_proto::ledger::v1::{AppendRequest, AppendResponse, EraseResponse, Fact, RetractResponse};
use uuid::Uuid;

use crate::trace::{datetime, envelope_json};

/// A UTC instant.
pub type Timestamp = DateTime<Utc>;

/// A row the ledger holds for the subject, as the model knows it.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModelFact {
    /// Known once acknowledged or learned.
    pub id: Option<i64>,
    /// Known once acknowledged or learned.
    pub recorded_at: Option<Timestamp>,
    /// Path.
    pub path: String,
    /// Source as the proto number.
    pub source: i32,
    /// `None` is a tombstone.
    pub value: Option<serde_json::Value>,
    /// Origin.
    pub origin: serde_json::Value,
    /// Confidence.
    pub confidence: Option<f32>,
    /// Counterparty subject.
    pub counterparty: Option<Uuid>,
    /// Observed.
    pub observed_at: Option<Timestamp>,
    /// Expiry.
    pub expires_at: Option<Timestamp>,
    /// Consent scope ids, as sent.
    pub consent: Vec<String>,
    /// Stub flag.
    pub stub: bool,
}

impl ModelFact {
    /// From what the ledger answered.
    #[must_use]
    pub fn from_wire(f: &Fact) -> Self {
        Self {
            id: Some(f.id),
            recorded_at: f.recorded_at.as_ref().and_then(datetime),
            path: f.path.clone(),
            source: f.source,
            value: f.value.as_ref().map(|v| envelope_json(Some(v))),
            origin: envelope_json(f.origin.as_ref()),
            confidence: f.confidence,
            counterparty: f.counterparty_id.as_deref().and_then(|s| s.parse().ok()),
            observed_at: f.observed_at.as_ref().and_then(datetime),
            expires_at: f.expires_at.as_ref().and_then(datetime),
            consent: f.consent.clone(),
            stub: f.stub,
        }
    }

    /// The `(path, source)` key `current` is keyed by.
    #[must_use]
    pub fn key(&self) -> (&str, i32) {
        (&self.path, self.source)
    }

    /// Whether this is a tombstone.
    #[must_use]
    pub fn is_tombstone(&self) -> bool {
        self.value.is_none()
    }

    /// Where the fact stands against expiry at `now`, with a skew band the
    /// model does not judge.
    #[must_use]
    pub fn expiry(&self, now: Timestamp, skew: chrono::Duration) -> Expiry {
        match self.expires_at {
            None => Expiry::Live,
            Some(e) if e <= now - skew => Expiry::Expired,
            Some(e) if e > now + skew => Expiry::Live,
            Some(_) => Expiry::Ambiguous,
        }
    }

    /// `(recorded_at, id)`, the ledger's total order; unknown stamps sort last.
    #[must_use]
    pub fn stamp(&self) -> Option<(Timestamp, i64)> {
        Some((self.recorded_at?, self.id?))
    }

    /// Consent as a set, for comparisons that ignore order.
    #[must_use]
    pub fn consent_set(&self) -> BTreeSet<&str> {
        self.consent.iter().map(String::as_str).collect()
    }
}

/// A fact's standing against its expiry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Expiry {
    /// Not expired.
    Live,
    /// Expired.
    Expired,
    /// Within the clock-skew band; not compared.
    Ambiguous,
}

/// The erasure state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "state", rename_all = "snake_case")]
pub enum Erased {
    /// Open.
    No,
    /// Requested; every call is denied until restore or the window ends.
    Pending {
        /// When.
        requested_at: Option<Timestamp>,
        /// When the cascade may run.
        executes_after: Option<Timestamp>,
    },
    /// Cascaded; the subject is gone for good.
    Executed,
}

/// One idempotency key the worker used.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct KeyUse {
    /// SHA-256 of the request content that first used the key.
    pub fingerprint: String,
    /// The fact the key produced.
    pub fact_id: Option<i64>,
    /// The fact was retracted since (the key now answers `Aborted`).
    pub retracted: bool,
}

/// An append whose acknowledgement was lost and whose re-drive did not settle
/// it: it may or may not be in the ledger. The next full history read decides.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PendingAppend {
    /// What was sent, minus the id and stamp the ledger would have minted.
    pub fact: ModelFact,
    /// Its idempotency key.
    pub key: Option<String>,
}

/// The model of one subject.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubjectModel {
    /// The subject.
    pub id: Uuid,
    /// Rows the ledger holds.
    pub facts: Vec<ModelFact>,
    /// Keys used, by key.
    pub keys: BTreeMap<String, KeyUse>,
    /// Erasure state.
    pub erased: Erased,
    /// Appends with an unknown outcome.
    pub pending: Vec<PendingAppend>,
    /// The greatest `(recorded_at, id)` acknowledged.
    pub last_stamp: Option<(Timestamp, i64)>,
}

impl SubjectModel {
    /// An empty model for `id`.
    #[must_use]
    pub fn new(id: Uuid) -> Self {
        Self {
            id,
            facts: Vec::new(),
            keys: BTreeMap::new(),
            erased: Erased::No,
            pending: Vec::new(),
            last_stamp: None,
        }
    }

    /// Whether the subject exists in the ledger as far as the model knows.
    #[must_use]
    pub fn exists(&self) -> bool {
        !self.facts.is_empty() || !self.pending.is_empty() || self.erased != Erased::No
    }

    /// Rows in `(recorded_at, id)` order, unknown stamps last.
    #[must_use]
    pub fn held(&self) -> Vec<&ModelFact> {
        let mut rows: Vec<&ModelFact> = self.facts.iter().collect();
        rows.sort_by_key(|f| (f.stamp().is_none(), f.stamp()));
        rows
    }

    /// Whether any held row lacks its stamp (a cut cannot be judged then).
    #[must_use]
    pub fn has_unknown_stamps(&self) -> bool {
        self.facts.iter().any(|f| f.stamp().is_none())
    }

    /// Rows recorded at or before `at`.
    #[must_use]
    pub fn held_at(&self, at: Timestamp) -> Vec<&ModelFact> {
        self.held()
            .into_iter()
            .filter(|f| f.recorded_at.is_some_and(|r| r <= at))
            .collect()
    }

    /// What `current` must return: per `(path, source)` the row with the
    /// greatest stamp when it is valued and live. Rows whose expiry is inside
    /// the skew band are returned apart, to be excluded on both sides.
    #[must_use]
    pub fn latest_valued(&self, now: Timestamp, skew: chrono::Duration) -> Latest<'_> {
        let mut by_key: BTreeMap<(&str, i32), &ModelFact> = BTreeMap::new();
        for f in &self.facts {
            let e = by_key.entry(f.key()).or_insert(f);
            if f.stamp() > e.stamp() || (f.stamp().is_none() && e.stamp().is_some()) {
                // An unknown stamp is newer than anything known: it was minted
                // after the last acknowledged write.
                *e = f;
            }
        }
        let mut live = Vec::new();
        let mut ambiguous = Vec::new();
        for f in by_key.into_values() {
            if f.is_tombstone() {
                continue;
            }
            match f.expiry(now, skew) {
                Expiry::Live => live.push(f),
                Expiry::Expired => {}
                Expiry::Ambiguous => ambiguous.push(f),
            }
        }
        live.sort_by_key(|f| (f.stamp().is_none(), f.stamp()));
        Latest { live, ambiguous }
    }

    /// The latest valued row for a key, if any.
    #[must_use]
    pub fn valued(&self, path: &str, source: i32) -> Option<&ModelFact> {
        self.facts
            .iter()
            .filter(|f| f.path == path && f.source == source && !f.is_tombstone())
            .max_by_key(|f| (f.stamp().is_none(), f.stamp()))
    }

    /// Keys that currently hold a valued row.
    #[must_use]
    pub fn valued_keys(&self) -> Vec<(String, i32)> {
        let mut keys: Vec<(String, i32)> = self
            .facts
            .iter()
            .filter(|f| !f.is_tombstone())
            .map(|f| (f.path.clone(), f.source))
            .collect();
        keys.sort();
        keys.dedup();
        keys
    }

    /// Every known id.
    #[must_use]
    pub fn known_ids(&self) -> BTreeSet<i64> {
        self.facts.iter().filter_map(|f| f.id).collect()
    }

    /// Record an acknowledged append.
    pub fn apply_append(&mut self, req: &AppendRequest, resp: &AppendResponse) {
        let Some(fact) = &resp.fact else { return };
        if resp.replayed {
            return;
        }
        let f = ModelFact::from_wire(fact);
        if let Some(s) = f.stamp()
            && self.last_stamp.is_none_or(|l| s > l)
        {
            self.last_stamp = Some(s);
        }
        if !req.idempotency_key.is_empty() {
            self.keys.insert(
                req.idempotency_key.clone(),
                KeyUse {
                    fingerprint: generate::fingerprint(req),
                    fact_id: Some(fact.id),
                    retracted: false,
                },
            );
        }
        self.facts.push(f);
    }

    /// Record an append whose outcome is unknown.
    pub fn apply_pending(&mut self, req: &AppendRequest) {
        self.pending.push(PendingAppend {
            fact: generate::model_fact_of(req),
            key: (!req.idempotency_key.is_empty()).then(|| req.idempotency_key.clone()),
        });
    }

    /// Record an acknowledged retraction: the values go, the tombstone stays.
    pub fn apply_retract(&mut self, path: &str, source: i32, resp: &RetractResponse) {
        self.remove_values(path, source);
        if let Some(t) = &resp.tombstone {
            let f = ModelFact::from_wire(t);
            if let Some(s) = f.stamp()
                && self.last_stamp.is_none_or(|l| s > l)
            {
                self.last_stamp = Some(s);
            }
            self.facts.push(f);
        }
    }

    /// Record a retraction the ledger says already happened (a re-driven
    /// retract answered `NotFound`): the tombstone exists with a stamp the
    /// model never saw; `learn` fills it in.
    pub fn apply_retract_unknown(&mut self, path: &str, source: i32, origin: serde_json::Value) {
        let consent = self
            .valued(path, source)
            .map(|f| f.consent.clone())
            .unwrap_or_default();
        self.remove_values(path, source);
        self.facts.push(ModelFact {
            id: None,
            recorded_at: None,
            path: path.to_owned(),
            source,
            value: None,
            origin,
            confidence: None,
            counterparty: None,
            observed_at: None,
            expires_at: None,
            consent,
            stub: false,
        });
    }

    fn remove_values(&mut self, path: &str, source: i32) {
        for f in &self.facts {
            if f.path == path && f.source == source && !f.is_tombstone() {
                for k in self.keys.values_mut() {
                    if k.fact_id.is_some() && k.fact_id == f.id {
                        k.retracted = true;
                    }
                }
            }
        }
        self.facts
            .retain(|f| !(f.path == path && f.source == source && !f.is_tombstone()));
    }

    /// Record an erasure request.
    pub fn apply_erase(&mut self, resp: &EraseResponse) {
        self.erased = Erased::Pending {
            requested_at: resp.requested_at.as_ref().and_then(datetime),
            executes_after: resp.executes_after.as_ref().and_then(datetime),
        };
    }

    /// Record a restore.
    pub fn apply_restore(&mut self) {
        self.erased = Erased::No;
    }

    /// Record the cascade: the subject is gone.
    pub fn apply_executed(&mut self) {
        self.erased = Erased::Executed;
    }

    /// Record a peer's cascade on this subject: every valued row naming the
    /// erased peer is replaced by a tombstone the model has not seen the id of.
    pub fn apply_counterparty_erased(&mut self, erased: Uuid) {
        let latest = self.latest_naming(erased);
        self.facts
            .retain(|f| f.is_tombstone() || f.counterparty != Some(erased));
        for ((path, source), consent) in latest {
            self.facts.push(ModelFact {
                id: None,
                recorded_at: None,
                path,
                source,
                value: None,
                origin: serde_json::json!({"cause": "counterparty_erased"}),
                confidence: None,
                counterparty: None,
                observed_at: None,
                expires_at: None,
                consent,
                stub: false,
            });
        }
    }

    /// Per `(path, source)`, the consent of the newest valued row naming
    /// `erased`: what a cascade tombstone carries.
    #[must_use]
    pub fn latest_naming(&self, erased: Uuid) -> BTreeMap<(String, i32), Vec<String>> {
        let mut latest: BTreeMap<(String, i32), (&ModelFact, Vec<String>)> = BTreeMap::new();
        for f in self
            .facts
            .iter()
            .filter(|f| !f.is_tombstone() && f.counterparty == Some(erased))
        {
            let key = (f.path.clone(), f.source);
            let newer = latest.get(&key).is_none_or(|(have, _)| {
                (f.stamp().is_none(), f.stamp()) > (have.stamp().is_none(), have.stamp())
            });
            if newer {
                latest.insert(key, (f, f.consent.clone()));
            }
        }
        latest.into_iter().map(|(k, (_, c))| (k, c)).collect()
    }

    /// Fill ids and stamps the model lacks from a complete history page, and
    /// settle pending appends: present in the page, they become facts; absent,
    /// they never happened.
    pub fn learn(&mut self, complete: &[Fact]) {
        let wire: Vec<ModelFact> = complete.iter().map(ModelFact::from_wire).collect();
        let known = self.known_ids();
        // Tombstones with unknown stamps take the id of a matching unclaimed
        // wire tombstone, oldest first.
        for f in self.facts.iter_mut().filter(|f| f.id.is_none()) {
            if let Some(w) = wire.iter().find(|w| {
                w.is_tombstone()
                    && w.path == f.path
                    && w.source == f.source
                    && w.id.is_some_and(|id| !known.contains(&id))
            }) {
                f.id = w.id;
                f.recorded_at = w.recorded_at;
                f.origin.clone_from(&w.origin);
            }
        }
        let known = self.known_ids();
        let pending = std::mem::take(&mut self.pending);
        for p in pending {
            if let Some(w) = wire.iter().find(|w| {
                w.id.is_some_and(|id| !known.contains(&id)) && generate::same_content(&p.fact, w)
            }) {
                if let Some(key) = p.key {
                    self.keys.insert(
                        key,
                        KeyUse {
                            fingerprint: generate::fingerprint_of(&p.fact),
                            fact_id: w.id,
                            retracted: false,
                        },
                    );
                }
                self.facts.push(w.clone());
            }
        }
        if let Some(s) = self.facts.iter().filter_map(ModelFact::stamp).max()
            && self.last_stamp.is_none_or(|l| s > l)
        {
            self.last_stamp = Some(s);
        }
    }
}

/// What `current` must hold.
#[derive(Debug)]
pub struct Latest<'a> {
    /// Rows that must be present.
    pub live: Vec<&'a ModelFact>,
    /// Rows whose expiry is too close to now to judge.
    pub ambiguous: Vec<&'a ModelFact>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tbd_proto::ledger::v1::Envelope;

    fn wire(id: i64, path: &str, value: Option<&str>, secs: i64) -> Fact {
        Fact {
            subject_id: String::new(),
            id,
            path: path.into(),
            source: 2,
            value: value.map(|v| Envelope {
                version: 0,
                bytes: serde_json::to_vec(&serde_json::json!(v)).unwrap(),
            }),
            origin: Some(Envelope {
                version: 0,
                bytes: b"{}".to_vec(),
            }),
            confidence: Some(1.0),
            counterparty_id: None,
            observed_at: None,
            recorded_at: Some(prost_types::Timestamp {
                seconds: secs,
                nanos: 0,
            }),
            expires_at: None,
            consent: vec!["self".into()],
            stub: false,
        }
    }

    fn append(path: &str, value: &str, key: &str) -> AppendRequest {
        AppendRequest {
            subject_id: String::new(),
            path: path.into(),
            source: 2,
            value: Some(Envelope {
                version: 0,
                bytes: serde_json::to_vec(&serde_json::json!(value)).unwrap(),
            }),
            origin: Some(Envelope {
                version: 0,
                bytes: b"{}".to_vec(),
            }),
            confidence: Some(1.0),
            counterparty_id: None,
            observed_at: None,
            expires_at: None,
            consent: vec!["self".into()],
            stub: false,
            idempotency_key: key.into(),
        }
    }

    #[test]
    fn appends_retracts_and_latest() {
        let mut m = SubjectModel::new(Uuid::now_v7());
        m.apply_append(
            &append("profile.name", "a", "k1"),
            &AppendResponse {
                fact: Some(wire(1, "profile.name", Some("a"), 100)),
                replayed: false,
            },
        );
        m.apply_append(
            &append("profile.name", "b", "k2"),
            &AppendResponse {
                fact: Some(wire(2, "profile.name", Some("b"), 101)),
                replayed: false,
            },
        );
        let now = Utc::now();
        let latest = m.latest_valued(now, chrono::Duration::milliseconds(500));
        assert_eq!(latest.live.len(), 1);
        assert_eq!(latest.live[0].id, Some(2));
        assert_eq!(m.held().len(), 2);
        m.apply_retract(
            "profile.name",
            2,
            &RetractResponse {
                tombstone: Some(wire(3, "profile.name", None, 102)),
            },
        );
        assert!(
            m.latest_valued(now, chrono::Duration::zero())
                .live
                .is_empty()
        );
        assert_eq!(
            m.held().len(),
            1,
            "the values are gone, the tombstone stays"
        );
        assert!(m.keys["k1"].retracted && m.keys["k2"].retracted);
        assert_eq!(m.last_stamp.map(|s| s.1), Some(3));
    }

    #[test]
    fn learn_settles_pending_and_unknown_stamps() {
        let mut m = SubjectModel::new(Uuid::now_v7());
        m.apply_pending(&append("profile.bio", "x", "k9"));
        m.apply_retract_unknown("profile.name", 2, serde_json::json!({}));
        assert!(m.has_unknown_stamps());
        let page = vec![
            wire(5, "profile.name", None, 200),
            wire(6, "profile.bio", Some("x"), 201),
        ];
        m.learn(&page);
        assert!(!m.has_unknown_stamps());
        assert!(m.pending.is_empty());
        assert_eq!(m.known_ids().into_iter().collect::<Vec<_>>(), vec![5, 6]);
        assert_eq!(m.keys["k9"].fact_id, Some(6));
        // A pending append absent from the page never happened.
        m.apply_pending(&append("traits.warmth", "y", "k10"));
        m.learn(&page);
        assert!(m.pending.is_empty() && !m.keys.contains_key("k10"));
    }
}
