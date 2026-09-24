//! The MCP transport at `/mcp`: every public RPC as a tool, over streamable
//! HTTP, as a client speaks it on the wire (JSON-RPC over POST).

use serde_json::{Value, json};

use crate::support;

const PROTOCOL_VERSION: &str = "2025-11-25";

fn claims(sub: &str) -> String {
    use base64::Engine as _;
    base64::engine::general_purpose::URL_SAFE_NO_PAD
        .encode(json!({"sub": sub, "client_id": sub}).to_string())
}

/// One JSON-RPC request; the response body's JSON, whether the server
/// answered as JSON or as one server-sent event.
async fn rpc(stack: &support::Stack, method: &str, params: Value, caller: Option<&str>) -> Value {
    let mut request = stack
        .client()
        .post(stack.url("/mcp"))
        .header("content-type", "application/json")
        .header("accept", "application/json, text/event-stream")
        .header("mcp-protocol-version", PROTOCOL_VERSION)
        .json(&json!({"jsonrpc": "2.0", "id": 1, "method": method, "params": params}));
    if let Some(sub) = caller {
        request = request.header("x-jwt-payload", claims(sub));
    }
    let response = request.send().await.unwrap();
    let status = response.status();
    let text = response.text().await.unwrap();
    assert!(status.is_success(), "{status}: {text}");
    let body = text
        .lines()
        .filter_map(|l| l.strip_prefix("data:"))
        .next_back()
        .map_or(text.as_str(), str::trim);
    serde_json::from_str(body).unwrap_or_else(|e| panic!("{e}: {text}"))
}

/// A tool's answer: the JSON in its one text block, and whether it is an error.
fn answer(response: &Value) -> (Value, bool) {
    let result = &response["result"];
    let text = result["content"][0]["text"]
        .as_str()
        .unwrap_or_else(|| panic!("{response}"));
    (
        serde_json::from_str(text).unwrap(),
        result["isError"].as_bool().unwrap_or(false),
    )
}

#[tokio::test]
async fn initialize_names_the_platform_and_offers_tools() {
    let stack = support::start().await;
    let response = rpc(
        &stack,
        "initialize",
        json!({"protocolVersion": PROTOCOL_VERSION, "capabilities": {}, "clientInfo": {"name": "test", "version": "0"}}),
        Some("agent-1"),
    )
    .await;
    assert_eq!(
        response["result"]["serverInfo"]["name"], "tbd",
        "{response}"
    );
    assert!(
        response["result"]["capabilities"]["tools"].is_object(),
        "{response}"
    );
}

#[tokio::test]
async fn every_public_rpc_is_a_tool_with_a_description_and_an_object_schema() {
    let stack = support::start().await;
    let response = rpc(&stack, "tools/list", json!({}), Some("agent-1")).await;
    let tools = response["result"]["tools"]
        .as_array()
        .unwrap_or_else(|| panic!("{response}"));
    let names: Vec<&str> = tools.iter().filter_map(|t| t["name"].as_str()).collect();
    for want in [
        "ledger_ping",
        "engine_subscribe",
        "llm_generate",
        "llm_list_models",
    ] {
        assert!(names.contains(&want), "{want} missing from {names:?}");
    }
    for tool in tools {
        assert_eq!(tool["inputSchema"]["type"], "object", "{tool}");
        assert!(
            tool["description"].as_str().is_some_and(|d| !d.is_empty()),
            "{tool}"
        );
    }
    let generate = tools.iter().find(|t| t["name"] == "llm_generate").unwrap();
    assert!(
        generate["description"]
            .as_str()
            .unwrap()
            .contains("collects the stream"),
        "a streaming tool says it collects: {generate}"
    );
    assert!(
        generate["inputSchema"]["properties"]["messages"].is_object(),
        "{generate}"
    );
}

#[tokio::test]
async fn a_call_runs_the_rpc_as_the_caller() {
    let stack = support::start().await;
    let response = rpc(
        &stack,
        "tools/call",
        json!({"name": "ledger_ping", "arguments": {"message": "hi"}}),
        Some("agent-1"),
    )
    .await;
    let (value, error) = answer(&response);
    assert!(!error, "{response}");
    assert_eq!(value["message"], "hi");
}

#[tokio::test]
async fn a_call_without_a_verified_caller_is_refused() {
    let stack = support::start().await;
    let response = rpc(
        &stack,
        "tools/call",
        json!({"name": "ledger_ping", "arguments": {"message": "hi"}}),
        None,
    )
    .await;
    assert!(
        response["error"]["message"]
            .as_str()
            .is_some_and(|m| m.contains("unauthenticated")),
        "{response}"
    );
}

#[tokio::test]
async fn a_backend_that_is_down_is_an_error_the_agent_can_read() {
    let stack = support::start().await;
    let response = rpc(
        &stack,
        "tools/call",
        json!({"name": "llm_list_models", "arguments": {}}),
        Some("agent-1"),
    )
    .await;
    let (value, error) = answer(&response);
    assert!(error, "{response}");
    assert_eq!(value["code"], "unavailable", "{value}");
}

#[tokio::test]
async fn bad_arguments_are_refused_with_the_field() {
    let stack = support::start().await;
    let response = rpc(
        &stack,
        "tools/call",
        json!({"name": "ledger_ping", "arguments": {"nope": 1}}),
        Some("agent-1"),
    )
    .await;
    let (value, error) = answer(&response);
    assert!(error, "{response}");
    assert_eq!(value["code"], "bad_request", "{value}");
}

#[tokio::test]
async fn a_stream_is_collected_and_cut_at_the_cap() {
    let stack = support::start().await;
    let response = rpc(
        &stack,
        "tools/call",
        json!({"name": "engine_subscribe", "arguments": {"subject_id": "s-1"}}),
        Some("agent-1"),
    )
    .await;
    let (value, error) = answer(&response);
    assert!(!error, "{response}");
    assert_eq!(value["messages"], 3, "{value}");
    assert!(
        value["cut"].as_str().is_some_and(|c| c.contains("cap")),
        "{value}"
    );
    assert_eq!(value["items"].as_array().map(Vec::len), Some(3), "{value}");
}
