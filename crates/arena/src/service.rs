//! `tbd.arena.v1.ArenaService` implementation.
//!
//! `Ping` is the scaffold's stub and says so (`stub = true`). `GetSnapshot`
//! and `Watch` read the [`World`]: the snapshot the collectors keep current.
//! Both take the caller's role first (`[watch] require_role`, `admin` while
//! the lab is private), and `Watch` counts its viewers against
//! `[watch] max_viewers`. Every RPC first consults the fault handle in
//! [`Runtime`], so an embedder can make this service slow, failing or hung.

use std::{
    pin::Pin,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

use futures::{Stream, StreamExt as _};
use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, StreamGuard, names},
    principal::Principal,
};
use tbd_proto::arena::v1::{
    GetSnapshotRequest, GetSnapshotResponse, PingRequest, PingResponse, WatchRequest,
    WatchResponse, arena_service_server::ArenaService,
};
use tokio_stream::wrappers::BroadcastStream;
use tonic::{Code, Request, Response, Status};

use crate::{
    Runtime,
    config::{Ping, Watch},
    world::World,
};

/// The service. Cheap to clone; holds its configuration sections and shared handles.
#[derive(Debug, Clone)]
pub struct Arena {
    ping: Ping,
    watch: Watch,
    runtime: Runtime,
    world: World,
    viewers: Arc<AtomicUsize>,
}

/// One open `Watch`: counted while it lives.
struct Viewer {
    count: Arc<AtomicUsize>,
}

impl Drop for Viewer {
    fn drop(&mut self) {
        let left = self.count.fetch_sub(1, Ordering::SeqCst).saturating_sub(1);
        #[allow(clippy::cast_precision_loss)] // viewers in the tens
        metrics::gauge!(names::ARENA_VIEWERS).set(left as f64);
    }
}

impl Arena {
    /// Build a service over a world.
    #[must_use]
    pub fn new(ping: Ping, watch: Watch, runtime: Runtime, world: World) -> Self {
        Self {
            ping,
            watch,
            runtime,
            world,
            viewers: Arc::new(AtomicUsize::new(0)),
        }
    }

    /// The caller may read the arena: any caller when no role is required,
    /// else a verified one with that role.
    fn allowed<T>(&self, request: &Request<T>) -> Result<(), Status> {
        let need = self.watch.require_role.trim();
        if need.is_empty() {
            return Ok(());
        }
        let headers = request.metadata().clone().into_headers();
        let caller = Principal::from_headers(&headers, &[])
            .ok_or_else(|| Status::unauthenticated("no verified caller"))?;
        if caller.role.as_deref() == Some(need) {
            Ok(())
        } else {
            Err(Status::permission_denied(format!(
                "the arena is for the {need} role while the lab is private"
            )))
        }
    }

    /// A viewer slot, or `RESOURCE_EXHAUSTED` when every one is taken.
    fn viewer(&self) -> Result<Viewer, Status> {
        let before = self.viewers.fetch_add(1, Ordering::SeqCst);
        if before >= self.watch.max_viewers {
            self.viewers.fetch_sub(1, Ordering::SeqCst);
            return Err(Status::resource_exhausted(format!(
                "busy: {before} are watching the arena; try again shortly"
            )));
        }
        #[allow(clippy::cast_precision_loss)] // viewers in the tens
        metrics::gauge!(names::ARENA_VIEWERS).set((before + 1) as f64);
        Ok(Viewer {
            count: Arc::clone(&self.viewers),
        })
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
impl ArenaService for Arena {
    type WatchStream = Pin<Box<dyn Stream<Item = Result<WatchResponse, Status>> + Send + 'static>>;

    async fn get_snapshot(
        &self,
        request: Request<GetSnapshotRequest>,
    ) -> Result<Response<GetSnapshotResponse>, Status> {
        let mut timer = self.admit("ArenaService/GetSnapshot").await?;
        self.allowed(&request)
            .map_err(|s| self.reject(&mut timer, s))?;
        Ok(Response::new(GetSnapshotResponse {
            snapshot: Some(self.world.snapshot()),
        }))
    }

    async fn watch(
        &self,
        request: Request<WatchRequest>,
    ) -> Result<Response<Self::WatchStream>, Status> {
        let mut timer = self.admit("ArenaService/Watch").await?;
        self.allowed(&request)
            .map_err(|s| self.reject(&mut timer, s))?;
        let viewer = self.viewer().map_err(|s| self.reject(&mut timer, s))?;
        let guard = StreamGuard::open("arena_watch");
        let first = self.world.snapshot();
        // A reader that lags skips to the next whole frame, which catches it up.
        let updates =
            BroadcastStream::new(self.world.watch()).filter_map(|item| async move { item.ok() });
        let stream = futures::stream::iter(std::iter::once(first))
            .chain(updates)
            .map(move |snapshot| {
                let _keep = &viewer;
                guard.item("snapshot");
                Ok(WatchResponse {
                    snapshot: Some(snapshot),
                })
            });
        Ok(Response::new(Box::pin(stream)))
    }

    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("ArenaService/Ping").await?;
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
            stub: true,
        }))
    }
}
