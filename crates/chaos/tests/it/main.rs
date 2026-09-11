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

/// A campaign runs through chaos's own glue: the stack from the file, the
/// workers on it, the report; and findings round-trip through their files.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn stress_campaign_runs_on_the_files_stack_and_findings_round_trip() {
    let file = tbd_chaos::stress::parse_campaign(
        "[campaign]\nname = \"glue\"\nduration = \"1s\"\nwarmup = \"100ms\"\nseed = 5\n\n[stack.ledgers.l]\ngrace = \"0s\"\n\n[workload.owner]\nworkers = 2\nsubjects = 2\n\n[[timeline]]\nat = \"300ms\"\naction = \"log\"\nmessage = \"half way\"\n",
    )
    .unwrap();
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel();
    let hooks = tbd_chaos::stress::Hooks {
        events: Some(tx),
        cancel: tokio_util::sync::CancellationToken::default(),
    };
    let result = tbd_chaos::stress::run_campaign_with(
        &file,
        &tbd_chaos::stress::RunOptions::default(),
        &hooks,
    )
    .await;
    drop(hooks);
    assert!(result.passed, "{}", tbd_stress::render(&result));
    assert_eq!(result.store.as_deref(), Some("memory"));
    assert_eq!(result.targets, vec!["l".to_owned()]);
    let mut phases = Vec::new();
    let mut timeline = 0;
    while let Ok(e) = rx.try_recv() {
        match e {
            tbd_chaos::stress::RunEvent::Phase { name } => phases.push(name),
            tbd_chaos::stress::RunEvent::Timeline { .. } => timeline += 1,
            _ => {}
        }
    }
    assert_eq!(phases, ["setup", "warmup", "run", "done", "teardown"]);
    assert_eq!(timeline, 1, "the log action fired");

    // Findings: written one file each, read back by id or path, and a replay
    // with no ledger target is an error, not a hang.
    let dir = std::env::temp_dir().join(format!("chaos-findings-{}", uuid::Uuid::now_v7()));
    let mut with_finding = result.clone();
    let violation = tbd_stress::trace::Violation {
        invariant: "append_echo",
        message: "made up".into(),
        expected: serde_json::json!("x"),
        actual: serde_json::json!("y"),
    };
    with_finding.findings.push(tbd_stress::Finding::new(
        &violation,
        vec![],
        uuid::Uuid::now_v7(),
        tbd_stress::WorkerClass::Owner,
        "glue",
        "l",
        Some("memory"),
    ));
    let written = tbd_chaos::stress::write_findings(&dir, &with_finding).unwrap();
    assert_eq!(written.len(), 1);
    let id = &with_finding.findings[0].id;
    let (path, f) = tbd_chaos::stress::read_finding(&dir, id).unwrap();
    assert_eq!(path, written[0]);
    assert_eq!(f.id, *id);
    let (_, f2) = tbd_chaos::stress::read_finding(&dir, &written[0].display().to_string()).unwrap();
    assert_eq!(f2.signature, f.signature);
    let mut f = f;
    let err = tbd_chaos::stress::replay_finding(
        &path,
        &mut f,
        &[],
        &tbd_chaos::tls::Trust::default(),
        1,
        Duration::from_secs(1),
    )
    .await
    .unwrap_err();
    assert!(err.contains("no ledger target"), "{err}");
    std::fs::remove_dir_all(&dir).unwrap();
}

/// A campaign may fault the store of a kind that has one and of no other, and
/// may not stop an instance whose store forgets on restart.
#[test]
fn campaign_timelines_follow_the_store_fault_capability() {
    let with_store = kinds::ALL
        .iter()
        .find(|k| k.store_fault)
        .expect("a kind with store faults");
    let without = kinds::ALL
        .iter()
        .find(|k| !k.store_fault)
        .expect("a kind without store faults");
    let campaign = |plural: &str, name: &str, action: &str| {
        format!(
            "[campaign]\nname = \"t\"\n[stack.{plural}.{name}]\n[[timeline]]\nat = \"1s\"\naction = \"{action}\"\nservice = \"{name}\"\nbehavior = {{ type = \"healthy\" }}\n"
        )
    };
    let ok = campaign(with_store.plural, "s-1", "set_store_behavior");
    let ok = if with_store.name == "ledger" {
        ok
    } else {
        // Another kind may need a dependency; only the ledger is asserted to parse.
        campaign("ledgers", "s-1", "set_store_behavior")
    };
    tbd_chaos::stress::parse_campaign(&ok).unwrap();
    if without.fields.is_empty() {
        let e = tbd_chaos::stress::parse_campaign(&campaign(
            without.plural,
            "w-1",
            "set_store_behavior",
        ))
        .unwrap_err();
        assert!(e.contains("no store to fail"), "{e}");
    }
    let e = tbd_chaos::stress::parse_campaign(
        "[campaign]\nname = \"t\"\n[stack.ledgers.l]\n[[timeline]]\nat = \"1s\"\naction = \"stop\"\nservice = \"l\"\n",
    )
    .unwrap_err();
    assert!(e.contains("forgets on restart"), "{e}");
}

/// A store fault behaves like a database outage: the write commits and loses
/// its acknowledgement, the read fails, and healthy again the fact is there.
#[tokio::test]
async fn store_faults_lose_acknowledgements_but_keep_commits() {
    let stack = topology("[stack.ledgers.l]\ngrace = \"0s\"\n")
        .start()
        .await
        .unwrap();
    let addr = stack.get("l").unwrap().addr;
    let mut client = tbd_proto::ledger::v1::ledger_service_client::LedgerServiceClient::connect(
        format!("http://{addr}"),
    )
    .await
    .unwrap();
    let subject = uuid::Uuid::now_v7().to_string();
    let envelope = |v: serde_json::Value| {
        Some(tbd_proto::ledger::v1::Envelope {
            version: 0,
            bytes: serde_json::to_vec(&v).unwrap(),
        })
    };
    let append = tbd_proto::ledger::v1::AppendRequest {
        subject_id: subject.clone(),
        path: "profile.name".into(),
        source: 2,
        value: envelope(serde_json::json!("Ada")),
        origin: envelope(serde_json::json!({})),
        confidence: Some(1.0),
        counterparty_id: None,
        observed_at: Some(prost_types::Timestamp::from(std::time::SystemTime::now())),
        expires_at: None,
        consent: vec!["self".into()],
        stub: false,
        idempotency_key: "k-lost".into(),
    };
    let outage = Behavior::Error {
        kind: ErrorKind::Unavailable,
        rate: 1.0,
        message: "db down".into(),
    };
    stack.set_store_behavior("l", outage).unwrap();
    assert_eq!(
        stack.describe()[0]
            .store_behavior
            .as_ref()
            .map(|b| matches!(b, Behavior::Error { .. })),
        Some(true)
    );
    let lost = client.append(append.clone()).await.unwrap_err();
    assert_eq!(lost.code(), tonic::Code::Unavailable, "{lost}");
    let read = tbd_proto::ledger::v1::HistoryRequest {
        subject_id: subject.clone(),
        paths: vec![],
        sources: vec![],
        scopes: vec!["self".into()],
        cursor: String::new(),
        limit: 0,
        at: None,
    };
    assert_eq!(
        client.history(read.clone()).await.unwrap_err().code(),
        tonic::Code::Unavailable
    );
    stack.set_store_behavior("l", Behavior::Healthy).unwrap();
    // The commit stood: the key replays it and history holds exactly one row.
    let again = client.append(append).await.unwrap().into_inner();
    assert!(again.replayed);
    assert_eq!(
        client.history(read).await.unwrap().into_inner().facts.len(),
        1
    );
    assert!(matches!(
        stack.set_store_behavior("nope", Behavior::Healthy),
        Err(tbd_chaos::stack::StackError::Unknown(_))
    ));
    stack.shutdown().await;
}
