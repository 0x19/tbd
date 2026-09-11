//! `tbd.ledger.v1.LedgerService` implementation: a thin adapter over the
//! [`Store`]. Every RPC first consults the fault handle in [`Runtime`], so an
//! embedder can make this service slow, failing or hung at runtime; then it
//! translates the wire types, calls the store, and maps [`StoreError`] onto a
//! gRPC status. No business rule lives here.

use std::sync::Arc;

use tbd_common::{
    fault::{ErrorKind, Fault},
    metrics::{RequestTimer, names},
};
use tbd_proto::ledger::v1::{
    self as pb, AppendRequest, AppendResponse, CurrentRequest, CurrentResponse, EraseRequest,
    EraseResponse, HistoryRequest, HistoryResponse, PingRequest, PingResponse, RestoreRequest,
    RestoreResponse, RetractRequest, RetractResponse, ledger_service_server::LedgerService,
};
use tonic::{Code, Request, Response, Status};

use crate::{
    Runtime,
    config::{ErasureConfig, Ping},
    store::{
        Cursor, Envelope, Fact, NewFact, PathPattern, Query, ScopeId, Source, Store, StoreError,
        StoreKind, SubjectId, Timestamp,
    },
};

/// The service. Cheap to clone; holds its configuration section and shared handles.
#[derive(Clone)]
pub struct Ledger {
    ping: Ping,
    erasure: ErasureConfig,
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
    /// Build a service from its configuration sections and a store.
    #[must_use]
    pub fn new(
        ping: Ping,
        erasure: ErasureConfig,
        runtime: Runtime,
        store: Arc<dyn Store>,
    ) -> Self {
        Self {
            ping,
            erasure,
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

    /// Run a store call under an admitted request: every error is counted
    /// and mapped once, here.
    async fn call<T>(
        &self,
        route: &'static str,
        f: impl AsyncFnOnce() -> Result<T, Status>,
    ) -> Result<Response<T>, Status> {
        let mut timer = self.admit(route).await?;
        match f().await {
            Ok(v) => Ok(Response::new(v)),
            Err(status) => Err(self.reject(&mut timer, status)),
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

/// The one place a store error becomes a status.
fn status_of(e: &StoreError) -> Status {
    match e {
        StoreError::NotFound { .. } => Status::not_found(e.to_string()),
        StoreError::Erased => Status::failed_precondition(e.to_string()),
        StoreError::Invalid { .. } => Status::invalid_argument(e.to_string()),
        StoreError::Forbidden { .. } => Status::permission_denied(e.to_string()),
        StoreError::Conflict { .. } => Status::aborted(e.to_string()),
        StoreError::Unavailable { .. } => Status::unavailable(e.to_string()),
        StoreError::Internal(_) => {
            tracing::error!(error = %e, "store internal error");
            Status::internal("store error")
        }
    }
}

fn invalid(field: &str, reason: impl std::fmt::Display) -> Status {
    Status::invalid_argument(format!("invalid {field}: {reason}"))
}

fn subject_id(s: &str) -> Result<SubjectId, Status> {
    s.parse().map_err(|e| invalid("subject_id", e))
}

fn source_of(raw: i32) -> Result<Source, Status> {
    match pb::Source::try_from(raw) {
        Ok(pb::Source::Verified) => Ok(Source::Verified),
        Ok(pb::Source::Declared) => Ok(Source::Declared),
        Ok(pb::Source::Inferred) => Ok(Source::Inferred),
        Ok(pb::Source::Symbolic) => Ok(Source::Symbolic),
        Ok(pb::Source::Observed) => Ok(Source::Observed),
        Ok(pb::Source::Unspecified) | Err(_) => Err(invalid("source", "unspecified")),
    }
}

fn source_to(s: Source) -> i32 {
    let v = match s {
        Source::Verified => pb::Source::Verified,
        Source::Declared => pb::Source::Declared,
        Source::Inferred => pb::Source::Inferred,
        Source::Symbolic => pb::Source::Symbolic,
        Source::Observed => pb::Source::Observed,
    };
    v as i32
}

fn envelope_of(field: &str, e: Option<pb::Envelope>) -> Result<Envelope, Status> {
    let e = e.ok_or_else(|| invalid(field, "required"))?;
    let version = u16::try_from(e.version).map_err(|_| invalid(field, "version out of range"))?;
    Ok(Envelope {
        version,
        bytes: e.bytes,
    })
}

fn envelope_to(e: &Envelope) -> pb::Envelope {
    pb::Envelope {
        version: u32::from(e.version),
        bytes: e.bytes.clone(),
    }
}

fn time_of(field: &str, t: Option<prost_types::Timestamp>) -> Result<Option<Timestamp>, Status> {
    match t {
        None => Ok(None),
        Some(t) => {
            let nanos = u32::try_from(t.nanos).map_err(|_| invalid(field, "negative nanos"))?;
            chrono::DateTime::from_timestamp(t.seconds, nanos)
                .map(Some)
                .ok_or_else(|| invalid(field, "out of range"))
        }
    }
}

fn time_to(t: Timestamp) -> prost_types::Timestamp {
    prost_types::Timestamp {
        seconds: t.timestamp(),
        nanos: i32::try_from(t.timestamp_subsec_nanos()).unwrap_or(0),
    }
}

fn fact_to(f: Fact) -> pb::Fact {
    pb::Fact {
        subject_id: f.subject_id.to_string(),
        id: f.id.0,
        path: f.path,
        source: source_to(f.source),
        value: f.value.as_ref().map(envelope_to),
        origin: Some(envelope_to(&f.origin)),
        confidence: f.confidence,
        counterparty_id: f.counterparty_id.map(|c| c.to_string()),
        observed_at: Some(time_to(f.observed_at)),
        recorded_at: Some(time_to(f.recorded_at)),
        expires_at: f.expires_at.map(time_to),
        consent: f.consent.into_iter().map(|s| s.0).collect(),
        stub: f.stub,
    }
}

/// The scopes a read runs under: required and non-empty on the wire.
fn scopes_of(raw: Vec<String>) -> Result<Vec<ScopeId>, Status> {
    if raw.is_empty() {
        return Err(invalid("scopes", "at least one scope id"));
    }
    Ok(raw.into_iter().map(ScopeId).collect())
}

fn query_of(
    paths: &[String],
    sources: Vec<i32>,
    scopes: Vec<String>,
    cursor: String,
    limit: u32,
    at: Option<prost_types::Timestamp>,
) -> Result<Query, Status> {
    let paths = paths
        .iter()
        .map(|p| PathPattern::parse(p).map_err(|e| status_of(&e)))
        .collect::<Result<Vec<_>, _>>()?;
    let sources = sources
        .into_iter()
        .map(source_of)
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Query {
        paths,
        sources,
        scopes: Some(scopes_of(scopes)?),
        at: time_of("at", at)?,
        cursor: (!cursor.is_empty()).then(|| Cursor::from_wire(cursor)),
        limit,
    })
}

#[tonic::async_trait]
impl LedgerService for Ledger {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        let max = self.ping.max_message_len;
        let store = self.store.kind();
        self.call("LedgerService/Ping", async || {
            let message = request.into_inner().message;
            if message.len() > max {
                return Err(Status::invalid_argument(format!(
                    "message longer than {max} bytes"
                )));
            }
            tracing::debug!(len = message.len(), %store, "ping");
            Ok(PingResponse {
                message,
                version: tbd_common::VERSION.to_owned(),
                stub: false,
                store: store.as_str().to_owned(),
            })
        })
        .await
    }

    async fn append(
        &self,
        request: Request<AppendRequest>,
    ) -> Result<Response<AppendResponse>, Status> {
        self.call("LedgerService/Append", async || {
            let r = request.into_inner();
            let subject = subject_id(&r.subject_id)?;
            let observed_at = time_of("observed_at", r.observed_at)?
                .ok_or_else(|| invalid("observed_at", "required"))?;
            let counterparty_id = match r.counterparty_id.as_deref() {
                None | Some("") => None,
                Some(c) => Some(c.parse().map_err(|e| invalid("counterparty_id", e))?),
            };
            let fact = NewFact {
                path: r.path,
                source: source_of(r.source)?,
                value: envelope_of("value", r.value)?,
                origin: envelope_of("origin", r.origin)?,
                confidence: r.confidence,
                counterparty_id,
                observed_at,
                expires_at: time_of("expires_at", r.expires_at)?,
                consent: r.consent.into_iter().map(ScopeId).collect(),
                stub: r.stub,
                idempotency_key: (!r.idempotency_key.is_empty()).then_some(r.idempotency_key),
            };
            let appended = self
                .store
                .append(subject, fact)
                .await
                .map_err(|e| status_of(&e))?;
            Ok(AppendResponse {
                fact: Some(fact_to(appended.fact)),
                replayed: appended.replayed,
            })
        })
        .await
    }

    async fn current(
        &self,
        request: Request<CurrentRequest>,
    ) -> Result<Response<CurrentResponse>, Status> {
        self.call("LedgerService/Current", async || {
            let r = request.into_inner();
            let subject = subject_id(&r.subject_id)?;
            let query = query_of(&r.paths, r.sources, r.scopes, r.cursor, r.limit, None)?;
            let page = self
                .store
                .current(subject, &query)
                .await
                .map_err(|e| status_of(&e))?;
            Ok(CurrentResponse {
                facts: page.items.into_iter().map(fact_to).collect(),
                next: page.next.map(|c| c.as_str().to_owned()).unwrap_or_default(),
            })
        })
        .await
    }

    async fn history(
        &self,
        request: Request<HistoryRequest>,
    ) -> Result<Response<HistoryResponse>, Status> {
        self.call("LedgerService/History", async || {
            let r = request.into_inner();
            let subject = subject_id(&r.subject_id)?;
            let query = query_of(&r.paths, r.sources, r.scopes, r.cursor, r.limit, r.at)?;
            let page = self
                .store
                .history(subject, &query)
                .await
                .map_err(|e| status_of(&e))?;
            Ok(HistoryResponse {
                facts: page.items.into_iter().map(fact_to).collect(),
                next: page.next.map(|c| c.as_str().to_owned()).unwrap_or_default(),
            })
        })
        .await
    }

    async fn retract(
        &self,
        request: Request<RetractRequest>,
    ) -> Result<Response<RetractResponse>, Status> {
        self.call("LedgerService/Retract", async || {
            let r = request.into_inner();
            let subject = subject_id(&r.subject_id)?;
            let source = source_of(r.source)?;
            let origin = envelope_of("origin", r.origin)?;
            let tomb = self
                .store
                .retract(subject, &r.path, source, origin)
                .await
                .map_err(|e| status_of(&e))?;
            Ok(RetractResponse {
                tombstone: Some(fact_to(tomb)),
            })
        })
        .await
    }

    async fn erase(
        &self,
        request: Request<EraseRequest>,
    ) -> Result<Response<EraseResponse>, Status> {
        let grace =
            chrono::TimeDelta::from_std(self.erasure.grace).unwrap_or(chrono::TimeDelta::MAX);
        self.call("LedgerService/Erase", async || {
            let subject = subject_id(&request.into_inner().subject_id)?;
            let erasure = self
                .store
                .request_erasure(subject)
                .await
                .map_err(|e| status_of(&e))?;
            let executes_after = erasure
                .requested_at
                .checked_add_signed(grace)
                .unwrap_or(erasure.requested_at);
            Ok(EraseResponse {
                requested_at: Some(time_to(erasure.requested_at)),
                executes_after: Some(time_to(executes_after)),
            })
        })
        .await
    }

    async fn restore(
        &self,
        request: Request<RestoreRequest>,
    ) -> Result<Response<RestoreResponse>, Status> {
        self.call("LedgerService/Restore", async || {
            let subject = subject_id(&request.into_inner().subject_id)?;
            self.store
                .restore(subject)
                .await
                .map_err(|e| status_of(&e))?;
            Ok(RestoreResponse {})
        })
        .await
    }
}
