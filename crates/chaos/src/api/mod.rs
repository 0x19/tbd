//! `chaos serve`: the tool as a long-running HTTP API for the admin UI.
//!
//! Everything the CLI does is reachable here, plus what a UI needs on top:
//! a stack that stays up between calls and can be perturbed by hand, scenario
//! files that can be read, checked, written and run, run records that are
//! kept on disk, and live progress over Server-Sent Events.
//!
//! Routes are mounted under `[serve] base_path` (default `/api/chaos/v1`); the
//! contract is `docs/chaos/api.md`. The built UI, when configured, is served
//! at `[serve] ui_path`.

mod added;
mod error;
mod findings;
mod jobs;
mod notify;
mod routes;
mod runs;
mod state;
mod ui;

use std::{future::Future, path::Path, sync::Arc};

use axum::Router;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;

pub use added::{AddedInstance, AddedSpec};
pub use error::ApiError;
pub use findings::{FindingFilter, FindingGroup, FindingStore};
pub use jobs::{Job, QueuedRun, Schedule, ScheduleSpec};
pub use notify::{Notifier, NotifyMode};
pub use runs::{RunFeed, RunKind, RunRecord, RunStatus, RunStore, RunSummary, StressResult};
pub use state::AppState;
pub use state::{ReplayRequest, StressRequest};

use crate::config::{ChaosConfig, Source};

/// Build the state: load run records and schedules, start the stack when
/// configured, and start the scheduler that fires due schedules once a second
/// until shutdown.
pub async fn state(config: ChaosConfig, source: Source) -> anyhow::Result<Arc<AppState>> {
    let state = AppState::new(config, source).await?;
    tokio::spawn(scheduler(Arc::clone(&state)));
    Ok(state)
}

async fn scheduler(state: Arc<AppState>) {
    let stop = state.stopped();
    let mut tick = tokio::time::interval(std::time::Duration::from_secs(1));
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
    loop {
        tokio::select! {
            () = stop.cancelled() => return,
            _ = tick.tick() => state.tick_schedules(chrono::Utc::now()).await,
        }
    }
}

/// The full router: API under `base_path`, UI under `ui_path` when set.
///
/// Unknown paths under `base_path` answer the API's JSON 404, not the UI's
/// 404 page: with the UI at the root its wildcard would otherwise catch them.
pub fn router(state: &Arc<AppState>) -> Router {
    let base = state.config.serve.base_path.clone();
    let api = routes::router()
        .fallback(|| async { ApiError::not_found("no such route") })
        .layer(CorsLayer::permissive());
    let mut app = Router::new().nest(&base, api.with_state(Arc::clone(state)));
    if !state.config.serve.ui_dir.is_empty() {
        let dir = Path::new(&state.config.serve.ui_dir);
        if dir.is_dir() {
            app = app.merge(ui::router(
                &state.config.serve.ui_path,
                dir,
                &state.config.serve.base_path,
            ));
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
