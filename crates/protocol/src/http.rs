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

use crate::{AppState, Problem, subject::Subject};
use tbd_common::metrics::StreamGuard;

pub fn routes() -> Router<AppState> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/v1/evaluate", post(evaluate))
        .route("/v1/me", get(me))
        .route("/v1/subjects/{subject_id}/events", get(events))
}

/// Liveness: the process is up.
async fn healthz() -> Json<Health> {
    Json(Health { status: "ok" })
}

/// `GET /healthz` body.
#[derive(Debug, Serialize)]
pub struct Health {
    /// Always `ok` when the process answers.
    pub status: &'static str,
}

/// Who the verified caller is; 401 when Envoy forwarded no identity.
async fn me(subject: Subject) -> Json<Me> {
    Json(Me { subject: subject.0 })
}

#[derive(Serialize)]
struct Me {
    subject: String,
}

/// Readiness: every required backend answers `SERVING` to a live health
/// check. The body lists every registered backend either way, so an optional
/// one that is down is visible without failing the probe.
async fn readyz(State(state): State<AppState>) -> impl IntoResponse {
    let readiness = state.readiness().await;
    let status = if readiness.ready {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    (status, Json(readiness))
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
    crate::json::Json(body): crate::json::Json<EvaluateBody>,
) -> Result<Json<Evaluation>, Problem> {
    if body.subject_id.is_empty() {
        return Err(Problem::field("subject_id", "is required"));
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
) -> Result<Sse<impl Stream<Item = Result<SseEvent, Infallible>>>, Problem> {
    let stream = state
        .engine()
        .subscribe(SubscribeRequest { subject_id })
        .await?
        .into_inner();

    let guard = StreamGuard::open("sse");
    let sse = stream.filter_map(move |item| {
        guard.item("out");
        async move {
            let ev = match item {
                Ok(ev) => ev,
                Err(status) => {
                    // The envelope as the event's JSON; the stream stays open,
                    // the client decides.
                    let problem = Problem::from(status);
                    problem.log();
                    let event = SseEvent::default().event("error");
                    let event = event.json_data(problem.wire()).unwrap_or_else(|_| {
                        SseEvent::default()
                            .event("error")
                            .data(r#"{"code":"internal","error":"serialize","details":[]}"#)
                    });
                    return Some(Ok(event));
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
        }
    });

    Ok(Sse::new(sse).keep_alive(KeepAlive::default()))
}
