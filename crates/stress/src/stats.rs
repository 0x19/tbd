//! The statistics a sweep reports: quantiles, a bootstrap confidence interval
//! around one, and where the curve turns.
//!
//! A sweep repeats every point, so a point is not one number but a sample. The
//! interval says how much of the difference between two points is real: two
//! points whose intervals overlap did not measure differently, however far
//! apart their estimates look.

#![allow(
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation,
    clippy::cast_sign_loss
)]

use rand::{Rng, SeedableRng, rngs::StdRng};
use serde::{Deserialize, Serialize};

/// An estimate and the interval around it, in the sample's own unit.
#[derive(Debug, Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
pub struct Ci {
    /// The quantile of the sample itself.
    pub estimate: f64,
    /// Lower bound of the interval.
    pub low: f64,
    /// Upper bound.
    pub high: f64,
}

impl Ci {
    /// Whether two intervals overlap: they did not measure differently.
    #[must_use]
    pub fn overlaps(&self, other: &Self) -> bool {
        self.low <= other.high && other.low <= self.high
    }
}

/// The `q` quantile of `sorted` (ascending), by nearest rank. `0.0` when empty.
#[must_use]
pub fn quantile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let q = q.clamp(0.0, 1.0);
    let rank = (q * (sorted.len() - 1) as f64).round() as usize;
    sorted[rank.min(sorted.len() - 1)]
}

/// Sort a copy, ascending, dropping anything that is not a number.
#[must_use]
pub fn sorted(samples: &[f64]) -> Vec<f64> {
    let mut v: Vec<f64> = samples.iter().copied().filter(|x| x.is_finite()).collect();
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    v
}

/// How many resamples a bootstrap draws.
pub const BOOTSTRAP_ITERATIONS: usize = 1000;

/// A percentile bootstrap around the `q` quantile: resample `samples` with
/// replacement `iterations` times, take the quantile of each resample, and cut
/// the middle `1 - alpha` of those. Seeded, so a campaign reports the same
/// interval twice for the same measurements.
#[must_use]
pub fn bootstrap_ci(samples: &[f64], q: f64, iterations: usize, seed: u64) -> Ci {
    let s = sorted(samples);
    let estimate = quantile(&s, q);
    if s.len() < 2 || iterations == 0 {
        return Ci {
            estimate,
            low: estimate,
            high: estimate,
        };
    }
    let mut rng = StdRng::seed_from_u64(seed);
    let mut draws: Vec<f64> = Vec::with_capacity(iterations);
    let mut resample: Vec<f64> = vec![0.0; s.len()];
    for _ in 0..iterations {
        for slot in &mut resample {
            *slot = s[rng.random_range(0..s.len())];
        }
        resample.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        draws.push(quantile(&resample, q));
    }
    draws.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    Ci {
        estimate,
        low: quantile(&draws, 0.025),
        high: quantile(&draws, 0.975),
    }
}

/// Where a curve turns: the first point whose p99 passes `p99_ms`, or whose p99
/// is at least `factor` times the previous point's, whichever comes first.
/// `None` when neither bound is set or nothing crosses.
#[must_use]
pub fn knee(p99s: &[f64], p99_ms: Option<f64>, factor: Option<f64>) -> Option<usize> {
    for (i, p99) in p99s.iter().enumerate() {
        if p99_ms.is_some_and(|bound| *p99 > bound) {
            return Some(i);
        }
        if i > 0
            && let Some(f) = factor
            && p99s[i - 1] > 0.0
            && *p99 >= p99s[i - 1] * f
        {
            return Some(i);
        }
    }
    None
}

#[cfg(test)]
#[allow(clippy::float_cmp)] // these compare values that came straight out of the input
mod tests {
    use super::*;

    #[test]
    fn quantiles_are_nearest_rank() {
        let s = sorted(&[5.0, 1.0, 3.0, 2.0, 4.0]);
        assert_eq!(quantile(&s, 0.0), 1.0);
        assert_eq!(quantile(&s, 0.5), 3.0);
        assert_eq!(quantile(&s, 1.0), 5.0);
        assert_eq!(quantile(&[], 0.5), 0.0);
        // Anything that is not a number is dropped rather than poisoning the sort.
        assert_eq!(sorted(&[1.0, f64::NAN, 2.0]).len(), 2);
    }

    #[test]
    fn the_interval_brackets_a_known_quantile() {
        // 0..1000: the median is 500 and the 99th percentile 990.
        let samples: Vec<f64> = (0..1000).map(f64::from).collect();
        let p50 = bootstrap_ci(&samples, 0.5, BOOTSTRAP_ITERATIONS, 1);
        assert!((p50.estimate - 500.0).abs() < 2.0, "{p50:?}");
        assert!(
            p50.low <= p50.estimate && p50.estimate <= p50.high,
            "{p50:?}"
        );
        assert!(p50.low > 440.0 && p50.high < 560.0, "{p50:?}");
        let p99 = bootstrap_ci(&samples, 0.99, BOOTSTRAP_ITERATIONS, 1);
        assert!((p99.estimate - 990.0).abs() < 2.0, "{p99:?}");
        assert!(p99.low <= 990.0 && p99.high >= 990.0, "{p99:?}");
        // The same seed and samples give the same interval.
        assert_eq!(p99, bootstrap_ci(&samples, 0.99, BOOTSTRAP_ITERATIONS, 1));
    }

    #[test]
    fn one_sample_has_no_spread_and_overlap_means_no_difference() {
        let one = bootstrap_ci(&[7.0], 0.5, BOOTSTRAP_ITERATIONS, 1);
        assert_eq!(
            one,
            Ci {
                estimate: 7.0,
                low: 7.0,
                high: 7.0
            }
        );
        let a = bootstrap_ci(&(0..200).map(f64::from).collect::<Vec<_>>(), 0.5, 500, 2);
        let b = bootstrap_ci(&(1..201).map(f64::from).collect::<Vec<_>>(), 0.5, 500, 3);
        assert!(a.overlaps(&b), "{a:?} {b:?}");
        let far = bootstrap_ci(&(900..1100).map(f64::from).collect::<Vec<_>>(), 0.5, 500, 4);
        assert!(!a.overlaps(&far), "{a:?} {far:?}");
    }

    #[test]
    fn the_knee_is_the_first_crossing() {
        let hockey_stick = [10.0, 11.0, 12.0, 48.0, 300.0];
        assert_eq!(knee(&hockey_stick, Some(50.0), None), Some(4));
        assert_eq!(knee(&hockey_stick, None, Some(2.0)), Some(3));
        assert_eq!(knee(&hockey_stick, Some(50.0), Some(2.0)), Some(3));
        assert_eq!(knee(&[10.0, 11.0, 12.0], Some(50.0), Some(2.0)), None);
        assert_eq!(knee(&[], Some(1.0), Some(1.5)), None);
        assert_eq!(knee(&hockey_stick, None, None), None);
    }
}
