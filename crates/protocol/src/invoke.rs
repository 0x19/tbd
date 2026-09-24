//! One RPC, invoked by name with a JSON body, as the caller: what every
//! transport that addresses RPCs by name shares (the multiplexed socket and
//! MCP). REST and SSE bind path and query first and call through
//! `transcode/call.rs`; once the request message exists, the gRPC call is the
//! same.

use prost_reflect::{DeserializeOptions, DynamicMessage};
use tonic::{
    client::Grpc,
    metadata::{Ascii, MetadataValue},
};

use crate::{
    AppState,
    error::{Code, Problem},
    transcode::{Rpc, codec::DynamicCodec},
};

/// What a call answered: one message, or the stream of them.
pub(crate) enum Invoked {
    /// A unary RPC's response.
    Unary(DynamicMessage),
    /// A server-streaming RPC's messages, in order; an error item ends it.
    Stream(tonic::Streaming<DynamicMessage>),
}

/// The request message: `body`, strictly (unknown fields are refused), or an
/// empty message when there is none. Both proto and JSON field names are read.
pub(crate) fn request(
    rpc: &Rpc,
    body: Option<serde_json::Value>,
) -> Result<DynamicMessage, Problem> {
    let input = rpc.method.input();
    match body {
        None | Some(serde_json::Value::Null) => Ok(DynamicMessage::new(input)),
        Some(value) => {
            let options = DeserializeOptions::new().deny_unknown_fields(true);
            DynamicMessage::deserialize_with_options(input, value, &options)
                .map_err(|error| Problem::field("body", error.to_string()))
        }
    }
}

/// Call `rpc` on its backend with `message`, carrying `payload` (the identity
/// Envoy verified) as every other surface does. The caller times the call.
pub(crate) async fn invoke(
    rpc: &Rpc,
    state: &AppState,
    payload: Option<MetadataValue<Ascii>>,
    message: DynamicMessage,
) -> Result<Invoked, Problem> {
    let backend = state.backend(&rpc.backend).ok_or_else(|| {
        Problem::new(
            Code::Unavailable,
            format!("backend {} is not registered", rpc.backend),
        )
    })?;
    let mut grpc = Grpc::new(backend.transport());
    grpc.ready().await.map_err(|error| {
        Problem::from(tonic::Status::unavailable(format!(
            "backend not ready: {}",
            Into::<tonic::codegen::StdError>::into(error)
        )))
    })?;
    let codec = DynamicCodec::new(rpc.method.output());
    let mut outbound = tonic::Request::new(message);
    if let Some(payload) = payload {
        outbound
            .metadata_mut()
            .insert(crate::principal::PAYLOAD_HEADER, payload);
    }
    if rpc.streaming {
        let stream = grpc
            .server_streaming(outbound, rpc.grpc_path.clone(), codec)
            .await?
            .into_inner();
        Ok(Invoked::Stream(stream))
    } else {
        let response = grpc
            .unary(outbound, rpc.grpc_path.clone(), codec)
            .await?
            .into_inner();
        Ok(Invoked::Unary(response))
    }
}
