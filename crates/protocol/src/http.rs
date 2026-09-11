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
};
use futures::{Stream, StreamExt};
use serde::{Deserialize, Serialize};
use tbd_proto::engine::v1::{EvaluateRequest, SubscribeRequest, subscribe_response};
use utoipa::ToSchema;
use utoipa_axum::{router::OpenApiRouter, routes};

use crate::{AppState, Principal, Problem, error::ErrorBody, principal::Key, state::Readiness};
use tbd_common::metrics::StreamGuard;

/// The REST routes with their `OpenAPI` paths: one source for both, so the
/// document cannot drift from the router.
pub fn openapi_router() -> OpenApiRouter<AppState> {
    OpenApiRouter::new()
        .routes(routes!(healthz))
        .routes(routes!(readyz))
        .routes(routes!(evaluate))
        .routes(routes!(me))
        .routes(routes!(events))
}

pub fn routes() -> Router<AppState> {
    openapi_router().split_for_parts().0
}

/// Liveness: the process is up.
#[utoipa::path(get, path = "/healthz", tag = "health",
    responses((status = 200, description = "The process answers", body = Health)))]
async fn healthz() -> Json<Health> {
    Json(Health { status: "ok" })
}

/// `GET /healthz` body.
#[derive(Debug, Serialize, ToSchema)]
pub struct Health {
    /// Always `ok` when the process answers.
    pub status: &'static str,
}

/// Who the verified caller is; 401 when Envoy forwarded no identity.
#[utoipa::path(get, path = "/v1/me", tag = "identity",
    responses(
        (status = 200, description = "The principal Envoy verified", body = Me),
        (status = 401, description = "Envoy forwarded no identity", body = ErrorBody)))]
async fn me(principal: Principal) -> Json<Me> {
    Json(Me {
        client_id: principal.client_id().map(str::to_owned),
        kind: principal.kind_slug(),
        subject: principal.sub,
        org: principal.org,
        key: principal.key,
        scopes: principal.scopes,
        role: principal.role,
    })
}

/// `GET /v1/me` body.
#[derive(Debug, Serialize, ToSchema)]
pub struct Me {
    /// The `sub` claim.
    pub subject: String,
    /// `person`, `client` or `service`.
    pub kind: &'static str,
    /// The OAuth client the call came through, if any.
    pub client_id: Option<String>,
    /// Organisation, when minted.
    pub org: Option<String>,
    /// Key and parent key, when minted.
    pub key: Option<Key>,
    /// Granted scopes.
    pub scopes: Vec<String>,
    /// Role, when the consent step stamped one.
    pub role: Option<String>,
}

/// Readiness: every required backend answers `SERVING` to a live health
/// check. The body lists every registered backend either way, so an optional
/// one that is down is visible without failing the probe.
#[utoipa::path(get, path = "/readyz", tag = "health",
    responses(
        (status = 200, description = "Every required backend is serving", body = Readiness),
        (status = 503, description = "A required backend is not serving", body = Readiness)))]
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
#[derive(Debug, Deserialize, ToSchema)]
pub struct EvaluateBody {
    /// Subject to score.
    pub subject_id: String,
    /// Opaque payload, passed through to the engine.
    #[serde(default)]
    pub payload: String,
}

/// `POST /v1/evaluate` response. `stub` is forwarded from the engine untouched.
#[derive(Debug, Serialize, ToSchema)]
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

/// Score a subject once; the engine's `stub` flag is forwarded untouched.
#[utoipa::path(post, path = "/v1/evaluate", tag = "engine",
    request_body = EvaluateBody,
    responses(
        (status = 200, description = "The score", body = Evaluation),
        (status = 400, description = "A malformed body or an empty subject_id", body = ErrorBody),
        (status = 415, description = "The body is not JSON", body = ErrorBody),
        (status = 503, description = "The engine is unavailable", body = ErrorBody)))]
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
#[derive(Debug, Serialize, ToSchema)]
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
#[utoipa::path(get, path = "/v1/subjects/{subject_id}/events", tag = "engine",
    params(("subject_id" = String, Path, description = "The subject to follow")),
    responses(
        (status = 200, description = "A stream of `EventBody` events; a failure is an `error` event carrying `Problem`",
            content_type = "text/event-stream", body = EventBody),
        (status = 503, description = "The engine is unavailable", body = ErrorBody)))]
async fn events(
    State(state): State<AppState>,
    principal: Option<Principal>,
    Path(subject_id): Path<String>,
) -> Result<Sse<impl Stream<Item = Result<SseEvent, Infallible>>>, Problem> {
    tracing::debug!(
        caller = principal.as_ref().map(Principal::kind_slug),
        "events stream requested"
    );
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
