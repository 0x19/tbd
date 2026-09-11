//! One transcoded request: bind, forward over the backend's transport, answer
//! as JSON (unary) or as server-sent events (server streaming).

use std::{collections::HashMap, convert::Infallible};

use axum::{
    Json,
    body::Bytes,
    extract::Request,
    response::{
        IntoResponse, Response,
        sse::{Event as SseEvent, KeepAlive, Sse},
    },
};
use futures::StreamExt;
use http::header;
use prost_reflect::{DynamicMessage, SerializeOptions};
use serde::Serialize;
use tbd_common::metrics::StreamGuard;
use tonic::client::Grpc;

use super::{Binding, BodyRule, codec::DynamicCodec};
use crate::{AppState, Code, Problem, json};

/// Largest request body a transcoded route reads.
const MAX_BODY: usize = 2 * 1024 * 1024;

/// How a dynamic message is written: proto field names (the hand-written
/// shapes use them), 64-bit integers as strings and enums by name (proto3
/// JSON), every field present so a `stub: false` is labelled like every other
/// default.
const OPTS: SerializeOptions = SerializeOptions::new()
    .use_proto_field_name(true)
    .skip_default_fields(false);

/// A dynamic message on the wire. The only way one is serialised.
pub struct Out<'a>(pub &'a DynamicMessage);

impl Serialize for Out<'_> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.0.serialize_with_options(serializer, &OPTS)
    }
}

/// Serve one request for `binding`.
pub async fn handle(
    binding: &Binding,
    state: &AppState,
    vars: HashMap<String, String>,
    query: Option<String>,
    request: Request,
) -> Response {
    match call(binding, state, &vars, query.as_deref(), request).await {
        Ok(response) => response,
        Err(problem) => problem.into_response(),
    }
}

async fn call(
    binding: &Binding,
    state: &AppState,
    vars: &HashMap<String, String>,
    query: Option<&str>,
    request: Request,
) -> Result<Response, Problem> {
    let backend = state.backend(&binding.backend).ok_or_else(|| {
        Problem::new(
            Code::Unavailable,
            format!("backend {} is not registered", binding.backend),
        )
    })?;
    let body = match binding.body {
        BodyRule::None => None,
        BodyRule::Whole | BodyRule::Field(_) => Some(read_json_body(request).await?),
    };
    let message = super::bind::request(binding, vars, query, body.as_deref())?;

    let mut grpc = Grpc::new(backend.transport());
    grpc.ready().await.map_err(|e| {
        Problem::from(tonic::Status::unavailable(format!(
            "backend not ready: {}",
            Into::<tonic::codegen::StdError>::into(e)
        )))
    })?;
    let codec = DynamicCodec::new(binding.method.output());
    if binding.streaming {
        let stream = grpc
            .server_streaming(
                tonic::Request::new(message),
                binding.grpc_path.clone(),
                codec,
            )
            .await?
            .into_inner();
        let guard = StreamGuard::open("sse");
        let events = stream.map(move |item| {
            guard.item("out");
            let event = match item {
                Ok(msg) => SseEvent::default()
                    .json_data(Out(&msg))
                    .unwrap_or_else(|_| Problem::new(Code::Internal, "serialize").sse_event()),
                Err(status) => Problem::from(status).sse_event(),
            };
            Ok::<_, Infallible>(event)
        });
        return Ok(Sse::new(events)
            .keep_alive(KeepAlive::default())
            .into_response());
    }
    let response = grpc
        .unary(
            tonic::Request::new(message),
            binding.grpc_path.clone(),
            codec,
        )
        .await?
        .into_inner();
    let body = match &binding.response_body {
        None => serde_json::to_value(Out(&response)),
        Some(field) => serde_json::to_value(Out(&response)).map(|mut v| {
            v.as_object_mut()
                .and_then(|o| o.remove(field.name()))
                .unwrap_or(serde_json::Value::Null)
        }),
    }
    .map_err(|e| Problem::new(Code::Internal, format!("serialize: {e}")))?;
    Ok(Json(body).into_response())
}

/// The body of a route that takes one: JSON by content type, bounded.
async fn read_json_body(request: Request) -> Result<Bytes, Problem> {
    let is_json = request
        .headers()
        .get(header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .is_some_and(|ct| {
            ct.trim()
                .split(';')
                .next()
                .is_some_and(|mime| mime.trim().eq_ignore_ascii_case("application/json"))
        });
    if !is_json {
        return Err(json::not_json());
    }
    axum::body::to_bytes(request.into_body(), MAX_BODY)
        .await
        .map_err(|_| json::too_large())
}
