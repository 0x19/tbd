//! REST and server-sent events.

use std::convert::Infallible;

use axum::{
    Json, Router,
    extract::{Path, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
    routing::{get, post},
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tbd_proto::engine::v1::{EvaluateRequest, SubscribeRequest, subscribe_response};

use crate::{ApiError, AppState};

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/evaluate", post(evaluate))
        .route("/v1/subjects/{subject_id}/events", get(events))
}

/// Liveness: the process is up.
async fn healthz() -> &'static str {
    "ok"
}

/// Readiness: the engine answers its health check.
async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    if state.engine_ready().await {
        (StatusCode::OK, "ready")
    } else {
        (StatusCode::SERVICE_UNAVAILABLE, "engine unavailable")
    }
}

/// `POST /v1/evaluate` body.
#[derive(Debug, Deserialize)]
pub struct EvaluateBody {
    /// Subject to score.
    pub subject_id: String,
    /// Opaque payload, passed through to the engine.
    #[serde(default)]
    pub payload: String,
}

/// `POST /v1/evaluate` response. `stub` is forwarded from the engine untouched.
#[derive(Debug, Serialize)]
pub struct Evaluation {
    /// Subject that was scored.
    pub subject_id: String,
    /// The score.
    pub score: f64,
    /// True while the score is a placeholder, not a model output.
    pub stub: bool,
    /// Model version that produced the score.
    pub model_version: String,
}

async fn evaluate(
    State(state): State<AppState>,
    Json(body): Json<EvaluateBody>,
) -> Result<Json<Evaluation>, ApiError> {
    if body.subject_id.is_empty() {
        return Err(ApiError::BadRequest("subject_id is required".into()));
    }
    let resp = state
        .engine()
        .evaluate(EvaluateRequest {
            subject_id: body.subject_id,
            payload: body.payload.into_bytes(),
        })
        .await?
        .into_inner();
    Ok(Json(Evaluation {
        subject_id: resp.subject_id,
        score: resp.score,
        stub: resp.stub,
        model_version: resp.model_version,
    }))
}

/// One SSE event body.
#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EventBody {
    /// Periodic liveness tick from the engine.
    Heartbeat {
        /// Monotonic sequence number.
        seq: u64,
    },
    /// The subject's score changed.
    ScoreUpdated {
        /// New score.
        score: f64,
        /// True while the score is a placeholder.
        stub: bool,
    },
}

/// `GET /v1/subjects/{subject_id}/events`: engine `Subscribe` as SSE.
async fn events(
    State(state): State<AppState>,
    Path(subject_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<SseEvent, Infallible>>>, ApiError> {
    let stream = state
        .engine()
        .subscribe(SubscribeRequest { subject_id })
        .await?
        .into_inner();

    let sse = stream.filter_map(|item| async move {
        let ev = match item {
            Ok(ev) => ev,
            Err(status) => {
                tracing::warn!(%status, "engine event stream error");
                return Some(Ok(SseEvent::default()
                    .event("error")
                    .data(status.message())));
            }
        };
        let body = match ev.kind? {
            subscribe_response::Kind::Heartbeat(hb) => EventBody::Heartbeat { seq: hb.seq },
            subscribe_response::Kind::ScoreUpdated(s) => EventBody::ScoreUpdated {
                score: s.score,
                stub: s.stub,
            },
        };
        let sse = SseEvent::default().id(ev.id).json_data(body).ok()?;
        Some(Ok(sse))
    });

    Ok(Sse::new(sse).keep_alive(KeepAlive::default()))
}
