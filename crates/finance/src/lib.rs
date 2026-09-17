//! The finance service: a gRPC server.
//!
//! The library exposes [`serve`], [`serve_on`] and [`serve_with`] so the same
//! server can be run from `main`, from integration tests on an ephemeral port,
//! and from the chaos tool with fault injection and counters attached.

pub mod banking;
pub mod categorise;
pub mod config;
pub mod import;
mod service;
pub mod store;
pub mod sync;

use std::net::SocketAddr;

use tbd_proto::finance::v1::finance_service_server::FinanceServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Overrides, Source};
pub use service::Finance;
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
    /// The store could not be opened. The URL did not parse; an unreachable
    /// database is not this, because the pool is lazy.
    #[error("store: {0}")]
    Store(String),
    /// The bank credentials are set but unusable.
    #[error("provider: {0}")]
    Provider(String),
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
/// Serve with an in-memory store seeded by the caller.
///
/// For in-process stacks only: the chaos tool uses it so an access-control
/// scenario can drive a live service without a database behind it. Production
/// never reaches this -- `serve` and `serve_with` build a Postgres-backed
/// service or none at all.
///
/// # Errors
/// The listener's address cannot be read, or the server fails.
pub async fn serve_seeded(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    store: store::MemoryStore,
    grants: Vec<(String, Vec<uuid::Uuid>)>,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let service = Finance::with_memory(config.ping.clone(), runtime, store, grants);
    serve_built(listener, &config, service, shutdown).await
}

/// Serve on a caller-supplied listener with an explicit `Runtime`.
///
/// Builds a Postgres-backed service when `[store] url` is set, and a service
/// with no store when it is not -- which is what a scaffolded deployment looks
/// like before its database exists, and what the chaos tool runs when a
/// scenario gives it neither a URL nor a seed.
///
/// # Errors
/// The listener's address cannot be read, the store URL does not parse, or the
/// server fails.
pub async fn serve_with(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    // Lazy: the first query connects. A service with an unreachable database
    // still starts and says so through readiness, rather than crash-looping
    // before it can report why. An empty URL means no store at all, which is
    // what a scaffolded deployment looks like before its database exists.
    let service = if config.store.url.is_empty() {
        tracing::info!("no store configured; data RPCs answer unavailable");
        Finance::new(config.ping.clone(), runtime)
    } else {
        let pool = tbd_db::connect_lazy(&tbd_db::PgOptions {
            url: config.store.url.clone(),
            max_connections: config.store.max_connections,
            ..tbd_db::PgOptions::default()
        })
        .map_err(|e| ServeError::Store(e.to_string()))?;

        // The sync worker runs beside the server, on the same pool, only
        // when a bank is configured. Without credentials there is nothing to
        // call, and saying so once at start beats a worker that wakes every
        // fifteen minutes to fail.
        if config.provider.configured() {
            let provider = banking::from_config(&config.provider)
                .map_err(|e| ServeError::Provider(e.to_string()))?;
            let syncer = sync::Syncer::new(
                pool.clone(),
                std::sync::Arc::new(provider),
                config.sync.clone(),
            );
            let cancel = tokio_util::sync::CancellationToken::new();
            let worker = tokio::spawn(syncer.run(cancel.clone()));
            tracing::info!(
                interval_secs = config.sync.interval_secs,
                "sync worker started"
            );
            let service = Finance::with_pool(config.ping.clone(), runtime, pool);
            let result = serve_built(listener, &config, service, shutdown).await;
            cancel.cancel();
            let _ = worker.await;
            return result;
        }
        tracing::info!("no provider configured; sync worker not started");
        Finance::with_pool(config.ping.clone(), runtime, pool)
    };
    serve_built(listener, &config, service, shutdown).await
}

/// Everything the entry points share: health, reflection, the listener and the
/// server itself. The only thing that differs between them is which store the
/// service was built with.
async fn serve_built(
    listener: TcpListener,
    config: &Config,
    service: Finance,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let addr = listener.local_addr().map_err(|source| ServeError::Bind {
        addr: config.server.listen,
        source,
    })?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<FinanceServiceServer<Finance>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::finance::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    tracing::info!(%addr, version = tbd_common::VERSION, "finance listening");

    // `tcp_nodelay` on the builder only applies to tonic's own listener. With
    // a caller-supplied listener it must be set on the incoming stream, or
    // small responses stall ~40 ms on Nagle + delayed ACK.
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    Server::builder()
        .trace_fn(tbd_common::telemetry::grpc_request_span)
        .add_service(health_service)
        .add_service(reflection)
        .add_service(FinanceServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await?;

    tracing::info!("finance stopped");
    Ok(())
}
