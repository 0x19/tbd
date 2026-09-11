//! The facts ledger: append-only facts about an opaque subject, with
//! provenance, tombstones and erasure, on a [`store::Store`] (Postgres, or in
//! memory for tests and embedders), and a thin gRPC service on top.
//!
//! The library exposes [`serve`], [`serve_on`] and [`serve_with`] so the same
//! server can be run from `main`, from integration tests on an ephemeral port,
//! and from the chaos tool with fault injection and counters attached;
//! [`serve_store`] takes an explicit store for embedders that build their own.

pub mod clickhouse;
pub mod config;
pub mod health;
pub mod outbox;
mod service;
pub mod store;
pub mod sweeper;

use std::{net::SocketAddr, sync::Arc, time::Duration};

use tokio_util::sync::CancellationToken;

use tbd_proto::ledger::v1::ledger_service_server::LedgerServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Overrides, Source};
pub use service::Ledger;
pub use store::{
    Store, StoreError, StoreKind,
    memory::MemoryStore,
    pg::{PgOptions, PgStore},
};
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
pub async fn build_store(config: &Config) -> Result<Arc<dyn Store>, ServeError> {
    config.validate()?;
    match config.store.kind {
        StoreKind::Memory => Ok(Arc::new(MemoryStore::new())),
        StoreKind::Postgres => {
            let store = PgStore::connect_lazy(&PgOptions {
                url: config.store.url.clone(),
                max_connections: config.store.max_connections,
                acquire_timeout: config.store.acquire_timeout,
            })?;
            if config.store.migrate_on_start {
                store.migrate().await?;
                tracing::info!("migrations applied");
            }
            Ok(Arc::new(store))
        }
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
    // Readiness follows the store: one probe before accepting, then a loop.
    let up = health::probe_once(&store, config.health.probe_timeout)
        .await
        .map_err(|error| {
            tracing::warn!(%error, store = %store.kind(), "store unreachable at start; not serving until it answers");
        })
        .is_ok();
    health::report(&health_reporter, up).await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::ledger::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let service = Ledger::new(
        config.ping.clone(),
        config.erasure.clone(),
        runtime,
        Arc::clone(&store),
    );

    // Background work stops with the server: the shutdown future cancels the
    // token, the tasks watch it, and the server waits for them at the end.
    let cancel = CancellationToken::new();
    let mut tasks = tokio::task::JoinSet::new();
    tasks.spawn(health::run(
        Arc::clone(&store),
        health_reporter,
        config.health.probe_interval,
        config.health.probe_timeout,
        up,
        cancel.clone(),
    ));
    tasks.spawn(
        sweeper::Sweeper::new(
            Arc::clone(&store),
            config.erasure.grace,
            config.erasure.batch,
            config.idempotency.ttl,
            config.erasure.sweep_interval,
        )
        .run(cancel.clone()),
    );
    let publisher = build_publisher(&config).await?;
    tasks.spawn(
        outbox::Drainer::new(
            Arc::clone(&store),
            publisher,
            config.analytics.batch,
            config.analytics.period,
            config.analytics.lease,
        )
        .run(cancel.clone()),
    );
    if let Some(pg) = store_as_pg(&store) {
        tasks.spawn(pool_gauges(pg, cancel.clone()));
    }
    let server_shutdown = {
        let cancel = cancel.clone();
        async move {
            shutdown.await;
            cancel.cancel();
        }
    };

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
        .serve_with_incoming_shutdown(incoming, server_shutdown)
        .await?;

    cancel.cancel();
    while tasks.join_next().await.is_some() {}
    tracing::info!("ledger stopped");
    Ok(())
}

/// The outbox sink: `ClickHouse` when `[analytics] clickhouse_url` is set,
/// otherwise a recorder that counts and drops (analytics off).
///
/// # Errors
/// The `ClickHouse` URL does not parse.
async fn build_publisher(config: &Config) -> Result<Arc<dyn outbox::Publisher>, ServeError> {
    if config.analytics.clickhouse_url.is_empty() {
        tracing::info!("analytics off: outbox events are counted and dropped");
        return Ok(Arc::new(outbox::RecordingPublisher::default()));
    }
    let publisher = clickhouse::ClickHousePublisher::new(&config.analytics.clickhouse_url)
        .map_err(|e| ServeError::Store(StoreError::Internal(e.to_string())))?;
    match publisher.ensure_schema().await {
        Ok(()) => tracing::info!(
            table = clickhouse::TABLE,
            "analytics on: outbox drains into ClickHouse"
        ),
        Err(error) => {
            tracing::warn!(%error, "ClickHouse unreachable at start; the drainer retries");
        }
    }
    Ok(Arc::new(publisher))
}

/// The Postgres store behind the trait object, if that is what it is.
fn store_as_pg(store: &Arc<dyn Store>) -> Option<PgStore> {
    (store.kind() == StoreKind::Postgres)
        .then(|| store.as_any().downcast_ref::<PgStore>().cloned())
        .flatten()
}

/// Sample the pool every five seconds, like the process collector does.
async fn pool_gauges(pg: PgStore, cancel: CancellationToken) {
    loop {
        let (idle, in_use) = pg.pool_counts();
        metrics::gauge!(tbd_common::metrics::names::DB_POOL_CONNECTIONS, "state" => "idle")
            .set(f64::from(idle));
        metrics::gauge!(tbd_common::metrics::names::DB_POOL_CONNECTIONS, "state" => "in_use")
            .set(f64::from(in_use));
        tokio::select! {
            () = cancel.cancelled() => break,
            () = tokio::time::sleep(Duration::from_secs(5)) => {}
        }
    }
}
