//! Integration tests: boot a real stack in-process and exercise the tool
//! against it, exactly as `chaos up` + `chaos validate` would.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod api;

use std::time::Duration;

use tbd_chaos::{
    kinds::{self, FieldKind},
    topology::StackConfig,
    validate,
};
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

/// Engine, protocol and ledger by name, for the tests that pin behaviour of
/// those three and run no validate; validate needs every registered kind
/// present (`full_stack`), or a scaffolded kind fails its own check.
const TWO_TIER: &str = r#"
[stack.engines.engine-1]
heartbeat = "50ms"

[stack.protocols.protocol-1]
engine = "engine-1"

[stack.ledgers.ledger-1]
"#;

/// One `<kind>-1` of every registered kind, built from the registry so a
/// kind added by `tbd new service` is covered without editing this file.
/// Dependency fields point at `<dep>-1`; durations get a short value so the
/// tests stay quick.
fn full_stack() -> StackConfig {
    let mut config = StackConfig::default();
    for kind in kinds::ALL {
        let mut table = toml::Table::new();
        for field in kind.fields {
            let value = match field.kind {
                FieldKind::InstanceOf(dep) => format!("{dep}-1"),
                FieldKind::Duration => "50ms".to_owned(),
                FieldKind::Text => match field.default {
                    Some(d) => d.to_owned(),
                    None => continue,
                },
            };
            table.insert(field.name.to_owned(), toml::Value::String(value));
        }
        config
            .insert(&format!("{}-1", kind.name), kind, table)
            .unwrap_or_else(|e| panic!("{}: {e}", kind.name));
    }
    config.check().unwrap();
    config
}

#[test]
fn registry_is_consistent() {
    let mut names: Vec<&str> = kinds::ALL.iter().map(|k| k.name).collect();
    let mut plurals: Vec<&str> = kinds::ALL.iter().map(|k| k.plural).collect();
    names.sort_unstable();
    plurals.sort_unstable();
    names.dedup();
    plurals.dedup();
    assert_eq!(names.len(), kinds::ALL.len(), "kind names must be unique");
    assert_eq!(plurals.len(), kinds::ALL.len(), "plurals must be unique");
    let mut checks: Vec<&str> = validate::checks().map(|(_, c)| c.name).collect();
    let total = checks.len();
    checks.sort_unstable();
    checks.dedup();
    assert_eq!(checks.len(), total, "check names must be unique");
    for kind in kinds::ALL {
        assert_eq!(kinds::by_name(kind.name).map(|k| k.name), Some(kind.name));
        assert_eq!(
            kinds::by_plural(kind.plural).map(|k| k.name),
            Some(kind.name)
        );
        assert!(
            kind.target.is_some() || kind.checks.is_empty(),
            "{}: checks without a target",
            kind.name
        );
        for f in kind.fields {
            if let FieldKind::InstanceOf(dep) = f.kind {
                assert!(
                    kinds::by_name(dep).is_some(),
                    "{}.{}: unknown kind {dep}",
                    kind.name,
                    f.name
                );
            }
            assert!(
                !(f.required && f.default.is_some()),
                "{}.{}: required with a default",
                kind.name,
                f.name
            );
        }
        assert_eq!(
            kind.env_var(),
            format!("CHAOS_{}_URL", kind.name.to_uppercase())
        );
        // A kind must accept an empty table when nothing is required.
        if kind.fields.iter().all(|f| !f.required) {
            (kind.parse)(toml::Table::new()).unwrap_or_else(|e| panic!("{}: {e}", kind.name));
        }
    }
    assert!(kinds::markdown().contains("| `engine` |"));
    let described = kinds::describe();
    assert_eq!(described.len(), kinds::ALL.len());
    assert_eq!(validate::catalogue().len(), total);
}

/// Every registered kind starts, reports the capabilities its entry claims,
/// and every check passes against the first instance of its kind.
#[tokio::test]
async fn every_registered_kind_starts_and_validate_passes() {
    let config = full_stack();
    let stack = config.start().await.unwrap();
    for kind in kinds::ALL {
        let instance = stack.get(&format!("{}-1", kind.name)).unwrap();
        assert_eq!(instance.kind, kind.name);
        assert_eq!(
            instance.fault().is_some(),
            kind.fault,
            "{}: fault",
            kind.name
        );
        assert_eq!(
            instance.requests().is_some(),
            kind.counters,
            "{}: counters",
            kind.name
        );
    }
    let targets = validate::Targets::of_stack(&stack, Duration::from_secs(5));
    assert_eq!(targets.urls.len(), kinds::with_target().count());
    let report = validate::run(targets).await;
    assert!(report.ok(), "validate failed:\n{}", report.render());
    assert_eq!(report.checks.len(), validate::checks().count());
    let load_targets = kinds::load_targets(&stack);
    assert_eq!(
        load_targets.len(),
        kinds::ALL.iter().filter(|k| k.load_target).count()
    );
    // The serialised form regroups by plural, as the file was written.
    let json = serde_json::to_value(&config).unwrap();
    for kind in kinds::ALL {
        assert!(
            json[kind.plural][format!("{}-1", kind.name)].is_object(),
            "{json}"
        );
    }
    stack.shutdown().await;
}

/// A fault set on any kind with fault injection makes that kind's checks
/// fail, and at most the checks of the kinds that depend on it.
#[tokio::test]
async fn set_behavior_on_any_fault_kind_fails_only_its_checks() {
    let stack = full_stack().start().await.unwrap();
    for kind in kinds::ALL
        .iter()
        .filter(|k| k.fault && !k.checks.is_empty())
    {
        // The kind and, transitively, whatever forwards to it.
        let mut affected = vec![kind.name];
        loop {
            let more: Vec<&str> = kinds::ALL
                .iter()
                .filter(|k| {
                    !affected.contains(&k.name)
                        && k.dependency().is_some_and(|d| affected.contains(&d.kind))
                })
                .map(|k| k.name)
                .collect();
            if more.is_empty() {
                break;
            }
            affected.extend(more);
        }
        let name = format!("{}-1", kind.name);
        stack
            .set_behavior(
                &name,
                Behavior::Error {
                    kind: ErrorKind::Unavailable,
                    rate: 1.0,
                    message: "injected".into(),
                },
            )
            .unwrap();
        let report =
            validate::run(validate::Targets::of_stack(&stack, Duration::from_secs(5))).await;
        let failed: Vec<&str> = report
            .checks
            .iter()
            .filter(|c| !c.passed)
            .map(|c| c.name.as_str())
            .collect();
        let allowed: Vec<&str> = validate::checks()
            .filter(|(k, _)| affected.contains(&k.name))
            .map(|(_, c)| c.name)
            .collect();
        let own: Vec<&str> = kind.checks.iter().map(|c| c.name).collect();
        assert!(
            failed.iter().any(|f| own.contains(f)),
            "{}: a fault must fail one of its own checks {own:?}, failed {failed:?}",
            kind.name
        );
        assert!(
            failed.iter().all(|f| allowed.contains(f)),
            "{}: failed {failed:?} beyond {affected:?}",
            kind.name
        );
        stack.set_behavior(&name, Behavior::Healthy).unwrap();
    }
    stack.shutdown().await;
}

#[tokio::test]
async fn stack_starts_in_dependency_order_and_validate_passes() {
    let stack = full_stack().start().await.unwrap();
    let engine = stack.get("engine-1").unwrap();

    let report = validate::run(validate::Targets::of_stack(&stack, Duration::from_secs(5))).await;
    assert!(report.ok(), "validate failed:\n{}", report.render());
    assert_eq!(report.checks.len(), validate::checks().count());

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
    assert!(
        dangling
            .check()
            .unwrap_err()
            .contains("references unknown engine")
    );
    let wrong_kind = topology("[stack.ledgers.l]\n[stack.protocols.p]\nengine = \"l\"\n");
    assert!(wrong_kind.check().unwrap_err().contains("not an engine"));
    let twice = try_topology("[stack.engines.x]\n[stack.ledgers.x]\n");
    assert!(twice.unwrap_err().to_string().contains("both"));
    let unknown = try_topology("[stack.widgets.w]\n");
    let msg = unknown.unwrap_err().to_string();
    assert!(
        msg.contains("stack.widgets") && msg.contains("engines"),
        "{msg}"
    );
    assert!(try_topology("[stack.engines.e]\nlisten = \"not an address\"\n").is_err());
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
        load.errors.get("http 500"),
        Some(&load.requests_failed),
        "INTERNAL maps to 500"
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

/// A fault set on a ledger instance surfaces through the ledger check and
/// nowhere else.
#[tokio::test]
async fn ledger_fault_surfaces_in_its_own_check() {
    let stack = full_stack().start().await.unwrap();
    let ledger = stack.get("ledger-1").unwrap();
    ledger.fault().unwrap().set(Behavior::Error {
        kind: ErrorKind::Unavailable,
        rate: 1.0,
        message: "injected".into(),
    });
    let report = validate::run(validate::Targets::of_stack(&stack, Duration::from_secs(5))).await;
    let failed: Vec<&str> = report
        .checks
        .iter()
        .filter(|c| !c.passed)
        .map(|c| c.name.as_str())
        .collect();
    let ledger_checks: Vec<&str> = kinds::by_name("ledger")
        .map(|k| k.checks.iter().map(|c| c.name).collect())
        .unwrap_or_default();
    assert_eq!(failed, ledger_checks, "every ledger check and nothing else");
    stack.shutdown().await;
}

/// A target with no URL fails its checks with a clear detail rather than
/// panicking or being skipped.
#[tokio::test]
async fn missing_target_url_is_a_failed_check() {
    let stack = full_stack().start().await.unwrap();
    let mut targets = validate::Targets::of_stack(&stack, Duration::from_secs(5));
    targets.urls.remove("ledger");
    let report = validate::run(targets).await;
    let ledger = report
        .checks
        .iter()
        .find(|c| c.name == "grpc_ledger_ping")
        .unwrap();
    assert!(!ledger.passed);
    assert!(
        ledger.detail.contains("no target URL for kind `ledger`"),
        "{}",
        ledger.detail
    );
    let ledger_checks = kinds::by_name("ledger").map_or(0, |k| k.checks.len());
    assert_eq!(report.failed, ledger_checks, "{}", report.render());
    stack.shutdown().await;
}

/// A scenario may `set_behavior` on any kind with fault injection, and not
/// on one without.
#[test]
fn scenario_set_behavior_follows_the_kind_capability() {
    let scenario = |service: &str| {
        format!(
            "[scenario]\nname = \"x\"\n{TWO_TIER}\n[[timeline]]\nat = \"1s\"\naction = \"set_behavior\"\nservice = \"{service}\"\n[timeline.behavior]\ntype = \"healthy\"\n"
        )
    };
    let ledger: tbd_chaos::scenario::ScenarioFile = toml::from_str(&scenario("ledger-1")).unwrap();
    ledger.check().unwrap();
    let protocol: tbd_chaos::scenario::ScenarioFile =
        toml::from_str(&scenario("protocol-1")).unwrap();
    assert!(
        protocol
            .check()
            .unwrap_err()
            .contains("without fault injection")
    );
    let no_load_target: tbd_chaos::scenario::ScenarioFile = toml::from_str(
        "[scenario]\nname = \"x\"\n[stack.engines.e]\n[load]\nrate = 1\nduration = \"1s\"\n",
    )
    .unwrap();
    assert!(
        no_load_target
            .check()
            .unwrap_err()
            .contains("load can target")
    );
}
