//! The engine: a gRPC streaming service.
//!
//! The library exposes [`serve`] and [`serve_on`] so the same server can be run
//! from `main` and from integration tests on an ephemeral port.

mod config;
mod service;

use std::net::SocketAddr;

use tbd_proto::engine::v1::engine_server::EngineServer;
use tokio::net::TcpListener;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

pub use config::Config;
pub use service::EngineService;

/// Errors from starting or running the server.
#[derive(Debug, thiserror::Error)]
pub enum ServeError {
    /// Could not bind the listen address.
    #[error("bind {addr}: {source}")]
    Bind {
        /// Address we tried to bind.
        addr: SocketAddr,
        /// Underlying I/O error.
        source: std::io::Error,
    },
    /// Reflection service could not be built from the descriptor set.
    #[error("reflection: {0}")]
    Reflection(#[from] tonic_reflection::server::Error),
    /// The gRPC server failed while running.
    #[error("transport: {0}")]
    Transport(#[from] tonic::transport::Error),
}

/// Bind `config.listen_addr` and serve until `shutdown` resolves.
pub async fn serve(
    config: Config,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let listener = TcpListener::bind(config.listen_addr)
        .await
        .map_err(|source| ServeError::Bind {
            addr: config.listen_addr,
            source,
        })?;
    serve_on(listener, config, shutdown).await
}

/// Serve on an already-bound listener. Tests bind port 0 and read the address
/// back before calling this.
pub async fn serve_on(
    listener: TcpListener,
    config: Config,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let addr = listener.local_addr().map_err(|source| ServeError::Bind {
        addr: config.listen_addr,
        source,
    })?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<EngineServer<EngineService>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::engine::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let service = EngineService::new(config.heartbeat_interval());

    tracing::info!(%addr, version = tbd_common::VERSION, "engine listening");

    Server::builder()
        .trace_fn(|_| tracing::info_span!("grpc"))
        .add_service(health_service)
        .add_service(reflection)
        .add_service(EngineServer::new(service))
        .serve_with_incoming_shutdown(TcpListenerStream::new(listener), shutdown)
        .await?;

    tracing::info!("engine stopped");
    Ok(())
}
