//! WebSocket bridged to an engine `Session` bidirectional stream.
//!
//! Client → protocol: text frames become `SessionFrame.data`.
//! Engine → protocol → client: every frame is sent as a JSON envelope.

use axum::{
    Router,
    extract::{
        State, WebSocketUpgrade,
        ws::{Message, WebSocket},
    },
    response::Response,
    routing::get,
};
use futures::{SinkExt, StreamExt};
use serde::Serialize;
use tbd_proto::engine::v1::{Close, Heartbeat, SessionRequest, session_request, session_response};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::AppState;

pub fn routes() -> Router<AppState> {
    Router::new().route("/ws", get(upgrade))
}

/// JSON envelope sent to WebSocket clients for every engine frame.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WsFrame<'a> {
    /// Echoed or produced data. `data` is UTF-8 text or lossily decoded.
    Data {
        /// Session identifier.
        session_id: &'a str,
        /// Sequence number from the engine.
        seq: u64,
        /// Payload.
        data: String,
    },
    /// Engine heartbeat.
    Heartbeat {
        /// Sequence number.
        seq: u64,
    },
    /// The engine closed the session.
    Close {
        /// Why.
        reason: &'a str,
    },
    /// The bridge failed.
    Error {
        /// Message.
        message: String,
    },
}

async fn upgrade(ws: WebSocketUpgrade, State(state): State<AppState>) -> Response {
    ws.on_upgrade(move |socket| bridge(socket, state))
}

async fn bridge(socket: WebSocket, state: AppState) {
    let session_id = uuid::Uuid::now_v7().to_string();
    let (mut ws_tx, mut ws_rx) = socket.split();
    let (to_engine, from_ws) = mpsc::channel::<SessionRequest>(32);

    let mut from_engine = match state.engine().session(ReceiverStream::new(from_ws)).await {
        Ok(resp) => resp.into_inner(),
        Err(status) => {
            let msg = WsFrame::Error {
                message: status.to_string(),
            };
            let _ = ws_tx.send(Message::Text(json(&msg).into())).await;
            let _ = ws_tx.close().await;
            return;
        }
    };
    tracing::info!(%session_id, "ws session opened");

    let mut seq = 0_u64;
    loop {
        tokio::select! {
            inbound = ws_rx.next() => match inbound {
                Some(Ok(Message::Text(text))) => {
                    seq += 1;
                    let frame = SessionRequest {
                        session_id: session_id.clone(),
                        seq,
                        body: Some(session_request::Body::Data(text.as_bytes().to_vec())),
                    };
                    if to_engine.send(frame).await.is_err() { break; }
                }
                Some(Ok(Message::Binary(bytes))) => {
                    seq += 1;
                    let frame = SessionRequest {
                        session_id: session_id.clone(),
                        seq,
                        body: Some(session_request::Body::Data(bytes.to_vec())),
                    };
                    if to_engine.send(frame).await.is_err() { break; }
                }
                Some(Ok(Message::Close(_))) | None => {
                    seq += 1;
                    let _ = to_engine
                        .send(SessionRequest {
                            session_id: session_id.clone(),
                            seq,
                            body: Some(session_request::Body::Close(Close {
                                reason: "client closed".into(),
                            })),
                        })
                        .await;
                    break;
                }
                Some(Ok(Message::Ping(_) | Message::Pong(_))) => {}
                Some(Err(error)) => {
                    tracing::debug!(%session_id, %error, "ws receive error");
                    break;
                }
            },
            outbound = from_engine.next() => match outbound {
                Some(Ok(frame)) => {
                    let msg = match frame.body {
                        Some(session_response::Body::Data(data)) => WsFrame::Data {
                            session_id: &frame.session_id,
                            seq: frame.seq,
                            data: String::from_utf8_lossy(&data).into_owned(),
                        },
                        Some(session_response::Body::Heartbeat(Heartbeat { seq })) => WsFrame::Heartbeat { seq },
                        Some(session_response::Body::Close(Close { ref reason })) => WsFrame::Close { reason },
                        None => continue,
                    };
                    if ws_tx.send(Message::Text(json(&msg).into())).await.is_err() { break; }
                }
                Some(Err(status)) => {
                    let msg = WsFrame::Error { message: status.to_string() };
                    let _ = ws_tx.send(Message::Text(json(&msg).into())).await;
                    break;
                }
                None => break,
            },
        }
    }

    let _ = ws_tx.close().await;
    tracing::info!(%session_id, "ws session closed");
}

fn json(frame: &WsFrame<'_>) -> String {
    // Serialising a struct of strings and integers cannot fail.
    serde_json::to_string(frame)
        .unwrap_or_else(|_| r#"{"type":"error","message":"serialize"}"#.to_owned())
}
