//! The multiplexed WebSocket: every public RPC over one socket.
//!
//! `/v1/ws` carries the same calls as the REST surface, addressed by RPC name
//! instead of by path. A `call` frame starts one, `data` frames carry the
//! answers, `end` closes it and `error` is the envelope every other surface
//! returns. Calls are independent: the `id` the client picks multiplexes them,
//! so one connection serves many unary calls and many open streams at once,
//! and a client needs one socket rather than a request per call and a stream
//! per subscription.
//!
//! The callable set is the annotated one: an RPC is here when it carries
//! `google.api.http` and its backend is registered, so this surface and the
//! REST routes cannot drift apart. The whole request message is the frame's
//! `body`; path and query rules belong to HTTP and do not apply.
//!
//! One task per call, one loop per connection. The loop owns the socket and
//! the table of calls in flight, so a call's last frame is sent exactly once:
//! either the task reports it finished, or the loop cancelled it. Frames from
//! the tasks go through one bounded channel, which is the backpressure: a
//! client that does not read stalls its own calls and nothing else.

use std::{
    collections::{BTreeMap, HashMap},
    sync::Arc,
};

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
use prost_reflect::{DeserializeOptions, DynamicMessage};
use serde::{Deserialize, Serialize};
use tbd_common::metrics::{RequestTimer, StreamGuard};
use tokio::{sync::mpsc, task::AbortHandle};
use tonic::client::Grpc;

use crate::{
    AppState,
    config::Socket,
    error::{Code, Detail, Problem},
    principal::Caller,
    transcode::{Rpc, Transcoder, call::Out, codec::DynamicCodec},
};

/// The path. Hand-written, so [`crate::http::reserved_paths`] carries it.
pub(crate) const PATH: &str = "/v1/ws";

/// The transport label of a call made here (`route` is the RPC name).
const TRANSPORT: &str = "ws";

/// The route, with the RPC table resolved once at startup.
pub fn routes(transcoder: &Transcoder, limits: Socket) -> Router<AppState> {
    let rpcs = Arc::new(transcoder.rpcs().clone());
    Router::new().route(
        PATH,
        get(
            move |ws: WebSocketUpgrade, caller: Option<Caller>, State(state): State<AppState>| {
                let rpcs = Arc::clone(&rpcs);
                async move { upgrade(ws, caller.as_deref(), state, rpcs, limits) }
            },
        ),
    )
}

fn upgrade(
    ws: WebSocketUpgrade,
    principal: Option<&tbd_common::principal::Principal>,
    state: AppState,
    rpcs: Arc<BTreeMap<String, Arc<Rpc>>>,
    limits: Socket,
) -> Response {
    tracing::debug!(
        caller = principal.map(tbd_common::principal::Principal::kind_slug),
        calls = rpcs.len(),
        "mux upgrade requested"
    );
    ws.max_message_size(limits.max_frame_bytes)
        .max_frame_size(limits.max_frame_bytes)
        .on_upgrade(move |socket| serve(socket, state, rpcs, limits))
}

/// A frame from the client.
#[derive(Debug, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum ClientFrame {
    /// Start a call. `body` is the whole request message; absent is an empty one.
    Call {
        /// The client's handle for this call, unique among those in flight.
        id: String,
        /// `tbd.ledger.v1.LedgerService/Current`.
        method: String,
        /// The request message as proto3 JSON.
        #[serde(default)]
        body: Option<serde_json::Value>,
    },
    /// Stop a call in flight. It ends with `end`, like any other.
    Cancel {
        /// The call's id.
        id: String,
    },
}

/// A frame to the client.
#[derive(Debug, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum ServerFrame<'a> {
    /// One response message. A unary call sends exactly one.
    Data {
        /// The call's id.
        id: &'a str,
        /// The message as proto3 JSON, the same shape REST answers.
        body: Out<'a>,
    },
    /// The call is over and its id is free again.
    End {
        /// The call's id.
        id: &'a str,
    },
    /// The call failed, or the frame did not make sense. The keys are the REST
    /// envelope's; `id` is absent when the frame named no call.
    Error {
        /// The call's id, when the frame carried one.
        #[serde(skip_serializing_if = "Option::is_none")]
        id: Option<&'a str>,
        /// The slug.
        code: Code,
        /// The sentence.
        error: &'a str,
        /// Typed details.
        details: &'a [Detail],
    },
}

/// What a call task reports to the connection loop.
enum Event {
    /// One `data` frame, already rendered.
    Data {
        /// The call's id.
        id: String,
        /// The frame.
        json: String,
    },
    /// The call finished, cleanly or not.
    Done {
        /// The call's id.
        id: String,
        /// How it ended.
        outcome: Result<(), Problem>,
    },
}

/// What accepting one client frame did.
enum Accepted {
    /// A call started; its task reports the end.
    Started,
    /// A call was cancelled here and needs its `end` now.
    Ended(String),
}

/// A refusal, with the call it belongs to when the frame named one.
type Refused = (Option<String>, Problem);

async fn serve(
    socket: WebSocket,
    state: AppState,
    rpcs: Arc<BTreeMap<String, Arc<Rpc>>>,
    limits: Socket,
) {
    let (mut sink, mut stream) = socket.split();
    let (events_tx, mut events_rx) =
        mpsc::channel::<Event>(limits.max_calls.saturating_mul(2).max(8));
    let mut calls: HashMap<String, AbortHandle> = HashMap::new();
    let guard = StreamGuard::open("mux");
    tracing::info!(callable = rpcs.len(), "mux socket opened");

    loop {
        tokio::select! {
            inbound = stream.next() => {
                let Some(message) = inbound else { break };
                let message = match message {
                    Ok(message) => message,
                    Err(error) => {
                        tracing::debug!(%error, "mux receive error");
                        break;
                    }
                };
                let text = match message {
                    Message::Text(text) => text.to_string(),
                    Message::Binary(bytes) => String::from_utf8_lossy(&bytes).into_owned(),
                    Message::Close(_) => break,
                    Message::Ping(_) | Message::Pong(_) => continue,
                };
                guard.item("in");
                let frame = match accept(&text, &state, &rpcs, limits, &mut calls, &events_tx) {
                    Ok(Accepted::Started) => continue,
                    Ok(Accepted::Ended(id)) => render(&ServerFrame::End { id: &id }),
                    Err((id, problem)) => {
                        problem.log();
                        let wire = problem.wire();
                        render(&ServerFrame::Error {
                            id: id.as_deref(),
                            code: wire.code,
                            error: wire.error,
                            details: wire.details,
                        })
                    }
                };
                if sink.send(Message::Text(frame.into())).await.is_err() {
                    break;
                }
            }
            event = events_rx.recv() => {
                // This loop holds a sender, so the channel never closes here.
                let Some(event) = event else { continue };
                let frame = match event {
                    Event::Data { id, json } => {
                        // A cancelled call's messages are already history.
                        if !calls.contains_key(&id) {
                            continue;
                        }
                        guard.item("out");
                        json
                    }
                    Event::Done { id, outcome } => {
                        if calls.remove(&id).is_none() {
                            continue;
                        }
                        match outcome {
                            Ok(()) => render(&ServerFrame::End { id: &id }),
                            Err(problem) => {
                                problem.log();
                                let wire = problem.wire();
                                render(&ServerFrame::Error {
                                    id: Some(&id),
                                    code: wire.code,
                                    error: wire.error,
                                    details: wire.details,
                                })
                            }
                        }
                    }
                };
                if sink.send(Message::Text(frame.into())).await.is_err() {
                    break;
                }
            }
        }
    }

    for (_, handle) in calls.drain() {
        handle.abort();
    }
    let _ = sink.close().await;
    tracing::info!("mux socket closed");
}

/// Parse one client frame and act on it.
fn accept(
    text: &str,
    state: &AppState,
    rpcs: &Arc<BTreeMap<String, Arc<Rpc>>>,
    limits: Socket,
    calls: &mut HashMap<String, AbortHandle>,
    events: &mpsc::Sender<Event>,
) -> Result<Accepted, Refused> {
    let frame: ClientFrame = serde_json::from_str(text)
        .map_err(|error| (None, Problem::field("frame", error.to_string())))?;
    match frame {
        ClientFrame::Cancel { id } => match calls.remove(&id) {
            Some(handle) => {
                handle.abort();
                Ok(Accepted::Ended(id))
            }
            None => Err((
                Some(id),
                Problem::new(Code::NotFound, "no call with that id is in flight"),
            )),
        },
        ClientFrame::Call { id, method, body } => {
            if calls.contains_key(&id) {
                return Err((
                    Some(id),
                    Problem::bad_request("a call with that id is already in flight"),
                ));
            }
            if calls.len() >= limits.max_calls {
                return Err((
                    Some(id),
                    Problem::new(
                        Code::RateLimited,
                        format!(
                            "at most {} calls may be in flight on one socket",
                            limits.max_calls
                        ),
                    ),
                ));
            }
            let Some(rpc) = rpcs.get(&method) else {
                return Err((
                    Some(id),
                    Problem::new(
                        Code::NotFound,
                        format!("{method} is not a public RPC of this gateway"),
                    ),
                ));
            };
            let message = request(rpc, body).map_err(|problem| (Some(id.clone()), problem))?;
            let task = tokio::spawn(run(
                Arc::clone(rpc),
                state.clone(),
                id.clone(),
                message,
                events.clone(),
            ));
            calls.insert(id, task.abort_handle());
            Ok(Accepted::Started)
        }
    }
}

/// The request message: the frame's `body`, strictly, or an empty message.
fn request(rpc: &Rpc, body: Option<serde_json::Value>) -> Result<DynamicMessage, Problem> {
    let input = rpc.method.input();
    match body {
        None | Some(serde_json::Value::Null) => Ok(DynamicMessage::new(input)),
        Some(value) => {
            let options = DeserializeOptions::new().deny_unknown_fields(true);
            DynamicMessage::deserialize_with_options(input, value, &options)
                .map_err(|error| Problem::field("body", error.to_string()))
        }
    }
}

/// One call, from its own task: measured like every other request path.
async fn run(
    rpc: Arc<Rpc>,
    state: AppState,
    id: String,
    message: DynamicMessage,
    events: mpsc::Sender<Event>,
) {
    let name = rpc.name();
    let mut timer = RequestTimer::start(TRANSPORT, name.clone());
    let outcome = forward(&rpc, &state, &id, message, &events).await;
    timer.set_status(match &outcome {
        Ok(()) => "ok",
        Err(problem) => problem.code.slug(),
    });
    let _ = events.send(Event::Done { id, outcome }).await;
}

async fn forward(
    rpc: &Rpc,
    state: &AppState,
    id: &str,
    message: DynamicMessage,
    events: &mpsc::Sender<Event>,
) -> Result<(), Problem> {
    let backend = state.backend(&rpc.backend).ok_or_else(|| {
        Problem::new(
            Code::Unavailable,
            format!("backend {} is not registered", rpc.backend),
        )
    })?;
    let mut grpc = Grpc::new(backend.transport());
    grpc.ready().await.map_err(|error| {
        Problem::from(tonic::Status::unavailable(format!(
            "backend not ready: {}",
            Into::<tonic::codegen::StdError>::into(error)
        )))
    })?;
    let codec = DynamicCodec::new(rpc.method.output());
    if rpc.streaming {
        let mut stream = grpc
            .server_streaming(tonic::Request::new(message), rpc.grpc_path.clone(), codec)
            .await?
            .into_inner();
        while let Some(item) = stream.next().await {
            data(events, id, &item?).await?;
        }
        Ok(())
    } else {
        let response = grpc
            .unary(tonic::Request::new(message), rpc.grpc_path.clone(), codec)
            .await?
            .into_inner();
        data(events, id, &response).await
    }
}

/// Render one message and hand it to the connection loop, waiting while the
/// channel is full: that wait is this call's backpressure.
async fn data(
    events: &mpsc::Sender<Event>,
    id: &str,
    message: &DynamicMessage,
) -> Result<(), Problem> {
    let json = serde_json::to_string(&ServerFrame::Data {
        id,
        body: Out(message),
    })
    .map_err(|error| Problem::new(Code::Internal, format!("serialize: {error}")))?;
    events
        .send(Event::Data {
            id: id.to_owned(),
            json,
        })
        .await
        .map_err(|_| Problem::new(Code::Cancelled, "the socket closed"))
}

/// Serialising a frame of strings cannot fail; the fallback keeps the socket
/// honest if it ever does.
fn render(frame: &ServerFrame<'_>) -> String {
    serde_json::to_string(frame).unwrap_or_else(|_| {
        r#"{"type":"error","code":"internal","error":"serialize","details":[]}"#.to_owned()
    })
}
