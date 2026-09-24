//! `tbd.cv.v1.CvService` implementation.
//!
//! `Ping` is the scaffold's stub and says so on the wire. Everything else is
//! real: a request is keyed by the subject Envoy verified, the owner is
//! whoever the token says is `admin`, and the document is rendered for the
//! one person downloading it. Every RPC first consults the fault handle in
//! [`Runtime`], so an embedder can make this service slow, failing or hung.

use std::{sync::Arc, time::Duration};

use chrono::{DateTime, Utc};
use sqlx::PgPool;
use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
    principal::Principal,
};
use tbd_proto::cv::v1::{
    AccessRequest, AccessState, DecideRequestRequest, DecideRequestResponse, DownloadCvRequest,
    DownloadCvResponse, GetAccessRequest, GetAccessResponse, ListRequestsRequest,
    ListRequestsResponse, PingRequest, PingResponse, RequestAccessRequest, RequestAccessResponse,
    cv_service_server::CvService,
};
use tonic::{Code, Request, Response, Status};
use uuid::Uuid;

use crate::{
    Runtime,
    config::Ping,
    notify::Notifier,
    private::Private,
    render::{self, Reader},
    store::{self, Decision, RequestRow, StoreError},
};

/// The role the token must carry to decide.
const OWNER_ROLE: &str = "admin";

/// The service. Cheap to clone; holds its configuration section and shared handles.
#[derive(Debug, Clone)]
pub struct Cv {
    ping: Ping,
    runtime: Runtime,
    pool: Option<PgPool>,
    private: Arc<Option<Private>>,
    notifier: Option<Arc<Notifier>>,
    render_timeout: Duration,
}

impl Cv {
    /// Build a service from its `[ping]` configuration, with no store: every
    /// RPC but `Ping` answers `UNAVAILABLE`.
    #[must_use]
    pub fn new(ping: Ping, runtime: Runtime) -> Self {
        Self {
            ping,
            runtime,
            pool: None,
            private: Arc::new(None),
            notifier: None,
            render_timeout: Duration::from_secs(20),
        }
    }

    /// With a database.
    #[must_use]
    pub fn with_pool(mut self, pool: PgPool) -> Self {
        self.pool = Some(pool);
        self
    }

    /// With the private fields the full CV carries.
    #[must_use]
    pub fn with_private(mut self, private: Option<Private>) -> Self {
        self.private = Arc::new(private);
        self
    }

    /// With a way to tell the owner.
    #[must_use]
    pub fn with_notifier(mut self, notifier: Option<Arc<Notifier>>) -> Self {
        self.notifier = notifier;
        self
    }

    /// With a cap on how long a render may take.
    #[must_use]
    pub const fn with_render_timeout(mut self, timeout: Duration) -> Self {
        self.render_timeout = timeout;
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

    fn pool(&self) -> Result<&PgPool, Status> {
        self.pool
            .as_ref()
            .ok_or_else(|| Status::unavailable("no store configured"))
    }

    /// Who is calling, from the claims Envoy verified and forwarded. Envoy
    /// strips `x-jwt-payload` from anything a client sends and is the only
    /// way in, so its presence means the token was checked.
    fn principal<T>(request: &Request<T>) -> Result<Principal, Status> {
        let headers = request.metadata().clone().into_headers();
        Principal::from_headers(&headers, &[])
            .ok_or_else(|| Status::unauthenticated("no verified caller"))
    }

    /// The caller, and only if they are the owner.
    fn owner<T>(request: &Request<T>) -> Result<Principal, Status> {
        let p = Self::principal(request)?;
        if p.role.as_deref() == Some(OWNER_ROLE) {
            Ok(p)
        } else {
            Err(Status::permission_denied("the owner only"))
        }
    }

    /// The person's standing as the page shows it.
    fn state(&self, row: Option<&RequestRow>) -> AccessState {
        let notifications = self.notifier.is_some();
        row.map_or_else(
            || AccessState {
                status: "none".into(),
                notifications,
                ..AccessState::default()
            },
            |r| AccessState {
                status: r.status.clone(),
                email: r.email.clone(),
                name: r.name.clone(),
                note: r.note.clone(),
                requested_at: when(Some(r.requested_at)),
                decided_at: when(r.decided_at),
                notified_at: when(r.notified_at),
                notifications,
            },
        )
    }
}

fn when(t: Option<DateTime<Utc>>) -> String {
    t.map(|t| t.to_rfc3339_opts(chrono::SecondsFormat::Secs, true))
        .unwrap_or_default()
}

fn wire(r: &RequestRow) -> AccessRequest {
    AccessRequest {
        id: r.id.to_string(),
        subject: r.subject.clone(),
        email: r.email.clone(),
        name: r.name.clone(),
        note: r.note.clone(),
        status: r.status.clone(),
        requested_at: when(Some(r.requested_at)),
        decided_at: when(r.decided_at),
        decided_by: r.decided_by.clone().unwrap_or_default(),
        notified_at: when(r.notified_at),
        downloads: i32::try_from(r.downloads).unwrap_or(i32::MAX),
        last_download_at: when(r.last_download_at),
    }
}

fn status_of(e: StoreError) -> Status {
    match e {
        StoreError::NotFound => Status::not_found("no such request"),
        StoreError::Refused(why) => Status::failed_precondition(why),
        StoreError::Db(e) => {
            tracing::error!(error = %e, "cv store");
            Status::unavailable("store unavailable")
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

/// A header the gateway forwarded, or empty.
fn header<T>(request: &Request<T>, name: &str) -> String {
    request
        .metadata()
        .get(name)
        .and_then(|v| v.to_str().ok())
        .map(str::trim)
        .unwrap_or_default()
        .to_owned()
}

#[tonic::async_trait]
impl CvService for Cv {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("CvService/Ping").await?;
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

    async fn get_access(
        &self,
        request: Request<GetAccessRequest>,
    ) -> Result<Response<GetAccessResponse>, Status> {
        let mut timer = self.admit("CvService/GetAccess").await?;
        let run = async {
            let who = Self::principal(&request)?;
            let row = store::get(self.pool()?, &who.sub)
                .await
                .map_err(status_of)?;
            Ok(GetAccessResponse {
                state: Some(self.state(row.as_ref())),
            })
        };
        match run.await {
            Ok(r) => Ok(Response::new(r)),
            Err(e) => Err(self.reject(&mut timer, e)),
        }
    }

    async fn request_access(
        &self,
        request: Request<RequestAccessRequest>,
    ) -> Result<Response<RequestAccessResponse>, Status> {
        let mut timer = self.admit("CvService/RequestAccess").await?;
        let run = async {
            let who = Self::principal(&request)?;
            let email = who
                .email
                .clone()
                .filter(|e| e.contains('@'))
                .ok_or_else(|| {
                    Status::invalid_argument(
                        "the token carries no e-mail address; sign in with one",
                    )
                })?;
            let name = who
                .name
                .clone()
                .unwrap_or_else(|| email.split('@').next().unwrap_or_default().to_owned());
            let note: String = request.get_ref().note.trim().chars().take(2000).collect();
            let pool = self.pool()?;
            let row = store::request(pool, &who.sub, &email, &name, &note)
                .await
                .map_err(status_of)?;
            // Tell the owner, off the request: a slow mailbox must not hold
            // the answer, and the internal listener would retry a slow call
            // and send twice. The row says when the mail went out.
            if row.status == store::REQUESTED
                && row.notified_at.is_none()
                && let Some(notifier) = self.notifier.clone()
            {
                let pool = pool.clone();
                let sent = row.clone();
                tokio::spawn(async move {
                    match notifier.owner_requested(&sent).await {
                        Ok(()) => {
                            if let Err(e) = store::mark_notified(&pool, sent.id).await {
                                tracing::warn!(error = %e, "could not record the notice");
                            }
                        }
                        Err(e) => {
                            tracing::warn!(error = %e, id = %sent.id, "the owner was not told");
                        }
                    }
                });
            }
            Ok(RequestAccessResponse {
                state: Some(self.state(Some(&row))),
            })
        };
        match run.await {
            Ok(r) => Ok(Response::new(r)),
            Err(e) => Err(self.reject(&mut timer, e)),
        }
    }

    async fn download_cv(
        &self,
        request: Request<DownloadCvRequest>,
    ) -> Result<Response<DownloadCvResponse>, Status> {
        let mut timer = self.admit("CvService/DownloadCv").await?;
        let run = async {
            let who = Self::principal(&request)?;
            let pool = self.pool()?;
            let row = store::get(pool, &who.sub)
                .await
                .map_err(status_of)?
                .filter(|r| r.status == store::APPROVED)
                .ok_or_else(|| {
                    Status::permission_denied("the full CV needs the owner's approval")
                })?;
            let reader = Reader {
                name: if row.name.is_empty() {
                    row.email.clone()
                } else {
                    row.name.clone()
                },
                email: row.email.clone(),
                date: Utc::now().format("%Y-%m-%d").to_string(),
            };
            let private = Arc::clone(&self.private);
            let ident = format!("inorbit-cv-{}", row.id);
            let rendered = tokio::time::timeout(
                self.render_timeout,
                tokio::task::spawn_blocking(move || {
                    render::render(private.as_ref().as_ref(), Some(&reader), &ident, Utc::now())
                }),
            )
            .await
            .map_err(|_| Status::deadline_exceeded("the render took too long"))?
            .map_err(|e| Status::internal(format!("render task: {e}")))?
            .map_err(|e| {
                tracing::error!(error = %e, "cv render");
                Status::internal("the CV could not be rendered")
            })?;
            let user_agent = header(&request, "user-agent");
            let ip = header(&request, "x-forwarded-for")
                .split(',')
                .next()
                .unwrap_or_default()
                .trim()
                .to_owned();
            store::record_download(pool, row.id, &user_agent, &ip)
                .await
                .map_err(status_of)?;
            tracing::info!(subject = %who.sub, pages = rendered.pages, "cv downloaded");
            Ok(DownloadCvResponse {
                content_type: "application/pdf".into(),
                pdf: rendered.pdf,
                filename: "nevio-vesic-cv.pdf".into(),
            })
        };
        match run.await {
            Ok(r) => Ok(Response::new(r)),
            Err(e) => Err(self.reject(&mut timer, e)),
        }
    }

    async fn list_requests(
        &self,
        request: Request<ListRequestsRequest>,
    ) -> Result<Response<ListRequestsResponse>, Status> {
        let mut timer = self.admit("CvService/ListRequests").await?;
        let run = async {
            Self::owner(&request)?;
            let status = request.get_ref().status.trim();
            let rows = store::list(self.pool()?, (!status.is_empty()).then_some(status))
                .await
                .map_err(status_of)?;
            Ok(ListRequestsResponse {
                requests: rows.iter().map(wire).collect(),
            })
        };
        match run.await {
            Ok(r) => Ok(Response::new(r)),
            Err(e) => Err(self.reject(&mut timer, e)),
        }
    }

    async fn decide_request(
        &self,
        request: Request<DecideRequestRequest>,
    ) -> Result<Response<DecideRequestResponse>, Status> {
        let mut timer = self.admit("CvService/DecideRequest").await?;
        let run = async {
            let owner = Self::owner(&request)?;
            let body = request.get_ref();
            let id = Uuid::parse_str(body.id.trim())
                .map_err(|_| Status::invalid_argument("id is not a uuid"))?;
            let decision = Decision::parse(body.decision.trim()).map_err(status_of)?;
            let by = owner.email.clone().unwrap_or_else(|| owner.sub.clone());
            let row = store::decide(self.pool()?, id, decision, &by)
                .await
                .map_err(status_of)?;
            if decision == Decision::Approve
                && let Some(notifier) = self.notifier.clone()
            {
                let approved = row.clone();
                tokio::spawn(async move {
                    if let Err(e) = notifier.requester_approved(&approved).await {
                        tracing::info!(error = %e, id = %approved.id, "the requester was not told");
                    }
                });
            }
            tracing::info!(id = %row.id, decision = decision.as_str(), by = %by, "cv request decided");
            Ok(DecideRequestResponse {
                request: Some(wire(&row)),
            })
        };
        match run.await {
            Ok(r) => Ok(Response::new(r)),
            Err(e) => Err(self.reject(&mut timer, e)),
        }
    }
}
