//! Workers: the loops that drive a target and judge every answer. Each class
//! has its own module; this one holds what they share.

pub mod owner;

use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use tokio::sync::{Semaphore, mpsc};
use tokio_util::sync::CancellationToken;

use crate::{
    campaign::Campaign, client::LedgerClient, finding::Finding, metrics::Metrics,
    report::CheckCount,
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
