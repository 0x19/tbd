//! gRPC services owned by the protocol, multiplexed on the HTTP port via h2c.

use axum::Router;
use tbd_proto::protocol::v1::{
    PingRequest, PingResponse,
    protocol_service_server::{ProtocolService, ProtocolServiceServer},
};
use tonic::{Request, Response, Status, service::Routes};
use tonic_health::{ServingStatus, server::HealthReporter};

use crate::AppState;

/// The engine's gRPC service name, as reported on this port's health service.
pub const ENGINE_SERVICE: &str = "tbd.engine.v1.EngineService";

/// How often the engine's status is refreshed on the health service.
const ENGINE_HEALTH_INTERVAL: std::time::Duration = std::time::Duration::from_secs(5);

/// `tbd.protocol.v1.Protocol` implementation.
#[derive(Debug, Clone, Default)]
pub struct ProtocolGrpc;

#[tonic::async_trait]
impl ProtocolService for ProtocolGrpc {
    async fn ping(&self, request: Request<PingRequest>) -> Result<Response<PingResponse>, Status> {
        Ok(Response::new(PingResponse {
            message: request.into_inner().message,
            protocol_version: tbd_common::VERSION.to_owned(),
        }))
    }
}

/// gRPC routes as an axum router, ready to merge with the HTTP routes.
///
/// Reflection is best-effort: a descriptor set that fails to parse is a build
/// bug, so it is logged and the protocol still serves without reflection.
///
/// Health reports two names: the protocol's own service (always SERVING) and
/// [`ENGINE_SERVICE`], which mirrors `/readyz` (whether the engine answers). Behind
/// the Envoy edge every `grpc.health.v1.Health` call lands here, so a client that
/// only sees the edge still gets a truthful answer for the engine. A background
/// task refreshes it; must be called on a Tokio runtime.
pub fn routes(state: &AppState) -> Router {
    let (reporter, health) = tonic_health::server::health_reporter();
    tokio::spawn(report_engine_health(state.clone(), reporter));
    let mut routes = Routes::new(health).add_service(ProtocolServiceServer::new(ProtocolGrpc));
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

async fn report_engine_health(state: AppState, reporter: HealthReporter) {
    let mut last = None;
    let mut tick = tokio::time::interval(ENGINE_HEALTH_INTERVAL);
    loop {
        tick.tick().await;
        let status = if state.engine_ready().await {
            ServingStatus::Serving
        } else {
            ServingStatus::NotServing
        };
        if last != Some(status) {
            tracing::info!(service = ENGINE_SERVICE, ?status, "engine health changed");
            last = Some(status);
        }
        reporter.set_service_status(ENGINE_SERVICE, status).await;
    }
}
