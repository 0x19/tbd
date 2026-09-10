//! `chaos serve` end to end: a real API server on port 0 with a real stack
//! behind it, driven the way the UI drives it.

use std::{path::PathBuf, sync::Arc, time::Duration};

use futures::StreamExt;
use serde_json::{Value, json};
use tbd_chaos::{
    api::{self, AppState},
    config::{ChaosConfig, Source},
};

struct Server {
    base: String,
    http: reqwest::Client,
    state: Arc<AppState>,
    _dir: TempDir,
}

struct TempDir(PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

async fn boot() -> Server {
    boot_with(false, "").await
}

async fn boot_with_ui(with_ui: bool) -> Server {
    boot_with(with_ui, "").await
}

/// `with_ui` serves a two-file stand-in for the built UI at the root, the way
/// the image does; `slack` is a webhook URL for `[notify.slack]`.
async fn boot_with(with_ui: bool, slack: &str) -> Server {
    let dir = std::env::temp_dir().join(format!(
        "chaos-api-{}-{}",
        std::process::id(),
        rand::random::<u32>()
    ));
    std::fs::create_dir_all(dir.join("scenarios")).unwrap();
    if with_ui {
        std::fs::create_dir_all(dir.join("ui/runs")).unwrap();
        std::fs::write(dir.join("ui/index.html"), "<title>chaos</title>").unwrap();
        std::fs::write(dir.join("ui/runs/index.html"), "<title>runs</title>").unwrap();
        std::fs::write(dir.join("ui/404.html"), "<title>404</title>").unwrap();
    }
    std::fs::write(
        dir.join("scenarios/quick.toml"),
        r#"
[scenario]
name = "quick"

[stack.engines.engine-1]
heartbeat = "50ms"

[stack.protocols.protocol-1]
engine = "engine-1"

[load]
rate = 100
duration = "1s"

[[timeline]]
at = "300ms"
action = "log"
message = "midway"

[assertions]
max_error_rate = 0.0
min_requests = 50
"#,
    )
    .unwrap();
    std::fs::write(
        dir.join("topology.toml"),
        "[stack.engines.engine-1]\nheartbeat = \"50ms\"\n[stack.protocols.protocol-1]\nengine = \"engine-1\"\n",
    )
    .unwrap();

    let mut config: ChaosConfig = toml::from_str(
        &std::fs::read_to_string(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../configs/chaos/base.toml"
        ))
        .unwrap(),
    )
    .unwrap();
    config.paths.topology = dir.join("topology.toml");
    config.paths.scenarios = dir.join("scenarios");
    config.paths.results = dir.join("results");
    config.paths.schedules = dir.join("schedules.json");
    config.paths.stack = dir.join("stack.json");
    config.notify.slack.webhook = slack.to_owned();
    "#chaos-test".clone_into(&mut config.notify.slack.channel);
    if with_ui {
        config.serve.ui_dir = dir.join("ui").to_string_lossy().into_owned();
        config.serve.ui_path = String::new();
    }
    let state = api::state(
        config,
        Source {
            env: "test".into(),
            files: vec![],
        },
    )
    .await
    .unwrap();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    let served = Arc::clone(&state);
    tokio::spawn(async move {
        api::serve(served, listener, std::future::pending())
            .await
            .unwrap();
    });
    Server {
        base: format!("http://{addr}/api/chaos/v1"),
        http: reqwest::Client::new(),
        state,
        _dir: TempDir(dir),
    }
}

impl Server {
    async fn get(&self, path: &str) -> (u16, Value) {
        let r = self
            .http
            .get(format!("{}{path}", self.base))
            .send()
            .await
            .unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    async fn send(&self, method: reqwest::Method, path: &str, body: Option<Value>) -> (u16, Value) {
        let mut req = self.http.request(method, format!("{}{path}", self.base));
        if let Some(b) = body {
            req = req.json(&b);
        }
        let r = req.send().await.unwrap();
        let status = r.status().as_u16();
        (status, r.json().await.unwrap_or(Value::Null))
    }

    async fn post(&self, path: &str, body: Value) -> (u16, Value) {
        self.send(reqwest::Method::POST, path, Some(body)).await
    }

    /// Poll `GET /runs` until `pred` holds for the list, within 20 s.
    async fn wait_runs(&self, what: &str, pred: impl Fn(&[Value]) -> bool) -> Vec<Value> {
        let deadline = tokio::time::Instant::now() + Duration::from_secs(20);
        loop {
            let (_, runs) = self.get("/runs?limit=100").await;
            let runs = runs.as_array().cloned().unwrap_or_default();
            if pred(&runs) {
                return runs;
            }
            assert!(
                tokio::time::Instant::now() < deadline,
                "timed out waiting for {what}"
            );
            tokio::time::sleep(Duration::from_millis(200)).await;
        }
    }

    /// Event names of a run's SSE stream until `finished`.
    async fn follow(&self, id: &str) -> Vec<String> {
        let r = self
            .http
            .get(format!("{}/runs/{id}/events", self.base))
            .send()
            .await
            .unwrap();
        let mut names = Vec::new();
        let mut stream = r.bytes_stream();
        let mut buf = String::new();
        while let Some(chunk) = tokio::time::timeout(Duration::from_secs(20), stream.next())
            .await
            .expect("run must finish within 20s")
        {
            buf.push_str(std::str::from_utf8(&chunk.unwrap()).unwrap());
            while let Some(idx) = buf.find("\n\n") {
                let frame = buf[..idx].to_owned();
                buf.drain(..idx + 2);
                if let Some(name) = frame.lines().find_map(|l| l.strip_prefix("event: ")) {
                    names.push(name.to_owned());
                    if name == "finished" {
                        return names;
                    }
                }
            }
        }
        names
    }
}

#[tokio::test]
async fn overview_and_stack_reflect_the_running_topology() {
    let s = boot().await;
    let (status, overview) = s.get("/overview").await;
    assert_eq!(status, 200);
    assert_eq!(overview["env"], "test");
    assert_eq!(overview["scenarios"], 1);
    let stack = overview["stack"].as_array().unwrap();
    assert_eq!(stack.len(), 2);
    assert!(stack.iter().all(|i| i["running"] == true));
    let engine = stack.iter().find(|i| i["kind"] == "engine").unwrap();
    assert_eq!(engine["behavior"]["type"], "healthy");
}

#[tokio::test]
async fn stack_can_be_perturbed_by_hand() {
    let s = boot().await;
    let (status, body) = s.post("/stack/engine-1/stop", json!({})).await;
    assert_eq!(status, 200, "{body}");
    let engine = body
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "engine-1")
        .unwrap();
    assert_eq!(engine["running"], false);

    let (status, _) = s.post("/stack/engine-1/stop", json!({})).await;
    assert_eq!(status, 404, "stopping a stopped instance is unknown");

    let (status, _) = s.post("/stack/engine-1/start", json!({})).await;
    assert_eq!(status, 200);

    let behavior = json!({"type": "error", "kind": "unavailable", "rate": 1.0, "message": "t"});
    let (status, body) = s
        .send(
            reqwest::Method::PUT,
            "/stack/engine-1/behavior",
            Some(behavior),
        )
        .await;
    assert_eq!(status, 200, "{body}");
    let engine = body
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "engine-1")
        .unwrap();
    assert_eq!(engine["behavior"]["type"], "error");

    let (status, _) = s
        .send(
            reqwest::Method::PUT,
            "/stack/protocol-1/behavior",
            Some(json!({"type": "healthy"})),
        )
        .await;
    assert_eq!(status, 422, "the protocol has no fault injection");
}

#[tokio::test]
async fn scenario_run_streams_progress_and_is_recorded() {
    let s = boot().await;
    let (status, run) = s.post("/runs", json!({"scenario": "quick"})).await;
    assert_eq!(status, 202, "{run}");
    let id = run["id"].as_str().unwrap().to_owned();

    let (status, _) = s.post("/runs", json!({"scenario": "quick"})).await;
    assert_eq!(status, 409, "one run at a time");

    let names = s.follow(&id).await;
    assert_eq!(names.first().map(String::as_str), Some("started"));
    assert!(names.contains(&"phase".to_owned()));
    assert!(names.contains(&"load".to_owned()), "{names:?}");
    assert!(names.contains(&"timeline".to_owned()), "{names:?}");
    assert_eq!(names.last().map(String::as_str), Some("finished"));

    let (status, record) = s.get(&format!("/runs/{id}")).await;
    assert_eq!(status, 200);
    assert_eq!(record["status"], "passed", "{record}");
    assert!(!record["samples"].as_array().unwrap().is_empty());
    assert_eq!(record["events"].as_array().unwrap().len(), 1);
    assert_eq!(
        record["scenario"]["assertions"].as_array().unwrap().len(),
        2
    );

    // Replaying a finished run yields just the final record.
    assert_eq!(s.follow(&id).await, vec!["finished".to_owned()]);

    let (_, list) = s.get("/runs").await;
    assert_eq!(list.as_array().unwrap().len(), 1);
    let (_, detail) = s.get("/scenarios/quick").await;
    assert_eq!(detail["last_run"]["id"], id);

    // Persisted: a fresh store sees it.
    let dir = s.state.config.paths.results.clone();
    let store = api::RunStore::open(&dir).unwrap();
    assert_eq!(store.count().await, 1);
}

#[tokio::test]
async fn adhoc_load_can_be_cancelled() {
    let s = boot().await;
    let (status, run) = s
        .post(
            "/runs",
            json!({"name": "adhoc", "load": {"rate": 100, "duration": "30s"}}),
        )
        .await;
    assert_eq!(status, 202, "{run}");
    let id = run["id"].as_str().unwrap().to_owned();
    tokio::time::sleep(Duration::from_millis(1200)).await;
    let (status, _) = s.post(&format!("/runs/{id}/cancel"), json!({})).await;
    assert_eq!(status, 202);
    let names = s.follow(&id).await;
    assert_eq!(names.last().map(String::as_str), Some("finished"));
    let (_, record) = s.get(&format!("/runs/{id}")).await;
    assert_eq!(record["status"], "cancelled");
    assert!(record["load"]["requests_total"].as_u64().unwrap() > 0);
    assert!(
        record["duration_s"].as_f64().unwrap() < 10.0,
        "cancel must not wait for the duration"
    );

    let (status, _) = s
        .post("/runs", json!({"targets": [{"name": "x", "http_url": "nope"}], "load": {"rate": 1, "duration": "1s"}}))
        .await;
    assert_eq!(status, 422);
}

#[tokio::test]
async fn scenarios_can_be_checked_written_and_deleted() {
    let s = boot().await;
    let (_, reply) = s
        .post(
            "/scenarios/check",
            json!({"text": "[scenario]\nname = \"x\"\n[stack.protocols.p]\nengine = \"nope\"\n"}),
        )
        .await;
    assert_eq!(reply["ok"], false);
    assert!(reply["error"].as_str().unwrap().contains("unknown engine"));

    let good =
        "[scenario]\nname = \"new\"\n[stack.engines.e]\n[stack.protocols.p]\nengine = \"e\"\n";
    let (status, entry) = s
        .send(
            reqwest::Method::PUT,
            "/scenarios/sub/new",
            Some(json!({"text": good})),
        )
        .await;
    assert_eq!(status, 200, "{entry}");
    assert_eq!(entry["id"], "sub/new");
    let (_, list) = s.get("/scenarios").await;
    assert_eq!(list.as_array().unwrap().len(), 2);

    let (status, _) = s
        .send(
            reqwest::Method::PUT,
            "/scenarios/bad",
            Some(json!({"text": "nonsense ="})),
        )
        .await;
    assert_eq!(status, 422, "a file that does not check is not written");
    assert!(s.state.scenario_path("../escape").is_err());
    assert!(s.state.scenario_path("a/./b").is_err());

    let (status, _) = s
        .send(reqwest::Method::DELETE, "/scenarios/sub/new", None)
        .await;
    assert_eq!(status, 204);
    let (status, _) = s.get("/scenarios/sub/new").await;
    assert_eq!(status, 404);
}

/// The API keeps its JSON error shape for unknown routes, with and without
/// the UI's root wildcard on the same host.
#[tokio::test]
async fn unknown_api_route_is_a_json_404() {
    for with_ui in [false, true] {
        let s = boot_with_ui(with_ui).await;
        let (status, body) = s.get("/nope/at/all").await;
        assert_eq!(status, 404, "with_ui={with_ui}");
        assert_eq!(body["error"], "no such route", "with_ui={with_ui}: {body}");
    }
}

/// With the UI at the root: pages serve their `index.html`, unknown pages the
/// `404.html`, and the API stays reachable under its prefix.
#[tokio::test]
async fn ui_at_the_root_serves_pages_next_to_the_api() {
    let s = boot_with_ui(true).await;
    let origin = s.base.trim_end_matches("/api/chaos/v1").to_owned();
    for (path, status, needle) in [
        ("/", 200, "<title>chaos</title>"),
        ("/runs/", 200, "<title>runs</title>"),
        ("/nope/", 404, "<title>404</title>"),
    ] {
        let r = s.http.get(format!("{origin}{path}")).send().await.unwrap();
        assert_eq!(r.status().as_u16(), status, "{path}");
        assert!(r.text().await.unwrap().contains(needle), "{path}");
    }
    let (status, body) = s.get("/healthz").await;
    assert_eq!(status, 200, "{body}");
}

#[tokio::test]
async fn validate_runs_against_config_targets_and_is_recorded() {
    let s = boot().await;
    // The serve stack has no ledger; run one beside it for the ledger check.
    let ledger_stack: tbd_chaos::topology::StackConfig =
        toml::from_str("[stack.ledgers.ledger-1]\n")
            .map(|t: tbd_chaos::topology::TopologyFile| t.stack)
            .unwrap();
    let ledger_stack = ledger_stack.start().await.unwrap();
    let ledger_url = ledger_stack.get("ledger-1").unwrap().http_url();
    // The test stack is on ephemeral ports; point validate at it explicitly.
    let stack = s.state.stack_info().await.unwrap();
    let url = |kind: &str| {
        format!(
            "http://{}",
            stack.iter().find(|i| i.kind == kind).unwrap().addr
        )
    };
    let (status, record) = s
        .post(
            "/validate",
            json!({"protocol": url("protocol"), "engine": url("engine"), "ledger": ledger_url}),
        )
        .await;
    assert_eq!(status, 200, "{record}");
    assert_eq!(record["status"], "passed");
    assert_eq!(record["validate"]["failed"], 0);
    let (_, overview) = s.get("/overview").await;
    assert_eq!(overview["last_validate"]["status"], "passed");
}

#[tokio::test]
async fn queue_expands_all_scenarios_and_runs_them_one_after_another() {
    let s = boot().await;
    // An unknown scenario is rejected before anything is queued.
    let (status, _) = s
        .post("/queue", json!({"jobs": [{"scenario": "nope"}]}))
        .await;
    assert_eq!(status, 404);
    let (status, _) = s.post("/queue", json!({"jobs": []})).await;
    assert_eq!(status, 422);

    let (status, items) = s
        .post(
            "/queue",
            json!({"jobs": ["all_scenarios", {"scenario": "quick"}]}),
        )
        .await;
    assert_eq!(status, 202, "{items}");
    assert_eq!(items.as_array().unwrap().len(), 2, "{items}");
    assert_eq!(items[0]["scenario_id"], "quick");

    // The first started at once and holds the slot; the second waits.
    let (_, overview) = s.get("/overview").await;
    assert_eq!(overview["active_run"]["scenario_id"], "quick", "{overview}");
    assert_eq!(overview["queue"].as_array().unwrap().len(), 1);
    let (status, _) = s.post("/runs", json!({"scenario": "quick"})).await;
    assert_eq!(status, 409);

    let runs = s
        .wait_runs("two finished runs", |runs| {
            runs.iter().filter(|r| r["status"] == "passed").count() == 2
        })
        .await;
    assert!(runs.iter().all(|r| r["schedule_id"].is_null()));
    let (_, queue) = s.get("/queue").await;
    assert!(queue.as_array().unwrap().is_empty());

    // Queued items can be removed one by one or all at once.
    let (_, items) = s
        .post(
            "/queue",
            json!({"jobs": [{"scenario": "quick"}, {"scenario": "quick"}, {"scenario": "quick"}]}),
        )
        .await;
    let waiting = items[1]["id"].as_str().unwrap();
    let (status, _) = s
        .send(reqwest::Method::DELETE, &format!("/queue/{waiting}"), None)
        .await;
    assert_eq!(status, 204);
    let (status, _) = s
        .send(reqwest::Method::DELETE, &format!("/queue/{waiting}"), None)
        .await;
    assert_eq!(status, 404);
    let (status, _) = s.send(reqwest::Method::DELETE, "/queue", None).await;
    assert_eq!(status, 204);
    let (_, queue) = s.get("/queue").await;
    assert!(queue.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn schedules_persist_fire_on_their_own_and_run_on_demand() {
    let s = boot().await;
    let (status, body) = s
        .post(
            "/schedules",
            json!({"name": "bad", "cron": "every day", "job": "all_scenarios"}),
        )
        .await;
    assert_eq!(status, 422, "{body}");

    let (status, sched) = s
        .post(
            "/schedules",
            json!({"name": "each second", "cron": "* * * * * *", "job": {"scenario": "quick"}}),
        )
        .await;
    assert_eq!(status, 201, "{sched}");
    let id = sched["id"].as_str().unwrap().to_owned();
    assert!(sched["next_at"].is_string(), "{sched}");
    assert!(
        std::fs::read_to_string(s.state.config.paths.schedules.clone())
            .unwrap()
            .contains("each second")
    );

    // The scheduler fires it within a couple of seconds; the run carries the id.
    s.wait_runs("a scheduled run", |runs| {
        runs.iter().any(|r| r["schedule_id"] == id.as_str())
    })
    .await;
    let (_, got) = s.get(&format!("/schedules/{id}")).await;
    assert!(got["fired"].as_u64().unwrap() >= 1, "{got}");
    assert!(got["last_fired_at"].is_string());

    // Off: kept, no next fire time.
    let (status, off) = s
        .send(
            reqwest::Method::PUT,
            &format!("/schedules/{id}"),
            Some(json!({"name": "each second", "cron": "* * * * * *", "job": {"scenario": "quick"}, "enabled": false})),
        )
        .await;
    assert_eq!(status, 200, "{off}");
    assert!(off["next_at"].is_null());
    assert!(!off["enabled"].as_bool().unwrap());

    // Run now still works while disabled.
    let (status, items) = s.post(&format!("/schedules/{id}/run"), json!({})).await;
    assert_eq!(status, 202, "{items}");
    assert_eq!(items[0]["schedule_id"], id.as_str());
    let (_, overview) = s.get("/overview").await;
    assert_eq!(overview["schedules"], 1);
    assert_eq!(overview["schedules_enabled"], 0);

    let (status, _) = s
        .send(reqwest::Method::DELETE, &format!("/schedules/{id}"), None)
        .await;
    assert_eq!(status, 204);
    let (status, _) = s.get(&format!("/schedules/{id}")).await;
    assert_eq!(status, 404);
    let (_, list) = s.get("/schedules").await;
    assert!(list.as_array().unwrap().is_empty());
}

#[tokio::test]
async fn me_reflects_the_identity_headers_envoy_sets() {
    let s = boot().await;
    let (status, me) = s.get("/me").await;
    assert_eq!(status, 200);
    assert!(me["user"].is_null(), "{me}");
    assert_eq!(me["signout"], "/oauth2/signout");

    let r = s
        .http
        .get(format!("{}/me", s.base))
        .header("x-user-sub", "u-1")
        .header("x-user-email", "ops@example.test")
        .header("x-user-name", "Ops")
        .header("x-user-role", "admin")
        .send()
        .await
        .unwrap();
    let me: Value = r.json().await.unwrap();
    assert_eq!(me["user"]["email"], "ops@example.test");
    assert_eq!(me["user"]["name"], "Ops");
    assert_eq!(me["user"]["role"], "admin");
}

/// A stand-in Slack webhook: records every body it receives.
async fn slack_stub() -> (String, Arc<tokio::sync::Mutex<Vec<Value>>>) {
    use axum::{Json, Router, extract::State, routing::post};
    let seen: Arc<tokio::sync::Mutex<Vec<Value>>> = Arc::default();
    let app = Router::new()
        .route(
            "/hook",
            post(
                |State(seen): State<Arc<tokio::sync::Mutex<Vec<Value>>>>,
                 Json(body): Json<Value>| async move {
                    seen.lock().await.push(body);
                    "ok"
                },
            ),
        )
        .with_state(Arc::clone(&seen));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let addr = listener.local_addr().unwrap();
    tokio::spawn(async move { axum::serve(listener, app).await.unwrap() });
    (format!("http://{addr}/hook"), seen)
}

#[tokio::test]
async fn failed_runs_are_posted_to_slack_and_passed_ones_are_not() {
    let (hook, seen) = slack_stub().await;
    let s = boot_with(false, &hook).await;
    let (_, overview) = s.get("/overview").await;
    assert_eq!(overview["notify"]["enabled"], true, "{overview}");
    assert_eq!(overview["notify"]["channel"], "#chaos-test");

    // The hello.
    let (status, _) = s.post("/notify/test", json!({})).await;
    assert_eq!(status, 204);

    // A scenario that cannot pass: it wants more requests than 1 s at 100 req/s gives.
    std::fs::write(
        s.state.config.paths.scenarios.join("doomed.toml"),
        r#"
[scenario]
name = "doomed"
[stack.engines.engine-1]
[stack.protocols.protocol-1]
engine = "engine-1"
[load]
rate = 100
duration = "1s"
[assertions]
min_requests = 100000
"#,
    )
    .unwrap();
    let (status, run) = s.post("/runs", json!({"scenario": "doomed"})).await;
    assert_eq!(status, 202, "{run}");
    s.follow(run["id"].as_str().unwrap()).await;
    let (status, run) = s.post("/runs", json!({"scenario": "quick"})).await;
    assert_eq!(status, 202, "{run}");
    s.follow(run["id"].as_str().unwrap()).await;

    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let got = seen.lock().await.clone();
        if got.len() >= 2 {
            assert_eq!(got.len(), 2, "passed run must not post: {got:?}");
            assert!(
                got[0]["text"]
                    .as_str()
                    .unwrap()
                    .contains("can reach this channel")
            );
            let msg = got[1].to_string();
            assert!(msg.contains("*doomed* failed on *test*"), "{msg}");
            assert!(msg.contains("min_requests"), "{msg}");
            assert_eq!(got[1]["channel"], "#chaos-test");
            break;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "no slack post: {got:?}"
        );
        tokio::time::sleep(Duration::from_millis(100)).await;
    }
    tokio::time::sleep(Duration::from_millis(300)).await;
    assert_eq!(seen.lock().await.len(), 2, "the passed run posted");
}

#[tokio::test]
async fn instances_can_be_replicated_added_and_removed_at_runtime() {
    let s = boot().await;
    let names = |v: &Value| -> Vec<String> {
        v.as_array()
            .unwrap()
            .iter()
            .map(|i| i["name"].as_str().unwrap().to_owned())
            .collect()
    };

    // A replica of the protocol: same engine, fresh port, marked added.
    let (status, body) = s.post("/stack/protocol-1/clone", json!({})).await;
    assert_eq!(status, 201, "{body}");
    assert_eq!(names(&body), ["engine-1", "protocol-1", "protocol-2"]);
    let p2 = body
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "protocol-2")
        .unwrap();
    assert_eq!(p2["running"], true);
    assert_eq!(p2["added"], true);
    assert_eq!(p2["depends_on"], json!(["engine-1"]));
    let p1 = body
        .as_array()
        .unwrap()
        .iter()
        .find(|i| i["name"] == "protocol-1")
        .unwrap();
    assert_ne!(p1["addr"], p2["addr"]);
    assert_eq!(p1["added"], false);

    // Two engine replicas at once, then a new protocol on one of them.
    let (status, body) = s.post("/stack/engine-1/clone", json!({"count": 2})).await;
    assert_eq!(status, 201, "{body}");
    assert!(names(&body).contains(&"engine-3".to_owned()));
    let (status, body) = s
        .post("/stack", json!({"kind": "protocol", "engine": "engine-3"}))
        .await;
    assert_eq!(status, 201, "{body}");
    assert!(names(&body).contains(&"protocol-3".to_owned()));
    let (status, body) = s
        .post(
            "/stack",
            json!({"kind": "engine", "name": "spare", "heartbeat": "50ms"}),
        )
        .await;
    assert_eq!(status, 201, "{body}");
    let (status, _) = s
        .post("/stack", json!({"kind": "protocol", "engine": "nope"}))
        .await;
    assert_eq!(status, 409, "unknown engine dependency");
    let (status, _) = s
        .post("/stack", json!({"kind": "engine", "name": "engine-1"}))
        .await;
    assert_eq!(status, 409, "duplicate name");

    // Load with no explicit targets spreads over every running protocol.
    let (status, run) = s
        .post(
            "/runs",
            json!({"name": "spread", "load": {"rate": 100, "duration": "1s"}}),
        )
        .await;
    assert_eq!(status, 202, "{run}");
    s.follow(run["id"].as_str().unwrap()).await;
    let (_, record) = s
        .get(&format!("/runs/{}", run["id"].as_str().unwrap()))
        .await;
    let targets = record["load"]["per_target"].as_object().unwrap();
    assert_eq!(targets.len(), 3, "{targets:?}");

    // Removal: dependents first, topology instances never.
    let (status, _) = s
        .send(reqwest::Method::DELETE, "/stack/engine-3", None)
        .await;
    assert_eq!(status, 409, "protocol-3 still needs it");
    let (status, _) = s
        .send(reqwest::Method::DELETE, "/stack/protocol-3", None)
        .await;
    assert_eq!(status, 200);
    let (status, _) = s
        .send(reqwest::Method::DELETE, "/stack/engine-3", None)
        .await;
    assert_eq!(status, 200);
    let (status, _) = s
        .send(reqwest::Method::DELETE, "/stack/engine-1", None)
        .await;
    assert_eq!(status, 409, "from the topology");
    let (status, _) = s.send(reqwest::Method::DELETE, "/stack/nope", None).await;
    assert_eq!(status, 404);
    let (_, body) = s.get("/stack").await;
    assert_eq!(
        names(&body),
        ["engine-1", "engine-2", "protocol-1", "protocol-2", "spare"]
    );
}

#[tokio::test]
async fn added_instances_come_back_after_a_restart() {
    let s = boot().await;
    let (status, _) = s.post("/stack/protocol-1/clone", json!({})).await;
    assert_eq!(status, 201);
    let (status, _) = s
        .post(
            "/stack",
            json!({"kind": "engine", "name": "spare", "heartbeat": "50ms"}),
        )
        .await;
    assert_eq!(status, 201);
    let (status, _) = s
        .post(
            "/stack",
            json!({"kind": "protocol", "name": "on-spare", "engine": "spare"}),
        )
        .await;
    assert_eq!(status, 201);
    let file = std::fs::read_to_string(&s.state.config.paths.stack).unwrap();
    assert!(
        file.contains("protocol-2") && file.contains("on-spare"),
        "{file}"
    );

    // A second serve on the same files: everything added is running again,
    // engines before the protocols that need them.
    let config = s.state.config.clone();
    s.state.shutdown().await;
    let again = api::state(
        config,
        Source {
            env: "test".into(),
            files: vec![],
        },
    )
    .await
    .unwrap();
    let info = again.stack_info().await.unwrap();
    let running: Vec<(String, bool)> = info
        .iter()
        .filter(|i| i.added)
        .map(|i| (i.name.clone(), i.running))
        .collect();
    assert_eq!(
        running,
        [
            ("on-spare".to_owned(), true),
            ("protocol-2".to_owned(), true),
            ("spare".to_owned(), true)
        ]
    );
    again.remove_instance("on-spare").await.unwrap();
    let file = std::fs::read_to_string(&again.config.paths.stack).unwrap();
    assert!(!file.contains("on-spare"), "{file}");
    again.shutdown().await;
}
