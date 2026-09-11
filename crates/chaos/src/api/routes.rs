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
    added::AddedSpec,
    error::ApiError,
    jobs::{Job, QueuedRun, Schedule, ScheduleSpec},
    runs::{RunFeed, RunKind, RunRecord, RunSummary},
    state::{AppState, LoadRequest, ScenarioEntry, ValidateRequest},
};
use super::{
    findings::{FindingFilter, FindingGroup},
    state::{CampaignEntry, ReplayRequest, StressRequest},
};
use crate::{kinds, scenario::ScenarioFile, stack::InstanceInfo, validate};

type Shared = Arc<AppState>;
type Result<T> = std::result::Result<T, ApiError>;

/// Every route.
pub fn router() -> Router<Shared> {
    Router::new()
        .route("/healthz", get(healthz))
        .route("/me", get(me))
        .route("/notify/test", post(notify_test))
        .route("/overview", get(overview))
        .route("/events", get(events))
        .route("/stack", get(stack).post(stack_add))
        .route("/stack/{name}", axum::routing::delete(stack_remove))
        .route("/stack/{name}/clone", post(stack_clone))
        .route("/stack/{name}/start", post(stack_start))
        .route("/stack/{name}/stop", post(stack_stop))
        .route("/stack/{name}/behavior", axum::routing::put(stack_behavior))
        .route(
            "/stack/{name}/store_behavior",
            axum::routing::put(stack_store_behavior),
        )
        .route("/scenarios", get(scenarios))
        .route("/scenarios/check", post(scenario_check))
        .route(
            "/scenarios/{*id}",
            get(scenario_get).put(scenario_put).delete(scenario_delete),
        )
        .route("/stress", get(campaigns))
        .route("/stress/check", post(campaign_check))
        .route(
            "/stress/{*id}",
            get(campaign_get).put(campaign_put).delete(campaign_delete),
        )
        .route("/findings", get(findings))
        .route("/findings/{id}", get(finding_get).delete(finding_delete))
        .route("/findings/{id}/replay", post(finding_replay))
        .route("/runs", get(runs).post(run_start))
        .route("/runs/{id}", get(run_get).delete(run_delete))
        .route("/runs/{id}/events", get(run_events))
        .route("/runs/{id}/cancel", post(run_cancel))
        .route("/validate", post(validate))
        .route("/queue", get(queue).post(queue_push).delete(queue_clear))
        .route("/queue/{id}", axum::routing::delete(queue_remove))
        .route("/schedules", get(schedules).post(schedule_create))
        .route(
            "/schedules/{id}",
            get(schedule_get)
                .put(schedule_update)
                .delete(schedule_delete),
        )
        .route("/schedules/{id}/run", post(schedule_run))
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
    kinds: Vec<kinds::KindInfo>,
    validate: ValidateInfo,
    active_run: Option<RunSummary>,
    queue: Vec<QueuedRun>,
    recent_runs: Vec<RunSummary>,
    last_validate: Option<RunSummary>,
    scenarios: usize,
    campaigns: usize,
    findings: usize,
    finding_signatures: usize,
    runs: usize,
    schedules: usize,
    schedules_enabled: usize,
    next_schedule: Option<Schedule>,
    notify: serde_json::Value,
}

/// What `validate` runs, for the UI's copy.
#[derive(Serialize)]
struct ValidateInfo {
    checks: Vec<validate::CheckInfo>,
}

async fn overview(State(state): State<Shared>) -> Json<Overview> {
    let schedules = state.schedules.list().await;
    let finding_counts = state.findings.counts().await;
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
        kinds: kinds::describe(),
        validate: ValidateInfo {
            checks: validate::catalogue(),
        },
        active_run: active,
        queue: state.queue.list().await,
        recent_runs: state.runs.list(10).await,
        last_validate: state
            .runs
            .last_of(RunKind::Validate)
            .await
            .map(|r| r.summary()),
        scenarios: state.list_scenarios().len(),
        campaigns: state.list_campaigns().len(),
        findings: finding_counts.0,
        finding_signatures: finding_counts.1,
        runs: state.runs.count().await,
        schedules: schedules.len(),
        schedules_enabled: schedules.iter().filter(|s| s.enabled).count(),
        next_schedule: schedules
            .iter()
            .filter(|s| s.next_at.is_some())
            .min_by(|a, b| a.next_at.cmp(&b.next_at))
            .cloned(),
        notify: state.notifier.describe(),
    })
}

/// `GET /me`: who Envoy says is calling, from the identity headers it sets
/// after verifying the ID-token cookie (docs/auth/README.md). `user` is
/// `null` on the open local host, where there is no login.
#[derive(Serialize)]
struct Me {
    user: Option<User>,
    /// This host's sign-out path, handled by Envoy's `OAuth2` filter.
    signout: &'static str,
    /// Global sign-out on the auth host, when a domain is configured.
    signout_all: String,
}

#[derive(Serialize)]
struct User {
    sub: String,
    email: String,
    name: String,
    role: String,
}

async fn me(State(state): State<Shared>, headers: axum::http::HeaderMap) -> Json<Me> {
    let h = |name: &str| {
        headers
            .get(name)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_owned()
    };
    let sub = h("x-user-sub");
    let email = h("x-user-email");
    let user = if sub.is_empty() && email.is_empty() {
        None
    } else {
        Some(User {
            sub,
            email,
            name: h("x-user-name"),
            role: h("x-user-role"),
        })
    };
    let auth = state.config.links.resolved().auth;
    Json(Me {
        user,
        signout: "/oauth2/signout",
        signout_all: if auth.is_empty() {
            String::new()
        } else {
            format!("{auth}/logout")
        },
    })
}

/// `POST /notify/test`: post a hello to the configured Slack webhook.
async fn notify_test(State(state): State<Shared>) -> Result<StatusCode> {
    state.notifier.test().await.map_err(ApiError::invalid)?;
    Ok(StatusCode::NO_CONTENT)
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

/// Body of `POST /stack`: a new instance, `kind` and `name` plus the same
/// keys as the topology's `[stack.<plural>.<name>]` table for that kind
/// (`GET /overview` lists them under `kinds[].fields`).
#[derive(Deserialize)]
struct AddInstance {
    /// A registered kind that is addable.
    kind: String,
    /// Omit for the next free `<kind>-<n>`.
    #[serde(default)]
    name: Option<String>,
    /// The kind's own keys.
    #[serde(flatten)]
    spec: toml::Table,
}

async fn stack_add(
    State(state): State<Shared>,
    Json(body): Json<AddInstance>,
) -> Result<(StatusCode, Json<Vec<InstanceInfo>>)> {
    let name = body.name.filter(|n| !n.trim().is_empty());
    if let Some(n) = &name
        && !n
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
    {
        return Err(ApiError::invalid(format!(
            "name {n:?}: use letters, digits, `_`, `-` and `.`"
        )));
    }
    let kind = kinds::by_name(&body.kind).ok_or_else(|| {
        ApiError::invalid(format!("kind {:?}: one of {}", body.kind, kinds::names()))
    })?;
    if !kind.addable {
        return Err(ApiError::invalid(format!(
            "kind {:?} cannot be added at runtime",
            kind.name
        )));
    }
    // Empty strings from a form mean "not given".
    let spec: toml::Table = body
        .spec
        .into_iter()
        .filter(|(_, v)| v.as_str().is_none_or(|s| !s.trim().is_empty()))
        .collect();
    (kind.parse)(spec.clone()).map_err(|e| ApiError::invalid(format!("{}: {e}", kind.name)))?;
    let info = state
        .add_instance(
            AddedSpec {
                kind: kind.name.to_owned(),
                spec,
            },
            name,
            None,
        )
        .await?;
    Ok((StatusCode::CREATED, Json(info)))
}

/// Body of `POST /stack/{name}/clone`.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields, default)]
struct CloneRequest {
    /// How many replicas to add. Default 1, at most 16 at a time.
    count: usize,
}

async fn stack_clone(
    State(state): State<Shared>,
    Path(name): Path<String>,
    body: Option<Json<CloneRequest>>,
) -> Result<(StatusCode, Json<Vec<InstanceInfo>>)> {
    let count = body.map_or(1, |Json(b)| b.count);
    let info = state.clone_instance(&name, count).await?;
    Ok((StatusCode::CREATED, Json(info)))
}

async fn stack_remove(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<Vec<InstanceInfo>>> {
    Ok(Json(state.remove_instance(&id).await?))
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

async fn stack_store_behavior(
    State(state): State<Shared>,
    Path(name): Path<String>,
    Json(behavior): Json<Behavior>,
) -> Result<Json<Vec<InstanceInfo>>> {
    state
        .with_stack(async |s| {
            s.set_store_behavior(&name, behavior)?;
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
    /// Only runs that exercised this service kind.
    #[serde(default)]
    service: Option<String>,
    /// Only runs of this kind.
    #[serde(default)]
    kind: Option<RunKind>,
}

fn default_limit() -> usize {
    50
}

async fn runs(State(state): State<Shared>, Query(q): Query<RunsQuery>) -> Json<Vec<RunSummary>> {
    let runs = state.runs.list(q.limit).await;
    Json(
        runs.into_iter()
            .filter(|r| q.service.as_ref().is_none_or(|s| r.services.contains(s)))
            .filter(|r| q.kind.is_none_or(|k| r.kind == k))
            .collect(),
    )
}

/// Body of `POST /runs`: a scenario by id, a stress campaign, a finding's
/// replay, or an ad-hoc load run.
#[derive(Deserialize)]
#[serde(untagged)]
enum RunRequest {
    Scenario { scenario: String },
    Stress { stress: StressRequest },
    Replay { replay: ReplayRequest },
    Load(LoadRequest),
}

async fn run_start(
    State(state): State<Shared>,
    Json(body): Json<RunRequest>,
) -> Result<(StatusCode, Json<RunSummary>)> {
    let summary = match body {
        RunRequest::Scenario { scenario } => state.spawn_scenario(&scenario, None).await?,
        RunRequest::Stress { stress } => state.spawn_stress(stress, None).await?,
        RunRequest::Replay { replay } => state.spawn_replay(replay, None).await?,
        RunRequest::Load(req) => state.spawn_load(req, None).await?,
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

// ---------------------------------------------------------- campaigns --

async fn campaigns(State(state): State<Shared>) -> Json<Vec<CampaignEntry>> {
    Json(state.list_campaigns())
}

/// `GET /stress/{id}`
#[derive(Serialize)]
struct CampaignDetail {
    #[serde(flatten)]
    entry: CampaignEntry,
    text: String,
    parsed: Option<tbd_stress::Campaign>,
    last_run: Option<RunSummary>,
}

async fn campaign_get(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<CampaignDetail>> {
    let (entry, text, parsed) = state.read_campaign(&id).await?;
    let last_run = state.last_run_of_campaign(&id).await;
    Ok(Json(CampaignDetail {
        entry,
        text,
        parsed,
        last_run,
    }))
}

async fn campaign_put(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(body): Json<ScenarioText>,
) -> Result<Json<CampaignEntry>> {
    Ok(Json(state.write_campaign(&id, &body.text).await?))
}

async fn campaign_delete(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    state.delete_campaign(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /stress/check` reply.
#[derive(Serialize)]
struct CampaignCheckReply {
    ok: bool,
    name: Option<String>,
    error: Option<String>,
    parsed: Option<tbd_stress::Campaign>,
}

async fn campaign_check(Json(body): Json<ScenarioText>) -> Json<CampaignCheckReply> {
    Json(match crate::stress::parse_campaign(&body.text) {
        Ok(file) => CampaignCheckReply {
            ok: true,
            name: Some(file.campaign.campaign.name.clone()),
            error: None,
            parsed: Some(file.campaign),
        },
        Err(error) => CampaignCheckReply {
            ok: false,
            name: None,
            error: Some(error),
            parsed: None,
        },
    })
}

// ----------------------------------------------------------- findings --

/// Query of `GET /findings`.
#[derive(Deserialize)]
struct FindingsQuery {
    #[serde(default = "default_limit")]
    limit: usize,
    #[serde(default)]
    grouped: bool,
    #[serde(flatten)]
    filter: FindingFilter,
}

/// `GET /findings`: summaries newest first, or groups by signature.
#[derive(Serialize)]
#[serde(untagged)]
enum FindingsReply {
    List(Vec<tbd_stress::FindingSummary>),
    Groups(Vec<FindingGroup>),
}

async fn findings(
    State(state): State<Shared>,
    Query(q): Query<FindingsQuery>,
) -> Json<FindingsReply> {
    Json(if q.grouped {
        FindingsReply::Groups(state.findings.grouped(&q.filter).await)
    } else {
        FindingsReply::List(state.findings.list(q.limit, &q.filter).await)
    })
}

async fn finding_get(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<tbd_stress::Finding>> {
    state
        .findings
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("no finding {id}")))
}

async fn finding_delete(State(state): State<Shared>, Path(id): Path<String>) -> Result<StatusCode> {
    if state.findings.delete(&id).await? {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found(format!("no finding {id}")))
    }
}

/// Body of `POST /findings/{id}/replay`.
#[derive(Deserialize, Default)]
#[serde(deny_unknown_fields)]
struct ReplayBody {
    #[serde(default)]
    targets: Vec<crate::load::Target>,
    #[serde(default)]
    attempts: Option<u32>,
}

async fn finding_replay(
    State(state): State<Shared>,
    Path(id): Path<String>,
    body: Option<Json<ReplayBody>>,
) -> Result<(StatusCode, Json<RunSummary>)> {
    let Json(body) = body.unwrap_or_default();
    let summary = state
        .spawn_replay(
            ReplayRequest {
                finding: id,
                targets: body.targets,
                attempts: body.attempts,
            },
            None,
        )
        .await?;
    Ok((StatusCode::ACCEPTED, Json(summary)))
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
    Ok(Json(state.run_validate(req, None).await?))
}

// ---------------------------------------------------------------- queue --

async fn queue(State(state): State<Shared>) -> Json<Vec<QueuedRun>> {
    Json(state.queue.list().await)
}

/// Body of `POST /queue`.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct QueueRequest {
    jobs: Vec<Job>,
}

async fn queue_push(
    State(state): State<Shared>,
    Json(body): Json<QueueRequest>,
) -> Result<(StatusCode, Json<Vec<QueuedRun>>)> {
    let items = state.enqueue(body.jobs, None).await?;
    Ok((StatusCode::ACCEPTED, Json(items)))
}

async fn queue_clear(State(state): State<Shared>) -> StatusCode {
    state.clear_queue().await;
    StatusCode::NO_CONTENT
}

async fn queue_remove(State(state): State<Shared>, Path(id): Path<String>) -> Result<StatusCode> {
    state.dequeue(&id).await?;
    Ok(StatusCode::NO_CONTENT)
}

// ------------------------------------------------------------ schedules --

async fn schedules(State(state): State<Shared>) -> Json<Vec<Schedule>> {
    Json(state.schedules.list().await)
}

async fn schedule_create(
    State(state): State<Shared>,
    Json(spec): Json<ScheduleSpec>,
) -> Result<(StatusCode, Json<Schedule>)> {
    let schedule = state.schedules.create(spec).await?;
    state.publish_schedules().await;
    Ok((StatusCode::CREATED, Json(schedule)))
}

async fn schedule_get(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<Json<Schedule>> {
    state
        .schedules
        .get(&id)
        .await
        .map(Json)
        .ok_or_else(|| ApiError::not_found(format!("no schedule {id}")))
}

async fn schedule_update(
    State(state): State<Shared>,
    Path(id): Path<String>,
    Json(spec): Json<ScheduleSpec>,
) -> Result<Json<Schedule>> {
    let schedule = state.schedules.update(&id, spec).await?;
    state.publish_schedules().await;
    Ok(Json(schedule))
}

async fn schedule_delete(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<StatusCode> {
    state.schedules.delete(&id).await?;
    state.publish_schedules().await;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /schedules/{id}/run`: queue the schedule's job now, whatever its cron says.
async fn schedule_run(
    State(state): State<Shared>,
    Path(id): Path<String>,
) -> Result<(StatusCode, Json<Vec<QueuedRun>>)> {
    let schedule = state
        .schedules
        .get(&id)
        .await
        .ok_or_else(|| ApiError::not_found(format!("no schedule {id}")))?;
    let items = state.enqueue(vec![schedule.job], Some(&id)).await?;
    Ok((StatusCode::ACCEPTED, Json(items)))
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
