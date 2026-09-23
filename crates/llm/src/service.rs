//! `tbd.llm.v1.LlmService` implementation: the L2.
//!
//! Every RPC first consults the fault handle in [`Runtime`], so an embedder can
//! make this service slow, failing or hung at runtime. Nothing here knows an
//! engine's wire format; the engines are behind [`Engine`] and chosen by tier.
//!
//! A generation's failures have two placements and both are the contract:
//! before the first chunk the RPC itself fails with a status; after it, the
//! stream carries chunks and then one error item. The deadline is the tier's
//! `timeout_secs`, measured from admission, and covers both.

use std::{
    collections::BTreeMap,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

use futures::{StreamExt as _, stream::BoxStream};
use sqlx::PgPool;
use tbd_common::{
    fault::{ErrorKind, Fault, FaultHandle},
    metrics::{RequestTimer, StreamGuard, names},
    principal::Principal,
};
use tbd_proto::llm::v1::{
    EmbedRequest, EmbedResponse, Embedding, GenerateRequest, GenerateResponse, GetBudgetRequest,
    GetBudgetResponse, ListModelsRequest, ListModelsResponse, ModelInfo, PingRequest, PingResponse,
    Tier as ProtoTier, Usage as ProtoUsage, llm_service_server::LlmService,
};
use tonic::{Code, Request, Response, Status};
use tracing::Instrument as _;
use uuid::Uuid;

use crate::{
    Runtime,
    config::{Budget, Config, Generate, Ping, Tier},
    engine::{ChunkStream, Engine, EngineError, GenerateSpec, Message, Usage},
    probe,
    store::{self, Outcome, StoreError},
};

/// The service. Cheap to clone; holds its configuration sections and shared handles.
#[derive(Debug, Clone)]
pub struct Llm {
    ping: Ping,
    runtime: Runtime,
    inner: Arc<Inner>,
}

#[derive(Debug)]
struct Inner {
    engines: BTreeMap<Tier, Arc<dyn Engine>>,
    timeouts: BTreeMap<Tier, Duration>,
    default_tier: Tier,
    limits: Generate,
    budget: Budget,
    probe: probe::Status,
    /// The record; `None` runs without one (nothing recorded, no limit).
    pool: Option<PgPool>,
}

impl Llm {
    /// Build a service from its configuration and the engines built for each tier.
    #[must_use]
    pub fn new(
        config: &Config,
        runtime: Runtime,
        engines: BTreeMap<Tier, Arc<dyn Engine>>,
        probe: probe::Status,
    ) -> Self {
        let timeouts = Tier::ALL
            .into_iter()
            .map(|t| (t, config.engine(t).timeout()))
            .collect();
        Self {
            ping: config.ping.clone(),
            runtime,
            inner: Arc::new(Inner {
                engines,
                timeouts,
                default_tier: config.engines.default_tier,
                limits: config.generate.clone(),
                budget: config.budget.clone(),
                probe,
                pool: None,
            }),
        }
    }

    /// Record sessions and generations in this pool, and enforce the budget
    /// from it. Call before the service is shared.
    #[must_use]
    pub fn with_pool(mut self, pool: PgPool) -> Self {
        if let Some(inner) = Arc::get_mut(&mut self.inner) {
            inner.pool = Some(pool);
        }
        self
    }

    /// Tokens the caller spent today, when there is a record.
    async fn used_today(&self, subject: &str) -> Result<Option<u64>, Status> {
        let Some(pool) = &self.inner.pool else {
            return Ok(None);
        };
        let since = store::day_start(chrono::Utc::now());
        store::used_since(pool, subject, since)
            .await
            .map(Some)
            .map_err(store_status)
    }

    /// Refuse a caller over their daily budget. One generation in flight may
    /// overrun: tokens are known only when it ends.
    async fn admit_budget(&self, subject: &str) -> Result<(), Status> {
        let limit = self.inner.budget.tokens_per_day;
        if limit == 0 {
            return Ok(());
        }
        if let Some(used) = self.used_today(subject).await?
            && used >= limit
        {
            return Err(Status::resource_exhausted(format!(
                "daily token budget spent: {used} of {limit}; it resets at 00:00 UTC"
            )));
        }
        Ok(())
    }

    /// The generation's session and row, when there is a record.
    async fn record_start(
        &self,
        subject: &str,
        wanted: &str,
        tier: Tier,
        engine: &Arc<dyn Engine>,
    ) -> Result<Option<Recording>, Status> {
        let Some(pool) = &self.inner.pool else {
            return Ok(None);
        };
        let wanted = if wanted.is_empty() {
            None
        } else {
            Some(
                Uuid::parse_str(wanted)
                    .map_err(|_| Status::invalid_argument("session_id is not a UUID"))?,
            )
        };
        let session_id = store::touch_session(pool, subject, wanted)
            .await
            .map_err(store_status)?;
        let id = Uuid::now_v7();
        let identity = self.inner.probe.identity(tier);
        store::start_generation(
            pool,
            store::Start {
                id,
                session_id,
                subject,
                tier,
                engine: engine.kind().as_str(),
                model: engine.model(),
                stub: engine.stub(),
                engine_version: &identity.engine_version,
                model_revision: &identity.model_revision,
            },
        )
        .await
        .map_err(store_status)?;
        Ok(Some(Recording {
            pool: pool.clone(),
            id,
            session_id,
            first_token_ms: None,
            closed: false,
        }))
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

    /// The verified caller, or `UNAUTHENTICATED`.
    fn principal<T>(request: &Request<T>) -> Result<Principal, Status> {
        let headers = request.metadata().clone().into_headers();
        Principal::from_headers(&headers, &[])
            .ok_or_else(|| Status::unauthenticated("no verified caller"))
    }

    /// The tier a request named, or the default.
    fn tier(&self, wire: i32) -> Result<Tier, Status> {
        match ProtoTier::try_from(wire) {
            Ok(ProtoTier::Unspecified) => Ok(self.inner.default_tier),
            Ok(ProtoTier::Fast) => Ok(Tier::Fast),
            Ok(ProtoTier::Deep) => Ok(Tier::Deep),
            Err(_) => Err(Status::invalid_argument(format!("unknown tier {wire}"))),
        }
    }

    fn engine(&self, tier: Tier) -> Result<(&Arc<dyn Engine>, Duration), Status> {
        let engine = self
            .inner
            .engines
            .get(&tier)
            .ok_or_else(|| Status::failed_precondition(format!("tier {tier} has no engine")))?;
        let timeout = self
            .inner
            .timeouts
            .get(&tier)
            .copied()
            .unwrap_or(Duration::from_secs(60));
        Ok((engine, timeout))
    }

    /// The request as an engine-neutral spec, or `INVALID_ARGUMENT`.
    fn spec(&self, req: &GenerateRequest) -> Result<GenerateSpec, Status> {
        let limits = &self.inner.limits;
        if req.messages.is_empty() {
            return Err(Status::invalid_argument("at least one message is required"));
        }
        if req.messages.len() > limits.max_messages {
            return Err(Status::invalid_argument(format!(
                "more than {} messages",
                limits.max_messages
            )));
        }
        let mut messages = Vec::with_capacity(req.messages.len());
        for (i, m) in req.messages.iter().enumerate() {
            if !matches!(m.role.as_str(), "system" | "user" | "assistant") {
                return Err(Status::invalid_argument(format!(
                    "message {i}: role must be system, user or assistant"
                )));
            }
            if m.content.len() > limits.max_message_len {
                return Err(Status::invalid_argument(format!(
                    "message {i}: longer than {} bytes",
                    limits.max_message_len
                )));
            }
            messages.push(Message {
                role: m.role.clone(),
                content: m.content.clone(),
            });
        }
        if let Some(t) = req.temperature
            && !(0.0..=2.0).contains(&t)
        {
            return Err(Status::invalid_argument(
                "temperature must be within 0 and 2",
            ));
        }
        let max_tokens = Some(
            req.max_tokens
                .unwrap_or(limits.max_tokens_cap)
                .min(limits.max_tokens_cap),
        );
        Ok(GenerateSpec {
            messages,
            max_tokens,
            temperature: req.temperature,
            reasoning: req.reasoning,
        })
    }

    fn today() -> String {
        chrono::Utc::now().format("%Y-%m-%d").to_string()
    }
}

/// Map the store's failures onto a gRPC status.
fn store_status(e: StoreError) -> Status {
    match e {
        StoreError::NotFound => Status::not_found("no such session"),
        StoreError::Db(e) => {
            tracing::error!(error = %e, "llm store");
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

/// Map an engine's failure onto a gRPC status. The one place this is decided.
fn status_of(e: &EngineError) -> Status {
    match e {
        EngineError::Unavailable(m) => Status::unavailable(format!("engine unavailable: {m}")),
        EngineError::Refused(m) => Status::failed_precondition(format!("engine refused: {m}")),
        EngineError::Protocol(m) => Status::internal(format!("engine protocol: {m}")),
        EngineError::Timeout => Status::deadline_exceeded("the engine did not answer in time"),
    }
}

fn wire_tier(tier: Tier) -> i32 {
    match tier {
        Tier::Fast => ProtoTier::Fast as i32,
        Tier::Deep => ProtoTier::Deep as i32,
    }
}

/// What every chunk of one generation carries.
#[derive(Debug, Clone)]
struct Meta {
    engine: &'static str,
    model: String,
    tier: Tier,
    stub: bool,
    session_id: String,
    generation_id: String,
}

/// The row of one generation while it runs. Closed with its outcome by the
/// stream; a caller who goes away first drops it unclosed, and the drop
/// records `cancelled` so no row is left `running` by a disconnect.
struct Recording {
    pool: PgPool,
    id: Uuid,
    session_id: Uuid,
    first_token_ms: Option<i32>,
    closed: bool,
}

impl Recording {
    fn close(&mut self, outcome: Outcome, error: String, usage: Option<Usage>) {
        if self.closed {
            return;
        }
        self.closed = true;
        let pool = self.pool.clone();
        let id = self.id;
        let first = self.first_token_ms;
        tokio::spawn(async move {
            if let Err(e) = store::finish_generation(&pool, id, outcome, &error, usage, first).await
            {
                tracing::warn!(%id, error = %e, "recording the generation's end");
            }
        });
    }
}

impl Drop for Recording {
    fn drop(&mut self) {
        if !self.closed && tokio::runtime::Handle::try_current().is_ok() {
            self.close(Outcome::Cancelled, "the caller went away".to_owned(), None);
        }
    }
}

/// The state of one live generation, moved through the unfold.
struct Live {
    chunks: ChunkStream,
    deadline: Pin<Box<tokio::time::Sleep>>,
    started: Instant,
    index: u32,
    first_seen: bool,
    over: bool,
    meta: Meta,
    guard: StreamGuard,
    fault: FaultHandle,
    span: tracing::Span,
    recording: Option<Recording>,
}

impl Live {
    fn response(
        &self,
        text: String,
        done: bool,
        usage: Option<ProtoUsage>,
        model: Option<String>,
        reasoning: bool,
    ) -> GenerateResponse {
        GenerateResponse {
            text,
            index: self.index,
            done,
            usage,
            reasoning,
            engine: self.meta.engine.to_owned(),
            model: model.unwrap_or_else(|| self.meta.model.clone()),
            tier: wire_tier(self.meta.tier),
            stub: self.meta.stub,
            session_id: self.meta.session_id.clone(),
            generation_id: self.meta.generation_id.clone(),
        }
    }

    fn end(&mut self, status: Status) -> Result<GenerateResponse, Status> {
        self.over = true;
        self.span.in_scope(
            || tracing::info!(code = ?status.code(), chunks = self.index, "generation failed"),
        );
        if let Some(r) = self.recording.as_mut() {
            r.close(Outcome::Failed, status.message().to_owned(), None);
        }
        Err(status)
    }
}

/// Wrap an engine's chunks as responses, enforcing the deadline, recording
/// the metrics and ending the stream after the done chunk or the first error.
fn live_stream(live: Live) -> BoxStream<'static, Result<GenerateResponse, Status>> {
    futures::stream::unfold(live, |mut st| async move {
        if st.over {
            return None;
        }
        let item = tokio::select! {
            biased;
            () = &mut st.deadline => {
                st.end(Status::deadline_exceeded("the generation did not finish in time"))
            }
            next = st.chunks.next().instrument(st.span.clone()) => match next {
                None => st.end(Status::unavailable("the engine ended the stream early")),
                Some(Err(e)) => st.end(status_of(&e)),
                Some(Ok(chunk)) => {
                    if let Some(injected) = st.fault.stream_error() {
                        st.end(status_from(injected))
                    } else {
                        st.guard.item("out");
                        if !chunk.text.is_empty() && !st.first_seen {
                            st.first_seen = true;
                            let elapsed = st.started.elapsed();
                            metrics::histogram!(names::LLM_TIME_TO_FIRST_TOKEN, "tier" => st.meta.tier.as_str(), "engine" => st.meta.engine)
                                .record(elapsed.as_secs_f64());
                            if let Some(r) = st.recording.as_mut() {
                                r.first_token_ms =
                                    Some(i32::try_from(elapsed.as_millis()).unwrap_or(i32::MAX));
                            }
                        }
                        let usage = chunk.usage.map(|u| ProtoUsage {
                            prompt_tokens: u.prompt_tokens,
                            completion_tokens: u.completion_tokens,
                        });
                        if chunk.done {
                            st.over = true;
                            if let Some(r) = st.recording.as_mut() {
                                r.close(Outcome::Ok, String::new(), chunk.usage);
                            }
                            if let Some(u) = chunk.usage {
                                for (kind, n) in [("prompt", u.prompt_tokens), ("completion", u.completion_tokens)] {
                                    metrics::counter!(names::LLM_TOKENS_TOTAL, "tier" => st.meta.tier.as_str(), "engine" => st.meta.engine, "model" => st.meta.model.clone(), "kind" => kind)
                                        .increment(u64::from(n));
                                }
                            }
                            st.span.in_scope(|| tracing::info!(chunks = st.index + 1, usage = ?chunk.usage, elapsed_ms = st.started.elapsed().as_millis(), "generation done"));
                        }
                        let resp = st.response(
                            chunk.text,
                            chunk.done,
                            usage,
                            chunk.model,
                            chunk.reasoning,
                        );
                        st.index += 1;
                        Ok(resp)
                    }
                }
            },
        };
        Some((item, st))
    })
    .boxed()
}

#[tonic::async_trait]
impl LlmService for Llm {
    type GenerateStream = BoxStream<'static, Result<GenerateResponse, Status>>;

    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let mut timer = self.admit("LlmService/Ping").await?;
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

    async fn generate(
        &self,
        request: Request<GenerateRequest>,
    ) -> Result<Response<Self::GenerateStream>, Status> {
        let mut timer = self.admit("LlmService/Generate").await?;
        let started = Instant::now();
        let principal = Self::principal(&request).map_err(|s| self.reject(&mut timer, s))?;
        let req = request.into_inner();
        let chosen = self
            .tier(req.tier)
            .map_err(|s| self.reject(&mut timer, s))?;
        let spec = self.spec(&req).map_err(|s| self.reject(&mut timer, s))?;
        let (engine, timeout) = self
            .engine(chosen)
            .map_err(|s| self.reject(&mut timer, s))?;
        self.admit_budget(&principal.sub)
            .await
            .map_err(|s| self.reject(&mut timer, s))?;
        let recording = self
            .record_start(&principal.sub, &req.session_id, chosen, engine)
            .await
            .map_err(|s| self.reject(&mut timer, s))?;
        let meta = Meta {
            engine: engine.kind().as_str(),
            model: engine.model().to_owned(),
            tier: chosen,
            stub: engine.stub(),
            session_id: recording
                .as_ref()
                .map(|r| r.session_id.to_string())
                .unwrap_or_default(),
            generation_id: recording
                .as_ref()
                .map(|r| r.id.to_string())
                .unwrap_or_default(),
        };
        let span = tracing::info_span!(
            "llm.generate",
            tier = %chosen,
            engine = meta.engine,
            model = %meta.model,
            subject = %principal.sub,
            messages = spec.messages.len(),
        );

        let mut recording = recording;
        let chunks = match tokio::time::timeout(timeout, engine.generate(spec))
            .instrument(span.clone())
            .await
        {
            Ok(Ok(chunks)) => chunks,
            Ok(Err(e)) => {
                let status = status_of(&e);
                if let Some(r) = recording.as_mut() {
                    r.close(Outcome::Failed, status.message().to_owned(), None);
                }
                return Err(self.reject(&mut timer, status));
            }
            Err(_) => {
                let status =
                    Status::deadline_exceeded("the engine did not start answering in time");
                if let Some(r) = recording.as_mut() {
                    r.close(Outcome::Failed, status.message().to_owned(), None);
                }
                return Err(self.reject(&mut timer, status));
            }
        };
        let remaining = timeout.saturating_sub(started.elapsed());
        let live = Live {
            chunks,
            deadline: Box::pin(tokio::time::sleep(remaining)),
            started,
            index: 0,
            first_seen: false,
            over: false,
            meta,
            guard: StreamGuard::open("llm_generate"),
            fault: self.runtime.fault.clone(),
            span,
            recording,
        };
        Ok(Response::new(live_stream(live)))
    }

    async fn embed(
        &self,
        request: Request<EmbedRequest>,
    ) -> Result<Response<EmbedResponse>, Status> {
        let mut timer = self.admit("LlmService/Embed").await?;
        Self::principal(&request).map_err(|s| self.reject(&mut timer, s))?;
        let req = request.into_inner();
        let chosen = self
            .tier(req.tier)
            .map_err(|s| self.reject(&mut timer, s))?;
        let limits = &self.inner.limits;
        if req.inputs.is_empty() {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument("at least one input is required"),
            ));
        }
        if req.inputs.len() > limits.max_embed_inputs {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument(format!("more than {} inputs", limits.max_embed_inputs)),
            ));
        }
        if let Some(i) = req
            .inputs
            .iter()
            .position(|s| s.len() > limits.max_message_len)
        {
            return Err(self.reject(
                &mut timer,
                Status::invalid_argument(format!(
                    "input {i}: longer than {} bytes",
                    limits.max_message_len
                )),
            ));
        }
        let (engine, timeout) = self
            .engine(chosen)
            .map_err(|s| self.reject(&mut timer, s))?;
        if !engine.embeds() {
            return Err(self.reject(
                &mut timer,
                Status::failed_precondition(format!(
                    "tier {chosen} has no embedding model; set [engines.{chosen}] embed_model"
                )),
            ));
        }
        let vectors = match tokio::time::timeout(timeout, engine.embed(req.inputs)).await {
            Ok(Ok(v)) => v,
            Ok(Err(e)) => return Err(self.reject(&mut timer, status_of(&e))),
            Err(_) => {
                return Err(self.reject(
                    &mut timer,
                    Status::deadline_exceeded("the engine did not embed in time"),
                ));
            }
        };
        Ok(Response::new(EmbedResponse {
            vectors: vectors
                .into_iter()
                .map(|values| Embedding { values })
                .collect(),
            engine: engine.kind().as_str().to_owned(),
            model: engine.embed_model().to_owned(),
            tier: wire_tier(chosen),
            stub: engine.stub(),
        }))
    }

    async fn list_models(
        &self,
        _request: Request<ListModelsRequest>,
    ) -> Result<Response<ListModelsResponse>, Status> {
        let _timer = self.admit("LlmService/ListModels").await?;
        let models = Tier::ALL
            .into_iter()
            .filter_map(|tier| {
                let engine = self.inner.engines.get(&tier)?;
                let identity = self.inner.probe.identity(tier);
                Some(ModelInfo {
                    tier: wire_tier(tier),
                    engine: engine.kind().as_str().to_owned(),
                    model: engine.model().to_owned(),
                    up: self.inner.probe.up(tier),
                    stub: engine.stub(),
                    engine_version: identity.engine_version,
                    model_revision: identity.model_revision,
                    embeds: engine.embeds(),
                })
            })
            .collect();
        Ok(Response::new(ListModelsResponse {
            models,
            default_tier: wire_tier(self.inner.default_tier),
        }))
    }

    async fn get_budget(
        &self,
        request: Request<GetBudgetRequest>,
    ) -> Result<Response<GetBudgetResponse>, Status> {
        let mut timer = self.admit("LlmService/GetBudget").await?;
        let principal = Self::principal(&request).map_err(|s| self.reject(&mut timer, s))?;
        let limit = self.inner.budget.tokens_per_day;
        // Without a store nothing is recorded, so no limit can be enforced and
        // the answer says so rather than pretending a count.
        let used = self
            .used_today(&principal.sub)
            .await
            .map_err(|s| self.reject(&mut timer, s))?;
        Ok(Response::new(GetBudgetResponse {
            tokens_per_day: limit,
            used_today: used.unwrap_or(0),
            remaining: used.map_or(0, |u| limit.saturating_sub(u)),
            unlimited: limit == 0 || used.is_none(),
            recorded: used.is_some(),
            day: Self::today(),
        }))
    }
}
