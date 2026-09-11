//! `tbd.ledger.v1.LedgerService` implementation: a thin adapter over the
//! [`Store`]. `Ping` is still a **stub** and says so on the wire; the facts
//! RPCs land beside it. Every RPC first consults the fault handle in
//! [`Runtime`], so an embedder can make this service slow, failing or hung at
//! runtime.

use std::sync::Arc;

use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
};
use tbd_proto::ledger::v1::{PingRequest, PingResponse, ledger_service_server::LedgerService};
use tonic::{Code, Request, Response, Status};

use crate::{
    Runtime,
    config::Ping,
    store::{Store, StoreKind},
};

/// The service. Cheap to clone; holds its configuration section and shared handles.
#[derive(Clone)]
pub struct Ledger {
    ping: Ping,
    runtime: Runtime,
    store: Arc<dyn Store>,
}

impl std::fmt::Debug for Ledger {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Ledger")
            .field("store", &self.store.kind())
            .finish_non_exhaustive()
    }
}

impl Ledger {
    /// Build a service from its `[ping]` configuration and a store.
    #[must_use]
    pub fn new(ping: Ping, runtime: Runtime, store: Arc<dyn Store>) -> Self {
        Self {
            ping,
            runtime,
            store,
        }
    }

    /// Which store backs this service.
    #[must_use]
    pub fn store_kind(&self) -> StoreKind {
        self.store.kind()
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
impl LedgerService for Ledger {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("LedgerService/Ping").await?;
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
        tracing::debug!(len = message.len(), store = %self.store.kind(), "ping (stub)");
        Ok(Response::new(PingResponse {
            message,
            version: tbd_common::VERSION.to_owned(),
            stub: true,
        }))
    }
}
