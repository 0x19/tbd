//! Run records: one JSON file per run under `[paths] results`, an index in
//! memory, and a live feed for the run in progress.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::SystemTime,
};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, broadcast};
use tokio_util::sync::CancellationToken;

use crate::{
    load::LoadSnapshot,
    scenario::{ScenarioResult, executor::EventOutcome},
    validate,
};

/// What kind of run.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunKind {
    /// A scenario file: stack, load, timeline, assertions.
    Scenario,
    /// Ad-hoc load against the serve stack or external targets.
    Load,
    /// `chaos validate`.
    Validate,
}

/// Where a run is.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RunStatus {
    /// In progress.
    Running,
    /// Finished, every assertion or check passed.
    Passed,
    /// Finished, at least one assertion or check failed.
    Failed,
    /// Did not finish properly: setup failed, a timeline action errored, or
    /// it was cancelled.
    Error,
    /// Cancelled by the caller.
    Cancelled,
    /// Finished; load runs have nothing to pass or fail.
    Completed,
}

/// One run, as stored and returned.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunRecord {
    /// UUID v7: time-ordered.
    pub id: String,
    /// Kind.
    pub kind: RunKind,
    /// Scenario name, load run name, or `validate`.
    pub name: String,
    /// Scenario id (`scenarios/<id>.toml`) for scenario runs.
    #[serde(default)]
    pub scenario_id: Option<String>,
    /// The schedule that queued it, when one did.
    #[serde(default)]
    pub schedule_id: Option<String>,
    /// Status.
    pub status: RunStatus,
    /// RFC 3339.
    pub started_at: String,
    /// RFC 3339, once finished.
    #[serde(default)]
    pub finished_at: Option<String>,
    /// Wall time in seconds, once finished.
    #[serde(default)]
    pub duration_s: f64,
    /// Full scenario result.
    #[serde(default)]
    pub scenario: Option<ScenarioResult>,
    /// Final load snapshot of a load run.
    #[serde(default)]
    pub load: Option<LoadSnapshot>,
    /// Validate report.
    #[serde(default)]
    pub validate: Option<validate::Report>,
    /// Load snapshots taken once per second while load ran.
    #[serde(default)]
    pub samples: Vec<LoadSnapshot>,
    /// Timeline events as applied (scenario runs).
    #[serde(default)]
    pub events: Vec<EventOutcome>,
    /// The request that started an ad-hoc load run, for re-running.
    #[serde(default)]
    pub request: Option<serde_json::Value>,
    /// Failure outside assertions.
    #[serde(default)]
    pub error: Option<String>,
    /// The service kinds this run exercised, sorted: the scenario's stack, the
    /// load's target kinds, or the validated kinds. Records written before the
    /// field existed derive it from `request` where they can.
    #[serde(default)]
    pub services: Vec<String>,
}

/// The list view: a record without its bulky parts.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    /// Id.
    pub id: String,
    /// Kind.
    pub kind: RunKind,
    /// Name.
    pub name: String,
    /// Scenario id.
    pub scenario_id: Option<String>,
    /// Schedule that queued it.
    #[serde(default)]
    pub schedule_id: Option<String>,
    /// Status.
    pub status: RunStatus,
    /// Started.
    pub started_at: String,
    /// Finished.
    pub finished_at: Option<String>,
    /// Duration.
    pub duration_s: f64,
    /// Requests sent, when load ran.
    pub requests_total: Option<u64>,
    /// Error rate, when load ran.
    pub error_rate: Option<f64>,
    /// Throughput in requests per second, when load ran.
    pub throughput_rps: Option<f64>,
    /// p50 in ms, when load ran.
    pub p50_ms: Option<f64>,
    /// p90 in ms, when load ran.
    pub p90_ms: Option<f64>,
    /// p99 in ms, when load ran.
    pub p99_ms: Option<f64>,
    /// Assertions or checks: passed and total.
    pub passed: Option<(usize, usize)>,
    /// Error.
    pub error: Option<String>,
    /// The service kinds this run exercised, sorted.
    #[serde(default)]
    pub services: Vec<String>,
}

impl RunRecord {
    /// A fresh running record.
    pub fn start(kind: RunKind, name: &str) -> Self {
        Self {
            id: uuid::Uuid::now_v7().to_string(),
            kind,
            name: name.to_owned(),
            scenario_id: None,
            schedule_id: None,
            status: RunStatus::Running,
            started_at: now(),
            finished_at: None,
            duration_s: 0.0,
            scenario: None,
            load: None,
            validate: None,
            samples: Vec::new(),
            events: Vec::new(),
            request: None,
            error: None,
            services: Vec::new(),
        }
    }

    /// Record the kinds a run exercises, deduplicated and sorted.
    pub fn set_services<I, S>(&mut self, kinds: I)
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let mut v: Vec<String> = kinds.into_iter().map(Into::into).collect();
        v.sort();
        v.dedup();
        self.services = v;
    }

    /// The kinds of a record, deriving them from `request` for records written
    /// before `services` existed: a load's targets (`kind`, default
    /// `protocol`), validate's per-kind URL map.
    fn services(&self) -> Vec<String> {
        if !self.services.is_empty() {
            return self.services.clone();
        }
        let Some(request) = &self.request else {
            return Vec::new();
        };
        let mut v: Vec<String> = match self.kind {
            RunKind::Load => request
                .get("targets")
                .and_then(serde_json::Value::as_array)
                .map(|targets| {
                    targets
                        .iter()
                        .map(|t| {
                            t.get("kind")
                                .and_then(serde_json::Value::as_str)
                                .unwrap_or("protocol")
                                .to_owned()
                        })
                        .collect()
                })
                .unwrap_or_default(),
            RunKind::Validate => request
                .as_object()
                .map(|m| m.keys().filter(|k| *k != "timeout").cloned().collect())
                .unwrap_or_default(),
            RunKind::Scenario => Vec::new(),
        };
        v.sort();
        v.dedup();
        v
    }

    /// Mark finished now.
    pub fn finish(&mut self, status: RunStatus, duration_s: f64) {
        self.status = status;
        self.finished_at = Some(now());
        self.duration_s = duration_s;
    }

    /// Summary for lists.
    pub fn summary(&self) -> RunSummary {
        let load = self
            .scenario
            .as_ref()
            .and_then(|s| s.load.as_ref())
            .or(self.load.as_ref());
        let passed = match self.kind {
            RunKind::Scenario => self.scenario.as_ref().map(|s| {
                (
                    s.assertions.iter().filter(|a| a.passed).count(),
                    s.assertions.len(),
                )
            }),
            RunKind::Validate => self.validate.as_ref().map(|r| (r.passed, r.checks.len())),
            RunKind::Load => None,
        };
        RunSummary {
            id: self.id.clone(),
            kind: self.kind,
            name: self.name.clone(),
            scenario_id: self.scenario_id.clone(),
            schedule_id: self.schedule_id.clone(),
            status: self.status,
            started_at: self.started_at.clone(),
            finished_at: self.finished_at.clone(),
            duration_s: self.duration_s,
            requests_total: load.map(|l| l.requests_total),
            error_rate: load.map(|l| l.error_rate),
            throughput_rps: load.map(|l| l.throughput_rps),
            p50_ms: load.map(|l| l.latency.p50_ms),
            p90_ms: load.map(|l| l.latency.p90_ms),
            p99_ms: load.map(|l| l.latency.p99_ms),
            passed,
            error: self.error.clone(),
            services: self.services(),
        }
    }
}

fn now() -> String {
    humantime::format_rfc3339_millis(SystemTime::now()).to_string()
}

/// What a client following a run receives, in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum RunFeed {
    /// The run began.
    Started {
        /// Summary.
        run: RunSummary,
    },
    /// Entered a phase.
    Phase {
        /// Run id.
        id: String,
        /// `setup`, `load`, `assert`, `teardown`.
        name: String,
    },
    /// A load snapshot.
    Load {
        /// Run id.
        id: String,
        /// Snapshot.
        snapshot: LoadSnapshot,
    },
    /// A timeline action fired.
    Timeline {
        /// Run id.
        id: String,
        /// Event.
        event: EventOutcome,
    },
    /// The run ended; the record is final.
    Finished {
        /// Full record.
        run: Box<RunRecord>,
    },
}

/// The run in progress.
pub struct ActiveRun {
    /// Its id.
    pub id: String,
    /// Cancel it.
    pub cancel: CancellationToken,
    feed: broadcast::Sender<RunFeed>,
    history: Mutex<Vec<RunFeed>>,
}

impl ActiveRun {
    fn new(id: &str) -> Self {
        let (feed, _) = broadcast::channel(1024);
        Self {
            id: id.to_owned(),
            cancel: CancellationToken::new(),
            feed,
            history: Mutex::new(Vec::new()),
        }
    }

    /// Record and broadcast.
    pub async fn publish(&self, item: RunFeed) {
        self.history.lock().await.push(item.clone());
        let _ = self.feed.send(item);
    }

    /// Everything so far, plus a receiver for what follows.
    pub async fn replay(&self) -> (Vec<RunFeed>, broadcast::Receiver<RunFeed>) {
        let history = self.history.lock().await;
        (history.clone(), self.feed.subscribe())
    }
}

/// Records on disk plus the active run.
pub struct RunStore {
    dir: PathBuf,
    records: Mutex<BTreeMap<String, RunRecord>>,
    active: Mutex<Option<Arc<ActiveRun>>>,
}

impl RunStore {
    /// Open `dir`, creating it, and index every `*.json` in it.
    pub fn open(dir: &Path) -> std::io::Result<Self> {
        std::fs::create_dir_all(dir)?;
        let mut records = BTreeMap::new();
        for entry in std::fs::read_dir(dir)? {
            let path = entry?.path();
            if path.extension().is_none_or(|e| e != "json") {
                continue;
            }
            match std::fs::read_to_string(&path)
                .map_err(|e| e.to_string())
                .and_then(|t| serde_json::from_str::<RunRecord>(&t).map_err(|e| e.to_string()))
            {
                Ok(mut r) => {
                    if r.status == RunStatus::Running {
                        // The process died mid-run.
                        r.status = RunStatus::Error;
                        r.error.get_or_insert_with(|| "interrupted".into());
                    }
                    records.insert(r.id.clone(), r);
                }
                Err(error) => {
                    tracing::warn!(path = %path.display(), %error, "skipping unreadable run record");
                }
            }
        }
        tracing::info!(dir = %dir.display(), runs = records.len(), "run records loaded");
        Ok(Self {
            dir: dir.to_path_buf(),
            records: Mutex::new(records),
            active: Mutex::new(None),
        })
    }

    /// Insert or replace, and write to disk.
    pub async fn put(&self, record: &RunRecord) -> std::io::Result<()> {
        let path = self.dir.join(format!("{}.json", record.id));
        let tmp = self.dir.join(format!("{}.json.tmp", record.id));
        let text = serde_json::to_string_pretty(record)?;
        tokio::fs::write(&tmp, text).await?;
        tokio::fs::rename(&tmp, &path).await?;
        self.records
            .lock()
            .await
            .insert(record.id.clone(), record.clone());
        Ok(())
    }

    /// One record.
    pub async fn get(&self, id: &str) -> Option<RunRecord> {
        self.records.lock().await.get(id).cloned()
    }

    /// Remove from index and disk. Refuses the active run.
    pub async fn delete(&self, id: &str) -> std::io::Result<bool> {
        if self.active(id).await.is_some() {
            return Ok(false);
        }
        let removed = self.records.lock().await.remove(id).is_some();
        if removed {
            let _ = tokio::fs::remove_file(self.dir.join(format!("{id}.json"))).await;
        }
        Ok(removed)
    }

    /// Newest first.
    pub async fn list(&self, limit: usize) -> Vec<RunSummary> {
        self.records
            .lock()
            .await
            .values()
            .rev()
            .take(limit)
            .map(RunRecord::summary)
            .collect()
    }

    /// How many.
    pub async fn count(&self) -> usize {
        self.records.lock().await.len()
    }

    /// The newest finished record of a kind.
    pub async fn last_of(&self, kind: RunKind) -> Option<RunRecord> {
        self.records
            .lock()
            .await
            .values()
            .rev()
            .find(|r| r.kind == kind && r.status != RunStatus::Running)
            .cloned()
    }

    /// Claim the single active slot. `None` when a run is already active.
    pub async fn begin(&self, id: &str) -> Option<Arc<ActiveRun>> {
        let mut slot = self.active.lock().await;
        if slot.is_some() {
            return None;
        }
        let active = Arc::new(ActiveRun::new(id));
        *slot = Some(Arc::clone(&active));
        Some(active)
    }

    /// Release the slot after publishing the final record.
    pub async fn end(&self, record: &RunRecord) {
        let active = self.active.lock().await.take();
        if let Some(active) = active {
            active
                .publish(RunFeed::Finished {
                    run: Box::new(record.clone()),
                })
                .await;
        }
    }

    /// The active run, or the active run with this id.
    pub async fn active(&self, id: &str) -> Option<Arc<ActiveRun>> {
        self.active
            .lock()
            .await
            .as_ref()
            .filter(|a| a.id == id)
            .map(Arc::clone)
    }

    /// Whatever is active.
    pub async fn current(&self) -> Option<Arc<ActiveRun>> {
        self.active.lock().await.as_ref().map(Arc::clone)
    }
}
