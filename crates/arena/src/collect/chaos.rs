//! The chaos tool: its current run, followed live, and its last end-to-end
//! check of every way in.
//!
//! Once a `[collect] chaos_every` the arena reads `GET /overview`. A run in
//! `active_run` is followed over its own feed (`GET /runs/{id}/events`), and
//! each `load` frame, cumulative since the run began, is diffed against the
//! one before into per-second rates. `last_validate` names the newest check
//! of every surface; when it changes, its report (`GET /runs/{id}`) becomes
//! the surfaces. The check itself is the chaos tool's own, on a schedule the
//! arena creates once if it is missing, with Slack notifications off: the
//! arena shows how old the check is and never infers a surface's state.

use std::{
    collections::BTreeMap,
    time::{Duration, SystemTime},
};

use futures::StreamExt as _;
use serde::Deserialize;
use serde_json::Value;
use tbd_proto::arena::v1::{ChaosRun, SurfaceState};

use crate::world::World;

/// The schedule the arena keeps in the chaos tool, by name.
pub const SCHEDULE_NAME: &str = "arena: every way in";

/// The protocol's checks that stand for each way in, in the order the page
/// lists them. `http_mcp_tools` also says how many tools there are.
pub const SURFACES: [(&str, &str); 5] = [
    ("rest", "rest_evaluate"),
    ("sse", "sse_events"),
    ("websocket", "ws_mux"),
    ("mcp", "http_mcp_tools"),
    ("grpc", "grpc_protocol_ping"),
];

#[derive(Debug, Deserialize)]
struct Overview {
    #[serde(default)]
    active_run: Option<Summary>,
    #[serde(default)]
    last_validate: Option<Summary>,
}

#[derive(Debug, Clone, Deserialize)]
struct Summary {
    id: String,
    #[serde(default)]
    kind: String,
    #[serde(default)]
    name: String,
    #[serde(default)]
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
struct Record {
    #[serde(default)]
    validate: Option<Report>,
    #[serde(default)]
    finished_at: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
struct Report {
    #[serde(default)]
    checks: Vec<Check>,
}

#[derive(Debug, Deserialize)]
struct Check {
    name: String,
    passed: bool,
    #[serde(default)]
    latency_ms: f64,
    #[serde(default)]
    detail: String,
}

/// Read the chaos tool forever.
pub fn start(
    url: &str,
    cron: &str,
    every: Duration,
    timeout: Duration,
    world: World,
) -> tokio::task::JoinHandle<()> {
    let base = url.trim_end_matches('/').to_owned();
    let cron = cron.trim().to_owned();
    // A read has a deadline; a followed feed does not, so it has its own client.
    let http = reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_default();
    let feed = reqwest::Client::builder()
        .connect_timeout(timeout)
        .build()
        .unwrap_or_default();
    tokio::spawn(async move {
        let mut scheduled = cron.is_empty();
        let mut seen_validate: Option<String> = None;
        let mut follower: Option<(String, tokio::task::JoinHandle<()>)> = None;
        let mut tick = tokio::time::interval(every);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            let overview: Overview = match get(&http, &format!("{base}/overview")).await {
                Ok(v) => v,
                Err(error) => {
                    world.source_failed("chaos", error);
                    world.set_chaos(None);
                    if let Some((_, task)) = follower.take() {
                        task.abort();
                    }
                    continue;
                }
            };
            world.source_ok("chaos");
            if !scheduled {
                match ensure_schedule(&http, &base, &cron).await {
                    Ok(()) => scheduled = true,
                    Err(error) => {
                        tracing::warn!(%error, "arena: the chaos schedule is not there yet");
                    }
                }
            }

            // The current run: follow a new one, drop a finished one.
            if let Some(run) = overview.active_run {
                if follower.as_ref().map(|(id, _)| id) != Some(&run.id) {
                    if let Some((_, task)) = follower.take() {
                        task.abort();
                    }
                    world.set_chaos(Some(ChaosRun {
                        state: "running".to_owned(),
                        id: run.id.clone(),
                        kind: run.kind.clone(),
                        name: run.name.clone(),
                        ..ChaosRun::default()
                    }));
                    let task = tokio::spawn(follow(
                        feed.clone(),
                        base.clone(),
                        run.id.clone(),
                        world.clone(),
                    ));
                    follower = Some((run.id, task));
                }
            } else {
                if let Some((_, task)) = follower.take() {
                    task.abort();
                }
                world.set_chaos(Some(ChaosRun {
                    state: "idle".to_owned(),
                    ..ChaosRun::default()
                }));
            }

            // The newest end-to-end check, once per new one.
            if let Some(last) = overview.last_validate
                && seen_validate.as_deref() != Some(last.id.as_str())
            {
                match get::<Record>(&http, &format!("{base}/runs/{}", last.id)).await {
                    Ok(record) => {
                        let at = record
                            .finished_at
                            .or(last.finished_at)
                            .map_or_else(SystemTime::now, SystemTime::from);
                        let (surfaces, tools) =
                            surfaces(record.validate.map(|r| r.checks).unwrap_or_default());
                        world.set_surfaces(surfaces, at, tools);
                        seen_validate = Some(last.id);
                    }
                    Err(error) => tracing::debug!(%error, "arena: reading the last check"),
                }
            }
        }
    })
}

async fn get<T: serde::de::DeserializeOwned>(
    http: &reqwest::Client,
    url: &str,
) -> Result<T, String> {
    let resp = http
        .get(url)
        .send()
        .await
        .map_err(|e| format!("{url}: {e}"))?;
    let status = resp.status();
    if !status.is_success() {
        return Err(format!("{url}: {status}"));
    }
    resp.json().await.map_err(|e| format!("{url}: {e}"))
}

/// Create the arena's check schedule unless one by its name exists.
async fn ensure_schedule(http: &reqwest::Client, base: &str, cron: &str) -> Result<(), String> {
    let schedules: Vec<Value> = get(http, &format!("{base}/schedules")).await?;
    if schedules.iter().any(|s| s["name"] == SCHEDULE_NAME) {
        return Ok(());
    }
    let resp = http
        .post(format!("{base}/schedules"))
        .json(&serde_json::json!({
            "name": SCHEDULE_NAME,
            "cron": cron,
            "job": { "validate": {} },
            "enabled": true,
            "notify": "off",
        }))
        .send()
        .await
        .map_err(|e| format!("schedules: {e}"))?;
    if resp.status().is_success() {
        tracing::info!(
            cron,
            "arena: created the chaos schedule that checks every way in"
        );
        Ok(())
    } else {
        Err(format!("schedules: {}", resp.status()))
    }
}

/// The report's checks as the surfaces the page names, and the MCP tool count.
fn surfaces(checks: Vec<Check>) -> (Vec<SurfaceState>, Option<u32>) {
    let by_name: BTreeMap<String, Check> =
        checks.into_iter().map(|c| (c.name.clone(), c)).collect();
    let mut tools = None;
    let list = SURFACES
        .iter()
        .filter_map(|(surface, check)| {
            let c = by_name.get(*check)?;
            if *surface == "mcp" && c.passed {
                tools = c
                    .detail
                    .strip_prefix("tools=")
                    .and_then(|n| n.split_whitespace().next())
                    .and_then(|n| n.parse().ok());
            }
            Some(SurfaceState {
                name: (*surface).to_owned(),
                up: c.passed,
                latency_ms: c.latency_ms,
                check: c.name.clone(),
                detail: c.detail.clone(),
            })
        })
        .collect();
    (list, tools)
}

/// The previous `load` frame's cumulative counts.
#[derive(Debug, Clone, Copy, Default)]
struct Totals {
    elapsed_s: f64,
    requests: f64,
    failed: f64,
    completion_tokens: Option<f64>,
}

impl Totals {
    fn of(snapshot: &Value) -> Self {
        let n = |v: &Value| v.as_f64().unwrap_or(0.0);
        let completion = snapshot["per_op"].as_object().and_then(|ops| {
            let tokens: Vec<f64> = ops
                .values()
                .filter_map(|op| op["counters"]["completion_tokens"].as_f64())
                .collect();
            (!tokens.is_empty()).then(|| tokens.iter().sum())
        });
        Self {
            elapsed_s: n(&snapshot["elapsed_s"]),
            requests: n(&snapshot["requests_total"]),
            failed: n(&snapshot["requests_failed"]),
            completion_tokens: completion,
        }
    }
}

/// Follow one run's feed until it ends, writing each frame into the world.
async fn follow(http: reqwest::Client, base: String, id: String, world: World) {
    let url = format!("{base}/runs/{id}/events");
    let resp = match http
        .get(&url)
        .header("accept", "text/event-stream")
        .send()
        .await
    {
        Ok(r) if r.status().is_success() => r,
        Ok(r) => return tracing::debug!(status = %r.status(), "arena: the run feed"),
        Err(error) => return tracing::debug!(%error, "arena: the run feed"),
    };
    let mut body = resp.bytes_stream();
    let mut buffer = String::new();
    let mut prev: Option<Totals> = None;
    while let Some(Ok(chunk)) = body.next().await {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(end) = buffer.find("\n\n") {
            let frame: String = buffer.drain(..end + 2).collect();
            let (event, data) = parse_frame(&frame);
            let Ok(data) = serde_json::from_str::<Value>(&data) else {
                continue;
            };
            match event.as_str() {
                "phase" => {
                    let phase = data["name"].as_str().unwrap_or_default().to_owned();
                    world.update_chaos(|run| run.phase = phase);
                }
                "load" => {
                    let now = Totals::of(&data["snapshot"]);
                    let p99 = data["snapshot"]["latency"]["p99_ms"]
                        .as_f64()
                        .unwrap_or(0.0);
                    let rates = prev.and_then(|p| rates(p, now));
                    world.update_chaos(|run| {
                        run.elapsed_s = now.elapsed_s;
                        run.p99_ms = p99;
                        if let Some((rps, error_rate, tps)) = rates {
                            run.rps = rps;
                            run.error_rate = error_rate;
                            run.tokens_per_second = tps;
                        }
                    });
                    prev = Some(now);
                }
                "finished" => {
                    world.set_chaos(Some(ChaosRun {
                        state: "idle".to_owned(),
                        ..ChaosRun::default()
                    }));
                    return;
                }
                _ => {}
            }
        }
    }
}

/// Requests a second, the failed share and tokens a second between two
/// cumulative frames; `None` when time did not move or the counts went back
/// (a phase that reset them), so the last rates stand.
fn rates(prev: Totals, now: Totals) -> Option<(f64, f64, Option<f64>)> {
    let dt = now.elapsed_s - prev.elapsed_s;
    let dr = now.requests - prev.requests;
    if dt <= 0.0 || dr < 0.0 {
        return None;
    }
    let error_rate = if dr > 0.0 {
        (now.failed - prev.failed).max(0.0) / dr
    } else {
        0.0
    };
    let tps = match (prev.completion_tokens, now.completion_tokens) {
        (Some(a), Some(b)) if b >= a => Some((b - a) / dt),
        _ => None,
    };
    Some((dr / dt, error_rate, tps))
}

/// `event:` and the joined `data:` lines of one server-sent event.
fn parse_frame(frame: &str) -> (String, String) {
    let mut event = "message".to_owned();
    let mut data = Vec::new();
    for line in frame.lines() {
        if let Some(v) = line.strip_prefix("event:") {
            v.trim().clone_into(&mut event);
        } else if let Some(v) = line.strip_prefix("data:") {
            data.push(v.strip_prefix(' ').unwrap_or(v));
        }
    }
    (event, data.join("\n"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn two_load_frames_become_rates() {
        let a = Totals::of(
            &serde_json::json!({"elapsed_s": 1.0, "requests_total": 10, "requests_failed": 0,
            "per_op": {"llm_generate": {"counters": {"completion_tokens": 100}}}}),
        );
        let b = Totals::of(
            &serde_json::json!({"elapsed_s": 2.0, "requests_total": 14, "requests_failed": 1,
            "per_op": {"llm_generate": {"counters": {"completion_tokens": 400}}}}),
        );
        let (rps, err, tps) = rates(a, b).unwrap();
        assert!((rps - 4.0).abs() < 1e-9);
        assert!((err - 0.25).abs() < 1e-9);
        assert_eq!(tps, Some(300.0));
        assert!(
            rates(b, a).is_none(),
            "counts that went back keep the last rates"
        );
    }

    #[test]
    fn the_checks_become_surfaces_and_the_tool_count() {
        let checks = vec![
            Check {
                name: "http_mcp_tools".into(),
                passed: true,
                latency_ms: 4.0,
                detail: "tools=8".into(),
            },
            Check {
                name: "ws_mux".into(),
                passed: false,
                latency_ms: 9.0,
                detail: "closed".into(),
            },
            Check {
                name: "http_healthz".into(),
                passed: true,
                latency_ms: 1.0,
                detail: String::new(),
            },
        ];
        let (list, tools) = surfaces(checks);
        assert_eq!(tools, Some(8));
        let names: Vec<_> = list.iter().map(|s| (s.name.as_str(), s.up)).collect();
        assert_eq!(names, vec![("websocket", false), ("mcp", true)]);
    }

    #[test]
    fn a_frame_parses_with_or_without_the_space() {
        assert_eq!(
            parse_frame("event: load\ndata:{\"a\":1}\n\n"),
            ("load".to_owned(), "{\"a\":1}".to_owned())
        );
    }
}
