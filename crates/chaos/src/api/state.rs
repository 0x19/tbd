//! Shared state: config, the long-lived stack, run records, the global feed,
//! and the jobs that drive runs in the background.

use std::{
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};
use tokio::sync::{Mutex, broadcast, mpsc};

use super::{
    error::ApiError,
    runs::{ActiveRun, RunFeed, RunKind, RunRecord, RunStatus, RunStore, RunSummary},
};
use crate::{
    config::{ChaosConfig, Source},
    load::{self, LoadConfig, LoadSnapshot, Metrics, Target},
    scenario::{self, RunEvent, ScenarioFile, executor::EventOutcome},
    stack::{InstanceInfo, Stack},
    topology::TopologyFile,
    validate,
};

/// Broadcast to every `GET /events` client.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum GlobalEvent {
    /// A run began.
    RunStarted {
        /// Summary.
        run: RunSummary,
    },
    /// A run ended.
    RunFinished {
        /// Summary.
        run: RunSummary,
    },
    /// The stack changed: an instance started, stopped or got a new behaviour.
    StackChanged {
        /// Every launcher.
        instances: Vec<InstanceInfo>,
    },
}

/// A scenario file as listed.
#[derive(Debug, Clone, Serialize)]
pub struct ScenarioEntry {
    /// Path under the scenarios directory without `.toml`.
    pub id: String,
    /// File path.
    pub file: String,
    /// `[scenario] name`, when it parses.
    pub name: Option<String>,
    /// `[scenario] description`.
    pub description: String,
    /// `[scenario] skip`.
    pub skip: bool,
    /// Parses and passes `check`.
    pub ok: bool,
    /// Why not.
    pub error: Option<String>,
}

/// Body of `POST /runs` for an ad-hoc load run.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct LoadRequest {
    /// Shown in the run list. Default `load`.
    #[serde(default)]
    pub name: Option<String>,
    /// Where to send load. Empty: every running protocol in the serve stack.
    #[serde(default)]
    pub targets: Vec<Target>,
    /// The `[load]` table of a scenario.
    pub load: LoadConfig,
}

/// Body of `POST /validate`. Every field defaults to the config.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValidateRequest {
    /// Protocol base URL.
    #[serde(default)]
    pub protocol: Option<String>,
    /// Engine gRPC URL.
    #[serde(default)]
    pub engine: Option<String>,
    /// Per-check timeout.
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,
}

/// Everything the routes share.
pub struct AppState {
    /// Effective configuration.
    pub config: ChaosConfig,
    /// Where it came from.
    pub source: Source,
    /// The long-lived stack, when `[serve] start_stack` is on.
    pub stack: Mutex<Option<Stack>>,
    /// Run records.
    pub runs: RunStore,
    global: broadcast::Sender<GlobalEvent>,
}

impl AppState {
    /// Open the run store, seed the scenario directory, start the stack.
    pub async fn new(config: ChaosConfig, source: Source) -> anyhow::Result<Arc<Self>> {
        let runs = RunStore::open(&config.paths.results)?;
        seed_scenarios(&config.paths.scenarios, &config.paths.scenarios_seed)?;
        let stack = if config.serve.start_stack {
            let path = &config.paths.topology;
            let text = std::fs::read_to_string(path)
                .map_err(|e| anyhow::anyhow!("read topology {}: {e}", path.display()))?;
            let topology: TopologyFile = toml::from_str(&text)
                .map_err(|e| anyhow::anyhow!("parse topology {}: {e}", path.display()))?;
            topology
                .stack
                .check()
                .map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
            Some(topology.stack.start().await?)
        } else {
            None
        };
        let (global, _) = broadcast::channel(256);
        Ok(Arc::new(Self {
            config,
            source,
            stack: Mutex::new(stack),
            runs,
            global,
        }))
    }

    /// Subscribe to the global feed.
    pub fn subscribe(&self) -> broadcast::Receiver<GlobalEvent> {
        self.global.subscribe()
    }

    fn publish(&self, event: GlobalEvent) {
        let _ = self.global.send(event);
    }

    /// Cancel the active run and stop the stack.
    pub async fn shutdown(&self) {
        if let Some(active) = self.runs.current().await {
            active.cancel.cancel();
        }
        if let Some(stack) = self.stack.lock().await.take() {
            stack.shutdown().await;
        }
    }

    // ------------------------------------------------------------ stack --

    /// Every launcher with its state. 409 when serve runs without a stack.
    pub async fn stack_info(&self) -> Result<Vec<InstanceInfo>, ApiError> {
        self.stack
            .lock()
            .await
            .as_ref()
            .map(Stack::describe)
            .ok_or_else(no_stack)
    }

    /// Run `f` on the stack, then broadcast the new state.
    pub async fn with_stack<T>(
        &self,
        f: impl AsyncFnOnce(&mut Stack) -> Result<T, ApiError>,
    ) -> Result<T, ApiError> {
        let mut guard = self.stack.lock().await;
        let stack = guard.as_mut().ok_or_else(no_stack)?;
        let out = f(stack).await?;
        self.publish(GlobalEvent::StackChanged {
            instances: stack.describe(),
        });
        Ok(out)
    }

    async fn stack_targets(&self) -> Vec<Target> {
        self.stack
            .lock()
            .await
            .as_ref()
            .map(|s| {
                s.of_kind("protocol")
                    .iter()
                    .map(|i| Target {
                        name: i.name.clone(),
                        http_url: i.http_url(),
                    })
                    .collect()
            })
            .unwrap_or_default()
    }

    // -------------------------------------------------------- scenarios --

    /// `scenarios/<id>.toml`, after checking the id is a plain relative path.
    pub fn scenario_path(&self, id: &str) -> Result<PathBuf, ApiError> {
        let valid = !id.is_empty()
            && id.split('/').all(|seg| {
                !seg.is_empty()
                    && seg != "."
                    && seg != ".."
                    && seg
                        .chars()
                        .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.')
            });
        if !valid {
            return Err(ApiError::invalid(format!(
                "scenario id {id:?}: use letters, digits, `_`, `-` and `/`"
            )));
        }
        Ok(self.config.paths.scenarios.join(format!("{id}.toml")))
    }

    /// Every `*.toml` under the scenarios directory, sorted by id.
    pub fn list_scenarios(&self) -> Vec<ScenarioEntry> {
        let root = &self.config.paths.scenarios;
        let pattern = root.join("**/*.toml");
        let mut out = Vec::new();
        let Ok(paths) = glob::glob(&pattern.to_string_lossy()) else {
            return out;
        };
        for path in paths.flatten() {
            let Ok(rel) = path.strip_prefix(root) else {
                continue;
            };
            let id = rel
                .with_extension("")
                .to_string_lossy()
                .replace(std::path::MAIN_SEPARATOR, "/");
            out.push(entry(&id, &path));
        }
        out.sort_by(|a, b| a.id.cmp(&b.id));
        out
    }

    /// Text and parse state of one scenario.
    pub async fn read_scenario(
        &self,
        id: &str,
    ) -> Result<(ScenarioEntry, String, Option<ScenarioFile>), ApiError> {
        let path = self.scenario_path(id)?;
        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|_| ApiError::not_found(format!("no scenario {id:?}")))?;
        let parsed = parse_scenario(&text).ok();
        Ok((entry(id, &path), text, parsed))
    }

    /// Check, then write. The file is only written when it checks.
    pub async fn write_scenario(&self, id: &str, text: &str) -> Result<ScenarioEntry, ApiError> {
        let path = self.scenario_path(id)?;
        parse_scenario(text).map_err(ApiError::invalid)?;
        if let Some(parent) = path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        tokio::fs::write(&path, text).await?;
        Ok(entry(id, &path))
    }

    /// Remove a scenario file.
    pub async fn delete_scenario(&self, id: &str) -> Result<(), ApiError> {
        let path = self.scenario_path(id)?;
        tokio::fs::remove_file(&path)
            .await
            .map_err(|_| ApiError::not_found(format!("no scenario {id:?}")))
    }

    // ------------------------------------------------------------- runs --

    /// The newest run of a scenario.
    pub async fn last_run_of(&self, scenario_id: &str) -> Option<RunSummary> {
        self.runs
            .list(usize::MAX)
            .await
            .into_iter()
            .find(|r| r.scenario_id.as_deref() == Some(scenario_id))
    }

    /// Claim the active slot, persist the running record, announce it.
    async fn begin(&self, record: &RunRecord) -> Result<Arc<ActiveRun>, ApiError> {
        let active = self
            .runs
            .begin(&record.id)
            .await
            .ok_or_else(|| ApiError::conflict("a run is already active"))?;
        self.runs.put(record).await?;
        let summary = record.summary();
        active
            .publish(RunFeed::Started {
                run: summary.clone(),
            })
            .await;
        self.publish(GlobalEvent::RunStarted { run: summary });
        Ok(active)
    }

    /// Persist the final record, release the slot, announce it.
    async fn end(&self, mut record: RunRecord, status: RunStatus, elapsed: Duration) {
        record.finish(status, elapsed.as_secs_f64());
        if let Err(error) = self.runs.put(&record).await {
            tracing::error!(%error, id = record.id, "could not write run record");
        }
        self.runs.end(&record).await;
        self.publish(GlobalEvent::RunFinished {
            run: record.summary(),
        });
    }

    /// Start a scenario run in the background. 409 while another run is active.
    pub async fn spawn_scenario(self: &Arc<Self>, id: &str) -> Result<RunSummary, ApiError> {
        let path = self.scenario_path(id)?;
        let text = tokio::fs::read_to_string(&path)
            .await
            .map_err(|_| ApiError::not_found(format!("no scenario {id:?}")))?;
        let file = parse_scenario(&text).map_err(ApiError::invalid)?;

        let mut record = RunRecord::start(RunKind::Scenario, &file.scenario.name);
        record.scenario_id = Some(id.to_owned());
        let active = self.begin(&record).await?;
        let summary = record.summary();

        let state = Arc::clone(self);
        tokio::spawn(async move {
            let started = Instant::now();
            let (tx, rx) = mpsc::unbounded_channel();
            let hooks = scenario::Hooks {
                events: Some(tx),
                cancel: active.cancel.clone(),
            };
            let collector = tokio::spawn(collect(Arc::clone(&active), rx));
            let result = scenario::run_file_with(&path, &hooks).await;
            drop(hooks);
            let (samples, events) = collector.await.unwrap_or_default();
            record.samples = samples;
            record.events = events;
            let status = if active.cancel.is_cancelled() {
                RunStatus::Cancelled
            } else if result.passed {
                RunStatus::Passed
            } else if result.error.is_some() {
                RunStatus::Error
            } else {
                RunStatus::Failed
            };
            record.error.clone_from(&result.error);
            record.scenario = Some(result);
            state.end(record, status, started.elapsed()).await;
        });
        Ok(summary)
    }

    /// Start an ad-hoc load run in the background.
    pub async fn spawn_load(self: &Arc<Self>, req: LoadRequest) -> Result<RunSummary, ApiError> {
        req.load.check().map_err(ApiError::invalid)?;
        let targets = if req.targets.is_empty() {
            self.stack_targets().await
        } else {
            req.targets.clone()
        };
        if targets.is_empty() {
            return Err(ApiError::invalid(
                "no targets: give `targets` or run serve with a stack that has a protocol",
            ));
        }
        for t in &targets {
            url::Url::parse(&t.http_url)
                .map_err(|e| ApiError::invalid(format!("target {}: {e}", t.name)))?;
        }

        let mut record = RunRecord::start(RunKind::Load, req.name.as_deref().unwrap_or("load"));
        record.request = serde_json::to_value(&req).ok();
        let active = self.begin(&record).await?;
        let summary = record.summary();

        let state = Arc::clone(self);
        let config = req.load;
        tokio::spawn(async move {
            let started = Instant::now();
            let (tx, rx) = mpsc::unbounded_channel();
            let hooks = load::Hooks {
                progress: Some(tx),
                cancel: active.cancel.clone(),
            };
            let collector = tokio::spawn(collect_load(Arc::clone(&active), rx));
            let snapshot =
                load::run_with(&config, &targets, Arc::new(Metrics::new()), &hooks).await;
            drop(hooks);
            record.samples = collector.await.unwrap_or_default();
            let status = if active.cancel.is_cancelled() {
                RunStatus::Cancelled
            } else {
                RunStatus::Completed
            };
            record.load = Some(snapshot);
            state.end(record, status, started.elapsed()).await;
        });
        Ok(summary)
    }

    /// Run validate now and record it. Independent of the active run slot.
    pub async fn run_validate(&self, req: ValidateRequest) -> Result<RunRecord, ApiError> {
        let targets = validate::Targets {
            protocol: req
                .protocol
                .unwrap_or_else(|| self.config.targets.protocol.clone()),
            engine: req
                .engine
                .unwrap_or_else(|| self.config.targets.engine.clone()),
            timeout: req.timeout.unwrap_or(self.config.validate.timeout),
        };
        for (what, u) in [("protocol", &targets.protocol), ("engine", &targets.engine)] {
            url::Url::parse(u).map_err(|e| ApiError::invalid(format!("{what}: {e}")))?;
        }
        let mut record = RunRecord::start(RunKind::Validate, "validate");
        record.request = Some(targets_json(&targets));
        self.publish(GlobalEvent::RunStarted {
            run: record.summary(),
        });
        let started = Instant::now();
        let report = validate::run(targets).await;
        let status = if report.ok() {
            RunStatus::Passed
        } else {
            RunStatus::Failed
        };
        record.validate = Some(report);
        record.finish(status, started.elapsed().as_secs_f64());
        self.runs.put(&record).await?;
        self.publish(GlobalEvent::RunFinished {
            run: record.summary(),
        });
        Ok(record)
    }
}

/// Copy `*.toml` from `seed` into `dir` when `dir` is missing or has no TOML.
fn seed_scenarios(dir: &Path, seed: &Path) -> anyhow::Result<()> {
    if seed.as_os_str().is_empty() {
        return Ok(());
    }
    let has_files = std::fs::read_dir(dir).is_ok_and(|mut entries| {
        entries.any(|e| e.is_ok_and(|e| e.path().extension().is_some_and(|x| x == "toml")))
    });
    if has_files {
        return Ok(());
    }
    let pattern = seed.join("**/*.toml");
    let mut copied = 0;
    for path in glob::glob(&pattern.to_string_lossy())?.flatten() {
        let Ok(rel) = path.strip_prefix(seed) else {
            continue;
        };
        let target = dir.join(rel);
        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::copy(&path, &target)?;
        copied += 1;
    }
    tracing::info!(from = %seed.display(), to = %dir.display(), copied, "seeded scenarios");
    Ok(())
}

fn no_stack() -> ApiError {
    ApiError::conflict("serve is running without a stack ([serve] start_stack = false)")
}

fn targets_json(t: &validate::Targets) -> serde_json::Value {
    serde_json::json!({
        "protocol": t.protocol,
        "engine": t.engine,
        "timeout": humantime::format_duration(t.timeout).to_string(),
    })
}

/// Parse and check scenario text.
pub fn parse_scenario(text: &str) -> Result<ScenarioFile, String> {
    let file: ScenarioFile = toml::from_str(text).map_err(|e| e.to_string())?;
    file.check()?;
    Ok(file)
}

fn entry(id: &str, path: &Path) -> ScenarioEntry {
    let parsed = std::fs::read_to_string(path)
        .map_err(|e| e.to_string())
        .and_then(|t| parse_scenario(&t));
    match parsed {
        Ok(file) => ScenarioEntry {
            id: id.to_owned(),
            file: path.display().to_string(),
            name: Some(file.scenario.name),
            description: file.scenario.description,
            skip: file.scenario.skip,
            ok: true,
            error: None,
        },
        Err(error) => ScenarioEntry {
            id: id.to_owned(),
            file: path.display().to_string(),
            name: None,
            description: String::new(),
            skip: false,
            ok: false,
            error: Some(error),
        },
    }
}

/// Forward scenario events to the run feed and keep samples and events.
async fn collect(
    active: Arc<ActiveRun>,
    mut rx: mpsc::UnboundedReceiver<RunEvent>,
) -> (Vec<LoadSnapshot>, Vec<EventOutcome>) {
    let mut samples = Vec::new();
    let mut events = Vec::new();
    let id = active.id.clone();
    while let Some(event) = rx.recv().await {
        let item = match event {
            RunEvent::Phase { name } => RunFeed::Phase {
                id: id.clone(),
                name,
            },
            RunEvent::Load { snapshot } => {
                samples.push(snapshot.clone());
                RunFeed::Load {
                    id: id.clone(),
                    snapshot,
                }
            }
            RunEvent::Timeline { event } => {
                events.push(event.clone());
                RunFeed::Timeline {
                    id: id.clone(),
                    event,
                }
            }
        };
        active.publish(item).await;
    }
    (samples, events)
}

/// Forward load snapshots to the run feed and keep them.
async fn collect_load(
    active: Arc<ActiveRun>,
    mut rx: mpsc::UnboundedReceiver<LoadSnapshot>,
) -> Vec<LoadSnapshot> {
    let mut samples = Vec::new();
    while let Some(snapshot) = rx.recv().await {
        samples.push(snapshot.clone());
        active
            .publish(RunFeed::Load {
                id: active.id.clone(),
                snapshot,
            })
            .await;
    }
    samples
}
