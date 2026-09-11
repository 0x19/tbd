//! Write and query validation shared by every backend, so a raw store refuses
//! a bad write without a decorator in front of it.

use super::{
    Envelope, NewFact, Query, ScopeId, StoreError,
    registry::{Registry, RegistryError},
};

/// Longest path in bytes.
pub const MAX_PATH_LEN: usize = 256;
/// Longest scope id in bytes.
pub const MAX_SCOPE_LEN: usize = 64;
/// Longest idempotency key in bytes.
pub const MAX_KEY_LEN: usize = 255;
/// Most consent ids on one fact.
pub const MAX_CONSENT: usize = 64;

/// `[a-z0-9_]+(\.[a-z0-9_]+)*` with at least `min_segments` segments and at
/// most [`MAX_PATH_LEN`] bytes.
///
/// # Errors
/// The reason, as text.
pub fn path_shape(path: &str, min_segments: usize) -> Result<(), String> {
    if path.is_empty() {
        return Err("empty".into());
    }
    if path.len() > MAX_PATH_LEN {
        return Err(format!("longer than {MAX_PATH_LEN} bytes"));
    }
    let mut segments = 0;
    for segment in path.split('.') {
        if segment.is_empty() {
            return Err("empty segment".into());
        }
        if !segment
            .bytes()
            .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        {
            return Err(format!(
                "segment {segment:?}: only a-z, 0-9 and _ are allowed"
            ));
        }
        segments += 1;
    }
    if segments < min_segments {
        return Err(format!("at least {min_segments} dotted segments"));
    }
    Ok(())
}

/// A scope id is `[a-z0-9_.@-]{1,64}`.
///
/// # Errors
/// The reason, as text.
pub fn scope_id(scope: &ScopeId) -> Result<(), String> {
    let s = scope.as_str();
    if s.is_empty() || s.len() > MAX_SCOPE_LEN {
        return Err(format!("{s:?}: 1 to {MAX_SCOPE_LEN} bytes"));
    }
    if !s.bytes().all(|b| {
        b.is_ascii_lowercase() || b.is_ascii_digit() || matches!(b, b'_' | b'.' | b'@' | b'-')
    }) {
        return Err(format!("{s:?}: only a-z, 0-9, _ . @ - are allowed"));
    }
    Ok(())
}

/// Everything a new fact must satisfy before any backend touches it.
///
/// # Errors
/// [`StoreError::Invalid`] naming the field, or [`StoreError::Forbidden`] from the registry.
pub fn new_fact(fact: &NewFact, registry: &dyn Registry) -> Result<(), StoreError> {
    match registry.check_write(&fact.path, fact.source) {
        Ok(()) => {}
        Err(RegistryError::Malformed { reason, .. }) => {
            return Err(StoreError::invalid("path", reason));
        }
        Err(RegistryError::Unknown(path)) => {
            return Err(StoreError::invalid(
                "path",
                format!("unknown path {path:?}"),
            ));
        }
        Err(RegistryError::Forbidden { path, writer }) => {
            return Err(StoreError::Forbidden { path, writer });
        }
    }
    if let Some(c) = fact.confidence {
        if !fact.source.carries_confidence() {
            return Err(StoreError::invalid(
                "confidence",
                format!("{} facts carry no confidence", fact.source),
            ));
        }
        if !(0.0..=1.0).contains(&c) {
            return Err(StoreError::invalid("confidence", "must be within 0..=1"));
        }
    }
    envelope("value", &fact.value)?;
    envelope("origin", &fact.origin)?;
    if fact.value.version != fact.origin.version {
        return Err(StoreError::invalid(
            "origin",
            "value and origin must share an envelope version",
        ));
    }
    consent(&fact.consent)?;
    if let Some(key) = &fact.idempotency_key
        && (key.is_empty() || key.len() > MAX_KEY_LEN)
    {
        return Err(StoreError::invalid(
            "idempotency_key",
            format!("1 to {MAX_KEY_LEN} bytes"),
        ));
    }
    if let Some(e) = fact.expires_at
        && e <= fact.observed_at
    {
        return Err(StoreError::invalid(
            "expires_at",
            "must be after observed_at",
        ));
    }
    Ok(())
}

/// A plaintext envelope must hold JSON; other versions are opaque.
///
/// # Errors
/// [`StoreError::Invalid`] naming the field.
pub fn envelope(field: &'static str, e: &Envelope) -> Result<(), StoreError> {
    if e.version == Envelope::PLAINTEXT_JSON {
        serde_json::from_slice::<serde::de::IgnoredAny>(&e.bytes).map_err(|err| {
            StoreError::invalid(field, format!("plaintext envelope is not JSON: {err}"))
        })?;
    }
    Ok(())
}

/// Consent is non-empty, bounded, and every id well-formed.
///
/// # Errors
/// [`StoreError::Invalid`] on `consent`.
pub fn consent(consent: &[ScopeId]) -> Result<(), StoreError> {
    if consent.is_empty() {
        return Err(StoreError::invalid("consent", "at least one scope id"));
    }
    if consent.len() > MAX_CONSENT {
        return Err(StoreError::invalid(
            "consent",
            format!("at most {MAX_CONSENT} scope ids"),
        ));
    }
    for s in consent {
        scope_id(s).map_err(|reason| StoreError::invalid("consent", reason))?;
    }
    Ok(())
}

/// A query's bounds and cursor.
///
/// # Errors
/// [`StoreError::Invalid`] naming the field.
pub fn query(q: &Query, history: bool) -> Result<(), StoreError> {
    if q.limit > Query::MAX_LIMIT {
        return Err(StoreError::invalid(
            "limit",
            format!("at most {}", Query::MAX_LIMIT),
        ));
    }
    if !history && q.at.is_some() {
        return Err(StoreError::invalid("at", "only history takes `at`"));
    }
    if let Some(c) = &q.cursor {
        c.decode()?;
    }
    if let Some(scopes) = &q.scopes {
        for s in scopes {
            scope_id(s).map_err(|reason| StoreError::invalid("scopes", reason))?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::store::{Source, registry::ShapeOnly};

    fn fact() -> NewFact {
        NewFact {
            path: "profile.name".into(),
            source: Source::Declared,
            value: Envelope::plaintext(&serde_json::json!("Ada")),
            origin: Envelope::plaintext(&serde_json::json!({"by": "self"})),
            confidence: Some(1.0),
            counterparty_id: None,
            observed_at: chrono::Utc::now(),
            expires_at: None,
            consent: vec![ScopeId::new("self")],
            stub: false,
            idempotency_key: None,
        }
    }

    fn field(r: &Result<(), StoreError>) -> Option<&'static str> {
        match r {
            Err(StoreError::Invalid { field, .. }) => Some(field),
            _ => None,
        }
    }

    #[test]
    fn a_good_fact_passes() {
        assert!(new_fact(&fact(), &ShapeOnly).is_ok());
    }

    #[test]
    fn each_rule_names_its_field() {
        let mut f = fact();
        f.path = "nope".into();
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("path"));

        let mut f = fact();
        f.source = Source::Verified;
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("confidence"));

        let mut f = fact();
        f.confidence = Some(1.5);
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("confidence"));

        let mut f = fact();
        f.value = Envelope {
            version: 0,
            bytes: b"{not json".to_vec(),
        };
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("value"));

        let mut f = fact();
        f.origin.version = 1;
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("origin"));

        let mut f = fact();
        f.consent.clear();
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("consent"));

        let mut f = fact();
        f.consent = vec![ScopeId::new("Bad Scope")];
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("consent"));

        let mut f = fact();
        f.idempotency_key = Some(String::new());
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("idempotency_key"));

        let mut f = fact();
        f.expires_at = Some(f.observed_at);
        assert_eq!(field(&new_fact(&f, &ShapeOnly)), Some("expires_at"));
    }

    #[test]
    fn queries_are_bounded() {
        let q = Query {
            limit: 5000,
            ..Query::default()
        };
        assert_eq!(field(&query(&q, false)), Some("limit"));
        let q = Query {
            at: Some(chrono::Utc::now()),
            ..Query::default()
        };
        assert_eq!(field(&query(&q, false)), Some("at"));
        assert!(query(&q, true).is_ok());
    }
}
