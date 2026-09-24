//! The API: `POST /run` (the token required), `GET /languages`, `GET /healthz`.
//! Busy is answered at once: queueing is the runner's job, one layer up.

use std::sync::Arc;

use axum::{
    Json, Router,
    extract::{DefaultBodyLimit, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse, Response},
    routing::{get, post},
};
use serde_json::json;
use tbd_common::metrics::names;
use tokio::sync::Semaphore;

use crate::{
    config::Config,
    recipe::Language,
    run::{self, RunError, RunRequest},
};

/// What every handler shares.
#[derive(Clone)]
pub struct AppState {
    config: Arc<Config>,
    token: Arc<Vec<u8>>,
    slots: Arc<Semaphore>,
}

impl AppState {
    /// A state over a configuration and the token callers must present.
    #[must_use]
    pub fn new(config: Config, token: Vec<u8>) -> Self {
        let slots = Arc::new(Semaphore::new(config.limits.max_concurrent));
        Self {
            config: Arc::new(config),
            token: Arc::new(token),
            slots,
        }
    }
}

/// Equal without telling, through timing, how much of it matched.
fn same(a: &[u8], b: &[u8]) -> bool {
    let len = a.len() == b.len();
    let mut diff = 0u8;
    for (i, x) in a.iter().enumerate() {
        diff |= x ^ b.get(i).copied().unwrap_or(0);
    }
    len && diff == 0 && !b.is_empty()
}

fn problem(status: StatusCode, code: &str, error: impl Into<String>) -> Response {
    (status, Json(json!({ "code": code, "error": error.into() }))).into_response()
}

/// The router.
pub fn router(state: AppState) -> Router {
    let l = &state.config.limits;
    // A JSON-escaped source and input can be up to six times their size.
    let body = (l.max_source_bytes + l.max_stdin_bytes) * 6 + 4096;
    Router::new()
        .route("/run", post(run_handler))
        .route("/languages", get(languages))
        .route(
            "/healthz",
            get(|| async { Json(json!({ "status": "ok" })) }),
        )
        .layer(DefaultBodyLimit::max(body))
        .with_state(state)
}

async fn languages() -> Json<serde_json::Value> {
    Json(json!({ "languages": Language::ALL.iter().map(|l| l.as_str()).collect::<Vec<_>>() }))
}

async fn run_handler(
    State(state): State<AppState>,
    headers: HeaderMap,
    body: axum::body::Bytes,
) -> Response {
    let presented = headers
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(|v| v.strip_prefix("Bearer "))
        .unwrap_or_default();
    if !same(presented.as_bytes(), &state.token) {
        return problem(
            StatusCode::UNAUTHORIZED,
            "unauthenticated",
            "a valid token is required",
        );
    }
    let req: RunRequest = match serde_json::from_slice(&body) {
        Ok(r) => r,
        Err(e) => return problem(StatusCode::BAD_REQUEST, "bad_request", format!("body: {e}")),
    };
    let Ok(_slot) = state.slots.clone().try_acquire_owned() else {
        return problem(
            StatusCode::TOO_MANY_REQUESTS,
            "rate_limited",
            format!("busy: {} runs at once", state.config.limits.max_concurrent),
        );
    };
    let in_flight = state.config.limits.max_concurrent - state.slots.available_permits();
    #[allow(clippy::cast_precision_loss)] // runs in the tens
    metrics::gauge!(names::SANDBOX_IN_FLIGHT).set(in_flight as f64);
    let language = req.language;
    let result = run::run(&state.config, req).await;
    #[allow(clippy::cast_precision_loss)]
    metrics::gauge!(names::SANDBOX_IN_FLIGHT)
        .set((state.config.limits.max_concurrent - state.slots.available_permits() - 1) as f64);
    match result {
        Ok(resp) => {
            tracing::info!(
                target: "audit",
                id = %resp.id,
                language = language.as_str(),
                outcome = resp.outcome.as_str(),
                total_ms = resp.total_ms,
                "sandbox run"
            );
            Json(resp).into_response()
        }
        Err(RunError::Invalid(m)) => problem(StatusCode::BAD_REQUEST, "bad_request", m),
        Err(RunError::Unavailable(m)) => {
            tracing::error!(error = %m, "sandbox unavailable");
            problem(
                StatusCode::SERVICE_UNAVAILABLE,
                "unavailable",
                "the sandbox could not start",
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::same;

    #[test]
    fn tokens_compare_whole_and_never_empty() {
        assert!(same(b"abc", b"abc"));
        assert!(!same(b"abx", b"abc"));
        assert!(!same(b"ab", b"abc"));
        assert!(!same(b"abcd", b"abc"));
        assert!(!same(b"", b""), "an empty configured token lets nobody in");
    }
}
