//! The cv service: the full CV behind sign-in and the owner's approval.
//!
//! The public site carries the CV without contact details. Anyone who signed
//! in may ask for the full version; the owner is told by mail and decides;
//! an approved person downloads a PDF rendered for them, with their name on
//! every page, and every download is recorded. The library exposes
//! [`serve`], [`serve_on`] and [`serve_with`] so the same server runs from
//! `main`, from integration tests on an ephemeral port, and from the chaos
//! tool with fault injection and counters attached.

pub mod config;
pub mod notify;
pub mod private;
pub mod render;
mod service;
pub mod store;

use std::{net::SocketAddr, time::Duration};

use tbd_proto::cv::v1::cv_service_server::CvServiceServer;
use tokio::net::TcpListener;
use tonic::transport::{Server, server::TcpIncoming};

pub use config::{Config, Overrides, Source};
pub use service::Cv;
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
    /// The store URL does not parse.
    #[error("store: {0}")]
    Store(String),
    /// The private fields did not load.
    #[error("{0}")]
    Private(#[from] private::PrivateError),
    /// The finance URL does not parse.
    #[error("finance url: {0}")]
    Finance(tonic::transport::Error),
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
///
/// With `[store] url` set the service is real; without it every RPC but
/// `Ping` answers `UNAVAILABLE`, which is what a scaffolded deployment looks
/// like before its database exists. The private fields and the notifier are
/// each optional in the same way.
///
/// # Errors
/// The listener's address cannot be read, a URL does not parse, the private
/// fields do not load, or the server fails.
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
    health_reporter.set_serving::<CvServiceServer<Cv>>().await;

    let reflection = tonic_reflection::server::Builder::configure()
        .register_encoded_file_descriptor_set(tbd_proto::cv::v1::DESCRIPTOR_SET)
        .register_encoded_file_descriptor_set(tonic_health::pb::FILE_DESCRIPTOR_SET)
        .build_v1()?;

    let mut service = Cv::new(config.ping.clone(), runtime)
        .with_render_timeout(Duration::from_secs(config.render.timeout_secs));
    if config.store.url.is_empty() {
        tracing::info!("no store configured; every RPC but Ping answers unavailable");
    } else {
        // Lazy: the first query connects, so an unreachable database still
        // lets the service listen and say so through readiness.
        let pool = tbd_db::connect_lazy(&tbd_db::PgOptions {
            url: config.store.url.clone(),
            max_connections: config.store.max_connections,
            ..tbd_db::PgOptions::default()
        })
        .map_err(|e| ServeError::Store(e.to_string()))?;
        service = service.with_pool(pool);
    }
    let private = private::load(&config.private.json, &config.private.path)?;
    tracing::info!(private = ?private, "private fields");
    service = service.with_private(private);
    let notifier =
        notify::Notifier::new(&config.finance.url, &config.notify).map_err(ServeError::Finance)?;
    if notifier.is_none() {
        tracing::info!("no finance url or no notify addresses; the owner is not told of requests");
    }
    service = service.with_notifier(notifier);

    // Fonts are parsed on the first render; do it now, off the runtime, so
    // the first download does not pay for it.
    tokio::task::spawn_blocking(render::warm);

    tracing::info!(%addr, version = tbd_common::VERSION, "cv listening");

    // `tcp_nodelay` on the builder only applies to tonic's own listener. With
    // a caller-supplied listener it must be set on the incoming stream, or
    // small responses stall ~40 ms on Nagle + delayed ACK.
    let incoming = TcpIncoming::from(listener).with_nodelay(Some(true));

    Server::builder()
        .trace_fn(tbd_common::telemetry::grpc_request_span)
        .add_service(health_service)
        .add_service(reflection)
        .add_service(CvServiceServer::new(service))
        .serve_with_incoming_shutdown(incoming, shutdown)
        .await?;

    tracing::info!("cv stopped");
    Ok(())
}
