//! `tbd.engine.v1.EngineService` implementation.
//!
//! Everything that returns a score here is a **stub** and says so on the wire
//! (`stub = true`). No model exists yet. The streaming plumbing is real.
//!
//! Every RPC first consults the fault handle in [`Runtime`], so an embedder can
//! make this engine slow, failing or hung at runtime.

use std::{pin::Pin, time::Duration};

use futures::{Stream, StreamExt};
use tbd_common::fault::{ErrorKind, Fault};
use tbd_proto::engine::v1::{
    Close, EvaluateRequest, EvaluateResponse, Heartbeat, SessionRequest, SessionResponse,
    SubscribeRequest, SubscribeResponse, engine_service_server::EngineService, session_request,
    session_response, subscribe_response,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Code, Request, Response, Status, Streaming};

use crate::Runtime;

/// Model version reported while no model exists. The `stub-` prefix is part of
/// the contract: consumers may match on it.
pub const STUB_MODEL_VERSION: &str = "stub-0";

type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

/// The engine service. Cheap to clone; holds configuration and shared handles.
#[derive(Debug, Clone)]
pub struct Engine {
    heartbeat: Duration,
    runtime: Runtime,
}

impl Engine {
    /// Build a service that emits heartbeats at `heartbeat` on streams.
    pub fn new(heartbeat: Duration, runtime: Runtime) -> Self {
        Self { heartbeat, runtime }
    }

    fn now() -> prost_types::Timestamp {
        prost_types::Timestamp::from(std::time::SystemTime::now())
    }

    /// Count the request and apply any injected fault before real work.
    async fn admit(&self) -> Result<(), Status> {
        self.runtime.stats.request();
        if let Err(fault) = self.runtime.fault.apply().await {
            self.runtime.stats.failure();
            return Err(status_from(fault));
        }
        Ok(())
    }

    fn reject(&self, status: Status) -> Status {
        self.runtime.stats.failure();
        status
    }
}

/// Map a transport-neutral fault onto a gRPC status.
fn status_from(fault: Fault) -> Status {
    let code = match fault.kind {
        ErrorKind::Unavailable => Code::Unavailable,
        ErrorKind::Internal => Code::Internal,
        ErrorKind::Overloaded => Code::ResourceExhausted,
        ErrorKind::Timeout => Code::DeadlineExceeded,
    };
    Status::new(code, fault.message)
}

#[tonic::async_trait]
impl EngineService for Engine {
    async fn evaluate(
        &self,
        request: Request<EvaluateRequest>,
    ) -> Result<Response<EvaluateResponse>, Status> {
        self.admit().await?;
        let req = request.into_inner();
        if req.subject_id.is_empty() {
            return Err(self.reject(Status::invalid_argument("subject_id is required")));
        }
        tracing::debug!(subject_id = %req.subject_id, payload_len = req.payload.len(), "evaluate (stub)");
        Ok(Response::new(EvaluateResponse {
            subject_id: req.subject_id,
            score: 0.0,
            stub: true,
            model_version: STUB_MODEL_VERSION.to_owned(),
        }))
    }

    type SubscribeStream = BoxStream<SubscribeResponse>;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        self.admit().await?;
        let subject_id = request.into_inner().subject_id;
        if subject_id.is_empty() {
            return Err(self.reject(Status::invalid_argument("subject_id is required")));
        }
        tracing::debug!(%subject_id, "subscribe");
        let fault = self.runtime.fault.clone();

        // One interval for the life of the stream. It is moved through the unfold
        // state so it survives across awaits; the first tick fires immediately,
        // every later tick waits `heartbeat`.
        let mut interval = tokio::time::interval(self.heartbeat);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let stream = futures::stream::unfold(
            (0_u64, subject_id, interval, fault),
            |(seq, subject_id, mut interval, fault)| async move {
                if seq == u64::MAX {
                    return None;
                }
                interval.tick().await;
                if let Some(injected) = fault.stream_error() {
                    // Yield the injected status, then end the stream on the next poll.
                    let state = (u64::MAX, subject_id, interval, fault);
                    return Some((Err(status_from(injected)), state));
                }
                let event = SubscribeResponse {
                    id: uuid::Uuid::now_v7().to_string(),
                    subject_id: subject_id.clone(),
                    at: Some(Self::now()),
                    kind: Some(subscribe_response::Kind::Heartbeat(Heartbeat { seq })),
                };
                Some((Ok(event), (seq + 1, subject_id, interval, fault)))
            },
        );
        Ok(Response::new(Box::pin(stream)))
    }

    type SessionStream = BoxStream<SessionResponse>;

    async fn session(
        &self,
        request: Request<Streaming<SessionRequest>>,
    ) -> Result<Response<Self::SessionStream>, Status> {
        self.admit().await?;
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<SessionResponse, Status>>(32);
        let heartbeat = self.heartbeat;

        tokio::spawn(async move {
            // First heartbeat after one full interval, so it never races the
            // echo of the client's opening frame.
            let mut ticker =
                tokio::time::interval_at(tokio::time::Instant::now() + heartbeat, heartbeat);
            ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
            let mut session_id = String::new();
            let mut seq = 0_u64;

            loop {
                let frame = tokio::select! {
                    inbound = inbound.next() => match inbound {
                        Some(Ok(frame)) => frame,
                        Some(Err(status)) => {
                            tracing::warn!(%status, "session inbound error");
                            break;
                        }
                        None => break,
                    },
                    _ = ticker.tick() => {
                        seq += 1;
                        let hb = SessionResponse {
                            session_id: session_id.clone(),
                            seq,
                            body: Some(session_response::Body::Heartbeat(Heartbeat { seq })),
                        };
                        if tx.send(Ok(hb)).await.is_err() {
                            break;
                        }
                        continue;
                    }
                };

                if session_id.is_empty() {
                    session_id.clone_from(&frame.session_id);
                    tracing::debug!(%session_id, "session opened");
                }

                match frame.body {
                    Some(session_request::Body::Data(data)) => {
                        seq += 1;
                        let echo = SessionResponse {
                            session_id: session_id.clone(),
                            seq,
                            body: Some(session_response::Body::Data(data)),
                        };
                        if tx.send(Ok(echo)).await.is_err() {
                            break;
                        }
                    }
                    Some(session_request::Body::Heartbeat(_)) | None => {}
                    Some(session_request::Body::Close(Close { reason })) => {
                        tracing::debug!(%session_id, %reason, "session closed by client");
                        break;
                    }
                }
            }
            tracing::debug!(%session_id, "session task finished");
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
