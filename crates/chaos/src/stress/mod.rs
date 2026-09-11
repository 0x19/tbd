//! Stress campaigns: what chaos does around [`tbd_stress::run`], exactly what
//! the scenario executor does around load. A campaign file carries a
//! scenario's `[stack]` and `[[timeline]]`; chaos boots the stack, plays the
//! timeline and hands the ledgers to the stress crate as targets. With
//! explicit targets (a deployed ledger through Envoy) there is no stack and the
//! timeline is ignored. The workers, the model, the invariants and the findings
//! are the stress crate's; nothing here decides what the ledger must do.

use std::{
    path::Path,
    sync::Arc,
    time::{Duration, Instant},
};

use tbd_stress::{Campaign, CampaignResult, StressEvent, StressSnapshot};
use tokio::sync::{Mutex, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    load::{LoadSnapshot, Target},
    scenario::{
        executor::{EventOutcome, spawn_timeline},
        timeline::TimelineEvent,
    },
    tls::Trust,
    topology::StackConfig,
};

/// A campaign file as chaos runs it: the stress crate's schema plus the stack
/// and timeline parsed into chaos's own types.
#[derive(Debug, Clone)]
pub struct CampaignFile {
    /// Everything the stress crate reads.
    pub campaign: Campaign,
    /// `[stack]`, empty when the campaign runs against targets.
    pub stack: StackConfig,
    /// `[[timeline]]`, sorted by `at`.
    pub timeline: Vec<TimelineEvent>,
}

/// Parse and cross-check a campaign.
///
/// # Errors
/// The first problem, as a sentence: the stress crate's own checks, then the
/// stack's, then the timeline against the stack.
pub fn parse_campaign(text: &str) -> Result<CampaignFile, String> {
    let campaign = Campaign::parse(text).map_err(|e| e.to_string())?;
    let stack: StackConfig = if campaign.stack.is_empty() {
        StackConfig::default()
    } else {
        toml::Value::Table(campaign.stack.clone())
            .try_into()
            .map_err(|e| format!("stack: {e}"))?
    };
    if !stack.is_empty() {
        stack.check()?;
        if !stack.instances.values().any(|i| i.kind.name == "ledger") {
            return Err("stack: a campaign's stack needs at least one ledger".into());
        }
    }
    let mut timeline = Vec::with_capacity(campaign.timeline.len());
    for (i, t) in campaign.timeline.iter().enumerate() {
        let event: TimelineEvent = toml::Value::Table(t.clone())
            .try_into()
            .map_err(|e| format!("timeline[{i}]: {e}"))?;
        timeline.push(event);
    }
    if !timeline.is_empty() && stack.is_empty() {
        return Err("timeline without a [stack]: faults need instances chaos runs".into());
    }
    for event in &timeline {
        if let Some(service) = event.service()
            && stack.get(service).is_none()
        {
            return Err(format!("timeline references unknown service {service:?}"));
        }
        if matches!(event, TimelineEvent::SetBehavior { service, .. } if !stack.get(service).is_some_and(|i| i.kind.fault))
        {
            return Err(format!(
                "timeline: set_behavior on {}, whose kind has no fault injection",
                event.service().unwrap_or("?")
            ));
        }
    }
    timeline.sort_by_key(TimelineEvent::at);
    Ok(CampaignFile {
        campaign,
        stack,
        timeline,
    })
}

/// Read, parse and check a file.
///
/// # Errors
/// The file cannot be read, or [`parse_campaign`] refuses it.
pub fn load_campaign(path: &Path) -> Result<CampaignFile, String> {
    let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
    parse_campaign(&text)
}

/// What a running campaign emits, in order. `Phase`, `Load` and `Timeline`
/// are the scenario executor's frames; `Stress` and `Finding` are the stress
/// crate's.
#[derive(Debug, Clone)]
pub enum RunEvent {
    /// `setup`, `warmup`, `run`, `shrink`, `teardown`, `done`.
    Phase {
        /// The phase.
        name: String,
    },
    /// Load numbers, once a second.
    Load {
        /// Snapshot.
        snapshot: LoadSnapshot,
    },
    /// A timeline action fired.
    Timeline {
        /// The outcome.
        event: EventOutcome,
    },
    /// Checks and findings, once a second.
    Stress {
        /// Snapshot.
        snapshot: StressSnapshot,
    },
    /// A finding, as found.
    Finding {
        /// The list view.
        finding: tbd_stress::FindingSummary,
    },
}

/// Progress sink and cancellation.
#[derive(Debug, Clone, Default)]
pub struct Hooks {
    /// Where events go; `None` discards them.
    pub events: Option<mpsc::UnboundedSender<RunEvent>>,
    /// Cancel: the workers stop, the stack is torn down, the result is what
    /// was measured.
    pub cancel: CancellationToken,
}

impl Hooks {
    fn emit(&self, event: RunEvent) {
        if let Some(tx) = &self.events {
            let _ = tx.send(event);
        }
    }
}

/// How to run: against the file's stack, or against targets the caller names.
#[derive(Debug, Clone, Default)]
pub struct RunOptions {
    /// Explicit ledger targets; `None` boots the campaign's `[stack]`.
    pub targets: Option<Vec<Target>>,
    /// Overrides `[campaign] seed`.
    pub seed: Option<u64>,
    /// Trust and bearer for explicit targets.
    pub trust: Trust,
}

/// Run a file with default hooks and options, as the CLI does.
pub async fn run_file(path: &Path) -> CampaignResult {
    run_file_with(path, &RunOptions::default(), &Hooks::default()).await
}

/// Run a file.
pub async fn run_file_with(path: &Path, options: &RunOptions, hooks: &Hooks) -> CampaignResult {
    let name = path
        .file_stem()
        .map(|s| s.to_string_lossy().into_owned())
        .unwrap_or_default();
    let mut result = match load_campaign(path) {
        Ok(file) => run_campaign_with(&file, options, hooks).await,
        Err(e) => failed(&name, format!("parse: {e}")),
    };
    result.file = Some(path.display().to_string());
    result
}

fn failed(name: &str, error: String) -> CampaignResult {
    CampaignResult {
        name: name.to_owned(),
        file: None,
        passed: false,
        skipped: false,
        duration_s: 0.0,
        store: None,
        targets: Vec::new(),
        load: None,
        checks: std::collections::BTreeMap::default(),
        tolerated: 0,
        redriven: 0,
        findings: Vec::new(),
        stopped_early: false,
        error: Some(error),
    }
}

/// Run a parsed campaign: setup (stack or targets), the timeline alongside the
/// workers, teardown.
pub async fn run_campaign_with(
    file: &CampaignFile,
    options: &RunOptions,
    hooks: &Hooks,
) -> CampaignResult {
    let started = Instant::now();
    let mut campaign = file.campaign.clone();
    if let Some(seed) = options.seed {
        campaign.campaign.seed = seed;
    }
    if campaign.campaign.skip {
        return tbd_stress::run(&campaign, Vec::new(), &tbd_stress::Hooks::default()).await;
    }
    hooks.emit(RunEvent::Phase {
        name: "setup".into(),
    });

    // Where the workers go.
    let (stack, targets) = match resolve_targets(file, options, campaign.campaign.timeout).await {
        Ok(v) => v,
        Err(e) => return failed(&campaign.campaign.name, e),
    };

    // The timeline runs alongside the workers from the moment they start.
    let load_start = Instant::now();
    let timeline = stack.as_ref().map(|stack| {
        let hooks = hooks.clone();
        spawn_timeline(
            file.timeline.clone(),
            Arc::clone(stack),
            load_start,
            hooks.cancel.clone(),
            move |event| hooks.emit(RunEvent::Timeline { event }),
        )
    });
    if stack.is_none() && !file.timeline.is_empty() {
        tracing::info!("timeline ignored: the campaign runs against external targets");
    }

    // The stress crate's events become run events.
    let (tx, mut rx) = mpsc::unbounded_channel::<StressEvent>();
    let forward_hooks = hooks.clone();
    let forward = tokio::spawn(async move {
        while let Some(e) = rx.recv().await {
            forward_hooks.emit(match e {
                StressEvent::Phase { name } => RunEvent::Phase { name },
                StressEvent::Load { snapshot } => RunEvent::Load { snapshot },
                StressEvent::Stress { snapshot } => RunEvent::Stress { snapshot },
                StressEvent::Finding { finding } => RunEvent::Finding { finding },
            });
        }
    });
    let stress_hooks = tbd_stress::Hooks {
        events: Some(tx),
        cancel: hooks.cancel.clone(),
    };
    let mut result = tbd_stress::run(&campaign, targets, &stress_hooks).await;
    drop(stress_hooks);
    let _ = forward.await;

    // Timeline outcomes, then teardown.
    if let Some(timeline) = timeline {
        match timeline.await {
            Ok(outcomes) => {
                if let Some(failed_action) = outcomes.iter().find(|o| o.error.is_some()) {
                    result.error.get_or_insert_with(|| {
                        format!(
                            "timeline: {} failed: {}",
                            failed_action.action,
                            failed_action.error.as_deref().unwrap_or("")
                        )
                    });
                }
            }
            Err(_) => {
                result
                    .error
                    .get_or_insert_with(|| "timeline task panicked".into());
            }
        }
    }
    if let Some(stack) = stack {
        hooks.emit(RunEvent::Phase {
            name: "teardown".into(),
        });
        match Arc::try_unwrap(stack) {
            Ok(stack) => stack.into_inner().shutdown().await,
            Err(_) => {
                result
                    .error
                    .get_or_insert_with(|| "stack still shared after the timeline finished".into());
            }
        }
    }
    result.passed = result.error.is_none() && result.findings.is_empty();
    result.duration_s = started.elapsed().as_secs_f64();
    result
}

/// The ledgers the workers run against: the caller's targets, or the campaign's
/// stack, booted here.
async fn resolve_targets(
    file: &CampaignFile,
    options: &RunOptions,
    timeout: Duration,
) -> Result<
    (
        Option<Arc<Mutex<crate::stack::Stack>>>,
        Vec<tbd_stress::Target>,
    ),
    String,
> {
    if let Some(explicit) = &options.targets {
        let ledgers: Vec<&Target> = explicit.iter().filter(|t| t.kind == "ledger").collect();
        if ledgers.is_empty() {
            return Err("no ledger among the targets: give `--target ledger=URL`".into());
        }
        return Ok((None, build_targets(&ledgers, &options.trust, timeout)?));
    }
    if file.stack.is_empty() {
        return Err("no [stack] and no targets: a campaign needs a ledger to run against".into());
    }
    let stack = file
        .stack
        .start()
        .await
        .map_err(|e| format!("setup: {e}"))?;
    let ledgers: Vec<Target> = crate::kinds::load_targets(&stack)
        .into_iter()
        .filter(|t| t.kind == "ledger")
        .collect();
    let refs: Vec<&Target> = ledgers.iter().collect();
    let targets = build_targets(&refs, &Trust::default(), timeout)?;
    Ok((Some(Arc::new(Mutex::new(stack))), targets))
}

/// Ledger targets for the stress crate: a channel per URL with the trust the
/// caller decided on.
fn build_targets(
    ledgers: &[&Target],
    trust: &Trust,
    timeout: Duration,
) -> Result<Vec<tbd_stress::Target>, String> {
    let bearer = trust.authorization();
    ledgers
        .iter()
        .map(|t| {
            let channel = trust
                .channel(&t.http_url, Some(timeout))
                .map_err(|e| format!("target {}: {e}", t.name))?;
            Ok(tbd_stress::Target {
                name: t.name.clone(),
                channel,
                bearer: bearer.clone(),
            })
        })
        .collect()
}

/// Write every finding of `result` as `<dir>/<id>.json`; returns the paths.
///
/// # Errors
/// The directory cannot be created or a file cannot be written.
pub fn write_findings(
    dir: &Path,
    result: &CampaignResult,
) -> Result<Vec<std::path::PathBuf>, String> {
    if result.findings.is_empty() {
        return Ok(Vec::new());
    }
    std::fs::create_dir_all(dir).map_err(|e| format!("{}: {e}", dir.display()))?;
    let mut paths = Vec::with_capacity(result.findings.len());
    for f in &result.findings {
        let path = dir.join(format!("{}.json", f.id));
        let text = serde_json::to_string_pretty(f).map_err(|e| e.to_string())?;
        std::fs::write(&path, text).map_err(|e| format!("{}: {e}", path.display()))?;
        paths.push(path);
    }
    Ok(paths)
}

/// Read a finding by path, or by id under `dir`.
///
/// # Errors
/// Not found or not a finding.
pub fn read_finding(
    dir: &Path,
    id_or_path: &str,
) -> Result<(std::path::PathBuf, tbd_stress::Finding), String> {
    let candidate = Path::new(id_or_path);
    let path = if candidate.is_file() {
        candidate.to_path_buf()
    } else {
        dir.join(format!("{}.json", id_or_path.trim_end_matches(".json")))
    };
    let text = std::fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
    let finding = serde_json::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok((path, finding))
}

/// Replay a finding against ledger targets `attempts` times, record the
/// outcome on the finding and rewrite its file.
///
/// # Errors
/// No ledger target, or the file cannot be rewritten.
pub async fn replay_finding(
    path: &Path,
    finding: &mut tbd_stress::Finding,
    targets: &[Target],
    trust: &Trust,
    attempts: u32,
    timeout: Duration,
) -> Result<tbd_stress::ReplayOutcome, String> {
    let ledgers: Vec<&Target> = targets.iter().filter(|t| t.kind == "ledger").collect();
    let Some(first) = ledgers.first() else {
        return Err("no ledger target: give `--target ledger=URL`".into());
    };
    let built = build_targets(&[first], trust, timeout)?;
    let client: Arc<dyn tbd_stress::LedgerClient> =
        Arc::new(tbd_stress::GrpcLedger::new(&built[0], timeout));
    let outcome = tbd_stress::replay_finding(
        &finding.trace,
        &finding.invariant,
        &finding.signature,
        client,
        &first.name,
        attempts,
        Duration::from_millis(500),
        &[],
    )
    .await;
    finding.replays.push(outcome.clone());
    let text = serde_json::to_string_pretty(&finding).map_err(|e| e.to_string())?;
    std::fs::write(path, text).map_err(|e| format!("{}: {e}", path.display()))?;
    Ok(outcome)
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEAD: &str = "[campaign]\nname = \"t\"\nduration = \"1s\"\n";

    #[test]
    fn a_stack_and_timeline_parse_into_chaos_types() {
        let f = parse_campaign(&format!(
            "{HEAD}[stack.ledgers.l]\ngrace = \"0s\"\n[[timeline]]\nat = \"500ms\"\naction = \"set_behavior\"\nservice = \"l\"\n[timeline.behavior]\ntype = \"error\"\nkind = \"unavailable\"\nrate = 0.5\n"
        ))
        .unwrap();
        assert_eq!(f.stack.instances.len(), 1);
        assert_eq!(f.timeline.len(), 1);
        assert_eq!(f.timeline[0].service(), Some("l"));
    }

    #[test]
    fn cross_checks_name_the_problem() {
        let e = parse_campaign(&format!(
            "{HEAD}[[timeline]]\nat = \"1s\"\naction = \"log\"\nmessage = \"x\"\n"
        ))
        .unwrap_err();
        assert!(e.contains("without a [stack]"), "{e}");
        let e = parse_campaign(&format!(
            "{HEAD}[stack.ledgers.l]\n[[timeline]]\nat = \"1s\"\naction = \"stop\"\nservice = \"nope\"\n"
        ))
        .unwrap_err();
        assert!(e.contains("unknown service"), "{e}");
        let e = parse_campaign(&format!("{HEAD}[stack.engines.e]\n")).unwrap_err();
        assert!(e.contains("at least one ledger"), "{e}");
        let e = parse_campaign(&format!("{HEAD}nope = 1\n")).unwrap_err();
        assert!(e.contains("nope"), "{e}");
    }

    #[tokio::test]
    async fn no_stack_and_no_targets_is_an_error() {
        let f = parse_campaign(HEAD).unwrap();
        let r = run_campaign_with(&f, &RunOptions::default(), &Hooks::default()).await;
        assert!(!r.passed);
        assert!(
            r.error.as_deref().unwrap_or("").contains("no [stack]"),
            "{r:?}"
        );
    }
}
