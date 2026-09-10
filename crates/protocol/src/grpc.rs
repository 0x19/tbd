//! gRPC services owned by the gateway, multiplexed on the HTTP port via h2c.

use axum::Router;
use tbd_proto::protocol::v1::{
    PingRequest, PingResponse,
    gateway_server::{Gateway, GatewayServer},
};
use tonic::{Request, Response, Status, service::Routes};

use crate::AppState;

/// `tbd.protocol.v1.Gateway` implementation.
#[derive(Debug, Clone, Default)]
pub struct GatewayService;

#[tonic::async_trait]
impl Gateway for GatewayService {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse {
            message: request.into_inner().message,
            gateway_version: tbd_common::VERSION.to_owned(),
        }))
    }
}

/// gRPC routes as an axum router, ready to merge with the HTTP routes.
///
/// Reflection is best-effort: a descriptor set that fails to parse is a build
/// bug, so it is logged and the gateway still serves without reflection.
pub fn routes(_state: &AppState) -> Router {
    let (_, health) = tonic_health::server::health_reporter();
    let mut routes = Routes::new(health).add_service(GatewayServer::new(GatewayService));
    match tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::protocol::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()
    {
        Ok(reflection) => routes = routes.add_service(reflection),
        Err(error) => tracing::error!(%error, "gRPC reflection disabled"),
    }
    routes.into_axum_router()
}
