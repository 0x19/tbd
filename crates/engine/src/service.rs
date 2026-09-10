//! `tbd.engine.v1.Engine` implementation.
//!
//! Everything that returns a score here is a **stub** and says so on the wire
//! (`stub = true`). No model exists yet. The streaming plumbing is real.

use std::{pin::Pin, time::Duration};

use futures::{Stream, StreamExt};
use tbd_proto::engine::v1::{
    Close, EvaluateRequest, EvaluateResponse, Event, Heartbeat, SessionFrame, SubscribeRequest,
    engine_server::Engine, event, session_frame,
};
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

/// Model version reported while no model exists. The `stub-` prefix is part of
/// the contract: consumers may match on it.
pub const STUB_MODEL_VERSION: &str = "stub-0";

type BoxStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send + 'static>>;

/// The engine service. Cheap to clone; holds only configuration.
#[derive(Debug, Clone)]
pub struct EngineService {
    heartbeat: Duration,
}

impl EngineService {
    /// Build a service that emits heartbeats at `heartbeat` on streams.
    pub fn new(heartbeat: Duration) -> Self {
        Self { heartbeat }
    }

    fn now() -> prost_types::Timestamp {
        prost_types::Timestamp::from(std::time::SystemTime::now())
    }
}

#[tonic::async_trait]
impl Engine for EngineService {
    async fn evaluate(
        &self,
        request: Request<EvaluateRequest>,
    ) -> Result<Response<EvaluateResponse>, Status> {
        let req = request.into_inner();
        if req.subject_id.is_empty() {
            return Err(Status::invalid_argument("subject_id is required"));
        }
        tracing::info!(subject_id = %req.subject_id, payload_len = req.payload.len(), "evaluate (stub)");
        Ok(Response::new(EvaluateResponse {
            subject_id: req.subject_id,
            score: 0.0,
            stub: true,
            model_version: STUB_MODEL_VERSION.to_owned(),
        }))
    }

    type SubscribeStream = BoxStream<Event>;

    async fn subscribe(
        &self,
        request: Request<SubscribeRequest>,
    ) -> Result<Response<Self::SubscribeStream>, Status> {
        let subject_id = request.into_inner().subject_id;
        if subject_id.is_empty() {
            return Err(Status::invalid_argument("subject_id is required"));
        }
        tracing::info!(%subject_id, "subscribe");

        // One interval for the life of the stream. It is moved through the unfold
        // state so it survives across awaits; the first tick fires immediately,
        // every later tick waits `heartbeat`.
        let mut interval = tokio::time::interval(self.heartbeat);
        interval.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
        let stream = futures::stream::unfold(
            (0_u64, subject_id, interval),
            |(seq, subject_id, mut interval)| async move {
                interval.tick().await;
                let event = Event {
                    id: uuid::Uuid::now_v7().to_string(),
                    subject_id: subject_id.clone(),
                    at: Some(Self::now()),
                    kind: Some(event::Kind::Heartbeat(Heartbeat { seq })),
                };
                Some((Ok(event), (seq + 1, subject_id, interval)))
            },
        );
        Ok(Response::new(Box::pin(stream)))
    }

    type SessionStream = BoxStream<SessionFrame>;

    async fn session(
        &self,
        request: Request<Streaming<SessionFrame>>,
    ) -> Result<Response<Self::SessionStream>, Status> {
        let mut inbound = request.into_inner();
        let (tx, rx) = mpsc::channel::<Result<SessionFrame, Status>>(32);
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
                        let hb = SessionFrame {
                            session_id: session_id.clone(),
                            seq,
                            body: Some(session_frame::Body::Heartbeat(Heartbeat { seq })),
                        };
                        if tx.send(Ok(hb)).await.is_err() {
                            break;
                        }
                        continue;
                    }
                };

                if session_id.is_empty() {
                    session_id.clone_from(&frame.session_id);
                    tracing::info!(%session_id, "session opened");
                }

                match frame.body {
                    Some(session_frame::Body::Data(data)) => {
                        seq += 1;
                        let echo = SessionFrame {
                            session_id: session_id.clone(),
                            seq,
                            body: Some(session_frame::Body::Data(data)),
                        };
                        if tx.send(Ok(echo)).await.is_err() {
                            break;
                        }
                    }
                    Some(session_frame::Body::Heartbeat(_)) | None => {}
                    Some(session_frame::Body::Close(Close { reason })) => {
                        tracing::info!(%session_id, %reason, "session closed by client");
                        break;
                    }
                }
            }
            tracing::debug!(%session_id, "session task finished");
        });

        Ok(Response::new(Box::pin(ReceiverStream::new(rx))))
    }
}
