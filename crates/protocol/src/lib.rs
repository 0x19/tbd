//! The protocol service.
//!
//! One listener, four surfaces:
//!
//! | Path        | Surface                                  |
//! |-------------|------------------------------------------|
//! | `/v1/*`     | REST (JSON) and server-sent events       |
//! | `/ws`       | WebSocket bridged to an engine session   |
//! | `/graphql`  | GraphQL (POST) and `GraphiQL` (GET)      |
//! | gRPC        | `tbd.protocol.v1.Protocol` + health, h2c  |
//!
//! The protocol holds no business logic. Every request is translated and
//! forwarded to a registered backend (`[services]` in `configs/protocol`); every
//! engine `stub` flag is forwarded untouched. Every body is JSON, in and out:
//! HTTP responses, SSE payloads, WebSocket text frames, request bodies; errors
//! are one envelope ([`Problem`]) on every surface.

mod config;
mod error;
mod graphql;
mod grpc;
mod http;
pub mod json;
mod observe;
mod state;
pub mod subject;
mod ws;

use std::net::SocketAddr;

use axum::{Router, serve::ListenerExt as _};
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub use config::{
    Config, ENGINE, Health, Metrics, Overrides, Principals, Server, ServiceConfig, Source,
    grpc_service_name, service_url_var,
};
pub use error::{Code, Detail, Problem, Wire};
pub use grpc::ENGINE_SERVICE;
pub use state::{AppState, Backend, EngineClient, Readiness, ServiceState, Transport};

/// Errors from starting or running the protocol.
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
    /// A backend URL did not parse.
    #[error("services.{name}.url {url:?}: {source}")]
    BackendUrl {
        /// Registry name.
        name: String,
        /// The configured URL.
        url: String,
        /// Underlying transport error.
        source: tonic::transport::Error,
    },
    /// The registry lacks a backend the protocol cannot run without.
    #[error("[services.{0}] is not configured")]
    MissingBackend(&'static str),
    /// The HTTP server failed while running.
    #[error("serve: {0}")]
    Io(#[from] std::io::Error),
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
    let addr = listener.local_addr()?;
    let state = AppState::connect_lazy(&config)?;
    let app = router(&state);

    let backends: Vec<String> = config
        .services
        .iter()
        .map(|(name, s)| format!("{name}={}", s.url))
        .collect();
    tracing::info!(%addr, ?backends, version = tbd_common::VERSION, "protocol listening");

    // Accepted sockets keep Nagle on by default. A WebSocket upgrade writes
    // the 101 and the first frame back to back, so without this the first
    // frame waits ~40 ms for the client's delayed ACK. Found by the chaos
    // baseline scenario.
    let listener = listener.tap_io(|tcp| {
        if let Err(error) = tcp.set_nodelay(true) {
            tracing::warn!(%error, "failed to set TCP_NODELAY");
        }
    });

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    tracing::info!("protocol stopped");
    Ok(())
}

/// Unmatched paths. tonic's router carries its own fallback (HTTP 200 with
/// `grpc-status: 12`), which the merge would otherwise apply to every unknown
/// REST path too; on a public edge that turns every scanner probe into a 200.
/// gRPC callers keep the gRPC answer, everything else gets a JSON 404.
async fn fallback(request: axum::extract::Request) -> axum::response::Response {
    if observe::is_grpc(request.headers()) {
        tonic::Status::unimplemented("unknown gRPC method").into_http()
    } else {
        axum::response::IntoResponse::into_response(Problem::not_found(request.uri().path()))
    }
}

/// The full router: REST, SSE, WebSocket, GraphQL and gRPC on one port, each
/// request traced and measured.
pub fn router(state: &AppState) -> Router {
    Router::new()
        .merge(http::routes())
        .merge(ws::routes())
        .merge(graphql::routes(state))
        .with_state(state.clone())
        .merge(grpc::routes(state))
        .fallback(fallback)
        .layer(axum::middleware::from_fn(subject::attach))
        .layer(axum::middleware::from_fn(observe::metrics))
        .layer(TraceLayer::new_for_http().make_span_with(observe::make_span))
}
