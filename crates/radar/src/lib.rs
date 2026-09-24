//! The radar service: what changed in Go and Rust this week. It reads the
//! official sources on a timer, keeps every item, and once a week asks the
//! llm service for a digest per language and reader language (RFC 0007).
//!
//! The library exposes [`serve`], [`serve_on`] and [`serve_with`] so the same
//! server can be run from `main`, from integration tests on an ephemeral port,
//! and from the chaos tool with fault injection and counters attached.

pub mod config;
pub mod digest;
pub mod fetch;
mod service;
pub mod store;
pub mod worker;

use std::net::SocketAddr;

use tbd_proto::radar::v1::radar_service_server::RadarServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Overrides, Source};
pub use service::{ADMIN_ROLE, Radar};
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
    /// The store's pool could not be built.
    #[error("store: {0}")]
    Store(String),
    /// The HTTP client for the sources could not be built.
    #[error("fetch: {0}")]
    Fetch(#[from] fetch::FetchError),
    /// The llm URL is not usable.
    #[error("llm: {0}")]
    Llm(#[from] digest::DigestError),
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
pub async fn serve_with(
    listener: TcpListener,
    config: Config,
    runtime: Runtime,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> Result<(), ServeError> {
    let addr = listener.local_addr().map_err(|source| ServeError::Bind {
        addr: config.server.listen,
        source,
    })?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter
        .set_serving::<RadarServiceServer<Radar>>()
        .await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::radar::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let mut service = Radar::new(config.ping.clone(), runtime);
    let cancel = tokio_util::sync::CancellationToken::new();
    let mut worker_task = None;
    if config.store.url.is_empty() {
        tracing::info!("no store configured; every RPC but Ping answers unavailable, no timers");
    } else {
        // Lazy: the first query connects, so an unreachable database still
        // lets the service listen and say so through readiness.
        let pool = tbd_db::connect_lazy(&tbd_db::PgOptions {
            url: config.store.url.clone(),
            max_connections: config.store.max_connections,
            ..tbd_db::PgOptions::default()
        })
        .map_err(|e| ServeError::Store(e.to_string()))?;
        let store = store::Store::new(pool);
        let writer = digest::Writer::new(&config.llm)?;
        if writer.is_none() {
            tracing::info!("no llm url; items are read, digests are not written");
        }
        let worker = worker::Worker::new(
            store.clone(),
            fetch::Fetcher::new(config.fetch.clone())?,
            writer,
            config.sources.clone(),
            config.fetch.clone(),
            config.digest.clone(),
        );
        service = service.with_store(store, worker.clone(), config.digest.page_size);
        tracing::info!(
            sources = config.sources.len(),
            fetch_every_secs = config.fetch.interval_secs,
            digest_weekday = config.digest.weekday,
            digest_hour = config.digest.hour,
            "radar worker started"
        );
        worker_task = Some(tokio::spawn(worker.run(cancel.clone())));
    }

    tracing::info!(%addr, version = tbd_common::VERSION, "radar listening");

    // `tcp_nodelay` on the builder only applies to tonic's own listener. With
    // a caller-supplied listener it must be set on the incoming stream, or
    // small responses stall ~40 ms on Nagle + delayed ACK.
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    let result = Server::builder()
        .trace_fn(tbd_common::telemetry::grpc_request_span)
        .add_service(health_service)
        .add_service(reflection)
        .add_service(RadarServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await;
    cancel.cancel();
    if let Some(task) = worker_task {
        let _ = task.await;
    }
    result?;

    tracing::info!("radar stopped");
    Ok(())
}
