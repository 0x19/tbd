//! JSON request bodies with the envelope on rejection.
//!
//! axum's own `Json` extractor answers a bad body with plain text. Every body
//! on this surface is JSON in and JSON out, so the rejection becomes a
//! [`Problem`]: `unsupported_media_type` (415) when the content type is not
//! JSON, `payload_too_large` (413) over the body limit, and `bad_request`
//! (400) with one `field` detail naming the body when it does not parse.

use axum::{
    extract::{FromRequest, Request, rejection::JsonRejection},
    response::{IntoResponse, Response},
};
use serde::de::DeserializeOwned;

use crate::error::{Code, Problem};

/// A JSON request body; the request-side twin of `axum::Json` for responses.
#[derive(Debug, Clone, Copy, Default)]
pub struct Json<T>(pub T);

impl<T, S> FromRequest<S> for Json<T>
where
    T: DeserializeOwned,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
        match axum::Json::<T>::from_request(request, state).await {
            Ok(axum::Json(value)) => Ok(Self(value)),
            Err(rejection) => Err(problem(&rejection).into_response()),
        }
    }
}

/// The envelope for a body axum refused.
#[must_use]
pub fn problem(rejection: &JsonRejection) -> Problem {
    match rejection {
        JsonRejection::MissingJsonContentType(_) => Problem::new(
            Code::UnsupportedMediaType,
            "the request body must be JSON (content-type: application/json)",
        ),
        JsonRejection::BytesRejection(bytes)
            if bytes.status() == http::StatusCode::PAYLOAD_TOO_LARGE =>
        {
            Problem::new(Code::PayloadTooLarge, "the request body is too large")
        }
        other => Problem::field("body", other.body_text()),
    }
}
