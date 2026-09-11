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
    let started = Instant::now();
    let mut result = empty_result(campaign, &targets);
    if campaign.campaign.skip {
        result.passed = true;
        return result;
    }
    if targets.is_empty() {
        result.error = Some("no targets: give a ledger to run against".into());
        result.duration_s = started.elapsed().as_secs_f64();
        return result;
    }

    let timeout = campaign.campaign.timeout;
    let clients = match ping_all(&targets, timeout).await {
        Ok((clients, store)) => {
            result.store = store;
            clients
        }
        Err(e) => {
            result.error = Some(e);
            result.duration_s = started.elapsed().as_secs_f64();
            return result;
        }
    };

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
        &clients,
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
    let progress = Progress {
        metrics: Arc::clone(&metrics),
        checks: Arc::clone(&checks),
        found: Arc::clone(&collected.found),
        subjects: subjects_total,
        workers: [("owner".to_owned(), campaign.workload.owner.workers)]
            .into_iter()
            .collect(),
    };
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
    let (findings, stopped_early) = collected.finish().await;

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
async fn ping_all(
    targets: &[Target],
    timeout: Duration,
) -> Result<(Vec<Arc<dyn LedgerClient>>, Option<String>), String> {
    let mut clients: Vec<Arc<dyn LedgerClient>> = Vec::with_capacity(targets.len());
    let mut store = None;
    for t in targets {
        let client = GrpcLedger::new(t, timeout);
        let p = client
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
        clients.push(Arc::new(client));
    }
    Ok((clients, store))
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
    targets: &[Target],
    clients: &[Arc<dyn LedgerClient>],
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
            client: Arc::clone(&clients[ti]),
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

/// The result before anything ran.
fn empty_result(campaign: &Campaign, targets: &[Target]) -> CampaignResult {
    CampaignResult {
        name: campaign.campaign.name.clone(),
        file: None,
        passed: false,
        skipped: campaign.campaign.skip,
        duration_s: 0.0,
        store: None,
        targets: targets.iter().map(|t| t.name.clone()).collect(),
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
