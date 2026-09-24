//! The arena service: a gRPC server.
//!
//! The library exposes [`serve`], [`serve_on`] and [`serve_with`] so the same
//! server can be run from `main`, from integration tests on an ephemeral port,
//! and from the chaos tool with fault injection and counters attached.

pub mod collect;
pub mod config;
mod service;
pub mod world;

use std::net::SocketAddr;

use tbd_proto::arena::v1::arena_service_server::ArenaServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Overrides, Source};
pub use service::Arena;
pub use tbd_common::{
    fault::{Behavior, FaultHandle},
    runtime::{Runtime, Stats, StatsHandle, StatsSnapshot},
};

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
    /// A source's URL does not parse.
    #[error("source: {0}")]
    Source(tonic::transport::Error),
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
    serve_on(listener, config, shutdown).await
}

/// Serve on an already-bound listener, with every configured collector
/// running. Tests bind port 0 and read the address back before calling this.
pub async fn serve_on(
    listener: TcpListener,
    config: Config,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    serve_inner(listener, config, Runtime::default(), true, shutdown).await
}

/// Serve on an already-bound listener with the embedder's [`Runtime`]
/// attached and no collector running: the chaos tool embeds the arena to
/// test its surface, and an arena inside the chaos tool reading the chaos
/// tool would be the tool watching itself. Its snapshot says every source is
/// not read.
pub async fn serve_with(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    serve_inner(listener, config, runtime, false, shutdown).await
}

async fn serve_inner(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    collect: bool,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let addr = listener.local_addr().map_err(|source| ServeError::Bind {
        addr: config.server.listen,
        source,
    })?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<ArenaServiceServer<Arena>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::arena::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let world = if collect {
        world::World::new(&collect::configured(&config))
    } else {
        world::World::new(&[])
    };
    let tasks = if collect {
        collect::start(&config, &world).map_err(ServeError::Source)?
    } else {
        // Still a snapshot a tick, so a watcher of the embedded arena sees frames.
        vec![tokio::spawn(world.clone().tick(config.watch.tick))]
    };
    let service = Arena::new(config.ping.clone(), config.watch.clone(), runtime, world);

    tracing::info!(%addr, version = tbd_common::VERSION, "arena listening");

    // `tcp_nodelay` on the builder only applies to tonic's own listener. With
    // a caller-supplied listener it must be set on the incoming stream, or
    // small responses stall ~40 ms on Nagle + delayed ACK.
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    Server::builder()
        .trace_fn(tbd_common::telemetry::grpc_request_span)
        .add_service(health_service)
        .add_service(reflection)
        .add_service(ArenaServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await?;

    for task in tasks {
        task.abort();
    }
    tracing::info!("arena stopped");
    Ok(())
}
