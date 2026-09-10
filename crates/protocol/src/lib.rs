//! The protocol gateway.
//!
//! One listener, four surfaces:
//!
//! | Path        | Surface                                  |
//! |-------------|------------------------------------------|
//! | `/v1/*`     | REST (JSON) and server-sent events       |
//! | `/ws`       | WebSocket bridged to an engine session   |
//! | `/graphql`  | GraphQL (POST) and `GraphiQL` (GET)      |
//! | gRPC        | `tbd.protocol.v1.Gateway` + health, h2c  |
//!
//! The gateway holds no business logic. Every request is translated and
//! forwarded to the engine; every engine `stub` flag is forwarded untouched.

mod config;
mod error;
mod graphql;
mod grpc;
mod http;
mod state;
mod ws;

use std::net::SocketAddr;

use axum::Router;
use tokio::net::TcpListener;
use tower_http::trace::TraceLayer;

pub use config::Config;
pub use error::ApiError;
pub use state::AppState;

/// Errors from starting or running the gateway.
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
    /// The engine URL did not parse.
    #[error("engine url {url:?}: {source}")]
    EngineUrl {
        /// The configured URL.
        url: String,
        /// Underlying transport error.
        source: tonic::transport::Error,
    },
    /// The HTTP server failed while running.
    #[error("serve: {0}")]
    Io(#[from] std::io::Error),
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
    let addr = listener.local_addr()?;
    let state = AppState::connect_lazy(&config)?;
    let app = router(&state);

    tracing::info!(%addr, engine = %config.engine_url, version = tbd_common::VERSION, "protocol listening");

    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;

    tracing::info!("protocol stopped");
    Ok(())
}

/// The full router: REST, SSE, WebSocket, GraphQL and gRPC on one port.
pub fn router(state: &AppState) -> Router {
    Router::new()
        .merge(http::routes())
        .merge(ws::routes())
        .merge(graphql::routes(state))
        .with_state(state.clone())
        .merge(grpc::routes(state))
        .layer(TraceLayer::new_for_http())
}
