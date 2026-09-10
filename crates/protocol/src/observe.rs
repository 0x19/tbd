//! Request-level observability for every surface on the port: one span per
//! request parented to the caller's trace, and one metrics sample per request.
//!
//! gRPC shares the port, so both layers classify by content type and record
//! `transport = "grpc"` for it. Streaming bodies are timed to first response,
//! the same as the engine; open streams are tracked by the WebSocket and SSE
//! handlers with [`StreamGuard`](tbd_common::metrics::StreamGuard).

use axum::{
    extract::{MatchedPath, Request},
    middleware::Next,
    response::Response,
};
use tbd_common::{metrics::RequestTimer, telemetry::propagation};

struct Headers<'a>(&'a http::HeaderMap);

impl propagation::Extractor for Headers<'_> {
    fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.to_str().ok())
    }
    fn keys(&self) -> Vec<&str> {
        self.0.keys().map(http::HeaderName::as_str).collect()
    }
}

pub(crate) fn is_grpc(headers: &http::HeaderMap) -> bool {
    headers
        .get(http::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| ct.starts_with("application/grpc"))
}

/// The route label: the matched axum pattern for HTTP, the RPC path for gRPC.
/// An HTTP request that matched nothing is labelled `unmatched`, never its raw
/// path: on a public edge scanners would otherwise mint a label value per probe.
fn route_of(request: &Request) -> String {
    match request.extensions().get::<MatchedPath>() {
        Some(m) => m.as_str().to_owned(),
        None if is_grpc(request.headers()) => {
            request.uri().path().trim_start_matches('/').to_owned()
        }
        None => "unmatched".to_owned(),
    }
}

/// Span factory for `tower_http::trace::TraceLayer`.
pub fn make_span(request: &http::Request<axum::body::Body>) -> tracing::Span {
    let grpc = is_grpc(request.headers());
    let route = request.extensions().get::<MatchedPath>().map_or_else(
        || request.uri().path().to_owned(),
        |m| m.as_str().to_owned(),
    );
    let span = if grpc {
        tracing::info_span!(
            "grpc.request",
            rpc.system = "grpc",
            rpc.method = %route.trim_start_matches('/'),
            trace_id = tracing::field::Empty,
            enduser.id = tracing::field::Empty,
        )
    } else {
        tracing::info_span!(
            "http.request",
            http.request.method = %request.method(),
            http.route = %route,
            trace_id = tracing::field::Empty,
            enduser.id = tracing::field::Empty,
        )
    };
    if let Some(id) = propagation::adopt_parent(&span, &Headers(request.headers())) {
        span.record("trace_id", id);
    }
    span
}

/// Metrics middleware for `axum::middleware::from_fn`.
pub async fn metrics(request: Request, next: Next) -> Response {
    let transport = if is_grpc(request.headers()) {
        "grpc"
    } else {
        "http"
    };
    let mut timer = RequestTimer::start(transport, route_of(&request));
    let response = next.run(request).await;
    let status = match transport {
        "grpc" => response
            .headers()
            .get("grpc-status")
            .and_then(|v| v.to_str().ok())
            .map_or_else(
                || "ok".to_owned(),
                |code| {
                    if code == "0" {
                        "ok".to_owned()
                    } else {
                        format!("grpc-{code}")
                    }
                },
            ),
        _ => response.status().as_u16().to_string(),
    };
    timer.set_status(status);
    response
}
