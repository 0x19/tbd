//! The one executor: setup, load with timeline, assert, teardown.

use std::{collections::BTreeMap, path::Path, sync::Arc, time::Instant};

use serde::Serialize;
use tokio::sync::Mutex;

use crate::{
    load::{self, Metrics, Target},
    service::RequestCounts,
};

use super::{
    ScenarioFile,
    assertions::{AssertionResult, Snapshot},
};

/// Outcome of one scenario.
#[derive(Debug, Clone, Serialize)]
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
    pub load: Option<load::LoadSnapshot>,
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
#[derive(Debug, Clone, Serialize)]
pub struct EventOutcome {
    /// Seconds after load start when it fired.
    pub at_s: f64,
    /// What it did.
    pub action: String,
    /// Error, if applying failed.
    pub error: Option<String>,
}

/// Run a scenario from a file.
pub async fn run_file(path: &Path) -> ScenarioResult {
    match ScenarioFile::from_path(path) {
        Ok(file) => {
            let mut result = run_scenario(&file).await;
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

    let stack = match file.stack.start().await {
        Ok(s) => Arc::new(Mutex::new(s)),
        Err(e) => {
            result.error = Some(format!("setup: {e}"));
            result.duration_s = started.elapsed().as_secs_f64();
            return result;
        }
    };

    let targets: Vec<Target> = {
        let s = stack.lock().await;
        s.of_kind("protocol")
            .iter()
            .map(|i| Target {
                name: i.name.clone(),
                http_url: i.http_url(),
            })
            .collect()
    };

    // Timeline runs alongside load, from the moment load starts.
    let mut events = file.timeline.clone();
    events.sort_by_key(super::timeline::TimelineEvent::at);
    let timeline_stack = Arc::clone(&stack);
    let load_start = Instant::now();
    let timeline = tokio::spawn(async move {
        let mut outcomes = Vec::new();
        for event in events {
            tokio::time::sleep_until((load_start + event.at()).into()).await;
            tracing::info!(at = ?event.at(), action = %event.describe(), "timeline");
            let outcome = {
                let mut s = timeline_stack.lock().await;
                event.apply(&mut s).await
            };
            if let Err(error) = &outcome {
                tracing::error!(%error, action = %event.describe(), "timeline action failed");
            }
            outcomes.push(EventOutcome {
                at_s: load_start.elapsed().as_secs_f64(),
                action: event.describe(),
                error: outcome.err(),
            });
        }
        outcomes
    });

    if let Some(load_config) = &file.load {
        let metrics = Arc::new(Metrics::new());
        result.load = Some(load::run(load_config, &targets, metrics).await);
    } else if let Some(last) = file
        .timeline
        .iter()
        .map(super::timeline::TimelineEvent::at)
        .max()
    {
        // No load: just let the timeline play out.
        tokio::time::sleep_until((load_start + last).into()).await;
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

    result.assertions = file.assertions.evaluate(&Snapshot {
        load: result.load.as_ref(),
        engines: &result.services,
    });

    tracing::info!(scenario = %file.scenario.name, "teardown");
    stack.shutdown().await;

    let events_ok = result.events.iter().all(|e| e.error.is_none());
    result.passed =
        result.error.is_none() && events_ok && result.assertions.iter().all(|a| a.passed);
    result.duration_s = started.elapsed().as_secs_f64();
    result
}
