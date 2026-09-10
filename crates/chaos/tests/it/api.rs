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
    let dir = std::env::temp_dir().join(format!(
        "chaos-api-{}-{}",
        std::process::id(),
        rand::random::<u32>()
    ));
    std::fs::create_dir_all(dir.join("scenarios")).unwrap();
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
        base: format!("http://{addr}/api/chaos"),
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

#[tokio::test]
async fn validate_runs_against_config_targets_and_is_recorded() {
    let s = boot().await;
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
            json!({"protocol": url("protocol"), "engine": url("engine")}),
        )
        .await;
    assert_eq!(status, 200, "{record}");
    assert_eq!(record["status"], "passed");
    assert_eq!(record["validate"]["failed"], 0);
    let (_, overview) = s.get("/overview").await;
    assert_eq!(overview["last_validate"]["status"], "passed");
}
