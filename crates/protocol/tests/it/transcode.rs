//! The transcoded surface: the ledger's RPCs over REST and the engine's
//! `Subscribe` over SSE, from the `google.api.http` annotations alone.

use std::time::Duration;

use futures::StreamExt;
use serde_json::{Value, json};

use crate::support;

const JWT: &str = "x-jwt-payload";

fn claims(sub: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(json!({"sub": sub, "client_id": sub}).to_string())
}

fn envelope(bytes: &[u8]) -> Value {
    use base64::Engine as _;
    json!({"version": 0, "bytes": base64::engine::general_purpose::STANDARD.encode(bytes)})
}

async fn body(response: reqwest::Response) -> (u16, Value) {
    let status = response.status().as_u16();
    let text = response.text().await.unwrap();
    let value = serde_json::from_str(&text).unwrap_or_else(|e| panic!("{status}: {text}: {e}"));
    (status, value)
}

#[tokio::test]
async fn ping_answers_json_with_the_stub_flag_present() {
    let stack = support::start().await;
    let (status, ping) = body(
        stack
            .client()
            .get(stack.url("/v1/ledger/ping?message=hi"))
            .header(JWT, claims("c1"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200, "{ping}");
    assert_eq!(ping["message"], "hi");
    assert_eq!(ping["stub"], false, "defaults are emitted: {ping}");
    assert_eq!(ping["store"], "memory");
    assert!(ping["version"].is_string());
}

#[tokio::test]
async fn the_ledger_round_trip_over_rest() {
    let stack = support::start().await;
    let client = stack.client();
    let subject = format!("/v1/ledger/subjects/{}", uuid::Uuid::new_v4());

    // Append: whole body, the path variable overrides subject_id.
    let (status, appended) = body(
        client
            .post(stack.url(&format!("{subject}/facts")))
            .header(JWT, claims("c1"))
            .json(&json!({
                "path": "profile.name",
                "source": "SOURCE_DECLARED",
                "consent": ["self"],
                "observed_at": "2026-01-01T00:00:00Z",
                "value": envelope(b"\"Ada\""),
                "origin": envelope(br#"{"by":"test"}"#),
            }))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200, "{appended}");
    let fact = &appended["fact"];
    assert!(
        fact["id"].is_string(),
        "64-bit ids travel as strings: {appended}"
    );
    assert_eq!(fact["path"], "profile.name");
    assert_eq!(fact["source"], "SOURCE_DECLARED");
    assert_eq!(fact["value"]["bytes"], envelope(b"\"Ada\"")["bytes"]);

    // Current: repeated query parameter.
    let (status, current) = body(
        client
            .get(stack.url(&format!("{subject}/facts?scopes=self&scopes=other")))
            .header(JWT, claims("c1"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200, "{current}");
    assert_eq!(
        current["facts"].as_array().map(Vec::len),
        Some(1),
        "{current}"
    );

    // History with a timestamp query parameter.
    let (status, history) = body(
        client
            .get(stack.url(&format!(
                "{subject}/history?scopes=self&at=2100-01-01T00:00:00Z"
            )))
            .header(JWT, claims("c1"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200, "{history}");
    assert!(history["facts"].is_array(), "{history}");

    // Retract the fact.
    let (status, retracted) = body(
        client
            .post(stack.url(&format!("{subject}/retractions")))
            .header(JWT, claims("c1"))
            .json(&json!({
                "path": "profile.name",
                "source": "SOURCE_DECLARED",
                "origin": envelope(br#"{"by":"test"}"#),
            }))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200, "{retracted}");
    assert!(retracted["tombstone"]["id"].is_string(), "{retracted}");

    // Erase: DELETE, no body.
    let (status, erased) = body(
        client
            .delete(stack.url(&subject))
            .header(JWT, claims("c1"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200, "{erased}");
    assert!(erased["executes_after"].is_string(), "{erased}");

    // Restore it within the grace period.
    let (status, restored) = body(
        client
            .post(stack.url(&format!("{subject}/restore")))
            .header(JWT, claims("c1"))
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 200, "{restored}");
}

#[tokio::test]
async fn a_registered_backend_that_is_down_is_503_unavailable() {
    let stack = support::start().await;
    let (status, problem) = body(
        stack
            .client()
            .get(stack.url("/v1/humans/ping"))
            .header(JWT, claims("c1"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 503, "{problem}");
    assert_eq!(problem["code"], "unavailable");
}

#[tokio::test]
async fn grpc_status_becomes_the_envelope() {
    let stack = support::start().await;
    let (status, problem) = body(
        stack
            .client()
            .post(stack.url(&format!(
                "/v1/ledger/subjects/{}/restore",
                uuid::Uuid::new_v4()
            )))
            .header(JWT, claims("c1"))
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 404, "{problem}");
    assert_eq!(problem["code"], "not_found");
    assert!(problem["details"].is_array());
}

#[tokio::test]
async fn bad_input_is_a_field_detail() {
    let stack = support::start().await;
    let client = stack.client();
    let facts = stack.url("/v1/ledger/subjects/s-1/facts");

    let field = |status: u16, problem: Value| -> String {
        assert_eq!(status, 400, "{problem}");
        assert_eq!(problem["code"], "bad_request", "{problem}");
        problem["details"][0]["field"]
            .as_str()
            .unwrap_or_else(|| panic!("{problem}"))
            .to_owned()
    };

    let (s, p) = body(
        client
            .get(format!("{facts}?nope=1"))
            .header(JWT, claims("c1"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(field(s, p), "nope");

    let (s, p) = body(
        client
            .get(stack.url("/v1/ledger/subjects/s-1/history?sources=NOPE"))
            .header(JWT, claims("c1"))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(field(s, p), "sources");

    let (s, p) = body(
        client
            .post(&facts)
            .header(JWT, claims("c1"))
            .json(&json!({"nope": 1}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(field(s, p), "body");

    let (s, p) = body(
        client
            .post(format!("{facts}?limit=1"))
            .header(JWT, claims("c1"))
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(field(s, p), "limit");

    // A body that is not JSON is 415, as on the hand-written routes.
    let (s, p) = body(
        client
            .post(&facts)
            .header(JWT, claims("c1"))
            .header("content-type", "text/plain")
            .body("{}")
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(s, 415, "{p}");
    assert_eq!(p["code"], "unsupported_media_type");
}

#[tokio::test]
async fn a_known_path_with_the_wrong_verb_is_a_405_envelope() {
    let stack = support::start().await;
    let (status, problem) = body(
        stack
            .client()
            .put(stack.url("/v1/ledger/subjects/s-1/facts"))
            .header(JWT, claims("c1"))
            .json(&json!({}))
            .send()
            .await
            .unwrap(),
    )
    .await;
    assert_eq!(status, 405, "{problem}");
    assert_eq!(problem["code"], "method_not_allowed");
    assert!(problem["error"].as_str().is_some_and(|m| m.contains("PUT")));
}

#[tokio::test]
async fn a_server_stream_is_served_as_sse() {
    let stack = support::start().await;
    let response = stack
        .client()
        .get(stack.url("/v1/engine/subjects/s1/events"))
        .header(JWT, claims("c1"))
        .send()
        .await
        .unwrap();
    assert_eq!(response.status(), 200);
    assert!(
        response
            .headers()
            .get("content-type")
            .and_then(|v| v.to_str().ok())
            .is_some_and(|ct| ct.starts_with("text/event-stream")),
        "{:?}",
        response.headers()
    );
    let mut stream = response.bytes_stream();
    let mut buffer = String::new();
    let first = tokio::time::timeout(Duration::from_secs(5), async {
        while let Some(chunk) = stream.next().await {
            buffer.push_str(&String::from_utf8_lossy(&chunk.unwrap()));
            if let Some(line) = buffer.lines().find(|l| l.starts_with("data:")) {
                return line.trim_start_matches("data:").trim().to_owned();
            }
        }
        panic!("stream ended: {buffer}");
    })
    .await
    .unwrap();
    let event: Value = serde_json::from_str(&first).unwrap_or_else(|e| panic!("{first}: {e}"));
    assert_eq!(event["subject_id"], "s1");
    assert!(
        event["heartbeat"]["seq"].is_string(),
        "64-bit sequence as a string: {event}"
    );
    assert!(event["id"].is_string(), "{event}");
}
