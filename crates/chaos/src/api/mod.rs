//! `chaos serve`: the tool as a long-running HTTP API for the admin UI.
//!
//! Everything the CLI does is reachable here, plus what a UI needs on top:
//! a stack that stays up between calls and can be perturbed by hand, scenario
//! files that can be read, checked, written and run, run records that are
//! kept on disk, and live progress over Server-Sent Events.
//!
//! Routes are mounted under `[serve] base_path` (default `/api/chaos`); the
//! contract is `docs/chaos/api.md`. The built UI, when configured, is served
//! at `[serve] ui_path`.

mod error;
mod routes;
mod runs;
mod state;
mod ui;

use std::{future::Future, path::Path, sync::Arc};

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

pub use error::ApiError;
pub use runs::{RunFeed, RunKind, RunRecord, RunStatus, RunStore, RunSummary};
pub use state::AppState;

use crate::config::{ChaosConfig, Source};

/// Build the state: load run records and, when configured, start the stack.
pub async fn state(config: ChaosConfig, source: Source) -> anyhow::Result<Arc<AppState>> {
    AppState::new(config, source).await
}

/// The full router: API under `base_path`, UI under `ui_path` when set.
pub fn router(state: &Arc<AppState>) -> Router {
    let base = state.config.serve.base_path.clone();
    let api = routes::router().layer(CorsLayer::permissive());
    let mut app = Router::new().nest(&base, api.with_state(Arc::clone(state)));
    if !state.config.serve.ui_dir.is_empty() {
        let dir = Path::new(&state.config.serve.ui_dir);
        if dir.is_dir() {
            app = app.merge(ui::router(&state.config.serve.ui_path, dir));
        } else {
            tracing::warn!(dir = %dir.display(), "ui_dir does not exist; UI not served");
        }
    }
    app
}

/// Serve on `listener` until `shutdown` resolves, then stop the stack.
pub async fn serve(
    state: Arc<AppState>,
    listener: TcpListener,
    shutdown: impl Future<Output = ()> + Send + 'static,
) -> anyhow::Result<()> {
    let app = router(&state);
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown)
        .await?;
    state.shutdown().await;
    Ok(())
}
