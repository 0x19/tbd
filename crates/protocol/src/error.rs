//! One error type for every HTTP surface, mapped from `tonic::Status`.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;

/// Error returned to HTTP clients as JSON.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    /// The engine returned a gRPC status.
    #[error("engine: {0}")]
    Engine(#[from] tonic::Status),
    /// The request was malformed.
    #[error("bad request: {0}")]
    BadRequest(String),
}

#[derive(Serialize)]
struct Body<'a> {
    error: &'a str,
    code: &'a str,
}

impl ApiError {
    fn status(&self) -> StatusCode {
        use tonic::Code as C;
        match self {
            Self::BadRequest(_) => StatusCode::BAD_REQUEST,
            Self::Engine(s) => match s.code() {
                C::InvalidArgument | C::OutOfRange => StatusCode::BAD_REQUEST,
                C::NotFound => StatusCode::NOT_FOUND,
                C::AlreadyExists | C::Aborted => StatusCode::CONFLICT,
                C::PermissionDenied => StatusCode::FORBIDDEN,
                C::Unauthenticated => StatusCode::UNAUTHORIZED,
                C::ResourceExhausted => StatusCode::TOO_MANY_REQUESTS,
                C::FailedPrecondition => StatusCode::PRECONDITION_FAILED,
                C::Unimplemented => StatusCode::NOT_IMPLEMENTED,
                C::Unavailable => StatusCode::SERVICE_UNAVAILABLE,
                C::DeadlineExceeded => StatusCode::GATEWAY_TIMEOUT,
                _ => StatusCode::BAD_GATEWAY,
            },
        }
    }

    fn code(&self) -> &'static str {
        match self {
            Self::BadRequest(_) => "bad_request",
            Self::Engine(s) => match s.code() {
                tonic::Code::Unavailable => "engine_unavailable",
                _ => "engine_error",
            },
        }
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
        }
        let message = self.to_string();
        (
            status,
            Json(Body {
                error: &message,
                code: self.code(),
            }),
        )
            .into_response()
    }
}
