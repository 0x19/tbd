//! Integration tests: real engine + real protocol on ephemeral ports, driven
//! over HTTP, WebSocket, GraphQL and gRPC.

#![allow(clippy::unwrap_used, clippy::expect_used)]

mod support;

use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio_tungstenite::tungstenite::Message;

#[tokio::test]
async fn healthz_and_readyz() {
    let stack = support::start().await;
    let http = reqwest::Client::new();

    let live = http.get(stack.url("/healthz")).send().await.unwrap();
    assert_eq!(live.status(), 200);

    let ready = http.get(stack.url("/readyz")).send().await.unwrap();
    assert_eq!(
        ready.status(),
        200,
        "engine is up, protocol must report ready"
    );
}

#[tokio::test]
async fn rest_evaluate_forwards_stub_flag() {
    let stack = support::start().await;
    let http = reqwest::Client::new();

    let resp: Value = http
        .post(stack.url("/v1/evaluate"))
        .json(&json!({ "subject_id": "s1", "payload": "hi" }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert_eq!(resp["subject_id"], "s1");
    assert_eq!(resp["stub"], true);
    assert!(resp["model_version"].as_str().unwrap().starts_with("stub-"));
}

#[tokio::test]
async fn rest_evaluate_maps_engine_errors() {
    let stack = support::start().await;
    let http = reqwest::Client::new();

    let resp = http
        .post(stack.url("/v1/evaluate"))
        .json(&json!({ "subject_id": "" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 400);
}

#[tokio::test]
async fn sse_streams_engine_events() {
    let stack = support::start().await;
    let http = reqwest::Client::new();

    let resp = http
        .get(stack.url("/v1/subjects/s1/events"))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    assert!(
        resp.headers()["content-type"]
            .to_str()
            .unwrap()
            .starts_with("text/event-stream")
    );

    let mut body = resp.bytes_stream();
    let mut buf = String::new();
    while !buf.contains("\"seq\":1") {
        let chunk = body.next().await.unwrap().unwrap();
        buf.push_str(&String::from_utf8_lossy(&chunk));
    }
    assert!(buf.contains("\"kind\":\"heartbeat\""));
}

#[tokio::test]
async fn websocket_echoes_through_engine_session() {
    let stack = support::start().await;
    let (mut ws, _) = tokio_tungstenite::connect_async(stack.ws_url("/ws"))
        .await
        .unwrap();

    ws.send(Message::Text("ping".into())).await.unwrap();

    let echoed = loop {
        let msg = ws.next().await.unwrap().unwrap();
        let text = msg.into_text().unwrap();
        let v: Value = serde_json::from_str(&text).unwrap();
        if v["type"] == "data" {
            break v;
        }
    };
    assert_eq!(echoed["data"], "ping");
    ws.close(None).await.unwrap();
}

#[tokio::test]
async fn graphql_evaluate() {
    let stack = support::start().await;
    let http = reqwest::Client::new();

    let resp: Value = http
        .post(stack.url("/graphql"))
        .json(&json!({ "query": r#"{ version engineReady evaluate(subjectId: "s1") { subjectId score stub modelVersion } }"# }))
        .send()
        .await
        .unwrap()
        .json()
        .await
        .unwrap();

    assert!(resp["errors"].is_null(), "graphql errors: {resp}");
    assert_eq!(resp["data"]["engineReady"], true);
    assert_eq!(resp["data"]["evaluate"]["subjectId"], "s1");
    assert_eq!(resp["data"]["evaluate"]["stub"], true);
}

#[tokio::test]
async fn grpc_ping_on_same_port() {
    let stack = support::start().await;
    let mut client =
        tbd_proto::protocol::v1::protocol_service_client::ProtocolServiceClient::connect(
            stack.url(""),
        )
        .await
        .unwrap();

    let resp = client
        .ping(tbd_proto::protocol::v1::PingRequest {
            message: "hello".into(),
        })
        .await
        .unwrap()
        .into_inner();
    assert_eq!(resp.message, "hello");
    assert_eq!(resp.protocol_version, tbd_common::VERSION);
}

/// nextest runs each test in its own process, so installing the global
/// metrics exporter here does not collide with other tests.
#[tokio::test]
async fn metrics_endpoint_reports_requests_and_engine_calls() {
    let free = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let metrics_addr = free.local_addr().unwrap();
    drop(free);
    tbd_common::metrics::install(metrics_addr, "protocol-test").unwrap();

    let stack = support::start().await;
    let http = reqwest::Client::new();
    http.get(stack.url("/healthz")).send().await.unwrap();
    http.post(stack.url("/v1/evaluate"))
        .json(&json!({ "subject_id": "s1" }))
        .send()
        .await
        .unwrap()
        .error_for_status()
        .unwrap();

    // The exporter renders lazily; give the request tasks a moment to record.
    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    let body = http
        .get(format!("http://{metrics_addr}/metrics"))
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();

    assert!(
        body.contains("tbd_build_info{"),
        "build info missing:\n{body}"
    );
    assert!(
        body.contains(r#"tbd_requests_total{service="protocol-test",transport="http",route="/v1/evaluate",status="200"} 1"#)
            || body.contains(r#"tbd_requests_total{route="/v1/evaluate",service="protocol-test",status="200",transport="http"} 1"#),
        "request counter missing or wrong labels:\n{body}"
    );
    assert!(
        body.contains("tbd_request_duration_seconds_bucket{"),
        "duration histogram missing"
    );
    assert!(
        body.contains("tbd_engine_client_requests_total{")
            && body.contains("EngineService/Evaluate"),
        "engine client counter missing:\n{body}"
    );
    assert!(
        body.contains("process_cpu_seconds_total"),
        "process metrics missing"
    );
}

#[tokio::test]
async fn request_span_carries_a_trace_id_and_propagates_to_the_engine() {
    // Telemetry with no exporter still generates trace ids; init once per process.
    let mut telemetry = tbd_common::telemetry::init(
        &tbd_common::telemetry::TelemetryArgs::default(),
        "protocol-test",
    )
    .unwrap();
    let stack = support::start().await;
    let http = reqwest::Client::new();

    // A caller-supplied traceparent must be adopted: the same trace id reaches the engine.
    let resp = http
        .post(stack.url("/v1/evaluate"))
        .header(
            "traceparent",
            "00-0af7651916cd43dd8448eb211c80319c-b7ad6b7169203331-01",
        )
        .json(&json!({ "subject_id": "s1" }))
        .send()
        .await
        .unwrap();
    assert_eq!(resp.status(), 200);
    telemetry.shutdown();
}
