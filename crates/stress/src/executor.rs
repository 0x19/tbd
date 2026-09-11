//! Run one campaign: ping the targets, spawn the workers, measure, collect the
//! findings, report.

use std::{
    collections::BTreeMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use tbd_proto::ledger::v1::PingRequest;
use tokio::sync::{Semaphore, mpsc};

use crate::{
    campaign::Campaign,
    client::{GrpcLedger, LedgerClient, Target},
    finding::Finding,
    hooks::{Hooks, StressEvent},
    metrics::Metrics,
    report::{CampaignResult, StressSnapshot},
    workers::{Checks, Context, owner::Owner},
};

/// How often progress is emitted.
pub const PROGRESS_INTERVAL: Duration = Duration::from_secs(1);

/// Run `campaign` against `targets` until its duration passes, it is
/// cancelled, or `[stop] max_findings` is reached.
pub async fn run(campaign: &Campaign, targets: Vec<Target>, hooks: &Hooks) -> CampaignResult {
    if campaign.campaign.skip {
        let mut result = empty_result(campaign, &[]);
        result.passed = true;
        return result;
    }
    let timeout = campaign.campaign.timeout;
    let clients: Vec<Arc<dyn LedgerClient>> = targets
        .iter()
        .map(|t| Arc::new(GrpcLedger::new(t, timeout)) as Arc<dyn LedgerClient>)
        .collect();
    let names: Vec<String> = targets.iter().map(|t| t.name.clone()).collect();
    run_with_clients(campaign, names, clients, hooks).await
}

/// Run against clients the caller built: what [`run`] does after it has
/// connected, and what a test does with a client that lies.
pub async fn run_with_clients(
    campaign: &Campaign,
    names: Vec<String>,
    clients: Vec<Arc<dyn LedgerClient>>,
    hooks: &Hooks,
) -> CampaignResult {
    let started = Instant::now();
    let targets: Vec<Named> = names
        .into_iter()
        .zip(clients)
        .map(|(name, client)| Named { name, client })
        .collect();
    let mut result = empty_result(
        campaign,
        &targets.iter().map(|t| t.name.clone()).collect::<Vec<_>>(),
    );
    if targets.is_empty() {
        result.error = Some("no targets: give a ledger to run against".into());
        result.duration_s = started.elapsed().as_secs_f64();
        return result;
    }
    match ping_all(&targets).await {
        Ok(store) => result.store = store,
        Err(e) => {
            result.error = Some(e);
            result.duration_s = started.elapsed().as_secs_f64();
            return result;
        }
    }

    let campaign = Arc::new(campaign.clone());
    let metrics = Arc::new(Metrics::new());
    let checks = Arc::new(Checks::default());
    let semaphore = Arc::new(Semaphore::new(campaign.workload.max_in_flight));
    let cancel = hooks.cancel.child_token();
    let (ftx, frx) = mpsc::unbounded_channel::<Finding>();
    let run_start = Instant::now();

    // Workers, spread over the targets.
    let (mut workers, subjects_total) = spawn_owners(
        &campaign,
        &targets,
        result.store.as_deref(),
        &Shared {
            metrics: Arc::clone(&metrics),
            checks: Arc::clone(&checks),
            findings: ftx.clone(),
            semaphore: Arc::clone(&semaphore),
            cancel: cancel.clone(),
            started: run_start,
        },
    );
    drop(ftx);

    let collected = Collected::spawn(
        frx,
        campaign.stop.max_findings,
        cancel.clone(),
        hooks.clone(),
    );
    let progress = Progress::new(&campaign, &metrics, &checks, &collected, subjects_total);
    let snapshot = |phase: &str, elapsed: Duration| progress.snapshot(phase, elapsed);

    warmup(&campaign, hooks, &metrics, &checks, &cancel).await;

    hooks.emit(StressEvent::Phase { name: "run".into() });
    let measured_start = Instant::now();
    measure(
        hooks,
        &metrics,
        &snapshot,
        measured_start,
        campaign.campaign.duration,
        &cancel,
    )
    .await;
    cancel.cancel();
    while let Some(joined) = workers.join_next().await {
        if let Err(e) = joined
            && result.error.is_none()
        {
            result.error = Some(format!("a worker panicked: {e}"));
        }
    }
    let (mut findings, stopped_early) = collected.finish().await;

    if campaign.stop.shrink && !findings.is_empty() && !hooks.cancel.is_cancelled() {
        findings = shrink_all(&campaign, &targets, findings, hooks, |phase| {
            snapshot(phase, measured_start.elapsed())
        })
        .await;
    }

    result.load = Some(metrics.snapshot());
    result.checks = checks.snapshot();
    (result.tolerated, result.redriven) = checks.faults();
    result.findings = findings;
    result.stopped_early = stopped_early;
    if hooks.cancel.is_cancelled() && result.error.is_none() && !result.stopped_early {
        result.error = Some("cancelled".into());
    }
    emit_done(hooks, &metrics, snapshot("done", measured_start.elapsed()));
    result.passed = result.error.is_none() && result.findings.is_empty();
    result.duration_s = started.elapsed().as_secs_f64();
    result
}

/// Ping every target; the first store name wins.
async fn ping_all(targets: &[Named]) -> Result<Option<String>, String> {
    let mut store = None;
    for t in targets {
        let p = t
            .client
            .ping(PingRequest {
                message: "stress".into(),
            })
            .await
            .map_err(|e| format!("target {} unreachable: {e}", t.name))?;
        if p.stub {
            return Err(format!("target {}: the ledger is a stub", t.name));
        }
        if store.is_none() {
            store = Some(p.store);
        }
    }
    Ok(store)
}

/// The findings of a run, collected as they come; the cap stops the run early.
struct Collected {
    found: Arc<AtomicU64>,
    stopped_early: Arc<AtomicU64>,
    task: tokio::task::JoinHandle<Vec<Finding>>,
}

impl Collected {
    fn spawn(
        mut rx: mpsc::UnboundedReceiver<Finding>,
        max: u64,
        cancel: tokio_util::sync::CancellationToken,
        hooks: Hooks,
    ) -> Self {
        let found = Arc::new(AtomicU64::new(0));
        let stopped_early = Arc::new(AtomicU64::new(0));
        let task = {
            let found = Arc::clone(&found);
            let stopped_early = Arc::clone(&stopped_early);
            tokio::spawn(async move {
                let mut all = Vec::new();
                while let Some(f) = rx.recv().await {
                    hooks.emit(StressEvent::Finding {
                        finding: f.summary(),
                    });
                    all.push(f);
                    let n = found.fetch_add(1, Ordering::Relaxed) + 1;
                    if max > 0 && n >= max {
                        stopped_early.store(1, Ordering::Relaxed);
                        cancel.cancel();
                    }
                }
                all
            })
        };
        Self {
            found,
            stopped_early,
            task,
        }
    }

    async fn finish(self) -> (Vec<Finding>, bool) {
        let findings = self.task.await.unwrap_or_default();
        (findings, self.stopped_early.load(Ordering::Relaxed) == 1)
    }
}

/// What every worker shares.
struct Shared {
    metrics: Arc<Metrics>,
    checks: Arc<Checks>,
    findings: mpsc::UnboundedSender<Finding>,
    semaphore: Arc<Semaphore>,
    cancel: tokio_util::sync::CancellationToken,
    started: Instant,
}

/// Spawn the owner workers over the targets, round robin.
fn spawn_owners(
    campaign: &Arc<Campaign>,
    targets: &[Named],
    store: Option<&str>,
    shared: &Shared,
) -> (tokio::task::JoinSet<()>, u64) {
    let mut workers = tokio::task::JoinSet::new();
    let owner_cfg = &campaign.workload.owner;
    let mut subjects_total = 0u64;
    for w in 0..owner_cfg.workers {
        let ti = usize::try_from(w).unwrap_or(0) % targets.len();
        let ctx = Arc::new(Context {
            campaign: Arc::clone(campaign),
            client: Arc::clone(&targets[ti].client),
            target: targets[ti].name.clone(),
            store: store.map(str::to_owned),
            metrics: Arc::clone(&shared.metrics),
            checks: Arc::clone(&shared.checks),
            findings: shared.findings.clone(),
            semaphore: Arc::clone(&shared.semaphore),
            cancel: shared.cancel.clone(),
            started: shared.started,
        });
        let seed = campaign
            .campaign
            .seed
            .wrapping_mul(1_000_003)
            .wrapping_add(u64::from(w));
        let owner = Owner::new(ctx, seed, owner_cfg.subjects);
        subjects_total += owner.subject_count() as u64;
        workers.spawn(owner.run());
    }
    (workers, subjects_total)
}

/// Emit progress once a second until the duration passes or the run is cancelled.
async fn measure(
    hooks: &Hooks,
    metrics: &Metrics,
    snapshot: &impl Fn(&str, Duration) -> StressSnapshot,
    measured_start: Instant,
    duration: Duration,
    cancel: &tokio_util::sync::CancellationToken,
) {
    let deadline = measured_start + duration;
    let mut tick = tokio::time::interval(PROGRESS_INTERVAL);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tokio::select! {
            _ = tick.tick() => {
                hooks.emit(StressEvent::Load { snapshot: metrics.snapshot() });
                hooks.emit(StressEvent::Stress { snapshot: snapshot("run", measured_start.elapsed()) });
            }
            () = tokio::time::sleep_until(deadline.into()) => break,
            () = cancel.cancelled() => break,
        }
    }
}

/// A client with the target's name.
#[derive(Clone)]
pub struct Named {
    /// The target's name, for metrics and findings.
    pub name: String,
    /// The ledger.
    pub client: Arc<dyn LedgerClient>,
}

/// The result before anything ran.
fn empty_result(campaign: &Campaign, targets: &[String]) -> CampaignResult {
    CampaignResult {
        name: campaign.campaign.name.clone(),
        file: None,
        passed: false,
        skipped: campaign.campaign.skip,
        duration_s: 0.0,
        store: None,
        targets: targets.to_vec(),
        load: None,
        checks: BTreeMap::new(),
        tolerated: 0,
        redriven: 0,
        findings: Vec::new(),
        stopped_early: false,
        error: None,
    }
}

/// The last frames: final numbers, then the phase.
fn emit_done(hooks: &Hooks, metrics: &Metrics, snapshot: StressSnapshot) {
    hooks.emit(StressEvent::Load {
        snapshot: metrics.snapshot(),
    });
    hooks.emit(StressEvent::Stress { snapshot });
    hooks.emit(StressEvent::Phase {
        name: "done".into(),
    });
}

/// Warmup: the same workload, then the numbers are discarded.
async fn warmup(
    campaign: &Campaign,
    hooks: &Hooks,
    metrics: &Metrics,
    checks: &Checks,
    cancel: &tokio_util::sync::CancellationToken,
) {
    let Some(warmup) = campaign.campaign.warmup else {
        return;
    };
    if warmup.is_zero() {
        return;
    }
    hooks.emit(StressEvent::Phase {
        name: "warmup".into(),
    });
    tokio::select! {
        () = tokio::time::sleep(warmup) => {}
        () = cancel.cancelled() => {}
    }
    metrics.reset();
    checks.reset();
}

/// What a per-second snapshot reads.
struct Progress {
    metrics: Arc<Metrics>,
    checks: Arc<Checks>,
    found: Arc<AtomicU64>,
    subjects: u64,
    workers: BTreeMap<String, u32>,
}

impl Progress {
    fn new(
        campaign: &Campaign,
        metrics: &Arc<Metrics>,
        checks: &Arc<Checks>,
        collected: &Collected,
        subjects: u64,
    ) -> Self {
        Self {
            metrics: Arc::clone(metrics),
            checks: Arc::clone(checks),
            found: Arc::clone(&collected.found),
            subjects,
            workers: [("owner".to_owned(), campaign.workload.owner.workers)]
                .into_iter()
                .collect(),
        }
    }

    fn snapshot(&self, phase: &str, elapsed: Duration) -> StressSnapshot {
        let (tolerated, redriven) = self.checks.faults();
        let load = self.metrics.snapshot();
        StressSnapshot {
            elapsed_s: elapsed.as_secs_f64(),
            phase: phase.to_owned(),
            ops_total: load.requests_total,
            ops_failed: load.requests_failed,
            tolerated,
            redriven,
            checks: self.checks.snapshot(),
            findings: self.found.load(Ordering::Relaxed),
            subjects: self.subjects,
            workers: self.workers.clone(),
        }
    }
}

/// Shrink: every finding replayed down to the steps that matter, once the
/// workers are quiet so nothing competes with the replays.
async fn shrink_all(
    campaign: &Campaign,
    targets: &[Named],
    findings: Vec<Finding>,
    hooks: &Hooks,
    snapshot: impl Fn(&str) -> StressSnapshot,
) -> Vec<Finding> {
    hooks.emit(StressEvent::Phase {
        name: "shrink".into(),
    });
    let budget = crate::shrink::Budget {
        attempts: campaign.stop.shrink_attempts,
        timeout: campaign.stop.shrink_timeout,
    };
    let mut shrunk = Vec::with_capacity(findings.len());
    for f in findings {
        let client = targets
            .iter()
            .find(|t| t.name == f.target)
            .map(|t| Arc::clone(&t.client));
        shrunk.push(match client {
            Some(client) if !hooks.cancel.is_cancelled() => {
                crate::shrink::shrink(
                    f,
                    client,
                    budget,
                    campaign.faults.clock_skew,
                    &campaign.faults.tolerate,
                )
                .await
            }
            _ => f,
        });
        hooks.emit(StressEvent::Stress {
            snapshot: snapshot("shrink"),
        });
    }
    shrunk
}
