//! The runner's contract on the stub engine, and its engine against a wiremock
//! playing the sandbox daemon: who may run, the bounds, the allowance, the
//! queue, the audit line, and how the daemon's answers map.

use std::{
    io::Write,
    sync::{Arc, Mutex},
};

use tbd_proto::runner::v1::{Language, ListLanguagesRequest, RunRequest};
use tbd_runner::{Runtime, config::EngineKind};
use tonic::{Code, Request};
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{header, method, path},
};

use crate::support::{self, as_role};

fn go(source: &str) -> RunRequest {
    RunRequest {
        language: Language::Go as i32,
        source: source.into(),
        stdin: String::new(),
    }
}

#[tokio::test]
async fn a_run_needs_a_verified_caller_with_the_role() {
    let s = support::start().await;
    let mut c = s.client().await;
    let none = c.run(Request::new(go("package main"))).await.unwrap_err();
    assert_eq!(none.code(), Code::Unauthenticated);
    let viewer = c
        .run(as_role("p1", "viewer", go("package main")))
        .await
        .unwrap_err();
    assert_eq!(viewer.code(), Code::PermissionDenied);
    let ok = c
        .run(as_role("p1", "admin", go("package main")))
        .await
        .unwrap()
        .into_inner();
    assert!(ok.stub, "the stub says so on the wire");
    assert_eq!(ok.outcome, "ok");
    assert_eq!(ok.runs_left_today, 199);
}

#[tokio::test]
async fn bounds_are_refused_with_the_field_before_anything_runs() {
    let s = support::start_on(Runtime::default(), |c| c.limits.max_source_bytes = 16).await;
    let mut c = s.client().await;
    for (req, field) in [
        (go("   "), "source"),
        (go(&"x".repeat(17)), "source"),
        (
            RunRequest {
                language: Language::Unspecified as i32,
                source: "x".into(),
                stdin: String::new(),
            },
            "language",
        ),
    ] {
        let e = c.run(as_role("p1", "admin", req)).await.unwrap_err();
        assert_eq!(e.code(), Code::InvalidArgument);
        assert!(e.message().starts_with(field), "{e:?}");
    }
    let left = c
        .list_languages(as_role("p1", "admin", ListLanguagesRequest {}))
        .await
        .unwrap()
        .into_inner()
        .runs_left_today;
    assert_eq!(left, Some(200), "a refused request spends nothing");
}

#[tokio::test]
async fn the_daily_allowance_runs_out_with_the_reset_time() {
    let s = support::start_on(Runtime::default(), |c| c.budget.runs_per_day = 1).await;
    let mut c = s.client().await;
    assert!(
        c.run(as_role("p1", "admin", go("package main")))
            .await
            .is_ok()
    );
    let e = c
        .run(as_role("p1", "admin", go("package main")))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::ResourceExhausted);
    assert!(e.message().contains("resets at 00:00 UTC"), "{e:?}");
    assert!(
        c.run(as_role("p2", "admin", go("package main")))
            .await
            .is_ok(),
        "per caller"
    );
}

#[tokio::test]
async fn a_full_queue_is_refused_at_once_as_busy() {
    let s = support::start_on(Runtime::default(), |c| {
        c.admission.max_in_flight = 1;
        c.admission.max_queued = 0;
    })
    .await;
    let mut held = s.client().await;
    let first = tokio::spawn(async move { held.run(as_role("p1", "admin", go("<<hang>>"))).await });
    tokio::time::sleep(std::time::Duration::from_millis(200)).await;
    let mut c = s.client().await;
    let started = std::time::Instant::now();
    let e = c
        .run(as_role("p2", "admin", go("package main")))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::ResourceExhausted);
    assert!(e.message().starts_with("busy:"), "{e:?}");
    assert!(started.elapsed().as_millis() < 500, "refused at once");
    first.abort();
}

#[tokio::test]
async fn the_engine_down_is_unavailable_and_says_nothing_more() {
    let s = support::start().await;
    let mut c = s.client().await;
    let e = c
        .run(as_role("p1", "admin", go("<<down>>")))
        .await
        .unwrap_err();
    assert_eq!(e.code(), Code::Unavailable);
    assert!(!e.message().contains("stub"), "{e:?}");
}

/// A privacy claim is a test: the audit line names the run, never its code.
#[tokio::test]
async fn the_audit_line_never_carries_the_source() {
    #[derive(Clone, Default)]
    struct Buf(Arc<Mutex<Vec<u8>>>);
    impl Write for Buf {
        fn write(&mut self, b: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(b);
            Ok(b.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    let buf = Buf::default();
    let writer = buf.clone();
    let subscriber = tracing_subscriber::fmt()
        .with_writer(move || writer.clone())
        .with_max_level(tracing::Level::INFO)
        .finish();
    let _guard = tracing::subscriber::set_default(subscriber);

    let s = support::start().await;
    let mut c = s.client().await;
    let secret = "package main // SECRET-CODE-4d1f";
    c.run(as_role("p1", "admin", go(secret))).await.unwrap();
    let log = String::from_utf8(buf.0.lock().unwrap().clone()).unwrap();
    assert!(log.contains("code run"), "the run is audited: {log}");
    assert!(log.contains("source_bytes"), "{log}");
    assert!(
        !log.contains("SECRET-CODE"),
        "the source never reaches a log: {log}"
    );
}

#[tokio::test]
async fn the_sandbox_daemon_is_called_with_the_token_and_its_answers_mapped() {
    let daemon = MockServer::start().await;
    Mock::given(method("POST"))
        .and(path("/run"))
        .and(header("authorization", "Bearer the-token"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "id": "r1", "language": "go", "outcome": "ok", "total_ms": 1200,
            "compile": {"exit_code": 0, "stdout": "", "stderr": "", "truncated": false, "wall_ms": 900, "killed": "none"},
            "run": {"exit_code": 0, "stdout": "hi\n", "stderr": "", "truncated": false, "wall_ms": 50, "killed": "none"}
        })))
        .mount(&daemon)
        .await;
    let url = daemon.uri();
    let s = support::start_on(Runtime::default(), move |c| {
        c.engine.kind = EngineKind::Sandboxd;
        c.engine.url = url;
        c.sandbox_token = Some("the-token".into());
    })
    .await;
    let mut c = s.client().await;
    let r = c
        .run(as_role("p1", "admin", go("package main")))
        .await
        .unwrap()
        .into_inner();
    assert!(!r.stub);
    assert_eq!(r.run.unwrap().stdout, "hi\n");
    assert_eq!(r.compile.unwrap().wall_ms, 900);

    for (status, code) in [
        (429, Code::ResourceExhausted),
        (401, Code::Unavailable),
        (503, Code::Unavailable),
    ] {
        daemon.reset().await;
        Mock::given(method("POST"))
            .and(path("/run"))
            .respond_with(
                ResponseTemplate::new(status)
                    .set_body_json(serde_json::json!({"code": "x", "error": "why"})),
            )
            .mount(&daemon)
            .await;
        let e = c
            .run(as_role("p1", "admin", go("package main")))
            .await
            .unwrap_err();
        assert_eq!(e.code(), code, "daemon {status}: {e:?}");
        if status == 401 {
            assert!(
                !e.message().contains("token"),
                "an operator's problem is not told to the caller: {e:?}"
            );
        }
    }
}
