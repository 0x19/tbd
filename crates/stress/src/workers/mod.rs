//! Workers: the loops that drive a target and judge every answer. Each class
//! has its own module; this one holds what they share.

pub mod contention;
pub mod fuzz;
pub mod owner;

use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::{
    campaign::Campaign,
    client::{CallError, LedgerClient},
    finding::{Finding, WorkerClass},
    metrics::Metrics,
    model::invariants,
    report::CheckCount,
    trace::{Request, Step, Violation},
};

/// The invariant counters, shared by every worker of a run.
#[derive(Debug, Default)]
pub struct Checks {
    counts: Mutex<BTreeMap<&'static str, CheckCount>>,
    tolerated: AtomicU64,
    redriven: AtomicU64,
}

impl Checks {
    /// An evaluation that held.
    pub fn pass(&self, invariant: &'static str) {
        self.lock().entry(invariant).or_default().passed += 1;
    }

    /// An evaluation that broke.
    pub fn violate(&self, invariant: &'static str) {
        self.lock().entry(invariant).or_default().violated += 1;
    }

    /// A failure the campaign tolerates.
    pub fn tolerated(&self) {
        self.tolerated.fetch_add(1, Ordering::Relaxed);
    }

    /// A write re-driven to a known outcome.
    pub fn redriven(&self) {
        self.redriven.fetch_add(1, Ordering::Relaxed);
    }

    /// Copy the counters out.
    #[must_use]
    pub fn snapshot(&self) -> BTreeMap<String, CheckCount> {
        self.lock()
            .iter()
            .map(|(k, v)| ((*k).to_owned(), *v))
            .collect()
    }

    /// Tolerated and re-driven so far.
    #[must_use]
    pub fn faults(&self) -> (u64, u64) {
        (
            self.tolerated.load(Ordering::Relaxed),
            self.redriven.load(Ordering::Relaxed),
        )
    }

    /// Start over (after warmup).
    pub fn reset(&self) {
        self.lock().clear();
        self.tolerated.store(0, Ordering::Relaxed);
        self.redriven.store(0, Ordering::Relaxed);
    }

    fn lock(&self) -> std::sync::MutexGuard<'_, BTreeMap<&'static str, CheckCount>> {
        self.counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Everything a worker needs from the run around it.
pub struct Context {
    /// The campaign.
    pub campaign: Arc<Campaign>,
    /// The ledger.
    pub client: Arc<dyn LedgerClient>,
    /// The target's name, for metrics and findings.
    pub target: String,
    /// The store the target reported.
    pub store: Option<String>,
    /// Load numbers.
    pub metrics: Arc<Metrics>,
    /// Invariant counters.
    pub checks: Arc<Checks>,
    /// Where findings go.
    pub findings: mpsc::UnboundedSender<Finding>,
    /// Requests in flight across the run.
    pub semaphore: Arc<Semaphore>,
    /// Stop.
    pub cancel: CancellationToken,
    /// When the run started, for step stamps.
    pub started: Instant,
}

/// One subject's recording: the bounded trace a finding is cut from. The owner
/// worker keeps its own (it predates this and carries a model per lane); the
/// contention and fuzz workers record through here.
pub struct Lane {
    /// The subject.
    pub subject: Uuid,
    trace: VecDeque<Step>,
    next_index: usize,
    depth: usize,
}

impl Lane {
    /// A lane keeping the last `depth` steps.
    #[must_use]
    pub fn new(subject: Uuid, depth: usize) -> Self {
        Self {
            subject,
            trace: VecDeque::new(),
            next_index: 0,
            depth,
        }
    }

    /// The steps kept, oldest first.
    #[must_use]
    pub fn steps(&self) -> Vec<Step> {
        self.trace.iter().cloned().collect()
    }

    /// The index the next step gets.
    #[must_use]
    pub fn next_index(&self) -> usize {
        self.next_index
    }

    /// Record a step.
    pub fn push(
        &mut self,
        at_ms: f64,
        request: Request,
        response: Option<serde_json::Value>,
        error: Option<CallError>,
        tolerated: bool,
    ) -> usize {
        let index = self.next_index;
        self.next_index += 1;
        self.trace.push_back(Step {
            index,
            request,
            response,
            error,
            at_ms,
            tolerated,
        });
        while self.trace.len() > self.depth {
            self.trace.pop_front();
        }
        index
    }
}

/// One call from a non-owner worker: permit, timing, metrics, the step, the
/// `clean_errors` judgement. Returns the result and whether it was tolerated.
pub async fn send<T>(
    ctx: &Context,
    lane: &mut Lane,
    class: WorkerClass,
    request: Request,
    fut: impl Future<Output = Result<T, CallError>>,
    json: impl FnOnce(&T) -> serde_json::Value,
) -> (Result<T, CallError>, bool) {
    let permit = ctx.semaphore.clone().acquire_owned().await;
    let started = Instant::now();
    let result = fut.await;
    drop(permit);
    ctx.metrics.record(
        &ctx.target,
        request.op(),
        result.as_ref().map(|_| ()).map_err(CallError::class),
        started.elapsed(),
    );
    let tolerate = &ctx.campaign.faults.tolerate;
    let tolerated = result
        .as_ref()
        .err()
        .is_some_and(|e| tolerate.iter().any(|t| *t == e.class()));
    if tolerated {
        ctx.checks.tolerated();
    }
    let (response, error) = match &result {
        Ok(v) => (Some(json(v)), None),
        Err(e) => (None, Some(e.clone())),
    };
    lane.push(
        ctx.started.elapsed().as_secs_f64() * 1000.0,
        request,
        response,
        error,
        tolerated,
    );
    if let Err(e) = &result
        && !tolerated
        && let Some(v) = invariants::clean_errors(e, tolerate)
    {
        judge(ctx, lane, class, &["clean_errors"], vec![v]);
    } else if result.is_ok() {
        ctx.checks.pass("clean_errors");
    }
    (result, tolerated)
}

/// Count an evaluation and turn every violation into a finding on `lane`.
pub fn judge(
    ctx: &Context,
    lane: &Lane,
    class: WorkerClass,
    evaluated: &[&'static str],
    violations: Vec<Violation>,
) {
    for name in evaluated {
        if !violations.iter().any(|v| v.invariant == *name) {
            ctx.checks.pass(name);
        }
    }
    for v in violations {
        if !ctx.campaign.invariant_on(v.invariant) {
            continue;
        }
        ctx.checks.violate(v.invariant);
        let mut f = Finding::new(
            &v,
            lane.steps(),
            lane.subject,
            class,
            &ctx.campaign.campaign.name,
            &ctx.target,
            ctx.store.as_deref(),
        );
        if class == WorkerClass::Contention {
            f.shrink_note = Some(
                "concurrent: the trace is this worker's alone, so it is kept whole and not shrunk"
                    .into(),
            );
        }
        tracing::warn!(invariant = v.invariant, message = %v.message, subject = %f.subject, worker = class.name(), "finding");
        let _ = ctx.findings.send(f);
    }
}
