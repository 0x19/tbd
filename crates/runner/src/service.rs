//! `tbd.runner.v1.RunnerService` implementation: the sandbox's L2 (RFC 0010).
//!
//! `Run` is, in order: the fault handle, a verified caller (always) with the
//! required role (while the lab is private), the request's bounds, the caller's
//! daily allowance, a slot among the runs at once, and then the engine. One
//! `audit` line per run says who ran what language, how big, how it ended and
//! how long it took; never the source or what it printed. `Ping` stays the
//! scaffold's labelled stub.

use std::{sync::Arc, time::Instant};

use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
    principal::Principal,
};
use tbd_proto::runner::v1::{
    Language, LanguageInfo, ListLanguagesRequest, ListLanguagesResponse, PingRequest, PingResponse,
    RunRequest, RunResponse, Step, runner_service_server::RunnerService,
};
use tonic::{Code, Request, Response, Status};

use crate::{
    Runtime,
    config::{Config, Limits, Ping},
    engine::{self, Engine, EngineError, Lang},
    gate::{Allowance, Gate},
};

/// The service. Cheap to clone; holds its configuration sections and shared handles.
#[derive(Debug, Clone)]
pub struct Runner {
    ping: Ping,
    runtime: Runtime,
    require_role: String,
    limits: Limits,
    engine: Arc<dyn Engine>,
    gate: Gate,
    allowance: Allowance,
}

impl Runner {
    /// Build a service over an engine.
    #[must_use]
    pub fn new(config: &Config, runtime: Runtime, engine: Box<dyn Engine>) -> Self {
        Self {
            ping: config.ping.clone(),
            runtime,
            require_role: config.access.require_role.trim().to_owned(),
            limits: config.limits.clone(),
            engine: Arc::from(engine),
            gate: Gate::new(&config.admission),
            allowance: Allowance::new(&config.budget),
        }
    }

    /// The verified caller, with the required role.
    fn caller<T>(&self, request: &Request<T>) -> Result<Principal, Status> {
        let headers = request.metadata().clone().into_headers();
        let caller = Principal::from_headers(&headers, &[])
            .ok_or_else(|| Status::unauthenticated("no verified caller"))?;
        if !self.require_role.is_empty()
            && caller.role.as_deref() != Some(self.require_role.as_str())
        {
            return Err(Status::permission_denied(format!(
                "running code is for the {} role while the lab is private",
                self.require_role
            )));
        }
        Ok(caller)
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

fn step(s: engine::Step) -> Step {
    Step {
        exit_code: s.exit_code,
        stdout: s.stdout,
        stderr: s.stderr,
        truncated: s.truncated,
        wall_ms: s.wall_ms,
        killed: s.killed,
    }
}

fn lang(wire: i32) -> Result<(Lang, Language), Status> {
    match Language::try_from(wire) {
        Ok(Language::Go) => Ok((Lang::Go, Language::Go)),
        Ok(Language::Rust) => Ok((Lang::Rust, Language::Rust)),
        _ => Err(Status::invalid_argument("language: go or rust")),
    }
}

#[tonic::async_trait]
impl RunnerService for Runner {
    async fn run(&self, request: Request<RunRequest>) -> Result<Response<RunResponse>, Status> {
        let mut timer = self.admit("RunnerService/Run").await?;
        let caller = self
            .caller(&request)
            .map_err(|s| self.reject(&mut timer, s))?;
        let req = request.into_inner();
        let (lang, wire) = lang(req.language).map_err(|s| self.reject(&mut timer, s))?;
        let bound = |field: &str, len: usize, max: usize| {
            (len > max)
                .then(|| Status::invalid_argument(format!("{field}: longer than {max} bytes")))
        };
        if req.source.trim().is_empty() {
            return Err(self.reject(&mut timer, Status::invalid_argument("source: empty")));
        }
        if let Some(s) = bound("source", req.source.len(), self.limits.max_source_bytes)
            .or_else(|| bound("stdin", req.stdin.len(), self.limits.max_stdin_bytes))
        {
            return Err(self.reject(&mut timer, s));
        }
        let left = self
            .allowance
            .take(&caller.sub)
            .map_err(|m| self.reject(&mut timer, Status::resource_exhausted(m)))?;
        let _slot = self
            .gate
            .enter()
            .await
            .map_err(|m| self.reject(&mut timer, Status::resource_exhausted(m)))?;

        let started = Instant::now();
        let source_bytes = req.source.len();
        let answer = self
            .engine
            .run(engine::Request {
                language: lang,
                source: req.source,
                stdin: req.stdin,
            })
            .await;
        let elapsed_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
        let outcome = answer
            .as_ref()
            .map_or("unavailable", |r| r.outcome.as_str())
            .to_owned();
        // The audit trail: who ran what, how big, how it ended. Never the code
        // or what it printed, which are the caller's.
        tracing::info!(
            target: "audit",
            service = "runner",
            subject = %caller.sub,
            caller = caller.kind_slug(),
            language = lang.as_str(),
            source_bytes,
            outcome = %outcome,
            elapsed_ms,
            "code run"
        );
        metrics::counter!(names::RUNNER_RUNS_TOTAL, "language" => lang.as_str(), "outcome" => outcome.clone())
            .increment(1);
        metrics::histogram!(names::RUNNER_DURATION, "language" => lang.as_str())
            .record(started.elapsed().as_secs_f64());
        let r = answer.map_err(|e| {
            let status = match e {
                EngineError::Invalid(m) => Status::invalid_argument(m),
                EngineError::Busy(m) => Status::resource_exhausted(format!("busy: {m}")),
                EngineError::Unavailable(m) => {
                    tracing::error!(error = %m, "the sandbox did not answer");
                    Status::unavailable("the sandbox is unavailable; try again shortly")
                }
            };
            self.reject(&mut timer, status)
        })?;
        Ok(Response::new(RunResponse {
            id: r.id,
            language: wire as i32,
            outcome: r.outcome,
            compile: Some(step(r.compile)),
            run: r.run.map(step),
            total_ms: r.total_ms,
            stub: self.engine.stub(),
            runs_left_today: left.unwrap_or(u32::MAX),
        }))
    }

    async fn list_languages(
        &self,
        request: Request<ListLanguagesRequest>,
    ) -> Result<Response<ListLanguagesResponse>, Status> {
        let _timer = self.admit("RunnerService/ListLanguages").await?;
        let headers = request.metadata().clone().into_headers();
        let caller = Principal::from_headers(&headers, &[]);
        let count = |n: usize| u32::try_from(n).unwrap_or(u32::MAX);
        Ok(Response::new(ListLanguagesResponse {
            languages: [(Language::Go, "go"), (Language::Rust, "rust")]
                .into_iter()
                .map(|(language, name)| LanguageInfo {
                    language: language as i32,
                    name: name.to_owned(),
                })
                .collect(),
            runs_left_today: caller.map(|c| self.allowance.left(&c.sub).unwrap_or(u32::MAX)),
            in_flight: count(self.gate.in_flight()),
            max_in_flight: count(self.gate.max()),
            stub: self.engine.stub(),
        }))
    }

    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("RunnerService/Ping").await?;
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
