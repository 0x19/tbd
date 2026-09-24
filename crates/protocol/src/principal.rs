//! The axum side of identity.
//!
//! [`Principal`] itself lives in `tbd-common`: every service behind Envoy needs
//! the *same* identity the gateway saw, and re-parsing `x-jwt-payload` in each
//! one would be two implementations that can disagree. What stays here is the
//! part that is axum's -- the middleware that puts the principal in the request
//! extensions and records it on the span, and the extractors handlers take.

use std::future::Future;

use axum::{
    extract::{FromRequestParts, OptionalFromRequestParts, Request, State},
    http::request::Parts,
    middleware::Next,
    response::Response,
};
pub use tbd_common::principal::{CallerKind, PAYLOAD_HEADER, Principal};

/// The key claims, as `OpenAPI` sees them.
///
/// A mirror of `tbd_common::principal::Key`. The type itself lives in
/// `tbd-common`, which cannot derive `utoipa::ToSchema` without taking a
/// dependency on a documentation crate it has no other use for. Mirroring it
/// here keeps `docs/protocol/openapi.json` byte-identical across this
/// refactor -- a published contract must not shift because of an internal
/// move.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, utoipa::ToSchema)]
pub struct Key {
    /// Key id.
    pub id: String,
    /// The parent key, for a sub-key.
    pub parent: Option<String>,
}

impl From<tbd_common::principal::Key> for Key {
    fn from(k: tbd_common::principal::Key) -> Self {
        Self {
            id: k.id,
            parent: k.parent,
        }
    }
}

use crate::{AppState, Problem};

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

/// The [`Principal`] as an axum extractor.
///
/// A newtype because `Principal` now lives in `tbd-common`, and the orphan rule
/// forbids implementing axum's `FromRequestParts` for a foreign type here.
/// Handlers destructure it: `Caller(principal): Caller`, or
/// `caller: Option<Caller>` where the route tolerates an anonymous request.
#[derive(Debug, Clone)]
pub struct Caller(pub Principal);

impl std::ops::Deref for Caller {
    type Target = Principal;

    fn deref(&self) -> &Principal {
        &self.0
    }
}

impl<S: Send + Sync> FromRequestParts<S> for Caller {
    type Rejection = Problem;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Self, Self::Rejection>> + Send {
        std::future::ready(
            parts
                .extensions
                .get::<Principal>()
                .cloned()
                .map(Caller)
                .ok_or_else(Problem::unauthenticated),
        )
    }
}

impl<S: Send + Sync> OptionalFromRequestParts<S> for Caller {
    type Rejection = Problem;

    fn from_request_parts(
        parts: &mut Parts,
        _state: &S,
    ) -> impl Future<Output = Result<Option<Self>, Self::Rejection>> + Send {
        std::future::ready(Ok(parts.extensions.get::<Principal>().cloned().map(Caller)))
    }
}
