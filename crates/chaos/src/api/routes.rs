//! HTTP handlers. Paths are relative to `[serve] base_path`.

use std::{convert::Infallible, sync::Arc};

use axum::{
    Json, Router,
    extract::{Path, Query, State},
    http::StatusCode,
    response::{
        IntoResponse,
        sse::{Event, KeepAlive, KeepAliveStream, Sse},
    },
    routing::{get, post},
};
use futures::stream::{self, BoxStream, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::json;
use tbd_common::fault::Behavior;
use tokio_stream::wrappers::BroadcastStream;

use super::{
    error::ApiError,
    runs::{RunFeed, RunKind, RunRecord, RunSummary},
    state::{AppState, LoadRequest, ScenarioEntry, ValidateRequest},
};
use crate::{scenario::ScenarioFile, stack::InstanceInfo};

type Shared = Arc<AppState>;
type Result<T> = std::result::Result<T, ApiError>;

/// Every route.
pub fn router() -> Router<Shared> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/overview", get(overview))
        .route("/events", get(events))
        .route("/stack", get(stack))
        .route("/stack/{name}/start", post(stack_start))
        .route("/stack/{name}/stop", post(stack_stop))
        .route("/stack/{name}/behavior", axum::routing::put(stack_behavior))
        .route("/scenarios", get(scenarios))
        .route("/scenarios/check", post(scenario_check))
        .route(
            "/scenarios/{*id}",
            get(scenario_get).put(scenario_put).delete(scenario_delete),
        )
        .route("/runs", get(runs).post(run_start))
        .route("/runs/{id}", get(run_get).delete(run_delete))
        .route("/runs/{id}/events", get(run_events))
        .route("/runs/{id}/cancel", post(run_cancel))
        .route("/validate", post(validate))
}

async fn healthz() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok", "version": tbd_common::VERSION }))
}

/// `GET /overview`
#[derive(Serialize)]
struct Overview {
    version: &'static str,
    env: String,
    config_files: Vec<String>,
    config: crate::config::ChaosConfig,
    stack: Option<Vec<InstanceInfo>>,
    active_run: Option<RunSummary>,
    recent_runs: Vec<RunSummary>,
    last_validate: Option<RunSummary>,
    scenarios: usize,
    runs: usize,
}

async fn overview(State(state): State<Shared>) -> Json<Overview> {
    let active = match state.runs.current().await {
        Some(a) => state.runs.get(&a.id).await.map(|r| r.summary()),
        None => None,
    };
    Json(Overview {
        version: tbd_common::VERSION,
        env: state.source.env.clone(),
        config_files: state
            .source
            .files
            .iter()
            .map(|p| p.display().to_string())
            .collect(),
        config: state.config.clone(),
        stack: state.stack_info().await.ok(),
        active_run: active,
        recent_runs: state.runs.list(10).await,
        last_validate: state
            .runs
            .last_of(RunKind::Validate)
            .await
            .map(|r| r.summary()),
        scenarios: state.list_scenarios().len(),
        runs: state.runs.count().await,
    })
}

/// `GET /events`: the global feed.
async fn events(State(state): State<Shared>) -> Sse<KeepAliveStream<BoxStream<'static, SseItem>>> {
    let stream = BroadcastStream::new(state.subscribe())
        .filter_map(|item| async move { item.ok() })
        .map(|event| sse_event(&event));
    Sse::new(stream.boxed()).keep_alive(KeepAlive::default())
}

// ---------------------------------------------------------------- stack --

async fn stack(State(state): State<Shared>) -> Result<Json<Vec<InstanceInfo>>> {
    Ok(Json(state.stack_info().await?))
}

async fn stack_start(
    State(state): State<Shared>,
    Path(name): Path<String>,
) -> Result<Json<Vec<InstanceInfo>>> {
    state
        .with_stack(async |s| {
            s.start_instance(&name).await?;
            Ok(s.describe())
        })
        .await
        .map(Json)
}

async fn stack_stop(
    State(state): State<Shared>,
    Path(name): Path<String>,
) -> Result<Json<Vec<InstanceInfo>>> {
    state
        .with_stack(async |s| {
            s.stop_instance(&name).await?;
            Ok(s.describe())
        })
        .await
        .map(Json)
}

async fn stack_behavior(
    State(state): State<Shared>,
    Path(name): Path<String>,
    Json(behavior): Json<Behavior>,
) -> Result<Json<Vec<InstanceInfo>>> {
    state
        .with_stack(async |s| {
            s.set_behavior(&name, behavior)?;
            Ok(s.describe())
        })
        .await
        .map(Json)
}

// ------------------------------------------------------------ scenarios --

async fn scenarios(State(state): State<Shared>) -> Json<Vec<ScenarioEntry>> {
    Json(state.list_scenarios())
}

/// `GET /scenarios/{id}`
#[derive(Serialize)]
struct ScenarioDetail {
    #[serde(flatten)]
    entry: ScenarioEntry,
    text: String,
    parsed: Option<ScenarioFile>,
    last_run: Option<RunSummary>,
}

async fn scenario_get(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<ScenarioDetail>> {
    let (entry, text, parsed) = state.read_scenario(&id).await?;
    let last_run = state.last_run_of(&id).await;
    Ok(Json(ScenarioDetail {
        entry,
        text,
        parsed,
        last_run,
    }))
}

/// Body of `PUT /scenarios/{id}` and `POST /scenarios/check`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ScenarioText {
    text: String,
}

async fn scenario_put(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(body): Json<ScenarioText>,
) -> Result<Json<ScenarioEntry>> {
    Ok(Json(state.write_scenario(&id, &body.text).await?))
}

async fn scenario_delete(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    state.delete_scenario(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /scenarios/check` reply.
#[derive(Serialize)]
struct CheckReply {
    ok: bool,
    name: Option<String>,
    error: Option<String>,
    parsed: Option<ScenarioFile>,
}

async fn scenario_check(Json(body): Json<ScenarioText>) -> Json<CheckReply> {
    Json(match super::state::parse_scenario(&body.text) {
        Ok(file) => CheckReply {
            ok: true,
            name: Some(file.scenario.name.clone()),
            error: None,
            parsed: Some(file),
        },
        Err(error) => CheckReply {
            ok: false,
            name: None,
            error: Some(error),
            parsed: None,
        },
    })
}

// ----------------------------------------------------------------- runs --

#[derive(Deserialize)]
struct RunsQuery {
    #[serde(default = "default_limit")]
    limit: usize,
}

fn default_limit() -> usize {
    50
}

async fn runs(State(state): State<Shared>, Query(q): Query<RunsQuery>) -> Json<Vec<RunSummary>> {
    Json(state.runs.list(q.limit).await)
}

/// Body of `POST /runs`: a scenario by id, or an ad-hoc load run.
#[derive(Deserialize)]
#[serde(untagged)]
enum RunRequest {
    Scenario { scenario: String },
    Load(LoadRequest),
}

async fn run_start(
    State(state): State<Shared>,
    Json(body): Json<RunRequest>,
) -> Result<(StatusCode, Json<RunSummary>)> {
    let summary = match body {
        RunRequest::Scenario { scenario } => state.spawn_scenario(&scenario).await?,
        RunRequest::Load(req) => state.spawn_load(req).await?,
    };
    Ok((StatusCode::ACCEPTED, Json(summary)))
}

async fn run_get(State(state): State<Shared>, Path(id): Path<String>) -> Result<Json<RunRecord>> {
    state
        .runs
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("no run {id}")))
}

async fn run_delete(State(state): State<Shared>, Path(id): Path<String>) -> Result<StatusCode> {
    if state.runs.active(&id).await.is_some() {
        return Err(ApiError::conflict("run is active; cancel it first"));
    }
    if state.runs.delete(&id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found(format!("no run {id}")))
    }
}

async fn run_cancel(State(state): State<Shared>, Path(id): Path<String>) -> Result<StatusCode> {
    let active = state
        .runs
        .active(&id)
        .await
        .ok_or_else(|| ApiError::conflict("run is not active"))?;
    active.cancel.cancel();
    Ok(StatusCode::ACCEPTED)
}

/// `GET /runs/{id}/events`: history so far, then live, ending at `finished`.
async fn run_events(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Sse<KeepAliveStream<BoxStream<'static, SseItem>>>> {
    if let Some(active) = state.runs.active(&id).await {
        let (history, rx) = active.replay().await;
        let done = history
            .iter()
            .any(|item| matches!(item, RunFeed::Finished { .. }));
        let replay = stream::iter(history);
        let live = if done {
            stream::empty().boxed()
        } else {
            until_finished(BroadcastStream::new(rx).filter_map(|item| async move { item.ok() }))
        };
        let all = replay.chain(live).map(|item| sse_event(&item));
        return Ok(Sse::new(all.boxed()).keep_alive(KeepAlive::default()));
    }
    let record = state
        .runs
        .get(&id)
        .await
        .ok_or_else(|| ApiError::not_found(format!("no run {id}")))?;
    let finished = RunFeed::Finished {
        run: Box::new(record),
    };
    let once = stream::once(async move { sse_event(&finished) });
    Ok(Sse::new(once.boxed()).keep_alive(KeepAlive::default()))
}

/// Pass items through until, and including, `finished`.
fn until_finished(
    feed: impl futures::Stream<Item = RunFeed> + Send + 'static,
) -> BoxStream<'static, RunFeed> {
    feed.scan(false, |done, item| {
        if *done {
            return futures::future::ready(None);
        }
        *done = matches!(item, RunFeed::Finished { .. });
        futures::future::ready(Some(item))
    })
    .boxed()
}

// ------------------------------------------------------------- validate --

async fn validate(
    State(state): State<Shared>,
    body: Option<Json<ValidateRequest>>,
) -> Result<Json<RunRecord>> {
    let req = body.map(|Json(b)| b).unwrap_or_default();
    Ok(Json(state.run_validate(req).await?))
}

// ------------------------------------------------------------------ sse --

type SseItem = std::result::Result<Event, Infallible>;

/// One SSE frame: `event:` is the tagged `type`, `data:` the JSON.
#[allow(clippy::unnecessary_wraps)]
fn sse_event<T: Serialize>(item: &T) -> SseItem {
    let value = serde_json::to_value(item).unwrap_or(serde_json::Value::Null);
    let kind = value
        .get("type")
        .and_then(|t| t.as_str())
        .unwrap_or("message")
        .to_owned();
    Ok(Event::default().event(kind).data(value.to_string()))
}

impl IntoResponse for RunSummary {
    fn into_response(self) -> axum::response::Response {
        Json(self).into_response()
    }
}
