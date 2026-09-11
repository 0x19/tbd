//! gRPC services owned by the protocol, multiplexed on the HTTP port via h2c.

use axum::Router;
use tbd_proto::protocol::v1::{
    PingRequest, PingResponse,
    protocol_service_server::{ProtocolService, ProtocolServiceServer},
};
use tonic::{Request, Response, Status, service::Routes};
use tonic_health::server::HealthReporter;

use crate::{AppState, state::Backend};

/// The engine's gRPC service name, as reported on this port's health service
/// and as `Config::embedded` names the `engine` backend.
pub const ENGINE_SERVICE: &str = "tbd.engine.v1.EngineService";

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
/// Health reports the protocol's own service (always SERVING) and every
/// registered backend under its `grpc.health.v1` name, refreshed every
/// `[health] probe_interval` by one task per backend. Behind the Envoy edge
/// every `grpc.health.v1.Health` call lands here, so a client that only sees
/// the edge still gets a truthful answer for each backend. Must be called on a
/// Tokio runtime.
pub fn routes(state: &AppState) -> Router {
    let (reporter, health) = tonic_health::server::health_reporter();
    for backend in state.backends() {
        tokio::spawn(report_backend_health(
            backend.clone(),
            reporter.clone(),
            state.probe_interval(),
            state.probe_timeout(),
        ));
    }
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

async fn report_backend_health(
    backend: Backend,
    reporter: HealthReporter,
    interval: std::time::Duration,
    timeout: std::time::Duration,
) {
    let mut last = None;
    let mut tick = tokio::time::interval(interval);
    loop {
        tick.tick().await;
        let state = backend.check(timeout).await;
        if last != Some(state) {
            tracing::info!(
                backend = backend.name(),
                service = %backend.health_name,
                ?state,
                "backend health changed"
            );
            last = Some(state);
        }
        reporter
            .set_service_status(&backend.health_name, state.serving_status())
            .await;
    }
}
