//! `tbd.finance.v1.FinanceService` implementation.
//!
//! Everything here is a **stub** and says so on the wire (`stub = true`).
//! Every RPC first consults the fault handle in [`Runtime`], so an embedder can
//! make this service slow, failing or hung at runtime.

use std::sync::Arc;

use sqlx::PgPool;
use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
    principal::Principal,
};
use tbd_db::{Access, DbError};
use tbd_proto::finance::v1::{
    ListTransactionsRequest, ListTransactionsResponse, PingRequest, PingResponse, Transaction,
    finance_service_server::FinanceService,
};
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use crate::{
    Runtime,
    config::Ping,
    store::{self, MemoryStore, PgStore, Store},
};

/// The service. Cheap to clone; holds its configuration section and shared handles.
#[derive(Debug, Clone)]
pub struct Finance {
    ping: Ping,
    runtime: Runtime,
    /// The store, when there is one. Without it every data RPC answers
    /// `UNAVAILABLE`, which is what a scaffolded deployment looks like before
    /// its database exists.
    store: Option<Arc<dyn Store>>,
    /// The pool, kept separately because resolving a caller's grants needs the
    /// identity tables and the memory store has none. A memory-backed stack
    /// therefore takes its caller from `Access::for_parties` instead.
    pool: Option<PgPool>,
    /// Subject to readable parties, for memory-backed stacks only. Empty with a
    /// database, where grants are rows in `public.party_access`.
    grants: Vec<(String, Vec<Uuid>)>,
}

impl Finance {
    /// Build a service from its `[ping]` configuration, without a store.
    ///
    /// Every data RPC then answers `UNAVAILABLE`, which is what a scaffolded
    /// deployment does before its database exists -- and what the chaos stack
    /// does when a scenario gives it no `database_url`.
    #[must_use]
    pub fn new(ping: Ping, runtime: Runtime) -> Self {
        Self {
            ping,
            runtime,
            store: None,
            pool: None,
            grants: Vec::new(),
        }
    }

    /// Build a service backed by a database.
    #[must_use]
    pub fn with_pool(ping: Ping, runtime: Runtime, pool: PgPool) -> Self {
        Self {
            ping,
            runtime,
            store: Some(Arc::new(PgStore::new(pool.clone()))),
            pool: Some(pool),
            grants: Vec::new(),
        }
    }

    /// Build a service over an in-memory store, for stacks with no database.
    ///
    /// Callers are resolved with [`tbd_db::Access::for_parties`] from
    /// `grants`, because there are no identity tables to read. Chaos uses this
    /// so an access-control scenario can run against a live service.
    #[must_use]
    pub fn with_memory(
        ping: Ping,
        runtime: Runtime,
        store: MemoryStore,
        grants: Vec<(String, Vec<Uuid>)>,
    ) -> Self {
        Self {
            ping,
            runtime,
            store: Some(Arc::new(store)),
            pool: None,
            grants: Vec::new(),
        }
        .with_grants(grants)
    }

    fn with_grants(mut self, grants: Vec<(String, Vec<Uuid>)>) -> Self {
        self.grants = grants;
        self
    }

    fn store(&self) -> Result<&Arc<dyn Store>, Status> {
        self.store
            .as_ref()
            .ok_or_else(|| Status::unavailable("no store configured"))
    }

    /// Who is calling, from the claims Envoy verified and forwarded.
    ///
    /// Envoy strips `x-jwt-payload` from anything a client sends and is the
    /// only way in, so its presence means the token was checked. Nothing here
    /// verifies anything; an absent or unparsable header is simply not
    /// authenticated.
    fn principal<T>(request: &Request<T>) -> Result<Principal, Status> {
        let headers = request.metadata().clone().into_headers();
        Principal::from_headers(&headers, &[])
            .ok_or_else(|| Status::unauthenticated("no verified caller"))
    }

    /// The caller's readable parties, provisioning the user on first sight.
    ///
    /// With a database this reads real grants. Without one -- a chaos stack --
    /// it uses the grants the stack was built with, keyed by subject. Either
    /// way the set comes from the verified subject and never from the request.
    async fn access(&self, request: &Request<impl Sized>) -> Result<Access, Status> {
        let principal = Self::principal(request)?;
        if let Some(pool) = &self.pool {
            return Access::resolve(pool, &principal.sub, None, "")
                .await
                .map_err(status_of);
        }
        let parties = self
            .grants
            .iter()
            .find(|(subject, _)| *subject == principal.sub)
            .map(|(_, parties)| parties.clone())
            .unwrap_or_default();
        Ok(Access::for_parties(tbd_db::UserId(Uuid::nil()), parties))
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

/// The one place a store error becomes a status.
fn status_of(e: DbError) -> Status {
    match e {
        DbError::NotFound { what } => Status::not_found(what),
        DbError::Invalid { field, reason } => {
            Status::invalid_argument(format!("invalid {field}: {reason}"))
        }
        DbError::Conflict { reason } => Status::aborted(reason),
        // Retryable, and the cause is ours: do not hand the caller the detail.
        DbError::Unavailable { .. } => Status::unavailable("store unavailable"),
        DbError::Internal(_) => Status::internal("store error"),
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
impl FinanceService for Finance {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("FinanceService/Ping").await?;
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

    async fn list_transactions(
        &self,
        request: Request<ListTransactionsRequest>,
    ) -> Result<Response<ListTransactionsResponse>, Status> {
        let mut timer = self.admit("FinanceService/ListTransactions").await?;
        let access = match self.access(&request).await {
            Ok(access) => access,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let req = request.into_inner();

        // A malformed id is a bad request. It is *not* treated as "no such
        // party", so a caller cannot probe which ids are well formed and which
        // exist by watching the error change.
        let mut narrow = Vec::with_capacity(req.party_ids.len());
        for id in &req.party_ids {
            match Uuid::parse_str(id) {
                Ok(id) => narrow.push(id),
                Err(_) => {
                    return Err(self.reject(
                        &mut timer,
                        Status::invalid_argument("party_ids: not a uuid"),
                    ));
                }
            }
        }

        let limit = if req.limit == 0 {
            store::DEFAULT_LIMIT
        } else {
            req.limit
        };
        let store = match self.store() {
            Ok(store) => store,
            Err(status) => return Err(self.reject(&mut timer, status)),
        };
        let rows = match store.transactions(&access, &narrow, limit).await {
            Ok(rows) => rows,
            Err(e) => return Err(self.reject(&mut timer, status_of(e))),
        };

        let view = if narrow.is_empty() {
            access.clone()
        } else {
            access.narrow(&narrow)
        };
        tracing::debug!(
            rows = rows.len(),
            parties = view.party_ids().len(),
            "list_transactions"
        );
        Ok(Response::new(ListTransactionsResponse {
            transactions: rows
                .into_iter()
                .map(|t| Transaction {
                    id: t.id.to_string(),
                    account_id: t.account_id.to_string(),
                    party_id: t.party_id.to_string(),
                    status: t.status.to_uppercase(),
                    amount_minor: t.amount_minor,
                    currency: t.currency,
                    scale: u32::try_from(t.scale).unwrap_or(2),
                    booking_date: t.booking_date.map(|d| d.to_string()).unwrap_or_default(),
                    counterparty_name: t.counterparty_name.unwrap_or_default(),
                    remittance: t.remittance.unwrap_or_default(),
                })
                .collect(),
            party_ids: view.party_ids().iter().map(ToString::to_string).collect(),
        }))
    }
}
