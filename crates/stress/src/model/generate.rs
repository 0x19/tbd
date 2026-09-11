//! Seeded generators and the fingerprint the model keys idempotency on.

use chrono::{DateTime, Utc};
use rand::{Rng, rngs::StdRng, seq::IndexedRandom};
use sha2::{Digest, Sha256};
use tbd_proto::ledger::v1::AppendRequest;

use super::ModelFact;
use crate::trace::{datetime, envelope_json};

/// Proto `Source` numbers the workers write with, and whether the ledger
/// requires a confidence for each.
pub const SOURCES: &[(i32, bool)] = &[(1, false), (2, true), (3, true), (4, false), (5, true)];

/// Names for the report: the proto enum in `snake_case`.
#[must_use]
pub fn source_name(source: i32) -> &'static str {
    match source {
        1 => "verified",
        2 => "declared",
        3 => "inferred",
        4 => "symbolic",
        5 => "observed",
        _ => "unspecified",
    }
}

/// A small JSON value: a string, a number, or a flat object.
pub fn value(rng: &mut StdRng) -> serde_json::Value {
    match rng.random_range(0..4) {
        0 => serde_json::json!(format!("v{}", rng.random::<u32>())),
        1 => serde_json::json!(rng.random_range(-1000..1000)),
        2 => serde_json::json!(f64::from(rng.random_range(0..10_000)) / 100.0),
        _ => serde_json::json!({"n": rng.random::<u16>(), "s": format!("s{}", rng.random::<u8>())}),
    }
}

/// A non-empty subset of the scope vocabulary, in vocabulary order.
pub fn consent(rng: &mut StdRng, scopes: &[String]) -> Vec<String> {
    let mut out: Vec<String> = scopes
        .iter()
        .filter(|_| rng.random_bool(0.5))
        .cloned()
        .collect();
    if out.is_empty()
        && let Some(s) = scopes.choose(rng)
    {
        out.push(s.clone());
    }
    out
}

/// A source and a confidence that fits it.
pub fn source(rng: &mut StdRng) -> (i32, Option<f32>) {
    let (s, needs_confidence) = SOURCES[rng.random_range(0..SOURCES.len())];
    if needs_confidence {
        (s, Some(f32::from(rng.random_range(0..=100u8)) / 100.0))
    } else {
        (s, None)
    }
}

/// A fresh idempotency key.
pub fn key(rng: &mut StdRng) -> String {
    format!("k-{:016x}", rng.random::<u64>())
}

/// The model's view of an append before the ledger stamped it.
#[must_use]
pub fn model_fact_of(req: &AppendRequest) -> ModelFact {
    ModelFact {
        id: None,
        recorded_at: None,
        path: req.path.clone(),
        source: req.source,
        value: Some(envelope_json(req.value.as_ref())),
        origin: envelope_json(req.origin.as_ref()),
        confidence: req.confidence,
        counterparty: req.counterparty_id.as_deref().and_then(|s| s.parse().ok()),
        observed_at: req.observed_at.as_ref().and_then(datetime),
        expires_at: req.expires_at.as_ref().and_then(datetime),
        consent: req.consent.clone(),
        stub: req.stub,
    }
}

/// SHA-256 over the content the ledger fingerprints for idempotency.
#[must_use]
pub fn fingerprint(req: &AppendRequest) -> String {
    fingerprint_of(&model_fact_of(req))
}

/// The same fingerprint from a model fact.
#[must_use]
pub fn fingerprint_of(f: &ModelFact) -> String {
    let mut h = Sha256::new();
    h.update(f.path.as_bytes());
    h.update(f.source.to_le_bytes());
    h.update(serde_json::to_vec(&f.value).unwrap_or_default());
    h.update(serde_json::to_vec(&f.origin).unwrap_or_default());
    h.update(format!("{:?}", f.confidence).as_bytes());
    h.update(format!("{:?}", f.counterparty).as_bytes());
    h.update(format!("{:?}", f.observed_at.map(micros)).as_bytes());
    h.update(format!("{:?}", f.expires_at.map(micros)).as_bytes());
    h.update(f.consent.join(",").as_bytes());
    h.update([u8::from(f.stub)]);
    format!("{:x}", h.finalize())
}

/// Whether a wire fact carries the content of a model fact (ids and stamps
/// aside): how a pending append is recognised in a history page.
#[must_use]
pub fn same_content(a: &ModelFact, b: &ModelFact) -> bool {
    a.path == b.path
        && a.source == b.source
        && a.value == b.value
        && a.origin == b.origin
        && close(a.confidence, b.confidence)
        && a.counterparty == b.counterparty
        && a.observed_at.map(micros) == b.observed_at.map(micros)
        && a.expires_at.map(micros) == b.expires_at.map(micros)
        && a.consent_set() == b.consent_set()
        && a.stub == b.stub
}

/// Microseconds since the epoch: Postgres keeps six digits, prost nine.
#[must_use]
pub fn micros(t: DateTime<Utc>) -> i64 {
    t.timestamp_micros()
}

/// `f32` confidences equal to what a `real` column keeps.
#[must_use]
pub fn close(a: Option<f32>, b: Option<f32>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => (a - b).abs() < 1e-5,
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use rand::SeedableRng;

    use super::*;

    #[test]
    fn generators_are_seeded_and_consent_is_never_empty() {
        let scopes = vec!["self".to_owned(), "engine.base".to_owned()];
        let mut a = StdRng::seed_from_u64(7);
        let mut b = StdRng::seed_from_u64(7);
        for _ in 0..50 {
            assert_eq!(value(&mut a), value(&mut b));
            let c = consent(&mut a, &scopes);
            assert!(!c.is_empty());
            let _ = consent(&mut b, &scopes);
            assert_eq!(key(&mut a), key(&mut b));
            let (s, conf) = source(&mut a);
            let _ = source(&mut b);
            assert_eq!(conf.is_some(), SOURCES.iter().any(|(x, n)| *x == s && *n));
        }
    }
}
