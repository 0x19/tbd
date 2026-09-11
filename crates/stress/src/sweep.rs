//! A sweep: the same campaign at every value of one parameter, repeated, so a
//! difference between two points can be told from noise.
//!
//! Every repeat is an independent measurement — fresh workers, fresh subjects,
//! its own warmup — and the latencies of a point's repeats pool into one
//! sample, which is what the confidence interval is computed from. The knee is
//! the first point that crosses `[sweep.knee]`.

use std::{sync::Arc, time::Instant};

use crate::{
    campaign::Campaign,
    executor::{Named, Phase, Progress, Shared, measured_phase},
    hooks::{Hooks, StressEvent},
    metrics::Metrics,
    report::{PointStats, SweepProgress, SweepResult},
    stats::{self, BOOTSTRAP_ITERATIONS},
};

/// Run every point of `campaign`'s sweep. Returns what it measured and the
/// first worker panic, if any.
pub(crate) async fn run(
    campaign: &Arc<Campaign>,
    targets: &[Named],
    store: Option<&str>,
    shared: &Shared,
    total: &Metrics,
    progress: &Progress,
    hooks: &Hooks,
) -> (SweepResult, Option<String>) {
    let sweep = campaign
        .sweep
        .clone()
        .unwrap_or_else(|| unreachable!("a sweep"));
    let started = Instant::now();
    let mut points = Vec::with_capacity(sweep.values.len());
    let mut error = None;
    for (i, value) in sweep.values.iter().enumerate() {
        let (stats, panicked) = point(At {
            campaign,
            sweep: &sweep,
            value: *value,
            index: i,
            targets,
            store,
            shared,
            total,
            progress,
            hooks,
        })
        .await;
        points.push(stats);
        if error.is_none() {
            error = panicked;
        }
        if shared.cancel.is_cancelled() {
            break;
        }
    }
    progress.set_sweep(None);
    let p99s: Vec<f64> = points.iter().map(|p| p.p99.estimate).collect();
    let at = stats::knee(&p99s, sweep.knee.p99_ms, sweep.knee.factor);
    let (knee, knee_reason) = match at.and_then(|i| points.get(i).map(|p| (i, p))) {
        Some((i, p)) => (
            Some(p.value),
            Some(reason(
                &sweep.parameter,
                p.p99.estimate,
                i,
                &p99s,
                sweep.knee,
            )),
        ),
        None => (None, None),
    };
    tracing::info!(
        parameter = %sweep.parameter,
        points = points.len(),
        took_s = started.elapsed().as_secs_f64(),
        knee = ?knee,
        "sweep finished"
    );
    (
        SweepResult {
            parameter: sweep.parameter,
            points,
            knee,
            knee_reason,
        },
        error,
    )
}

/// One point of a sweep and everything it needs.
struct At<'a> {
    campaign: &'a Arc<Campaign>,
    sweep: &'a crate::campaign::Sweep,
    value: u64,
    index: usize,
    targets: &'a [Named],
    store: Option<&'a str>,
    shared: &'a Shared,
    total: &'a Metrics,
    progress: &'a Progress,
    hooks: &'a Hooks,
}

/// Measure one point: `repeat` independent phases whose latencies pool into
/// one sample. `Some` when a worker panicked.
async fn point(at: At<'_>) -> (PointStats, Option<String>) {
    let (campaign, sweep, value, i) = (at.campaign, at.sweep, at.value, at.index);
    let (targets, store, shared, total, progress, hooks) = (
        at.targets,
        at.store,
        at.shared,
        at.total,
        at.progress,
        at.hooks,
    );
    let mut error = None;
    let point = Arc::new(campaign.at_point(value));
    let mut latencies: Vec<f64> = Vec::new();
    let mut requests = 0u64;
    let mut failed = 0u64;
    let mut rps = Vec::with_capacity(sweep.repeat as usize);
    let found_before = progress.found();
    let mut repeats_run = 0u32;
    for r in 0..sweep.repeat {
        if shared.cancel.is_cancelled() {
            break;
        }
        progress.set_sweep(Some(SweepProgress {
            parameter: sweep.parameter.clone(),
            value,
            point: i + 1,
            points: sweep.values.len(),
            repeat: r + 1,
            repeats: sweep.repeat,
        }));
        hooks.emit(StressEvent::Phase {
            name: format!(
                "sweep {} = {value} ({}/{})",
                sweep.parameter,
                r + 1,
                sweep.repeat
            ),
        });
        let panicked = measured_phase(
            Phase {
                campaign: &point,
                targets,
                store,
                shared,
                progress,
                hooks,
                warmup: point.campaign.warmup,
                // The first repeat of the first point clears what warmup
                // evaluated; from then on the counts accumulate.
                reset_checks: i == 0 && r == 0,
            },
            &|phase, elapsed| progress.snapshot(phase, elapsed),
        )
        .await;
        if error.is_none() {
            error = panicked;
        }
        let snapshot = shared.metrics.snapshot();
        requests += snapshot.requests_total;
        failed += snapshot.requests_failed;
        rps.push(snapshot.throughput_rps);
        latencies.extend(shared.metrics.latencies());
        total.absorb(&shared.metrics);
        shared.metrics.reset();
        repeats_run += 1;
    }
    // The seed makes the interval reproducible for the same measurements.
    let seed = campaign.campaign.seed.wrapping_add(value);
    let stats = PointStats {
        value,
        achieved_rps: mean(&rps),
        error_rate: if requests == 0 {
            0.0
        } else {
            #[allow(clippy::cast_precision_loss)]
            {
                failed as f64 / requests as f64
            }
        },
        requests,
        p50: stats::bootstrap_ci(&latencies, 0.50, BOOTSTRAP_ITERATIONS, seed),
        p99: stats::bootstrap_ci(&latencies, 0.99, BOOTSTRAP_ITERATIONS, seed ^ 0x9e37),
        repeats: repeats_run,
        findings: progress.found().saturating_sub(found_before),
    };
    (stats, error)
}

/// Why a point is the knee, in one sentence.
fn reason(
    parameter: &str,
    p99: f64,
    i: usize,
    p99s: &[f64],
    knee: crate::campaign::Knee,
) -> String {
    if knee.p99_ms.is_some_and(|bound| p99 > bound) {
        return format!(
            "p99 {p99:.2} ms passed the bound of {:.2} ms at {parameter}",
            knee.p99_ms.unwrap_or_default()
        );
    }
    let previous = if i > 0 { p99s[i - 1] } else { p99 };
    format!(
        "p99 {p99:.2} ms is {:.1}x the previous point's {previous:.2} ms at {parameter}",
        if previous > 0.0 { p99 / previous } else { 0.0 }
    )
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    #[allow(clippy::cast_precision_loss)]
    {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // these compare values that came straight out of the input
mod tests {
    use super::*;

    #[test]
    fn the_mean_of_nothing_is_zero() {
        assert!((mean(&[1.0, 2.0, 6.0]) - 3.0).abs() < f64::EPSILON);
        assert_eq!(mean(&[]), 0.0);
    }

    #[test]
    fn the_reason_names_the_bound_or_the_jump() {
        let knee = crate::campaign::Knee {
            p99_ms: Some(50.0),
            factor: Some(2.0),
        };
        let bound = reason("owner.workers", 80.0, 2, &[10.0, 12.0, 80.0], knee);
        assert!(bound.contains("passed the bound"), "{bound}");
        let jump = reason(
            "owner.workers",
            30.0,
            2,
            &[10.0, 12.0, 30.0],
            crate::campaign::Knee {
                p99_ms: None,
                factor: Some(2.0),
            },
        );
        assert!(jump.contains("2.5x"), "{jump}");
    }
}
