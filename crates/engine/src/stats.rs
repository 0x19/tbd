//! Per-instance request counters, readable by whoever holds the handle.
//!
//! Not production metrics. They exist so an in-process orchestrator can
//! assert which engine served what.

use std::sync::{
    Arc,
    atomic::{AtomicU64, Ordering},
};

use serde::Serialize;

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

    pub(crate) fn request(&self) {
        self.requests_total.fetch_add(1, Ordering::Relaxed);
    }

    pub(crate) fn failure(&self) {
        self.requests_failed.fetch_add(1, Ordering::Relaxed);
    }
}
