//! The facts ledger: append-only facts about an opaque subject, with
//! provenance, tombstones and erasure, on a [`store::Store`] (Postgres, or in
//! memory for tests and embedders), and a thin gRPC service on top.
//!
//! The library exposes [`serve`], [`serve_on`] and [`serve_with`] so the same
//! server can be run from `main`, from integration tests on an ephemeral port,
//! and from the chaos tool with fault injection and counters attached;
//! [`serve_store`] takes an explicit store for embedders that build their own.

pub mod config;
mod service;
pub mod store;

use std::{net::SocketAddr, sync::Arc};

use tbd_proto::ledger::v1::ledger_service_server::LedgerServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Overrides, Source};
pub use service::Ledger;
pub use store::{Store, StoreError, StoreKind, memory::MemoryStore};
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
    /// The configuration does not hold together (a Postgres store without a URL).
    #[error("config: {0}")]
    Config(#[from] tbd_common::config::ConfigError),
    /// The store could not be built.
    #[error("store: {0}")]
    Store(#[from] StoreError),
}

/// Build the store the configuration names.
///
/// # Errors
/// The configuration is inconsistent, or the backend cannot be reached.
// Async for the Postgres backend, which connects here; the memory store does not await.
#[allow(clippy::unused_async)]
pub async fn build_store(config: &Config) -> Result<Arc<dyn Store>, ServeError> {
    config.validate()?;
    match config.store.kind {
        StoreKind::Memory => Ok(Arc::new(MemoryStore::new())),
        StoreKind::Postgres => Err(ServeError::Store(StoreError::Internal(
            "the postgres store is not built yet; run with LEDGER_STORE_KIND=memory".into(),
        ))),
    }
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
/// The store comes from `[store]`.
pub async fn serve_with(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let store = build_store(&config).await?;
    serve_store(listener, config, runtime, store, shutdown).await
}

/// Serve on an already-bound listener with an explicit store.
pub async fn serve_store(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    store: Arc<dyn Store>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let addr = listener.local_addr().map_err(|source| ServeError::Bind {
        addr: config.server.listen,
        source,
    })?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<LedgerServiceServer<Ledger>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::ledger::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let service = Ledger::new(config.ping.clone(), runtime, store);

    tracing::info!(%addr, version = tbd_common::VERSION, store = %service.store_kind(), "ledger listening");

    // `tcp_nodelay` on the builder only applies to tonic's own listener. With
    // a caller-supplied listener it must be set on the incoming stream, or
    // small responses stall ~40 ms on Nagle + delayed ACK.
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    Server::builder()
        .trace_fn(tbd_common::telemetry::grpc_request_span)
        .add_service(health_service)
        .add_service(reflection)
        .add_service(LedgerServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await?;

    tracing::info!("ledger stopped");
    Ok(())
}
