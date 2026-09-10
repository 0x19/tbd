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
