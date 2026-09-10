//! Who is calling. Envoy verifies the bearer token and forwards the verified
//! claims as base64url JSON in `x-jwt-payload`; it strips that header from
//! anything a client sends, and services are reachable only through Envoy, so
//! its presence means "verified by Envoy". The protocol reads the subject out of
//! it and does no verification of its own.

use std::future::Future;

use axum::{
    extract::{FromRequestParts, Request},
    http::{HeaderMap, request::Parts},
    middleware::Next,
    response::Response,
};
use base64::Engine as _;

use crate::ApiError;

/// Header Envoy forwards the verified JWT payload in.
pub const PAYLOAD_HEADER: &str = "x-jwt-payload";

/// The authenticated caller: the `sub` claim of the verified token.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Subject(pub String);

/// Reads the verified claims into a request extension and the span.
pub async fn attach(mut request: Request, next: Next) -> Response {
    if let Some(subject) = from_headers(request.headers()) {
        tracing::Span::current().record("enduser.id", subject.0.as_str());
        request.extensions_mut().insert(subject);
    }
    next.run(request).await
}

/// The subject from `x-jwt-payload`, if present and well formed.
#[must_use]
pub fn from_headers(headers: &HeaderMap) -> Option<Subject> {
    let raw = headers.get(PAYLOAD_HEADER)?.to_str().ok()?;
    let bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(raw)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(raw))
        .ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&bytes).ok()?;
    let sub = claims.get("sub")?.as_str()?;
    (!sub.is_empty()).then(|| Subject(sub.to_owned()))
}

impl<S: Send + Sync> FromRequestParts<S> for Subject {
    type Rejection = ApiError;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        std::future::ready(
            parts
                .extensions
                .get::<Self>()
                .cloned()
                .ok_or(ApiError::Unauthenticated),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn subject_comes_from_the_verified_payload_only() {
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .encode(br#"{"sub":"person-1","aud":["tbd-api"]}"#);
        let mut headers = HeaderMap::new();
        assert_eq!(from_headers(&headers), None);
        headers.insert(PAYLOAD_HEADER, payload.parse().unwrap());
        assert_eq!(from_headers(&headers), Some(Subject("person-1".into())));
        headers.insert(PAYLOAD_HEADER, "not base64!".parse().unwrap());
        assert_eq!(from_headers(&headers), None);
        headers.insert(
            PAYLOAD_HEADER,
            base64::engine::general_purpose::URL_SAFE_NO_PAD
                .encode(br#"{"aud":"x"}"#)
                .parse()
                .unwrap(),
        );
        assert_eq!(from_headers(&headers), None);
    }
}
