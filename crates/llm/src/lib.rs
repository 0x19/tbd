//! The llm service: a gRPC server over the engines that run the weights.
//!
//! The library exposes [`serve`], [`serve_on`] and [`serve_with`] so the same
//! server can be run from `main`, from integration tests on an ephemeral port,
//! and from the chaos tool with fault injection and counters attached. The
//! engines (Ollama, llama.cpp, the test stub) are built from `[engines]` here
//! and handed to the service; nothing above [`engine`] names one.

pub mod admission;
pub mod config;
pub mod engine;
pub mod probe;
mod service;
pub mod store;

use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

use tbd_proto::llm::v1::llm_service_server::LlmServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Overrides, Source, Tier};
pub use service::Llm;
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
    /// The configuration cannot be served from.
    #[error("config: {0}")]
    Config(#[from] config::ValidationError),
    /// An engine could not be built from its configuration.
    #[error("{0}")]
    Engine(#[from] engine::BuildError),
    /// The store's pool could not be built from its URL.
    #[error("store: {0}")]
    Store(String),
    /// Reflection service could not be built from the descriptor set.
    #[error("reflection: {0}")]
    Reflection(#[from] tonic_reflection::server::Error),
    /// The gRPC server failed while running.
    #[error("transport: {0}")]
    Transport(#[from] tonic::transport::Error),
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
    config.validate()?;

    let (health_reporter, health_service) = tonic_health::server::health_reporter();
    health_reporter.set_serving::<LlmServiceServer<Llm>>().await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::llm::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let mut engines: BTreeMap<Tier, Arc<dyn engine::Engine>> = BTreeMap::new();
    for tier in Tier::ALL {
        let cfg = config.engine(tier);
        let built = engine::build(cfg)?;
        tracing::info!(%tier, engine = cfg.kind.as_str(), model = %cfg.model, stub = built.stub(), "engine");
        engines.insert(tier, built);
    }
    // Lazy: the first query connects, so an unreachable database still lets
    // the service listen; a generation then fails with UNAVAILABLE from the
    // store, which is the truthful answer.
    let pool = if config.store.url.is_empty() {
        tracing::warn!(
            "no store configured: generations are not recorded and the budget is unlimited"
        );
        None
    } else {
        Some(
            tbd_db::connect_lazy(&tbd_db::PgOptions {
                url: config.store.url.clone(),
                max_connections: config.store.max_connections,
                ..tbd_db::PgOptions::default()
            })
            .map_err(|e| ServeError::Store(e.to_string()))?,
        )
    };
    let status = probe::Status::default();
    let prober = tokio::spawn(probe::run(
        engines.clone(),
        status.clone(),
        config.engines.probe_interval,
        config.engines.probe_timeout,
    ));

    let mut service = Llm::new(&config, runtime, engines, status);
    if let Some(pool) = pool {
        service = service.with_pool(pool);
    }

    tracing::info!(%addr, version = tbd_common::VERSION, "llm listening");

    // `tcp_nodelay` on the builder only applies to tonic's own listener. With
    // a caller-supplied listener it must be set on the incoming stream, or
    // small responses stall ~40 ms on Nagle + delayed ACK.
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    let served = Server::builder()
        .trace_fn(tbd_common::telemetry::grpc_request_span)
        .add_service(health_service)
        .add_service(reflection)
        .add_service(LlmServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await;
    prober.abort();
    served?;

    tracing::info!("llm stopped");
    Ok(())
}
