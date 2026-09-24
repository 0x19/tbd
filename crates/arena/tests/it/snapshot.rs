//! The snapshot: who may read it, what each source puts in it, and what an
//! absent source leaves out.

use std::time::Duration;

use futures::StreamExt as _;
use tbd_arena::collect::{chaos::SCHEDULE_NAME, metrics};
use tbd_proto::arena::v1::{GetSnapshotRequest, Snapshot, WatchRequest};
use tonic::Code;
use wiremock::{
    Mock, MockServer, ResponseTemplate,
    matchers::{method, path, query_param},
};

use crate::support;

/// Poll `GetSnapshot` as an admin until `ok` holds, or fail after five seconds.
async fn until(server: &support::Server, what: &str, ok: impl Fn(&Snapshot) -> bool) -> Snapshot {
    let mut client = server.client().await;
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        let snap = client
            .get_snapshot(support::as_role("admin", GetSnapshotRequest {}))
            .await
            .unwrap()
            .into_inner()
            .snapshot
            .unwrap();
        if ok(&snap) {
            return snap;
        }
        assert!(
            tokio::time::Instant::now() < deadline,
            "never saw {what}: {snap:?}"
        );
        tokio::time::sleep(Duration::from_millis(20)).await;
    }
}

#[tokio::test]
async fn the_first_frame_arrives_at_once_and_the_tiers_follow_the_model_service() {
    let server = support::start_live(|_| {}).await;
    let mut client = server.client().await;
    let mut stream = client
        .watch(support::as_role("admin", WatchRequest {}))
        .await
        .unwrap()
        .into_inner();
    let first = tokio::time::timeout(Duration::from_secs(1), stream.next())
        .await
        .expect("the current snapshot is sent at once")
        .unwrap()
        .unwrap();
    assert!(first.snapshot.unwrap().now.is_some());

    let snap = until(&server, "both tiers", |s| s.tiers.len() == 2).await;
    for t in &snap.tiers {
        assert!(t.stub, "the stub engine says so: {t:?}");
        assert_eq!(t.max_in_flight, 64, "the stub config's slots: {t:?}");
        assert_eq!(
            t.tokens_per_second, None,
            "no metrics store, no rate: absent, never zero"
        );
    }
    let llm = snap.sources.iter().find(|s| s.name == "llm").unwrap();
    assert!(llm.ok, "{llm:?}");
    let chaos = snap.sources.iter().find(|s| s.name == "chaos").unwrap();
    assert_eq!(chaos.error, "not configured");
    assert_eq!(snap.chaos.unwrap().state, "absent");
}

#[tokio::test]
async fn rates_come_from_the_metrics_store_and_an_empty_window_is_absent() {
    let store = MockServer::start().await;
    let vector = |tier: &str, v: &str| {
        serde_json::json!({"status": "success", "data": {"resultType": "vector",
            "result": [{"metric": {"tier": tier}, "value": [1.0, v]}]}})
    };
    for (q, body) in [
        (metrics::TOKENS_PER_SECOND, vector("fast", "12.5")),
        (metrics::TTFT_P50, vector("fast", "NaN")),
        (metrics::TTFT_P99, vector("fast", "NaN")),
        (metrics::REFUSED, vector("fast", "3")),
    ] {
        Mock::given(method("GET"))
            .and(path("/api/v1/query"))
            .and(query_param("query", q))
            .respond_with(ResponseTemplate::new(200).set_body_json(body))
            .mount(&store)
            .await;
    }
    let url = store.uri();
    let server = support::start_live(move |c| c.sources.metrics_url = url).await;
    let snap = until(&server, "the fast tier's rate", |s| {
        s.tiers
            .iter()
            .any(|t| t.tier == "fast" && t.tokens_per_second.is_some())
    })
    .await;
    let fast = snap.tiers.iter().find(|t| t.tier == "fast").unwrap();
    assert_eq!(fast.tokens_per_second, Some(12.5));
    assert_eq!(fast.refused_per_minute, Some(3.0));
    assert_eq!(
        fast.ttft_p50_ms, None,
        "a histogram with nothing in it is absent"
    );
    let deep = snap.tiers.iter().find(|t| t.tier == "deep").unwrap();
    assert_eq!(deep.tokens_per_second, None);
}

#[tokio::test]
async fn a_chaos_run_becomes_rates_and_its_last_check_becomes_the_surfaces() {
    let chaos = MockServer::start().await;
    Mock::given(method("GET"))
        .and(path("/overview"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "active_run": {"id": "r1", "kind": "load", "name": "burst"},
            "last_validate": {"id": "v1", "finished_at": "2026-09-24T10:00:00Z"},
        })))
        .mount(&chaos)
        .await;
    Mock::given(method("GET"))
        .and(path("/runs/v1"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "finished_at": "2026-09-24T10:00:00Z",
            "validate": {"passed": 2, "failed": 1, "checks": [
                {"name": "http_mcp_tools", "surface": "http", "passed": true, "latency_ms": 4.2, "detail": "tools=8"},
                {"name": "ws_mux", "surface": "ws", "passed": true, "latency_ms": 2.0, "detail": ""},
                {"name": "sse_events", "surface": "sse", "passed": false, "latency_ms": 9.0, "detail": "timed out"}
            ]},
        })))
        .mount(&chaos)
        .await;
    let feed = [
        ("started", serde_json::json!({"run": {"id": "r1"}})),
        ("phase", serde_json::json!({"id": "r1", "name": "run"})),
        ("load", serde_json::json!({"id": "r1", "snapshot": {"elapsed_s": 1.0, "requests_total": 10, "requests_failed": 0,
            "latency": {"p99_ms": 80.0}, "per_op": {"llm_generate": {"counters": {"completion_tokens": 100}}}}})),
        ("load", serde_json::json!({"id": "r1", "snapshot": {"elapsed_s": 2.0, "requests_total": 14, "requests_failed": 1,
            "latency": {"p99_ms": 95.0}, "per_op": {"llm_generate": {"counters": {"completion_tokens": 400}}}}})),
    ]
    .iter()
    .fold(String::new(), |mut out, (e, d)| {
        use std::fmt::Write as _;
        let _ = write!(out, "event: {e}\ndata: {d}\n\n");
        out
    });
    Mock::given(method("GET"))
        .and(path("/runs/r1/events"))
        .respond_with(
            ResponseTemplate::new(200)
                .insert_header("content-type", "text/event-stream")
                .set_body_string(feed),
        )
        .mount(&chaos)
        .await;
    Mock::given(method("GET"))
        .and(path("/schedules"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!([])))
        .mount(&chaos)
        .await;
    Mock::given(method("POST"))
        .and(path("/schedules"))
        .respond_with(ResponseTemplate::new(201).set_body_json(serde_json::json!({"id": "s1"})))
        .mount(&chaos)
        .await;

    let url = chaos.uri();
    let server = support::start_live(move |c| c.sources.chaos_url = url).await;
    let snap = until(&server, "the run's rates and the surfaces", |s| {
        s.chaos.as_ref().is_some_and(|c| c.rps > 0.0) && !s.surfaces.is_empty()
    })
    .await;
    let run = snap.chaos.unwrap();
    assert_eq!(
        (run.state.as_str(), run.name.as_str(), run.phase.as_str()),
        ("running", "burst", "run")
    );
    assert!((run.rps - 4.0).abs() < 1e-9, "{run:?}");
    assert!((run.error_rate - 0.25).abs() < 1e-9, "{run:?}");
    assert_eq!(run.tokens_per_second, Some(300.0));
    assert!((run.p99_ms - 95.0).abs() < 1e-9);

    assert_eq!(snap.mcp_tools, Some(8));
    let states: Vec<_> = snap
        .surfaces
        .iter()
        .map(|s| (s.name.as_str(), s.up))
        .collect();
    assert_eq!(
        states,
        vec![("sse", false), ("websocket", true), ("mcp", true)]
    );
    assert!(snap.surfaces_checked_at.is_some());

    // The schedule that checks every way in was created once, with Slack off.
    let posts: Vec<serde_json::Value> = chaos
        .received_requests()
        .await
        .unwrap()
        .iter()
        .filter(|r| r.method.as_str() == "POST")
        .map(|r| serde_json::from_slice(&r.body).unwrap())
        .collect();
    assert_eq!(posts.len(), 1, "{posts:?}");
    assert_eq!(posts[0]["name"], SCHEDULE_NAME);
    assert_eq!(posts[0]["notify"], "off");
    assert_eq!(posts[0]["job"], serde_json::json!({"validate": {}}));
}

#[tokio::test]
async fn only_the_required_role_may_read_it() {
    let server = support::start_live(|_| {}).await;
    let mut client = server.client().await;
    let anonymous = client
        .get_snapshot(GetSnapshotRequest {})
        .await
        .unwrap_err();
    assert_eq!(anonymous.code(), Code::Unauthenticated);
    let viewer = client
        .watch(support::as_role("viewer", WatchRequest {}))
        .await
        .unwrap_err();
    assert_eq!(viewer.code(), Code::PermissionDenied);

    let open = support::start_live(|c| c.watch.require_role = String::new()).await;
    let mut client = open.client().await;
    assert!(
        client.get_snapshot(GetSnapshotRequest {}).await.is_ok(),
        "an open arena needs no caller"
    );
}

#[tokio::test]
async fn one_viewer_too_many_is_refused_and_a_closed_one_frees_its_place() {
    let server = support::start_live(|c| c.watch.max_viewers = 1).await;
    let mut client = server.client().await;
    let first = client
        .watch(support::as_role("admin", WatchRequest {}))
        .await
        .unwrap()
        .into_inner();
    let busy = client
        .watch(support::as_role("admin", WatchRequest {}))
        .await
        .unwrap_err();
    assert_eq!(busy.code(), Code::ResourceExhausted);
    assert!(busy.message().starts_with("busy:"), "{busy:?}");

    drop(first);
    let deadline = tokio::time::Instant::now() + Duration::from_secs(5);
    loop {
        match client
            .watch(support::as_role("admin", WatchRequest {}))
            .await
        {
            Ok(_) => break,
            Err(e) if e.code() == Code::ResourceExhausted => {
                assert!(
                    tokio::time::Instant::now() < deadline,
                    "the place never came back"
                );
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
            Err(e) => panic!("{e:?}"),
        }
    }
}
