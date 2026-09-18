//! The playground service: a gRPC server.
//!
//! The library exposes [`serve`], [`serve_on`] and [`serve_with`] so the same
//! server can be run from `main`, from integration tests on an ephemeral port,
//! and from the chaos tool with fault injection and counters attached.

pub mod config;
pub mod faults;
pub mod sandbox;
pub mod score;
mod service;
pub mod traffic;
pub mod world;

use std::{net::SocketAddr, sync::Arc};

use tbd_proto::playground::v1::playground_service_server::PlaygroundServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Game, Overrides, Sandbox as SandboxConfig, Source};
pub use sandbox::Sandbox;
pub use score::Scores;
pub use service::Playground;
pub use tbd_common::{
    fault::{Behavior, FaultHandle},
    runtime::{Runtime, Stats, StatsHandle, StatsSnapshot},
};
pub use world::World;

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
    /// The sandbox the game runs on would not start.
    #[error("sandbox: {0}")]
    Sandbox(#[from] sandbox::StartError),
}

/// Bind `[server] listen` and serve until `shutdown` resolves.
pub async fn serve(
    config: Config,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let listener = TcpListener::bind(config.server.listen)
        .await
        .map_err(|source| ServeError::Bind {
            addr: config.server.listen,
            source,
        })?;
    serve_inner(listener, config, Runtime::default(), true, shutdown).await
}

/// Serve on an already-bound listener. Tests bind port 0 and read the address
/// back before calling this.
pub async fn serve_on(
    listener: TcpListener,
    config: Config,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    serve_with(listener, config, Runtime::default(), shutdown).await
}

/// Serve on an already-bound listener with the embedder's [`Runtime`] attached.
///
/// No sandbox: the game RPCs answer `UNAVAILABLE` and `Ping` works. This is
/// what the chaos kind and the integration tests want — starting the world here
/// would have the chaos tool spawn four more services inside the one it just
/// started, recursively.
pub async fn serve_with(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    serve_inner(listener, config, runtime, false, shutdown).await
}

/// Serve, with the sandbox if one was started.
async fn serve_inner(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    game: bool,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let addr = listener.local_addr().map_err(|source| ServeError::Bind {
        addr: config.server.listen,
        source,
    })?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<PlaygroundServiceServer<Playground>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::playground::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let service = Playground::new(config.ping.clone(), runtime);

    // The sandbox starts beside the server rather than before it. Four services
    // take a few seconds to come up, and a server that has not bound yet fails
    // its liveness probe and gets killed mid-start. Until the world lands, the
    // game RPCs answer UNAVAILABLE and say why.
    //
    // An embedder — the chaos kind, an integration test — asks for no game at
    // all: starting one would have the chaos tool spawn four more services
    // inside the one it just started.
    let sandbox = if game {
        let slot = service.world_slot();
        let config = config.clone();
        Some(tokio::spawn(async move {
            match Sandbox::start(&config).await {
                Ok(sandbox) => {
                    let _ = slot.set(Arc::clone(&sandbox.world));
                    Some(sandbox)
                }
                Err(error) => {
                    tracing::error!(%error, "the sandbox did not start; the game RPCs stay unavailable");
                    None
                }
            }
        }))
    } else {
        None
    };

    tracing::info!(%addr, version = tbd_common::VERSION, "playground listening");

    // `tcp_nodelay` on the builder only applies to tonic's own listener. With
    // a caller-supplied listener it must be set on the incoming stream, or
    // small responses stall ~40 ms on Nagle + delayed ACK.
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    Server::builder()
        .trace_fn(tbd_common::telemetry::grpc_request_span)
        .add_service(health_service)
        .add_service(reflection)
        .add_service(PlaygroundServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await?;

    if let Some(task) = sandbox
        && let Ok(Some(sandbox)) = task.await
    {
        sandbox.shutdown().await;
    }
    tracing::info!("playground stopped");
    Ok(())
}
