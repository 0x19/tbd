//! One error shape for every route: `{ "error": "..." }` with a status.

use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

use crate::stack::StackError;

/// An error with the status it maps to.
#[derive(Debug)]
pub struct ApiError {
    /// HTTP status.
    pub status: StatusCode,
    /// Message for the caller.
    pub message: String,
}

impl ApiError {
    /// Build.
    pub fn new(status: StatusCode, message: impl Into<String>) -> Self {
        Self {
            status,
            message: message.into(),
        }
    }

    /// 404.
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, message)
    }

    /// 409.
    pub fn conflict(message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, message)
    }

    /// 422.
    pub fn invalid(message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, message)
    }

    /// 500.
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        (self.status, Json(json!({ "error": self.message }))).into_response()
    }
}

impl From<StackError> for ApiError {
    fn from(e: StackError) -> Self {
        let status = match &e {
            StackError::Unknown(_) => StatusCode::NOT_FOUND,
            StackError::AlreadyRunning(_)
            | StackError::Exists(_)
            | StackError::DependencyDown { .. }
            | StackError::InUse { .. }
            | StackError::FromTopology(_) => StatusCode::CONFLICT,
            StackError::NoFaults(_)
            | StackError::NoStoreFaults(_)
            | StackError::Unresolvable(_) => StatusCode::UNPROCESSABLE_ENTITY,
            StackError::Start { .. } | StackError::NotReady { .. } => {
                StatusCode::INTERNAL_SERVER_ERROR
            }
        };
        Self::new(status, e.to_string())
    }
}

impl From<std::io::Error> for ApiError {
    fn from(e: std::io::Error) -> Self {
        Self::internal(e.to_string())
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(e: serde_json::Error) -> Self {
        Self::internal(e.to_string())
    }
}
