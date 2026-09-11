//! The ledger as the workers see it: seven calls that never panic and never
//! lose the reason they failed. One implementation, gRPC over a channel the
//! caller built (chaos decides trust; this crate only adds the bearer).

use std::time::Duration;

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use tbd_proto::ledger::v1::{
    AppendRequest, AppendResponse, CurrentRequest, CurrentResponse, EraseRequest, EraseResponse,
    HistoryRequest, HistoryResponse, PingRequest, PingResponse, RestoreRequest, RestoreResponse,
    RetractRequest, RetractResponse, ledger_service_client::LedgerServiceClient,
};
use tonic::{
    Code, Request, Status,
    metadata::{Ascii, MetadataValue},
    service::{Interceptor, interceptor::InterceptedService},
    transport::Channel,
};

/// Why a call failed. Serialised into findings, so a reader sees the status
/// the ledger answered with, not a Rust type name.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, thiserror::Error)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum CallError {
    /// A gRPC status.
    #[error("{code}: {message}")]
    Status {
        /// `tonic::Code` as its `Debug` name, e.g. `NotFound`.
        code: String,
        /// The message.
        message: String,
    },
    /// The channel could not be built or the connection broke before a status.
    #[error("transport: {0}")]
    Transport(String),
    /// The caller's timeout elapsed.
    #[error("timeout")]
    Timeout,
}

/// Every class [`CallError::class`] can produce; `[faults] tolerate` names these.
pub const KNOWN_CLASSES: &[&str] = &[
    "transport",
    "timeout",
    "cancelled",
    "unknown",
    "invalid_argument",
    "deadline_exceeded",
    "not_found",
    "already_exists",
    "permission_denied",
    "resource_exhausted",
    "failed_precondition",
    "aborted",
    "out_of_range",
    "unimplemented",
    "internal",
    "unavailable",
    "data_loss",
    "unauthenticated",
];

impl CallError {
    /// From a status the ledger answered with.
    #[must_use]
    pub fn from_status(s: &Status) -> Self {
        Self::Status {
            code: format!("{:?}", s.code()),
            message: s.message().to_owned(),
        }
    }

    /// The gRPC code, when there is one.
    #[must_use]
    pub fn code(&self) -> Option<Code> {
        match self {
            Self::Status { code, .. } => code_from_name(code),
            Self::Transport(_) | Self::Timeout => None,
        }
    }

    /// A stable class for reports and `[faults] tolerate`: the code in
    /// `snake_case`, or `transport` / `timeout`.
    #[must_use]
    pub fn class(&self) -> String {
        match self {
            Self::Status { code, .. } => snake(code),
            Self::Transport(_) => "transport".into(),
            Self::Timeout => "timeout".into(),
        }
    }

    /// Whether the ledger broke its contract by the way it failed: an internal
    /// error, an unknown one, or data loss is never a valid answer.
    #[must_use]
    pub fn is_unclean(&self) -> bool {
        matches!(
            self.code(),
            Some(Code::Internal | Code::Unknown | Code::DataLoss)
        )
    }
}

fn snake(camel: &str) -> String {
    let mut out = String::with_capacity(camel.len() + 4);
    for (i, ch) in camel.chars().enumerate() {
        if ch.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(ch.to_ascii_lowercase());
        } else {
            out.push(ch);
        }
    }
    out
}

fn code_from_name(name: &str) -> Option<Code> {
    Some(match name {
        "Ok" => Code::Ok,
        "Cancelled" => Code::Cancelled,
        "Unknown" => Code::Unknown,
        "InvalidArgument" => Code::InvalidArgument,
        "DeadlineExceeded" => Code::DeadlineExceeded,
        "NotFound" => Code::NotFound,
        "AlreadyExists" => Code::AlreadyExists,
        "PermissionDenied" => Code::PermissionDenied,
        "ResourceExhausted" => Code::ResourceExhausted,
        "FailedPrecondition" => Code::FailedPrecondition,
        "Aborted" => Code::Aborted,
        "OutOfRange" => Code::OutOfRange,
        "Unimplemented" => Code::Unimplemented,
        "Internal" => Code::Internal,
        "Unavailable" => Code::Unavailable,
        "DataLoss" => Code::DataLoss,
        "Unauthenticated" => Code::Unauthenticated,
        _ => return None,
    })
}

/// The seven RPCs. Implemented by [`GrpcLedger`]; tests wrap it to lie.
#[async_trait]
pub trait LedgerClient: Send + Sync + 'static {
    /// Ping.
    async fn ping(&self, req: PingRequest) -> Result<PingResponse, CallError>;
    /// Append.
    async fn append(&self, req: AppendRequest) -> Result<AppendResponse, CallError>;
    /// Current.
    async fn current(&self, req: CurrentRequest) -> Result<CurrentResponse, CallError>;
    /// History.
    async fn history(&self, req: HistoryRequest) -> Result<HistoryResponse, CallError>;
    /// Retract.
    async fn retract(&self, req: RetractRequest) -> Result<RetractResponse, CallError>;
    /// Erase.
    async fn erase(&self, req: EraseRequest) -> Result<EraseResponse, CallError>;
    /// Restore.
    async fn restore(&self, req: RestoreRequest) -> Result<RestoreResponse, CallError>;
}

/// One ledger instance to run against. Both a stack the caller booted and a
/// deployed pod behind Envoy are gRPC, so one client type suffices.
#[derive(Clone)]
pub struct Target {
    /// Shown in reports and findings.
    pub name: String,
    /// The channel, connected lazily by the caller with its own trust.
    pub channel: Channel,
    /// `Authorization: Bearer ...` for a deployed target; none in-process.
    pub bearer: Option<String>,
}

impl std::fmt::Debug for Target {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Target")
            .field("name", &self.name)
            .field("bearer", &self.bearer.is_some())
            .finish_non_exhaustive()
    }
}

/// Inserts the bearer, if any, on every request.
#[derive(Clone)]
pub struct Bearer(Option<MetadataValue<Ascii>>);

impl Interceptor for Bearer {
    fn call(&mut self, mut req: Request<()>) -> Result<Request<()>, Status> {
        if let Some(v) = &self.0 {
            req.metadata_mut().insert("authorization", v.clone());
        }
        Ok(req)
    }
}

/// [`LedgerClient`] over gRPC.
#[derive(Clone)]
pub struct GrpcLedger {
    client: LedgerServiceClient<InterceptedService<Channel, Bearer>>,
    timeout: Duration,
}

impl GrpcLedger {
    /// A client on `target` with a per-call timeout.
    #[must_use]
    pub fn new(target: &Target, timeout: Duration) -> Self {
        let bearer = Bearer(
            target
                .bearer
                .as_deref()
                .and_then(|v| MetadataValue::try_from(v).ok()),
        );
        Self {
            client: LedgerServiceClient::with_interceptor(target.channel.clone(), bearer),
            timeout,
        }
    }

    async fn call<T, F>(&self, f: F) -> Result<T, CallError>
    where
        F: Future<Output = Result<tonic::Response<T>, Status>>,
    {
        match tokio::time::timeout(self.timeout, f).await {
            Ok(Ok(r)) => Ok(r.into_inner()),
            Ok(Err(s)) => Err(CallError::from_status(&s)),
            Err(_) => Err(CallError::Timeout),
        }
    }
}

#[async_trait]
impl LedgerClient for GrpcLedger {
    async fn ping(&self, req: PingRequest) -> Result<PingResponse, CallError> {
        let mut c = self.client.clone();
        self.call(c.ping(req)).await
    }
    async fn append(&self, req: AppendRequest) -> Result<AppendResponse, CallError> {
        let mut c = self.client.clone();
        self.call(c.append(req)).await
    }
    async fn current(&self, req: CurrentRequest) -> Result<CurrentResponse, CallError> {
        let mut c = self.client.clone();
        self.call(c.current(req)).await
    }
    async fn history(&self, req: HistoryRequest) -> Result<HistoryResponse, CallError> {
        let mut c = self.client.clone();
        self.call(c.history(req)).await
    }
    async fn retract(&self, req: RetractRequest) -> Result<RetractResponse, CallError> {
        let mut c = self.client.clone();
        self.call(c.retract(req)).await
    }
    async fn erase(&self, req: EraseRequest) -> Result<EraseResponse, CallError> {
        let mut c = self.client.clone();
        self.call(c.erase(req)).await
    }
    async fn restore(&self, req: RestoreRequest) -> Result<RestoreResponse, CallError> {
        let mut c = self.client.clone();
        self.call(c.restore(req)).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classes_are_snake_case_codes() {
        let e = CallError::from_status(&Status::failed_precondition("erased"));
        assert_eq!(e.class(), "failed_precondition");
        assert_eq!(e.code(), Some(Code::FailedPrecondition));
        assert!(!e.is_unclean());
        assert!(CallError::from_status(&Status::internal("x")).is_unclean());
        assert_eq!(CallError::Timeout.class(), "timeout");
        assert_eq!(CallError::Transport("x".into()).class(), "transport");
        for c in KNOWN_CLASSES {
            assert!(!c.is_empty());
        }
    }
}
