//! `tbd.playground.v1.PlaygroundService` implementation.
//!
//! Four public RPCs plus the liveness `Ping`. Everything here is reachable by
//! anyone: the service takes no identity, so it accepts no address, no rate and
//! no duration, and every parameter it does accept is an enum or a name it
//! looks up. The gateway turns these same four into REST, server-sent events
//! and calls over the multiplexed socket without any of it being written twice.

use std::{
    pin::Pin,
    sync::{Arc, OnceLock},
};

use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
};
use tbd_proto::playground::v1::{
    GetWorldRequest, GetWorldResponse, InjectFaultRequest, InjectFaultResponse, Move, PingRequest,
    PingResponse, ScoresRequest, ScoresResponse, WatchRequest, WatchResponse,
    playground_service_server::PlaygroundService,
};
use tokio_stream::{Stream, StreamExt, wrappers::BroadcastStream};
use tonic::{Code, Request, Response, Status};

use crate::{Runtime, config::Ping, faults, world::World};

/// The service. Cheap to clone; holds its configuration section and shared handles.
#[derive(Debug, Clone)]
pub struct Playground {
    ping: Ping,
    runtime: Runtime,
    /// Filled once the sandbox is warm. The server binds first and the world
    /// arrives a few seconds later, so Kubernetes sees a live service straight
    /// away instead of killing a process that is still starting four services.
    world: Arc<OnceLock<Arc<World>>>,
}

impl Playground {
    /// Build a service from its `[ping]` configuration.
    #[must_use]
    pub fn new(ping: Ping, runtime: Runtime) -> Self {
        Self {
            ping,
            runtime,
            world: Arc::new(OnceLock::new()),
        }
    }

    /// The slot the sandbox fills when it is ready. An embedder that only wants
    /// `Ping` never fills it, and the game RPCs answer `UNAVAILABLE`.
    #[must_use]
    pub fn world_slot(&self) -> Arc<OnceLock<Arc<World>>> {
        Arc::clone(&self.world)
    }

    fn world(&self) -> Result<&Arc<World>, Status> {
        self.world
            .get()
            .ok_or_else(|| Status::unavailable("the sandbox is still starting; try again shortly"))
    }

    /// Count the request, start its timer and apply any injected fault
    /// before real work.
    async fn admit(&self, route: &'static str) -> Result<RequestTimer, Status> {
        self.runtime.stats.request();
        let mut timer = RequestTimer::start("grpc", route);
        if let Err(fault) = self.runtime.fault.apply().await {
            self.runtime.stats.failure();
            metrics::counter!(names::FAULTS_INJECTED_TOTAL, "kind" => format!("{:?}", fault.kind).to_lowercase())
                .increment(1);
            let status = status_from(fault);
            timer.set_status(format!("{:?}", status.code()));
            return Err(status);
        }
        Ok(timer)
    }

    fn reject(&self, timer: &mut RequestTimer, status: Status) -> Status {
        self.runtime.stats.failure();
        timer.set_status(format!("{:?}", status.code()));
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
impl PlaygroundService for Playground {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("PlaygroundService/Ping").await?;
        let message = request.into_inner().message;
        if message.len() > self.ping.max_message_len {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument(format!(
                    "message longer than {} bytes",
                    self.ping.max_message_len
                )),
            ));
        }
        tracing::debug!(len = message.len(), "ping (stub)");
        Ok(Response::new(PingResponse {
            message,
            version: tbd_common::VERSION.to_owned(),
            // The sandbox behind the other RPCs is real; only this echo is a stub.
            stub: true,
        }))
    }

    async fn get_world(
        &self,
        _request: Request<GetWorldRequest>,
    ) -> Result<Response<GetWorldResponse>, Status> {
        let _timer = self.admit("PlaygroundService/GetWorld").await?;
        Ok(Response::new(self.world()?.snapshot().await))
    }

    type WatchStream = Pin<Box<dyn Stream<Item = Result<WatchResponse, Status>> + Send + 'static>>;

    async fn watch(
        &self,
        _request: Request<WatchRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let _timer = self.admit("PlaygroundService/Watch").await?;
        let world = self.world()?;
        let guard = tbd_common::metrics::StreamGuard::open("playground_watch");

        // The world as it stands, then every tick. A client that joins
        // mid-game is never left staring at an empty page waiting for one.
        let first = world.snapshot().await;
        let updates = BroadcastStream::new(world.watch()).filter_map(move |item| match item {
            Ok(world) => Some(Ok(WatchResponse { world: world.world })),
            // A slow reader misses ticks; the next whole world catches it up.
            Err(_) => None,
        });

        let stream = tokio_stream::iter(std::iter::once(Ok(WatchResponse { world: first.world })))
            .chain(updates)
            .map(move |item| {
                guard.item("world");
                item
            });

        Ok(Response::new(Box::pin(stream) as Self::WatchStream))
    }

    async fn inject_fault(
        &self,
        request: Request<InjectFaultRequest>,
    ) -> Result<Response<InjectFaultResponse>, Status> {
        let mut timer = self.admit("PlaygroundService/InjectFault").await?;
        let world = self.world()?.clone();
        let request = request.into_inner();

        let mv = Move::try_from(request.r#move).unwrap_or(Move::Unspecified);
        if mv == Move::Unspecified {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument("pick a move: latency, errors, kill or stall_store"),
            ));
        }
        let actor = faults::actor(&request.actor);
        let instance = (!request.instance.is_empty()).then_some(request.instance.as_str());

        match world.inject(mv, instance, &actor).await {
            Ok((cost, snapshot)) => {
                tracing::info!(?mv, %actor, cost, "a move landed");
                Ok(Response::new(InjectFaultResponse {
                    accepted: true,
                    reason: String::new(),
                    cost,
                    world: snapshot.world,
                }))
            }
            // A refusal is an answer, not a failure: the caller is told why and
            // handed the world unchanged, so the page can explain itself.
            Err(refusal) => Ok(Response::new(InjectFaultResponse {
                accepted: false,
                reason: refusal.reason(),
                cost: 0,
                world: world.snapshot().await.world,
            })),
        }
    }

    async fn scores(
        &self,
        _request: Request<ScoresRequest>,
    ) -> Result<Response<ScoresResponse>, Status> {
        let _timer = self.admit("PlaygroundService/Scores").await?;
        Ok(Response::new(ScoresResponse {
            scores: self.world()?.scores().list().await,
        }))
    }
}
