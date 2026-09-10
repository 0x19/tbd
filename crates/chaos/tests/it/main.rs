//! Integration tests: boot a real stack in-process and exercise the tool
//! against it, exactly as `chaos up` + `chaos validate` would.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::time::Duration;

use tbd_chaos::{topology::StackConfig, validate};
use tbd_common::fault::{Behavior, ErrorKind};

#[derive(serde::Deserialize)]
struct File {
    stack: StackConfig,
}

fn try_topology(toml_text: &str) -> Result<StackConfig, toml::de::Error> {
    toml::from_str::<File>(toml_text).map(|f| f.stack)
}

fn topology(toml_text: &str) -> StackConfig {
    try_topology(toml_text).unwrap()
}

const TWO_TIER: &str = r#"
[stack.engines.engine-1]
heartbeat = "50ms"

[stack.protocols.protocol-1]
engine = "engine-1"
"#;

#[tokio::test]
async fn stack_starts_in_dependency_order_and_validate_passes() {
    let stack = topology(TWO_TIER).start().await.unwrap();
    let engine = stack.get("engine-1").unwrap();
    let protocol = stack.get("protocol-1").unwrap();

    let report = validate::run(validate::Targets {
        protocol: protocol.http_url(),
        engine: engine.http_url(),
        timeout: Duration::from_secs(5),
    })
    .await;
    assert!(report.ok(), "validate failed:\n{}", report.render());
    assert_eq!(report.checks.len(), 11);

    // The engine counted the validate traffic.
    let counts = engine.requests().unwrap();
    assert!(counts.total >= 3, "engine saw {counts:?}");

    stack.shutdown().await;
}

#[tokio::test]
async fn fault_on_engine_shows_up_through_the_protocol() {
    let stack = topology(TWO_TIER).start().await.unwrap();
    let engine = stack.get("engine-1").unwrap();
    let protocol = stack.get("protocol-1").unwrap();

    engine.fault().unwrap().set(Behavior::Error {
        kind: ErrorKind::Unavailable,
        rate: 1.0,
        message: "chaos".into(),
    });
    let status = reqwest::Client::new()
        .post(format!("{}/v1/evaluate", protocol.http_url()))
        .json(&serde_json::json!({ "subject_id": "s1" }))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(status, 503, "UNAVAILABLE must map to 503 at the protocol");

    stack.shutdown().await;
}

#[tokio::test]
async fn engine_can_be_stopped_and_restarted_on_the_same_port() {
    let mut stack = topology(TWO_TIER).start().await.unwrap();
    let addr = stack.get("engine-1").unwrap().addr;
    let protocol_url = stack.get("protocol-1").unwrap().http_url();
    let http = reqwest::Client::new();

    stack.stop_instance("engine-1").await.unwrap();
    let down = http
        .get(format!("{protocol_url}/readyz"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(
        down, 503,
        "protocol must report not ready while the engine is down"
    );

    stack.start_instance("engine-1").await.unwrap();
    assert_eq!(
        stack.get("engine-1").unwrap().addr,
        addr,
        "restart must reuse the port"
    );
    let up = http
        .get(format!("{protocol_url}/readyz"))
        .send()
        .await
        .unwrap()
        .status();
    assert_eq!(up, 200, "protocol must recover once the engine is back");

    stack.shutdown().await;
}

#[test]
fn topology_rejects_unknown_keys_and_dangling_references() {
    assert!(
        try_topology("[stack.engines.e]\nbogus = 1\n").is_err(),
        "unknown keys must fail"
    );
    let dangling = topology("[stack.protocols.p]\nengine = \"nope\"\n");
    assert!(dangling.check().is_err());
}

#[tokio::test]
async fn scenario_with_fault_timeline_runs_end_to_end() {
    let file: tbd_chaos::scenario::ScenarioFile = toml::from_str(
        r#"
[scenario]
name = "inline"

[stack.engines.e]
heartbeat = "50ms"

[stack.protocols.p]
engine = "e"

[load]
rate = 100
duration = "1s"
timeout = "1s"

[[load.operations]]
op = "rest_evaluate"

[[timeline]]
at = "500ms"
action = "set_behavior"
service = "e"
[timeline.behavior]
type = "error"
kind = "internal"
rate = 1.0

[assertions]
min_requests = 50
max_error_rate = 0.80

[assertions.services.e]
min_requests = 50
"#,
    )
    .unwrap();
    file.check().unwrap();

    let result = tbd_chaos::scenario::run_scenario(&file).await;
    assert!(
        result.passed,
        "{}",
        tbd_chaos::scenario::report::render(&result)
    );
    let load = result.load.as_ref().unwrap();
    assert!(
        load.error_rate > 0.2 && load.error_rate < 0.8,
        "half the run failed: {}",
        load.error_rate
    );
    assert_eq!(
        load.errors.get("http 502"),
        Some(&load.requests_failed),
        "INTERNAL maps to 502"
    );
    assert_eq!(result.events.len(), 1);
    assert!(result.events[0].error.is_none());
}

#[test]
fn scenario_file_rejects_dangling_and_unknown() {
    let bad = r#"
[scenario]
name = "x"
[stack.engines.e]
[stack.protocols.p]
engine = "e"
[[timeline]]
at = "1s"
action = "stop"
service = "ghost"
"#;
    let parsed: tbd_chaos::scenario::ScenarioFile = toml::from_str(bad).unwrap();
    assert!(parsed.check().unwrap_err().contains("ghost"));
    assert!(
        toml::from_str::<tbd_chaos::scenario::ScenarioFile>(
            "[scenario]\nname=\"x\"\nnope=1\n[stack.engines.e]\n"
        )
        .is_err()
    );
}
