//! `tbd.radar.v1.RadarService` implementation.
//!
//! Every RPC first consults the fault handle in [`Runtime`], so an embedder can
//! make this service slow, failing or hung at runtime. Reads are public;
//! `Refresh` and `RunDigest` need the admin role on the caller Envoy verified.
//! Without a store every RPC but `Ping` answers `UNAVAILABLE`.

use chrono::Utc;
use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
    principal::Principal,
};
use tbd_proto::radar::v1::{
    Digest, GetDigestRequest, GetDigestResponse, Item, ListDigestsRequest, ListDigestsResponse,
    ListItemsRequest, ListItemsResponse, PingRequest, PingResponse, RefreshRequest,
    RefreshResponse, RunDigestRequest, RunDigestResponse, radar_service_server::RadarService,
};
use tonic::{Code, Request, Response, Status};

use crate::{
    Runtime,
    config::Ping,
    store::{DigestRow, ItemRow, Store},
    worker::{RunError, Worker},
};

/// The role that may refresh and write.
pub const ADMIN_ROLE: &str = "admin";

/// The service. Cheap to clone; holds its configuration section and shared handles.
#[derive(Debug, Clone)]
pub struct Radar {
    ping: Ping,
    runtime: Runtime,
    store: Option<Store>,
    worker: Option<Worker>,
    page_size: u32,
}

impl Radar {
    /// Build a service from its `[ping]` configuration, with no store yet.
    #[must_use]
    pub fn new(ping: Ping, runtime: Runtime) -> Self {
        Self {
            ping,
            runtime,
            store: None,
            worker: None,
            page_size: 20,
        }
    }

    /// Attach the store and the worker that reads and writes through it.
    #[must_use]
    pub fn with_store(mut self, store: Store, worker: Worker, page_size: u32) -> Self {
        self.store = Some(store);
        self.worker = Some(worker);
        self.page_size = page_size.max(1);
        self
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

    fn store(&self, timer: &mut RequestTimer) -> Result<&Store, Status> {
        match &self.store {
            Some(s) => Ok(s),
            None => Err(self.reject(timer, Status::unavailable("no store configured"))),
        }
    }

    /// The caller, and only if Envoy verified them as an admin.
    fn admin<T>(
        &self,
        request: &Request<T>,
        timer: &mut RequestTimer,
    ) -> Result<Principal, Status> {
        let headers = request.metadata().clone().into_headers();
        let Some(p) = Principal::from_headers(&headers, &[]) else {
            return Err(self.reject(timer, Status::unauthenticated("no verified caller")));
        };
        if p.role.as_deref() == Some(ADMIN_ROLE) {
            Ok(p)
        } else {
            Err(self.reject(timer, Status::permission_denied("admins only")))
        }
    }

    fn limit(&self, asked: i32) -> i64 {
        let max = i64::from(self.page_size) * 5;
        match i64::from(asked) {
            n if n <= 0 => i64::from(self.page_size),
            n => n.min(max),
        }
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

fn filter(value: &str, allowed: &[&str], what: &str) -> Result<String, Status> {
    if value.is_empty() || allowed.contains(&value) {
        Ok(value.to_owned())
    } else {
        Err(Status::invalid_argument(format!(
            "{what} must be one of {allowed:?} or empty"
        )))
    }
}

fn to_item(r: ItemRow) -> Item {
    Item {
        id: r.id,
        source: r.item.source,
        language: r.item.language,
        title: r.item.title,
        url: r.item.url,
        published_at: r.item.published_at.to_rfc3339(),
        summary: r.item.summary,
    }
}

fn to_digest(d: DigestRow) -> Digest {
    Digest {
        id: d.id,
        week: d.week,
        language: d.language,
        lang: d.lang,
        created_at: d.created_at.map(|t| t.to_rfc3339()).unwrap_or_default(),
        changed: d.changed,
        why: d.why,
        drill: d.drill,
        script: d.script,
        item_count: d.item_count,
        model: d.model,
        ai_written: true,
        stub: d.stub,
    }
}

fn internal(e: impl std::fmt::Display) -> Status {
    tracing::error!(error = %e, "radar store");
    Status::internal("the store failed")
}

#[tonic::async_trait]
impl RadarService for Radar {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("RadarService/Ping").await?;
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
        Ok(Response::new(PingResponse {
            message,
            version: tbd_common::VERSION.to_owned(),
            // Ping is the liveness echo in every service and stays labelled a
            // stub even where the service is real (the chaos check asserts it).
            stub: true,
        }))
    }

    async fn list_digests(
        &self,
        request: Request<ListDigestsRequest>,
    ) -> Result<Response<ListDigestsResponse>, Status> {
        let mut timer = self.admit("RadarService/ListDigests").await?;
        let req = request.into_inner();
        let language = filter(&req.language, &["go", "rust"], "language")
            .map_err(|s| self.reject(&mut timer, s))?;
        let lang =
            filter(&req.lang, &["en", "hr"], "lang").map_err(|s| self.reject(&mut timer, s))?;
        let limit = self.limit(req.limit);
        let store = self.store(&mut timer)?;
        let rows = store
            .list_digests(&language, &lang, limit)
            .await
            .map_err(|e| self.reject(&mut timer, internal(e)))?;
        Ok(Response::new(ListDigestsResponse {
            digests: rows.into_iter().map(to_digest).collect(),
        }))
    }

    async fn get_digest(
        &self,
        request: Request<GetDigestRequest>,
    ) -> Result<Response<GetDigestResponse>, Status> {
        let mut timer = self.admit("RadarService/GetDigest").await?;
        let id = request.into_inner().id;
        let store = self.store(&mut timer)?;
        match store
            .get_digest(id)
            .await
            .map_err(|e| self.reject(&mut timer, internal(e)))?
        {
            Some(d) => Ok(Response::new(GetDigestResponse {
                digest: Some(to_digest(d)),
            })),
            None => Err(self.reject(&mut timer, Status::not_found(format!("no digest {id}")))),
        }
    }

    async fn list_items(
        &self,
        request: Request<ListItemsRequest>,
    ) -> Result<Response<ListItemsResponse>, Status> {
        let mut timer = self.admit("RadarService/ListItems").await?;
        let req = request.into_inner();
        let language = filter(&req.language, &["go", "rust"], "language")
            .map_err(|s| self.reject(&mut timer, s))?;
        let limit = self.limit(req.limit);
        let store = self.store(&mut timer)?;
        let rows = store
            .list_items(&language, limit)
            .await
            .map_err(|e| self.reject(&mut timer, internal(e)))?;
        Ok(Response::new(ListItemsResponse {
            items: rows.into_iter().map(to_item).collect(),
        }))
    }

    async fn refresh(
        &self,
        request: Request<RefreshRequest>,
    ) -> Result<Response<RefreshResponse>, Status> {
        let mut timer = self.admit("RadarService/Refresh").await?;
        let who = self.admin(&request, &mut timer)?;
        self.store(&mut timer)?;
        let Some(worker) = &self.worker else {
            return Err(self.reject(&mut timer, Status::unavailable("no worker")));
        };
        tracing::info!(subject = %who.sub, "radar refresh requested");
        let done = worker
            .refresh(Utc::now())
            .await
            .map_err(|e| self.reject(&mut timer, internal(e)))?;
        Ok(Response::new(RefreshResponse {
            sources: i32::try_from(done.sources).unwrap_or(i32::MAX),
            failed: i32::try_from(done.failed).unwrap_or(i32::MAX),
            new_items: i32::try_from(done.new_items).unwrap_or(i32::MAX),
        }))
    }

    async fn run_digest(
        &self,
        request: Request<RunDigestRequest>,
    ) -> Result<Response<RunDigestResponse>, Status> {
        let mut timer = self.admit("RadarService/RunDigest").await?;
        let who = self.admin(&request, &mut timer)?;
        self.store(&mut timer)?;
        let Some(worker) = &self.worker else {
            return Err(self.reject(&mut timer, Status::unavailable("no worker")));
        };
        let req = request.into_inner();
        if !worker.has_writer() {
            return Err(self.reject(
                &mut timer,
                Status::failed_precondition("no llm service configured"),
            ));
        }
        let now = Utc::now();
        let week = Worker::week_of(now);
        tracing::info!(subject = %who.sub, force = req.force, wait = req.wait, "radar digest requested");
        if !req.wait {
            let started = worker.start_digest(now, req.force);
            return Ok(Response::new(RunDigestResponse {
                week,
                digests: Vec::new(),
                started,
                already_running: !started,
            }));
        }
        match worker.digest_now(now, req.force).await {
            Ok(Some((week, rows))) => Ok(Response::new(RunDigestResponse {
                week,
                digests: rows.into_iter().map(to_digest).collect(),
                started: true,
                already_running: false,
            })),
            Ok(None) => Ok(Response::new(RunDigestResponse {
                week,
                digests: Vec::new(),
                started: false,
                already_running: true,
            })),
            Err(RunError::NoWriter) => Err(self.reject(
                &mut timer,
                Status::failed_precondition("no llm service configured"),
            )),
            Err(RunError::Store(e)) => Err(self.reject(&mut timer, internal(e))),
        }
    }
}
