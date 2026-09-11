//! Who is calling. Envoy verifies the bearer token and forwards the verified
//! claims as base64url JSON in `x-jwt-payload`; it strips that header from
//! anything a client sends, and services are reachable only through Envoy, so
//! its presence means "verified by Envoy". The protocol reads the principal out
//! of it and does no verification of its own.
//!
//! A [`Principal`] is the subject plus its kind: a person (through a client or
//! not), a client (an organisation's machine key: `client_id` equals `sub`),
//! or one of our own services (`sub` listed in `[principals] services`). The
//! organisation and key claims (`org`, `key`, `parent`) are read here and
//! minted by the id plane later; until then they are `None`.

use std::future::Future;

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request, State},
    http::{HeaderMap, request::Parts},
    middleware::Next,
    response::Response,
};
use base64::Engine as _;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{AppState, Problem};

/// Header Envoy forwards the verified JWT payload in.
pub const PAYLOAD_HEADER: &str = "x-jwt-payload";

/// The authenticated caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Principal {
    /// The `sub` claim: a person's pairwise id, a client id, or a service id.
    pub sub: String,
    /// What kind of caller this is.
    pub kind: CallerKind,
    /// Granted scopes (`scp`, or a space-separated `scope`).
    pub scopes: Vec<String>,
    /// The person's role, when the consent step stamped one.
    pub role: Option<String>,
    /// The organisation the caller belongs to, when the id plane mints it.
    pub org: Option<String>,
    /// The key (a client or a sub-key of one) the call was made with.
    pub key: Option<Key>,
}

/// The three kinds of caller.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CallerKind {
    /// A person, through the client named in `client_id`/`azp` when there is one.
    Person {
        /// The OAuth client the person came through.
        client_id: Option<String>,
    },
    /// A machine caller: a client-credentials token whose `client_id` is the subject.
    Client {
        /// The client id, which is also `sub`.
        client_id: String,
    },
    /// One of our own services (`sub` in `[principals] services`).
    Service,
}

/// A key the id plane issued: a client, or a sub-key under a parent client.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct Key {
    /// Key id.
    pub id: String,
    /// The parent key, for a sub-key.
    pub parent: Option<String>,
}

impl Principal {
    /// The principal in a verified claim set. `services` lists the subjects
    /// that are our own services. `None` without a non-empty `sub`.
    #[must_use]
    pub fn from_claims(claims: &serde_json::Value, services: &[String]) -> Option<Self> {
        let sub = claim_str(claims, "sub")
            .filter(|s| !s.is_empty())?
            .to_owned();
        let client_id = claim_str(claims, "client_id")
            .or_else(|| claim_str(claims, "azp"))
            .filter(|s| !s.is_empty())
            .map(str::to_owned);
        let kind = if services.contains(&sub) {
            CallerKind::Service
        } else if client_id.as_deref() == Some(sub.as_str()) {
            CallerKind::Client {
                client_id: sub.clone(),
            }
        } else {
            CallerKind::Person { client_id }
        };
        Some(Self {
            sub,
            kind,
            scopes: scopes(claims),
            role: claim_str(claims, "role").map(str::to_owned),
            org: claim_str(claims, "org").map(str::to_owned),
            key: claim_str(claims, "key").map(|id| Key {
                id: id.to_owned(),
                parent: claim_str(claims, "parent").map(str::to_owned),
            }),
        })
    }

    /// The principal from `x-jwt-payload`, if present and well formed.
    #[must_use]
    pub fn from_headers(headers: &HeaderMap, services: &[String]) -> Option<Self> {
        let raw = headers.get(PAYLOAD_HEADER)?.to_str().ok()?;
        let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(raw)
            .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(raw))
            .ok()?;
        let claims: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
        Self::from_claims(&claims, services)
    }

    /// `person`, `client` or `service`.
    #[must_use]
    pub fn kind_slug(&self) -> &'static str {
        match self.kind {
            CallerKind::Person { .. } => "person",
            CallerKind::Client { .. } => "client",
            CallerKind::Service => "service",
        }
    }

    /// The OAuth client the call came through, if any.
    #[must_use]
    pub fn client_id(&self) -> Option<&str> {
        match &self.kind {
            CallerKind::Person { client_id } => client_id.as_deref(),
            CallerKind::Client { client_id } => Some(client_id),
            CallerKind::Service => None,
        }
    }
}

/// A string claim at the top level or under `ext`, where Hydra puts the
/// claims the consent step adds to an access token.
fn claim_str<'a>(claims: &'a serde_json::Value, name: &str) -> Option<&'a str> {
    claims
        .get(name)
        .or_else(|| claims.get("ext")?.get(name))?
        .as_str()
}

fn scopes(claims: &serde_json::Value) -> Vec<String> {
    let value = claims
        .get("scp")
        .or_else(|| claims.get("ext")?.get("scp"))
        .or_else(|| claims.get("scope"));
    match value {
        Some(serde_json::Value::Array(items)) => items
            .iter()
            .filter_map(|v| v.as_str())
            .map(str::to_owned)
            .collect(),
        Some(serde_json::Value::String(s)) => s.split_whitespace().map(str::to_owned).collect(),
        _ => Vec::new(),
    }
}

/// Reads the verified claims into a request extension and the span.
pub async fn attach(State(state): State<AppState>, mut request: Request, next: Next) -> Response {
    if let Some(principal) = Principal::from_headers(request.headers(), state.service_subs()) {
        let span = tracing::Span::current();
        span.record("enduser.id", principal.sub.as_str());
        span.record("enduser.kind", principal.kind_slug());
        if let Some(org) = &principal.org {
            span.record("enduser.org", org.as_str());
        }
        if let Some(key) = &principal.key {
            span.record("enduser.key", key.id.as_str());
        }
        request.extensions_mut().insert(principal);
    }
    next.run(request).await
}

impl<S: Send + Sync> FromRequestParts<S> for Principal {
    type Rejection = Problem;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        std::future::ready(
            parts
                .extensions
                .get::<Self>()
                .cloned()
                .ok_or_else(Problem::unauthenticated),
        )
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for Principal {
    type Rejection = Problem;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Option<Self>, Self::Rejection>> + Send {
        std::future::ready(Ok(parts.extensions.get::<Self>().cloned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn payload(v: &serde_json::Value) -> HeaderMap {
        let raw = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(v.to_string());
        let mut headers = HeaderMap::new();
        headers.insert(
            PAYLOAD_HEADER,
            raw.parse().unwrap_or_else(|_| unreachable!()),
        );
        headers
    }

    #[test]
    fn a_person_with_and_without_a_client() {
        let p = Principal::from_claims(&json!({"sub": "person-1", "aud": ["tbd-api"]}), &[])
            .unwrap_or_else(|| unreachable!());
        assert_eq!(p.kind, CallerKind::Person { client_id: None });
        assert_eq!(p.kind_slug(), "person");
        assert!(p.scopes.is_empty());

        // Through a client, with the consent-set claims under `ext` and a
        // space-separated scope string.
        let p = Principal::from_claims(
            &json!({"sub": "person-2", "azp": "tbd-app", "scope": "openid tbd.api",
                    "ext": {"role": "editor", "org": "acme", "key": "k-7", "parent": "k-1"}}),
            &[],
        )
        .unwrap_or_else(|| unreachable!());
        assert_eq!(p.client_id(), Some("tbd-app"));
        assert_eq!(p.kind_slug(), "person");
        assert_eq!(p.scopes, ["openid", "tbd.api"]);
        assert_eq!(p.role.as_deref(), Some("editor"));
        assert_eq!(p.org.as_deref(), Some("acme"));
        assert_eq!(
            p.key,
            Some(Key {
                id: "k-7".into(),
                parent: Some("k-1".into())
            })
        );
    }

    #[test]
    fn a_client_is_its_own_subject_and_a_service_is_listed() {
        let p = Principal::from_claims(
            &json!({"sub": "tbd-chaos", "client_id": "tbd-chaos", "scp": ["tbd.api"]}),
            &[],
        )
        .unwrap_or_else(|| unreachable!());
        assert_eq!(
            p.kind,
            CallerKind::Client {
                client_id: "tbd-chaos".into()
            }
        );
        assert_eq!(p.scopes, ["tbd.api"]);

        let services = vec!["svc-ledger".to_owned()];
        let p = Principal::from_claims(
            &json!({"sub": "svc-ledger", "client_id": "svc-ledger"}),
            &services,
        )
        .unwrap_or_else(|| unreachable!());
        assert_eq!(p.kind, CallerKind::Service);
        assert_eq!(p.client_id(), None);
    }

    #[test]
    fn principal_comes_from_the_verified_payload_only() {
        let mut headers = HeaderMap::new();
        assert_eq!(Principal::from_headers(&headers, &[]), None);
        headers = payload(&json!({"sub": "person-1", "aud": ["tbd-api"]}));
        assert_eq!(
            Principal::from_headers(&headers, &[]).map(|p| p.sub),
            Some("person-1".to_owned())
        );
        headers.insert(
            PAYLOAD_HEADER,
            "not base64!".parse().unwrap_or_else(|_| unreachable!()),
        );
        assert_eq!(Principal::from_headers(&headers, &[]), None);
        assert_eq!(
            Principal::from_headers(&payload(&json!({"aud": "x"})), &[]),
            None
        );
        assert_eq!(
            Principal::from_headers(&payload(&json!({"sub": ""})), &[]),
            None
        );
    }
}
