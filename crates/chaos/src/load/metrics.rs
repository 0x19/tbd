//! One metrics type for all load: HDR histograms plus counters, with
//! per-operation, per-target and per-error-class breakdowns.

#![allow(clippy::cast_precision_loss)]

use std::{
    collections::BTreeMap,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use hdrhistogram::Histogram;
use serde::Serialize;

/// Live metrics. Safe to share across tasks.
pub struct Metrics {
    hist: Mutex<Histogram<u64>>,
    started: Mutex<Instant>,
    total: AtomicU64,
    success: AtomicU64,
    failed: AtomicU64,
    per_target: Mutex<BTreeMap<String, TargetCounts>>,
    per_op: Mutex<BTreeMap<String, OpStats>>,
    errors: Mutex<BTreeMap<String, u64>>,
}

/// Success and failure counts for one key.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct TargetCounts {
    /// Requests sent.
    pub total: u64,
    /// Requests that failed.
    pub failed: u64,
}

/// Per-operation counts and latency.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct OpSnapshot {
    /// Requests sent.
    pub total: u64,
    /// Requests that failed.
    pub failed: u64,
    /// Latency of successful requests.
    pub latency: Latency,
}

struct OpStats {
    counts: TargetCounts,
    hist: Histogram<u64>,
}

/// Latency percentiles in milliseconds.
#[derive(Debug, Clone, Copy, Default, Serialize)]
pub struct Latency {
    /// Median.
    pub p50_ms: f64,
    /// 90th percentile.
    pub p90_ms: f64,
    /// 99th percentile.
    pub p99_ms: f64,
    /// Maximum.
    pub max_ms: f64,
    /// Mean.
    pub mean_ms: f64,
}

/// A point-in-time copy of [`Metrics`].
#[derive(Debug, Clone, Default, Serialize)]
pub struct LoadSnapshot {
    /// Seconds since the last reset.
    pub elapsed_s: f64,
    /// Requests sent.
    pub requests_total: u64,
    /// Requests that succeeded.
    pub requests_success: u64,
    /// Requests that failed, including timeouts.
    pub requests_failed: u64,
    /// `failed / total`, 0 when nothing was sent.
    pub error_rate: f64,
    /// Requests per second over the elapsed window.
    pub throughput_rps: f64,
    /// Latency of successful requests.
    pub latency: Latency,
    /// Per target name.
    pub per_target: BTreeMap<String, TargetCounts>,
    /// Per operation name.
    pub per_op: BTreeMap<String, OpSnapshot>,
    /// Failures by class.
    pub errors: BTreeMap<String, u64>,
}

impl Default for Metrics {
    fn default() -> Self {
        Self::new()
    }
}

/// 1 µs .. 60 s, three significant figures.
fn histogram() -> Histogram<u64> {
    // These bounds are valid, so construction cannot fail; the fallback keeps
    // the function total without an `unwrap`.
    Histogram::new_with_bounds(1, 60_000_000, 3)
        .or_else(|_| Histogram::new(3))
        .unwrap_or_else(|_| unreachable!("a 3-sigfig histogram is always constructible"))
}

fn micros(latency: Duration) -> u64 {
    u64::try_from(latency.as_micros())
        .unwrap_or(u64::MAX)
        .clamp(1, 60_000_000)
}

fn latency_of(hist: &Histogram<u64>) -> Latency {
    let ms = |v: u64| v as f64 / 1000.0;
    Latency {
        p50_ms: ms(hist.value_at_quantile(0.50)),
        p90_ms: ms(hist.value_at_quantile(0.90)),
        p99_ms: ms(hist.value_at_quantile(0.99)),
        max_ms: ms(hist.max()),
        mean_ms: hist.mean() / 1000.0,
    }
}

impl Metrics {
    /// Empty metrics, clock started now.
    pub fn new() -> Self {
        Self {
            hist: Mutex::new(histogram()),
            started: Mutex::new(Instant::now()),
            total: AtomicU64::new(0),
            success: AtomicU64::new(0),
            failed: AtomicU64::new(0),
            per_target: Mutex::new(BTreeMap::new()),
            per_op: Mutex::new(BTreeMap::new()),
            errors: Mutex::new(BTreeMap::new()),
        }
    }

    /// Record one completed request.
    pub fn record(&self, target: &str, op: &str, outcome: Result<(), String>, latency: Duration) {
        self.total.fetch_add(1, Ordering::Relaxed);
        let failed = outcome.is_err();
        if failed {
            self.failed.fetch_add(1, Ordering::Relaxed);
        } else {
            self.success.fetch_add(1, Ordering::Relaxed);
            let _ = self
                .hist
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .record(micros(latency));
        }
        {
            let mut m = self
                .per_target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let e = m.entry(target.to_owned()).or_default();
            e.total += 1;
            if failed {
                e.failed += 1;
            }
        }
        {
            let mut m = self
                .per_op
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            let e = m.entry(op.to_owned()).or_insert_with(|| OpStats {
                counts: TargetCounts::default(),
                hist: histogram(),
            });
            e.counts.total += 1;
            if failed {
                e.counts.failed += 1;
            } else {
                let _ = e.hist.record(micros(latency));
            }
        }
        if let Err(class) = outcome {
            let mut m = self
                .errors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            *m.entry(class).or_default() += 1;
        }
    }

    /// Discard everything and restart the clock. Used after warmup.
    pub fn reset(&self) {
        self.hist
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reset();
        *self
            .started
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner) = Instant::now();
        self.total.store(0, Ordering::Relaxed);
        self.success.store(0, Ordering::Relaxed);
        self.failed.store(0, Ordering::Relaxed);
        self.per_target
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.per_op
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
        self.errors
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clear();
    }

    /// Copy everything out.
    pub fn snapshot(&self) -> LoadSnapshot {
        let elapsed = self
            .started
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .elapsed();
        let total = self.total.load(Ordering::Relaxed);
        let failed = self.failed.load(Ordering::Relaxed);
        let hist = self
            .hist
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let per_op = self
            .per_op
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .iter()
            .map(|(k, v)| {
                (
                    k.clone(),
                    OpSnapshot {
                        total: v.counts.total,
                        failed: v.counts.failed,
                        latency: latency_of(&v.hist),
                    },
                )
            })
            .collect();
        LoadSnapshot {
            elapsed_s: elapsed.as_secs_f64(),
            requests_total: total,
            requests_success: self.success.load(Ordering::Relaxed),
            requests_failed: failed,
            error_rate: if total == 0 {
                0.0
            } else {
                failed as f64 / total as f64
            },
            throughput_rps: if elapsed.is_zero() {
                0.0
            } else {
                total as f64 / elapsed.as_secs_f64()
            },
            latency: latency_of(&hist),
            per_target: self
                .per_target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
            per_op,
            errors: self
                .errors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        }
    }
}
