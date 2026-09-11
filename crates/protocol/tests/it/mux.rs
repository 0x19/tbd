//! The multiplexed WebSocket at `/v1/ws`: unary calls, streams, cancellation,
//! and the refusals a client has to be able to tell apart.

use std::time::Duration;

use futures::{SinkExt, StreamExt};
use serde_json::{Value, json};
use tokio::net::TcpStream;
use tokio_tungstenite::{MaybeTlsStream, WebSocketStream, tungstenite::Message};

use crate::support;

type Socket = WebSocketStream<MaybeTlsStream<TcpStream>>;

const PING: &str = "tbd.ledger.v1.LedgerService/Ping";
const SUBSCRIBE: &str = "tbd.engine.v1.EngineService/Subscribe";

async fn connect(stack: &support::Stack) -> Socket {
    tokio_tungstenite::connect_async(stack.ws_url("/v1/ws"))
        .await
        .unwrap()
        .0
}

async fn send(ws: &mut Socket, frame: &Value) {
    ws.send(Message::Text(frame.to_string().into()))
        .await
        .unwrap();
}

/// The next JSON frame, or a panic when the socket goes quiet.
async fn next(ws: &mut Socket) -> Value {
    loop {
        let message = tokio::time::timeout(Duration::from_secs(5), ws.next())
            .await
            .expect("the socket went quiet")
            .expect("the socket closed")
            .unwrap();
        match message {
            Message::Text(text) => return serde_json::from_str(&text).unwrap(),
            Message::Close(frame) => panic!("closed: {frame:?}"),
            // Ping, pong and continuation frames are not ours to read.
            _ => {}
        }
    }
}

#[tokio::test]
async fn a_unary_rpc_answers_with_data_then_end() {
    let stack = support::start().await;
    let mut ws = connect(&stack).await;
    send(
        &mut ws,
        &json!({"type": "call", "id": "1", "method": PING, "body": {"message": "hi"}}),
    )
    .await;

    let data = next(&mut ws).await;
    assert_eq!(data["type"], "data", "{data}");
    assert_eq!(data["id"], "1");
    assert_eq!(data["body"]["message"], "hi");
    assert_eq!(data["body"]["stub"], false, "stubs stay labelled: {data}");
    assert_eq!(data["body"]["store"], "memory");

    let end = next(&mut ws).await;
    assert_eq!(end["type"], "end", "{end}");
    assert_eq!(end["id"], "1");
}

#[tokio::test]
async fn a_call_needs_no_body_and_ids_are_free_again_after_end() {
    let stack = support::start().await;
    let mut ws = connect(&stack).await;
    for _ in 0..2 {
        send(
            &mut ws,
            &json!({"type": "call", "id": "same", "method": PING}),
        )
        .await;
        let data = next(&mut ws).await;
        assert_eq!(data["type"], "data", "{data}");
        assert_eq!(data["body"]["message"], "", "an empty request message");
        assert_eq!(next(&mut ws).await["type"], "end");
    }
}

#[tokio::test]
async fn a_server_stream_runs_until_the_client_cancels_it() {
    let stack = support::start().await;
    let mut ws = connect(&stack).await;
    send(
        &mut ws,
        &json!({"type": "call", "id": "s", "method": SUBSCRIBE, "body": {"subject_id": "s1"}}),
    )
    .await;

    for _ in 0..2 {
        let data = next(&mut ws).await;
        assert_eq!(data["type"], "data", "{data}");
        assert_eq!(data["id"], "s");
        assert_eq!(data["body"]["subject_id"], "s1");
    }

    send(&mut ws, &json!({"type": "cancel", "id": "s"})).await;
    let end = loop {
        let frame = next(&mut ws).await;
        if frame["type"] != "data" {
            break frame;
        }
    };
    assert_eq!(
        end["type"], "end",
        "a cancelled call ends like any other: {end}"
    );
    assert_eq!(end["id"], "s");

    // The socket is still usable, and the id is free.
    send(&mut ws, &json!({"type": "call", "id": "s", "method": PING})).await;
    assert_eq!(next(&mut ws).await["type"], "data");
}

#[tokio::test]
async fn calls_are_multiplexed_over_one_socket() {
    let stack = support::start().await;
    let mut ws = connect(&stack).await;
    send(
        &mut ws,
        &json!({"type": "call", "id": "stream", "method": SUBSCRIBE, "body": {"subject_id": "s1"}}),
    )
    .await;
    send(
        &mut ws,
        &json!({"type": "call", "id": "unary", "method": PING, "body": {"message": "beside it"}}),
    )
    .await;

    let mut unary_answered = false;
    let mut stream_answered = false;
    for _ in 0..10 {
        let frame = next(&mut ws).await;
        match (frame["id"].as_str(), frame["type"].as_str()) {
            (Some("unary"), Some("data")) => {
                assert_eq!(frame["body"]["message"], "beside it");
                unary_answered = true;
            }
            (Some("stream"), Some("data")) => stream_answered = true,
            (Some("unary"), Some("end")) => {}
            other => panic!("unexpected frame {other:?}: {frame}"),
        }
        if unary_answered && stream_answered {
            break;
        }
    }
    assert!(
        unary_answered && stream_answered,
        "the unary call answered while the stream was open"
    );
}

#[tokio::test]
async fn refusals_name_the_call_and_leave_the_socket_open() {
    let stack = support::start().await;
    let mut ws = connect(&stack).await;

    // Not a public RPC.
    send(
        &mut ws,
        &json!({"type": "call", "id": "a", "method": "tbd.ledger.v1.LedgerService/Nope"}),
    )
    .await;
    let error = next(&mut ws).await;
    assert_eq!(error["type"], "error", "{error}");
    assert_eq!(error["id"], "a");
    assert_eq!(error["code"], "not_found");

    // An unannotated RPC is not public either.
    send(
        &mut ws,
        &json!({"type": "call", "id": "b", "method": "tbd.engine.v1.EngineService/Evaluate"}),
    )
    .await;
    assert_eq!(next(&mut ws).await["code"], "not_found");

    // A body the request message does not have.
    send(
        &mut ws,
        &json!({"type": "call", "id": "c", "method": PING, "body": {"nope": 1}}),
    )
    .await;
    let error = next(&mut ws).await;
    assert_eq!(error["code"], "bad_request", "{error}");
    assert_eq!(error["details"][0]["field"], "body");

    // A frame that is not a frame: no call to name.
    ws.send(Message::Text("{oops".into())).await.unwrap();
    let error = next(&mut ws).await;
    assert_eq!(error["code"], "bad_request", "{error}");
    assert!(error.get("id").is_none(), "no call to blame: {error}");

    // Cancelling something that is not running.
    send(&mut ws, &json!({"type": "cancel", "id": "ghost"})).await;
    let error = next(&mut ws).await;
    assert_eq!(error["code"], "not_found", "{error}");
    assert_eq!(error["id"], "ghost");

    // After all that the socket still serves calls.
    send(&mut ws, &json!({"type": "call", "id": "d", "method": PING})).await;
    assert_eq!(next(&mut ws).await["type"], "data");
}

#[tokio::test]
async fn an_id_in_flight_cannot_be_reused() {
    let stack = support::start().await;
    let mut ws = connect(&stack).await;
    send(
        &mut ws,
        &json!({"type": "call", "id": "x", "method": SUBSCRIBE, "body": {"subject_id": "s1"}}),
    )
    .await;
    send(
        &mut ws,
        &json!({"type": "call", "id": "x", "method": SUBSCRIBE, "body": {"subject_id": "s1"}}),
    )
    .await;
    let refusal = loop {
        let frame = next(&mut ws).await;
        if frame["type"] == "error" {
            break frame;
        }
    };
    assert_eq!(refusal["code"], "bad_request", "{refusal}");
    assert_eq!(refusal["id"], "x");
}

#[tokio::test]
async fn a_backend_that_is_down_fails_the_call_not_the_socket() {
    let stack = support::start().await;
    let mut ws = connect(&stack).await;
    send(
        &mut ws,
        &json!({"type": "call", "id": "h", "method": "tbd.humans.v1.HumansService/Ping"}),
    )
    .await;
    let error = next(&mut ws).await;
    assert_eq!(error["type"], "error", "{error}");
    assert_eq!(error["id"], "h");
    assert_eq!(error["code"], "unavailable");

    send(
        &mut ws,
        &json!({"type": "call", "id": "ok", "method": PING}),
    )
    .await;
    assert_eq!(next(&mut ws).await["type"], "data");
}
