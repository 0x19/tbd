//! The one executor: setup, load with timeline, assert, teardown.

use std::{collections::BTreeMap, path::Path, sync::Arc, time::Instant};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    load::{self, LoadSnapshot, Metrics, Target},
    service::RequestCounts,
};

use super::{
    ScenarioFile,
    assertions::{AssertionResult, Snapshot},
};

/// Outcome of one scenario.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScenarioResult {
    /// From `[scenario] name`.
    pub name: String,
    /// Source file, when run from one.
    pub file: Option<String>,
    /// All assertions passed and nothing broke.
    pub passed: bool,
    /// `skip = true`; nothing ran.
    pub skipped: bool,
    /// Wall time.
    pub duration_s: f64,
    /// Load metrics, when load ran.
    pub load: Option<LoadSnapshot>,
    /// Engine counters at the end.
    pub services: BTreeMap<String, RequestCounts>,
    /// Timeline events as applied, with outcome.
    pub events: Vec<EventOutcome>,
    /// Assertion results.
    pub assertions: Vec<AssertionResult>,
    /// Failure outside assertions: setup, load, or a timeline action.
    pub error: Option<String>,
}

/// One applied timeline event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventOutcome {
    /// Seconds after load start when it fired.
    pub at_s: f64,
    /// What it did.
    pub action: String,
    /// Error, if applying failed.
    pub error: Option<String>,
}

/// Something that happened while a scenario ran, in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunEvent {
    /// Entered a phase: `setup`, `load`, `assert`, `teardown`.
    Phase {
        /// Phase name.
        name: String,
    },
    /// A load snapshot, once per [`load::PROGRESS_INTERVAL`].
    Load {
        /// Snapshot.
        snapshot: LoadSnapshot,
    },
    /// A timeline action fired.
    Timeline {
        /// What happened.
        event: EventOutcome,
    },
}

/// Optional observation and control of a scenario run.
#[derive(Debug, Clone, Default)]
pub struct Hooks {
    /// Receives every [`RunEvent`].
    pub events: Option<mpsc::UnboundedSender<RunEvent>>,
    /// Cancel: load stops, pending timeline events are skipped, the stack is
    /// torn down and the result carries `error = "cancelled"`.
    pub cancel: CancellationToken,
}

impl Hooks {
    fn emit(&self, event: RunEvent) {
        if let Some(tx) = &self.events {
            let _ = tx.send(event);
        }
    }
}

/// Run a scenario from a file.
pub async fn run_file(path: &Path) -> ScenarioResult {
    run_file_with(path, &Hooks::default()).await
}

/// [`run_file`] with hooks.
pub async fn run_file_with(path: &Path, hooks: &Hooks) -> ScenarioResult {
    match ScenarioFile::from_path(path) {
        Ok(file) => {
            let mut result = run_scenario_with(&file, hooks).await;
            result.file = Some(path.display().to_string());
            result
        }
        Err(e) => ScenarioResult {
            name: path.display().to_string(),
            file: Some(path.display().to_string()),
            passed: false,
            skipped: false,
            duration_s: 0.0,
            load: None,
            services: BTreeMap::new(),
            events: Vec::new(),
            assertions: Vec::new(),
            error: Some(e.to_string()),
        },
    }
}

/// Run a parsed scenario.
pub async fn run_scenario(file: &ScenarioFile) -> ScenarioResult {
    run_scenario_with(file, &Hooks::default()).await
}

/// [`run_scenario`] with hooks.
#[allow(clippy::too_many_lines)]
pub async fn run_scenario_with(file: &ScenarioFile, hooks: &Hooks) -> ScenarioResult {
    let started = Instant::now();
    let mut result = ScenarioResult {
        name: file.scenario.name.clone(),
        file: None,
        passed: false,
        skipped: file.scenario.skip,
        duration_s: 0.0,
        load: None,
        services: BTreeMap::new(),
        events: Vec::new(),
        assertions: Vec::new(),
        error: None,
    };
    if file.scenario.skip {
        result.passed = true;
        return result;
    }
    tracing::info!(scenario = %file.scenario.name, "setup");
    hooks.emit(RunEvent::Phase {
        name: "setup".into(),
    });

    let stack = match file.stack.start().await {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => {
            result.error = Some(format!("setup: {e}"));
            result.duration_s = started.elapsed().as_secs_f64();
            return result;
        }
    };

    let targets: Vec<Target> = crate::kinds::load_targets(&*stack.lock().await);

    // Timeline runs alongside load, from the moment load starts.
    let load_start = Instant::now();
    hooks.emit(RunEvent::Phase {
        name: "load".into(),
    });
    let timeline = {
        let hooks = hooks.clone();
        spawn_timeline(
            file.timeline.clone(),
            Arc::clone(&stack),
            load_start,
            hooks.cancel.clone(),
            move |event| hooks.emit(RunEvent::Timeline { event }),
        )
    };

    if let Some(load_config) = &file.load {
        let metrics = Arc::new(Metrics::new());
        let (tx, mut rx) = mpsc::unbounded_channel();
        let forward_hooks = hooks.clone();
        let forward = tokio::spawn(async move {
            while let Some(snapshot) = rx.recv().await {
                forward_hooks.emit(RunEvent::Load { snapshot });
            }
        });
        let load_hooks = load::Hooks {
            progress: Some(tx),
            cancel: hooks.cancel.clone(),
        };
        result.load = Some(
            load::run_with(
                load_config,
                &targets,
                metrics,
                &load_hooks,
                &crate::tls::Trust::default(),
            )
            .await,
        );
        drop(load_hooks);
        let _ = forward.await;
    } else if let Some(last) = file
        .timeline
        .iter()
        .map(super::timeline::TimelineEvent::at)
        .max()
    {
        // No load: just let the timeline play out.
        tokio::select! {
            () = tokio::time::sleep_until((load_start + last).into()) => {}
            () = hooks.cancel.cancelled() => {}
        }
    }

    if let Ok(outcomes) = timeline.await {
        result.events = outcomes;
    } else {
        result.error = Some("timeline task panicked".into());
    }

    let Ok(stack) = Arc::try_unwrap(stack) else {
        result.error = Some("stack still shared after timeline finished".into());
        result.duration_s = started.elapsed().as_secs_f64();
        return result;
    };
    let stack = stack.into_inner();
    result.services = stack.request_counts();

    hooks.emit(RunEvent::Phase {
        name: "assert".into(),
    });
    result.assertions = file.assertions.evaluate(&Snapshot {
        load: result.load.as_ref(),
        engines: &result.services,
    });
    if hooks.cancel.is_cancelled() && result.error.is_none() {
        result.error = Some("cancelled".into());
    }

    tracing::info!(scenario = %file.scenario.name, "teardown");
    hooks.emit(RunEvent::Phase {
        name: "teardown".into(),
    });
    stack.shutdown().await;

    let events_ok = result.events.iter().all(|e| e.error.is_none());
    result.passed =
        result.error.is_none() && events_ok && result.assertions.iter().all(|a| a.passed);
    result.duration_s = started.elapsed().as_secs_f64();
    result
}

/// Play `events` against `stack` from `load_start`, in order, until cancelled;
/// every applied action goes through `emit` and comes back in the result. The
/// one timeline runner: scenarios and stress campaigns both use it.
pub(crate) fn spawn_timeline(
    mut events: Vec<super::timeline::TimelineEvent>,
    stack: Arc<Mutex<crate::stack::Stack>>,
    load_start: Instant,
    cancel: CancellationToken,
    emit: impl Fn(EventOutcome) + Send + 'static,
) -> tokio::task::JoinHandle<Vec<EventOutcome>> {
    events.sort_by_key(super::timeline::TimelineEvent::at);
    tokio::spawn(async move {
        let mut outcomes = Vec::new();
        for event in events {
            tokio::select! {
                () = tokio::time::sleep_until((load_start + event.at()).into()) => {}
                () = cancel.cancelled() => break,
            }
            tracing::info!(at = ?event.at(), action = %event.describe(), "timeline");
            let outcome = {
                let mut s = stack.lock().await;
                event.apply(&mut s).await
            };
            if let Err(error) = &outcome {
                tracing::error!(%error, action = %event.describe(), "timeline action failed");
            }
            let applied = EventOutcome {
                at_s: load_start.elapsed().as_secs_f64(),
                action: event.describe(),
                error: outcome.err(),
            };
            emit(applied.clone());
            outcomes.push(applied);
        }
        outcomes
    })
}
