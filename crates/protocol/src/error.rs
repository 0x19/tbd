//! One error envelope for every surface, downstream-agnostic.
//!
//! A [`Problem`] carries a [`Code`] from the frozen vocabulary, a sentence, and
//! typed details translated from `google.rpc` error details when a backend
//! sends them. REST answers it as `{"code","error","details"}` with the
//! standard gRPC-to-HTTP status; SSE sends it as the `error` event's JSON;
//! the WebSocket bridge as the `error` frame; GraphQL as the message plus
//! `extensions.code` and `extensions.details`.

use std::collections::BTreeMap;

use axum::{
    Json,
    http::{HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
};
use serde::{Deserialize, Serialize};
use tonic_types::StatusExt as _;
use utoipa::ToSchema;

/// The stable error vocabulary. The slug (`snake_case`) is what clients see in
/// `code`; the HTTP status follows the standard gRPC mapping, plus the two
/// HTTP-only codes for request bodies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum Code {
    /// `INVALID_ARGUMENT`, `OUT_OF_RANGE`, or a body that does not parse.
    BadRequest,
    /// `NOT_FOUND`, or no route.
    NotFound,
    /// `ALREADY_EXISTS`.
    AlreadyExists,
    /// `ABORTED`: a concurrent change or an idempotency mismatch.
    Conflict,
    /// `PERMISSION_DENIED`.
    Forbidden,
    /// `UNAUTHENTICATED`, or a route that needs a caller and got none.
    Unauthenticated,
    /// `RESOURCE_EXHAUSTED`.
    RateLimited,
    /// `FAILED_PRECONDITION`: the state does not allow the call (an erased subject).
    FailedPrecondition,
    /// `UNIMPLEMENTED`.
    Unimplemented,
    /// `UNAVAILABLE`: a backend is down or unreachable.
    Unavailable,
    /// `DEADLINE_EXCEEDED`.
    Timeout,
    /// `CANCELLED`.
    Cancelled,
    /// `INTERNAL`, `UNKNOWN`, `DATA_LOSS`: the message is redacted.
    Internal,
    /// A request body that is not JSON.
    UnsupportedMediaType,
    /// A request body over the size limit.
    PayloadTooLarge,
}

impl Code {
    /// The standard gRPC-to-HTTP mapping (`Cancelled` is nginx's 499).
    #[must_use]
    pub fn from_grpc(code: tonic::Code) -> Self {
        use tonic::Code as C;
        match code {
            C::InvalidArgument | C::OutOfRange => Self::BadRequest,
            C::NotFound => Self::NotFound,
            C::AlreadyExists => Self::AlreadyExists,
            C::Aborted => Self::Conflict,
            C::PermissionDenied => Self::Forbidden,
            C::Unauthenticated => Self::Unauthenticated,
            C::ResourceExhausted => Self::RateLimited,
            C::FailedPrecondition => Self::FailedPrecondition,
            C::Unimplemented => Self::Unimplemented,
            C::Unavailable => Self::Unavailable,
            C::DeadlineExceeded => Self::Timeout,
            C::Cancelled => Self::Cancelled,
            C::Ok | C::Unknown | C::Internal | C::DataLoss => Self::Internal,
        }
    }

    /// The HTTP status.
    #[must_use]
    pub fn http(self) -> StatusCode {
        match self {
            Self::BadRequest | Self::FailedPrecondition => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::AlreadyExists | Self::Conflict => StatusCode::CONFLICT,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::Unauthenticated => StatusCode::UNAUTHORIZED,
            Self::RateLimited => StatusCode::TOO_MANY_REQUESTS,
            Self::Unimplemented => StatusCode::NOT_IMPLEMENTED,
            Self::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
            Self::Timeout => StatusCode::GATEWAY_TIMEOUT,
            Self::Cancelled => {
                StatusCode::from_u16(499).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR)
            }
            Self::Internal => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnsupportedMediaType => StatusCode::UNSUPPORTED_MEDIA_TYPE,
            Self::PayloadTooLarge => StatusCode::PAYLOAD_TOO_LARGE,
        }
    }

    /// The `code` string on the wire.
    #[must_use]
    pub fn slug(self) -> &'static str {
        match self {
            Self::BadRequest => "bad_request",
            Self::NotFound => "not_found",
            Self::AlreadyExists => "already_exists",
            Self::Conflict => "conflict",
            Self::Forbidden => "forbidden",
            Self::Unauthenticated => "unauthenticated",
            Self::RateLimited => "rate_limited",
            Self::FailedPrecondition => "failed_precondition",
            Self::Unimplemented => "unimplemented",
            Self::Unavailable => "unavailable",
            Self::Timeout => "timeout",
            Self::Cancelled => "cancelled",
            Self::Internal => "internal",
            Self::UnsupportedMediaType => "unsupported_media_type",
            Self::PayloadTooLarge => "payload_too_large",
        }
    }
}

/// A typed detail, translated from `google.rpc` error details.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Detail {
    /// `google.rpc.BadRequest.FieldViolation`: which field, and why.
    Field {
        /// Field path in the request.
        field: String,
        /// What is wrong with it.
        description: String,
    },
    /// `google.rpc.ErrorInfo`: a machine-readable reason.
    Info {
        /// Reason, `UPPER_SNAKE_CASE` by convention.
        reason: String,
        /// Who owns the reason.
        domain: String,
        /// Extra key/values.
        metadata: BTreeMap<String, String>,
    },
    /// `google.rpc.RetryInfo`: when to try again; also the `Retry-After` header.
    Retry {
        /// Seconds to wait, rounded up.
        after_seconds: u64,
    },
}

/// The error on every surface.
#[derive(Debug, Clone, thiserror::Error)]
#[error("{}: {message}", code.slug())]
pub struct Problem {
    /// The vocabulary entry.
    pub code: Code,
    /// A sentence for a person.
    pub message: String,
    /// Typed details.
    pub details: Vec<Detail>,
    /// The downstream message when `message` was redacted; logged, never sent.
    downstream: Option<String>,
}

/// The wire shape as an owned type, for the `OpenAPI` document (`Problem`).
pub type ErrorBody = Wire<'static>;

/// `{"code","error","details"}`: the REST body and the SSE `error` payload.
#[derive(Debug, Serialize, ToSchema)]
#[schema(as = Problem)]
pub struct Wire<'a> {
    /// The slug.
    pub code: Code,
    /// The sentence.
    pub error: &'a str,
    /// Typed details.
    pub details: &'a [Detail],
}

impl Problem {
    /// A problem with a code and a sentence.
    #[must_use]
    pub fn new(code: Code, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: Vec::new(),
            downstream: None,
        }
    }

    /// A malformed request.
    #[must_use]
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new(Code::BadRequest, message)
    }

    /// A malformed field: 400 with one `field` detail.
    #[must_use]
    pub fn field(field: impl Into<String>, description: impl Into<String>) -> Self {
        let field = field.into();
        let description = description.into();
        Self::new(Code::BadRequest, format!("{field} {description}"))
            .with(Detail::Field { field, description })
    }

    /// No route for a path.
    #[must_use]
    pub fn not_found(path: &str) -> Self {
        Self::new(Code::NotFound, format!("no route for {path}"))
    }

    /// The route needs a caller and Envoy forwarded none.
    #[must_use]
    pub fn unauthenticated() -> Self {
        Self::new(Code::Unauthenticated, "authentication required")
    }

    /// Add a detail.
    #[must_use]
    pub fn with(mut self, detail: Detail) -> Self {
        self.details.push(detail);
        self
    }

    /// Seconds from a `Retry` detail, for the `Retry-After` header.
    #[must_use]
    pub fn retry_after(&self) -> Option<u64> {
        self.details.iter().find_map(|d| match d {
            Detail::Retry { after_seconds } => Some(*after_seconds),
            _ => None,
        })
    }

    /// The wire shape.
    #[must_use]
    pub fn wire(&self) -> Wire<'_> {
        Wire {
            code: self.code,
            error: &self.message,
            details: &self.details,
        }
    }

    /// The wire shape as a JSON value (for GraphQL extensions and frames).
    #[must_use]
    pub fn details_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.details).unwrap_or(serde_json::Value::Array(Vec::new()))
    }

    /// One `error` line for server-side codes, with the redacted downstream
    /// message when there is one. Called where the request span is current, so
    /// `trace_id` is on the line.
    pub fn log(&self) {
        if self.code.http().is_server_error() {
            match &self.downstream {
                Some(downstream) => {
                    tracing::error!(code = self.code.slug(), message = %self.message, %downstream, "request failed");
                }
                None => {
                    tracing::error!(code = self.code.slug(), message = %self.message, "request failed");
                }
            }
        }
    }
}

impl From<tonic::Status> for Problem {
    /// Code by the standard mapping; details from the `google.rpc` details the
    /// status carries; `internal` redacts the downstream message.
    fn from(status: tonic::Status) -> Self {
        let code = Code::from_grpc(status.code());
        let rich = status.get_error_details();
        let mut details = Vec::new();
        if let Some(bad) = rich.bad_request() {
            details.extend(bad.field_violations.iter().map(|v| Detail::Field {
                field: v.field.clone(),
                description: v.description.clone(),
            }));
        }
        if let Some(info) = rich.error_info() {
            details.push(Detail::Info {
                reason: info.reason.clone(),
                domain: info.domain.clone(),
                metadata: info
                    .metadata
                    .iter()
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect(),
            });
        }
        if let Some(retry) = rich.retry_info()
            && let Some(delay) = retry.retry_delay
        {
            let after_seconds = delay.as_secs() + u64::from(delay.subsec_nanos() > 0);
            details.push(Detail::Retry { after_seconds });
        }
        let (message, downstream) = if code == Code::Internal {
            (
                "internal error".to_owned(),
                Some(status.message().to_owned()),
            )
        } else {
            (status.message().to_owned(), None)
        };
        Self {
            code,
            message,
            details,
            downstream,
        }
    }
}

impl Problem {
    /// The GraphQL error: the sentence as the message, `code` and `details`
    /// in `extensions`. (A `From` impl would collide with async-graphql's
    /// blanket `From<impl Display>`.)
    #[must_use]
    pub fn into_graphql(self) -> async_graphql::Error {
        use async_graphql::ErrorExtensions as _;
        self.log();
        let details = async_graphql::Value::from_json(self.details_json())
            .unwrap_or(async_graphql::Value::Null);
        let slug = self.code.slug();
        async_graphql::Error::new(self.message).extend_with(|_, e| {
            e.set("code", slug);
            e.set("details", details.clone());
        })
    }
}

impl IntoResponse for Problem {
    fn into_response(self) -> Response {
        self.log();
        let mut response = (self.code.http(), Json(self.wire())).into_response();
        if let Some(seconds) = self.retry_after()
            && let Ok(value) = HeaderValue::from_str(&seconds.to_string())
        {
            response.headers_mut().insert(header::RETRY_AFTER, value);
        }
        response
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_grpc_code_maps_to_the_standard_status_and_slug() {
        use tonic::Code as C;
        let table = [
            (C::Ok, "internal", 500),
            (C::Cancelled, "cancelled", 499),
            (C::Unknown, "internal", 500),
            (C::InvalidArgument, "bad_request", 400),
            (C::DeadlineExceeded, "timeout", 504),
            (C::NotFound, "not_found", 404),
            (C::AlreadyExists, "already_exists", 409),
            (C::PermissionDenied, "forbidden", 403),
            (C::ResourceExhausted, "rate_limited", 429),
            (C::FailedPrecondition, "failed_precondition", 400),
            (C::Aborted, "conflict", 409),
            (C::OutOfRange, "bad_request", 400),
            (C::Unimplemented, "unimplemented", 501),
            (C::Internal, "internal", 500),
            (C::Unavailable, "unavailable", 503),
            (C::DataLoss, "internal", 500),
            (C::Unauthenticated, "unauthenticated", 401),
        ];
        for (grpc, slug, status) in table {
            let code = Code::from_grpc(grpc);
            assert_eq!(code.slug(), slug, "{grpc:?}");
            assert_eq!(code.http().as_u16(), status, "{grpc:?}");
            let json = serde_json::to_string(&code).unwrap_or_default();
            assert_eq!(json, format!("\"{slug}\""), "serde slug must equal slug()");
        }
        assert_eq!(Code::UnsupportedMediaType.http().as_u16(), 415);
        assert_eq!(Code::PayloadTooLarge.http().as_u16(), 413);
    }

    #[test]
    fn details_come_from_google_rpc_and_internal_is_redacted() {
        let mut rich = tonic_types::ErrorDetails::new();
        rich.set_bad_request(vec![tonic_types::FieldViolation::new(
            "subject_id",
            "required",
        )])
        .set_error_info(
            "SUBJECT_MISSING",
            "tbd",
            std::collections::HashMap::from([("hint".to_owned(), "give one".to_owned())]),
        )
        .set_retry_info(Some(std::time::Duration::from_millis(1500)));
        let status = tonic::Status::with_error_details(
            tonic::Code::InvalidArgument,
            "subject_id is required",
            rich,
        );
        let problem = Problem::from(status);
        assert_eq!(problem.code, Code::BadRequest);
        assert_eq!(problem.message, "subject_id is required");
        assert_eq!(problem.details.len(), 3);
        assert!(
            matches!(&problem.details[0], Detail::Field { field, .. } if field == "subject_id")
        );
        assert!(
            matches!(&problem.details[1], Detail::Info { reason, .. } if reason == "SUBJECT_MISSING")
        );
        assert_eq!(problem.retry_after(), Some(2), "1.5 s rounds up");

        let internal = Problem::from(tonic::Status::internal("pool exhausted at 17:04"));
        assert_eq!(internal.code, Code::Internal);
        assert_eq!(internal.message, "internal error");
        assert_eq!(
            internal.downstream.as_deref(),
            Some("pool exhausted at 17:04")
        );
        let wire = serde_json::to_value(internal.wire()).unwrap_or_default();
        assert_eq!(wire["code"], "internal");
        assert_eq!(wire["error"], "internal error");
        assert!(
            wire.get("source").is_none(),
            "the downstream text never reaches the wire"
        );
        assert_eq!(wire["details"], serde_json::json!([]));
    }

    #[test]
    fn helpers_build_the_expected_envelopes() {
        let field = Problem::field("subject_id", "is required");
        let wire = serde_json::to_value(field.wire()).unwrap_or_default();
        assert_eq!(wire["code"], "bad_request");
        assert_eq!(wire["error"], "subject_id is required");
        assert_eq!(wire["details"][0]["type"], "field");
        assert_eq!(wire["details"][0]["field"], "subject_id");
        assert_eq!(Problem::not_found("/x").code.http().as_u16(), 404);
        assert_eq!(Problem::unauthenticated().code.slug(), "unauthenticated");
    }
}
