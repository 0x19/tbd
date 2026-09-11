//! What an embedder keeps to observe and perturb a running service: a fault
//! handle and per-instance request counters.
//!
//! The counters are not production metrics. They exist so an in-process
//! orchestrator (the chaos tool, integration tests) can assert which instance
//! served what.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use serde::Serialize;

use crate::fault::FaultHandle;

/// Handles an embedder keeps to observe and perturb a running service.
#[derive(Debug, Clone, Default)]
pub struct Runtime {
    /// Fault injection at the request adapter. Healthy unless something sets it.
    pub fault: FaultHandle,
    /// Fault injection at the store, for services that have one: a read fails
    /// before it runs, a write fails after it committed (the acknowledgement is
    /// lost, the commit stands). Ignored by services without a store.
    pub store_fault: FaultHandle,
    /// Request counters.
    pub stats: StatsHandle,
}

/// Live counters. Share via [`StatsHandle`].
#[derive(Debug, Default)]
pub struct Stats {
    /// Calls and streams received, of every kind.
    pub requests_total: AtomicU64,
    /// Requests that ended in an error, injected or real.
    pub requests_failed: AtomicU64,
}

/// Shared stats.
pub type StatsHandle = Arc<Stats>;

/// A point-in-time copy of [`Stats`].
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize)]
pub struct StatsSnapshot {
    /// See [`Stats::requests_total`].
    pub requests_total: u64,
    /// See [`Stats::requests_failed`].
    pub requests_failed: u64,
}

impl Stats {
    /// Copy every counter.
    pub fn snapshot(&self) -> StatsSnapshot {
        StatsSnapshot {
            requests_total: self.requests_total.load(Ordering::Relaxed),
            requests_failed: self.requests_failed.load(Ordering::Relaxed),
        }
    }

    /// Count one request or stream received.
    pub fn request(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    /// Count one request that ended in an error.
    pub fn failure(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }
}
