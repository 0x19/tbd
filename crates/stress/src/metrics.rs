//! One metrics type for all load: HDR histograms plus counters, with
//! per-operation, per-target and per-error-class breakdowns.

#![allow(clippy::cast_precision_loss)]

use std::{
    collections::BTreeMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use hdrhistogram::Histogram;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Latency samples kept for the statistics a sweep reports. A histogram gives
/// quantiles but not a sample to resample, so successful latencies are also
/// kept in a bounded reservoir (algorithm R: every request has the same chance
/// of being in it, however many there were).
pub const RESERVOIR: usize = 50_000;

/// Live metrics. Safe to share across tasks.
pub struct Metrics {
    hist: Mutex<Histogram<u64>>,
    samples: Mutex<Reservoir>,
    started: Mutex<Instant>,
    total: AtomicU64,
    success: AtomicU64,
    failed: AtomicU64,
    per_target: Mutex<BTreeMap<String, TargetCounts>>,
    per_op: Mutex<BTreeMap<String, OpStats>>,
    errors: Mutex<BTreeMap<String, u64>>,
}

/// Success and failure counts for one key.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct TargetCounts {
    /// Requests sent.
    pub total: u64,
    /// Requests that failed.
    pub failed: u64,
}

/// Per-operation counts and latency, plus whatever the operation metered
/// itself (empty for one that meters nothing).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OpSnapshot {
    /// Requests sent.
    pub total: u64,
    /// Requests that failed.
    pub failed: u64,
    /// Latency of successful requests.
    pub latency: Latency,
    /// Counters the operation reported through [`Meter::count`], by name
    /// (an LLM operation's `completion_tokens`, say). A rate is the counter
    /// over the snapshot's `elapsed_s`.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub counters: BTreeMap<String, u64>,
    /// Timings the operation reported through [`Meter::sample`], by name
    /// (`ttft`, say), as latency quantiles.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub samples: BTreeMap<String, Latency>,
}

struct OpStats {
    counts: TargetCounts,
    hist: Histogram<u64>,
    counters: BTreeMap<String, u64>,
    samples: BTreeMap<String, Histogram<u64>>,
}

impl OpStats {
    fn new() -> Self {
        Self {
            counts: TargetCounts::default(),
            hist: histogram(),
            counters: BTreeMap::new(),
            samples: BTreeMap::new(),
        }
    }
}

/// What an operation may add beside the one latency the generator records
/// for it: named counters and named timings, kept per operation. A handle
/// over the run's [`Metrics`]; cheap to clone, and an operation that has
/// nothing to add never touches it.
#[derive(Clone)]
pub struct Meter {
    metrics: Option<Arc<Metrics>>,
}

impl Meter {
    /// A meter that records into `metrics`.
    #[must_use]
    pub fn new(metrics: Arc<Metrics>) -> Self {
        Self {
            metrics: Some(metrics),
        }
    }

    /// A meter that drops everything, for callers with no run behind them.
    #[must_use]
    pub fn none() -> Self {
        Self { metrics: None }
    }

    /// Add `n` to the counter `name` of operation `op`.
    pub fn count(&self, op: &str, name: &str, n: u64) {
        if let Some(m) = &self.metrics {
            m.count(op, name, n);
        }
    }

    /// Record one timing under `name` for operation `op`.
    pub fn sample(&self, op: &str, name: &str, value: Duration) {
        if let Some(m) = &self.metrics {
            m.sample(op, name, value);
        }
    }
}

impl std::fmt::Debug for Meter {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(if self.metrics.is_some() {
            "Meter(live)"
        } else {
            "Meter(none)"
        })
    }
}

/// A uniform sample of at most [`RESERVOIR`] latencies in milliseconds.
#[derive(Default)]
struct Reservoir {
    kept: Vec<f32>,
    seen: u64,
}

impl Reservoir {
    fn record(&mut self, ms: f32) {
        self.seen += 1;
        if self.kept.len() < RESERVOIR {
            self.kept.push(ms);
            return;
        }
        // Replace a slot with probability RESERVOIR/seen.
        let slot = rand::rng().random_range(0..self.seen);
        if let Ok(i) = usize::try_from(slot)
            && i < RESERVOIR
        {
            self.kept[i] = ms;
        }
    }

    fn absorb(&mut self, other: &Self) {
        self.seen += other.seen;
        for ms in &other.kept {
            if self.kept.len() < RESERVOIR {
                self.kept.push(*ms);
            } else {
                let slot = rand::rng().random_range(0..self.kept.len());
                self.kept[slot] = *ms;
            }
        }
    }

    fn clear(&mut self) {
        self.kept.clear();
        self.seen = 0;
    }
}

/// Latency percentiles in milliseconds.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
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
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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

/// 1 µs .. 10 min, three significant figures. A request path is normally
/// milliseconds, but a streamed generation queued behind a large model runs
/// for a minute or more, and a cap at 60 s flattened exactly those.
fn histogram() -> Histogram<u64> {
    // These bounds are valid, so construction cannot fail; the fallback keeps
    // the function total without an `unwrap`.
    Histogram::new_with_bounds(1, 600_000_000, 3)
        .or_else(|_| Histogram::new(3))
        .unwrap_or_else(|_| unreachable!("a 3-sigfig histogram is always constructible"))
}

fn micros(latency: Duration) -> u64 {
    u64::try_from(latency.as_micros())
        .unwrap_or(u64::MAX)
        .clamp(1, 600_000_000)
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
            samples: Mutex::new(Reservoir::default()),
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
            self.lock_samples().record(latency.as_secs_f32() * 1000.0);
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
            let e = m.entry(op.to_owned()).or_insert_with(OpStats::new);
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

    fn lock_samples(&self) -> std::sync::MutexGuard<'_, Reservoir> {
        self.samples
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
    }

    /// The latency sample kept for statistics, in milliseconds.
    pub fn latencies(&self) -> Vec<f64> {
        self.lock_samples()
            .kept
            .iter()
            .map(|ms| f64::from(*ms))
            .collect()
    }

    /// Add `n` to an operation's named counter (see [`Meter::count`]).
    pub fn count(&self, op: &str, name: &str, n: u64) {
        let mut m = self
            .per_op
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let e = m.entry(op.to_owned()).or_insert_with(OpStats::new);
        *e.counters.entry(name.to_owned()).or_default() += n;
    }

    /// Record one timing under an operation's named sample (see [`Meter::sample`]).
    pub fn sample(&self, op: &str, name: &str, value: Duration) {
        let mut m = self
            .per_op
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let e = m.entry(op.to_owned()).or_insert_with(OpStats::new);
        let _ = e
            .samples
            .entry(name.to_owned())
            .or_insert_with(histogram)
            .record(micros(value));
    }

    /// Add everything `other` counted to this, for a total over several
    /// measured phases. The clock is this one's: a total's throughput is over
    /// its own wall time.
    pub fn absorb(&self, other: &Self) {
        let mut hist = self
            .hist
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let their_hist = other
            .hist
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let _ = hist.add(&*their_hist);
        drop(their_hist);
        drop(hist);
        self.total
            .fetch_add(other.total.load(Ordering::Relaxed), Ordering::Relaxed);
        self.success
            .fetch_add(other.success.load(Ordering::Relaxed), Ordering::Relaxed);
        self.failed
            .fetch_add(other.failed.load(Ordering::Relaxed), Ordering::Relaxed);
        {
            let mut mine = self
                .per_target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (k, v) in &*other
                .per_target
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            {
                let e = mine.entry(k.clone()).or_default();
                e.total += v.total;
                e.failed += v.failed;
            }
        }
        {
            let mut mine = self
                .per_op
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (k, v) in &*other
                .per_op
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            {
                let e = mine.entry(k.clone()).or_insert_with(OpStats::new);
                e.counts.total += v.counts.total;
                e.counts.failed += v.counts.failed;
                let _ = e.hist.add(&v.hist);
                for (name, n) in &v.counters {
                    *e.counters.entry(name.clone()).or_default() += n;
                }
                for (name, h) in &v.samples {
                    let _ = e
                        .samples
                        .entry(name.clone())
                        .or_insert_with(histogram)
                        .add(h);
                }
            }
        }
        {
            let mut mine = self
                .errors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner);
            for (k, v) in &*other
                .errors
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
            {
                *mine.entry(k.clone()).or_default() += v;
            }
        }
        self.lock_samples().absorb(&other.lock_samples());
    }

    /// Discard everything and restart the clock. Used after warmup.
    pub fn reset(&self) {
        self.hist
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .reset();
        self.lock_samples().clear();
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

    /// Copy everything out and start again, so the next snapshot describes only
    /// what happened since this call.
    ///
    /// A caller that wants "the last second" rather than "the run so far" wants
    /// this: percentiles cannot be subtracted, so the only way to get a windowed
    /// p99 out of a histogram is to empty it each time.
    ///
    /// Not atomic. The counters and the histogram are separate locks, so a
    /// request that lands between the copy and the clear is counted in neither.
    /// At any sane rate that is a handful of requests a day; if it ever needs to
    /// be exact, the whole struct needs one lock rather than nine.
    pub fn drain(&self) -> LoadSnapshot {
        let snapshot = self.snapshot();
        self.reset();
        snapshot
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
                        counters: v.counters.clone(),
                        samples: v
                            .samples
                            .iter()
                            .map(|(name, h)| (name.clone(), latency_of(h)))
                            .collect(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn meters_are_per_operation_and_survive_absorb_but_not_reset() {
        let first = Metrics::new();
        first.record("t", "generate", Ok(()), Duration::from_millis(50));
        first.record("t", "ping", Ok(()), Duration::from_millis(1));
        let meter = Meter::new(Arc::new(Metrics::new()));
        drop(meter);
        first.count("generate", "completion_tokens", 40);
        first.count("generate", "completion_tokens", 8);
        first.sample("generate", "ttft", Duration::from_millis(120));
        first.sample("generate", "ttft", Duration::from_millis(80));

        let snap = first.snapshot();
        let generate = &snap.per_op["generate"];
        assert_eq!(generate.counters["completion_tokens"], 48);
        let ttft = &generate.samples["ttft"];
        assert!(ttft.p50_ms >= 80.0 && ttft.max_ms >= 120.0, "{ttft:?}");
        let ping = &snap.per_op["ping"];
        assert!(
            ping.counters.is_empty() && ping.samples.is_empty(),
            "an operation that meters nothing shows nothing"
        );

        let total = Metrics::new();
        total.count("generate", "completion_tokens", 2);
        total.absorb(&first);
        let snap = total.snapshot();
        assert_eq!(snap.per_op["generate"].counters["completion_tokens"], 50);
        assert!(snap.per_op["generate"].samples["ttft"].max_ms >= 120.0);

        total.reset();
        assert!(total.snapshot().per_op.is_empty());
    }

    #[test]
    fn a_meter_without_metrics_records_nothing_and_a_snapshot_round_trips() {
        Meter::none().count("x", "y", 1);
        Meter::none().sample("x", "y", Duration::from_millis(1));
        let m = Arc::new(Metrics::new());
        let meter = Meter::new(m.clone());
        meter.count("generate", "generations", 1);
        let snap = m.snapshot();
        let json = serde_json::to_string(&snap).unwrap_or_default();
        let back: LoadSnapshot = serde_json::from_str(&json).unwrap_or_default();
        assert_eq!(back.per_op["generate"].counters["generations"], 1);
        // An older record without the maps still parses.
        let old = r#"{"total":1,"failed":0,"latency":{"p50_ms":1.0,"p90_ms":1.0,"p99_ms":1.0,"max_ms":1.0,"mean_ms":1.0}}"#;
        let op: OpSnapshot = serde_json::from_str(old).unwrap_or_default();
        assert!(op.counters.is_empty() && op.samples.is_empty());
    }

    fn record(m: &Metrics, n: usize, ms: u64) {
        for _ in 0..n {
            m.record("t", "append", Ok(()), Duration::from_millis(ms));
        }
    }

    #[test]
    fn a_total_absorbs_every_phase() {
        let first = Metrics::new();
        record(&first, 10, 5);
        first.record(
            "t",
            "append",
            Err("unavailable".into()),
            Duration::from_millis(1),
        );
        let second = Metrics::new();
        record(&second, 10, 15);
        let total = Metrics::new();
        total.absorb(&first);
        total.absorb(&second);
        let s = total.snapshot();
        assert_eq!(s.requests_total, 21);
        assert_eq!(s.requests_failed, 1);
        assert_eq!(s.errors["unavailable"], 1);
        assert_eq!(s.per_op["append"].total, 21);
        // The merged histogram spans both phases.
        assert!(
            s.latency.p50_ms >= 5.0 && s.latency.max_ms >= 15.0,
            "{:?}",
            s.latency
        );
        assert_eq!(total.latencies().len(), 20);
    }

    #[test]
    fn the_reservoir_is_bounded_and_reset_clears_it() {
        let m = Metrics::new();
        record(&m, RESERVOIR + 100, 2);
        assert_eq!(m.latencies().len(), RESERVOIR);
        m.reset();
        assert!(m.latencies().is_empty());
        assert_eq!(m.snapshot().requests_total, 0);
    }
}
