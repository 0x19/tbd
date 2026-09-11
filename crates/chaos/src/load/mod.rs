//! Load generation: a paced, open-loop generator that spreads a weighted mix
//! of operations across targets and records everything into [`Metrics`].

pub mod generator;
pub mod ledger_ops;
pub mod metrics;
pub mod ops;

use std::time::Duration;

use serde::{Deserialize, Serialize};

pub use generator::{Hooks, PROGRESS_INTERVAL, run, run_with};
pub use metrics::{LoadSnapshot, Metrics};
pub use ops::{OpKind, Target};

/// `[load]` in a scenario.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LoadConfig {
    /// Requests per second, spread across all targets. With a `ramp` pattern
    /// this is ignored in favour of the pattern's rates.
    #[serde(default = "default_rate")]
    pub rate: f64,
    /// How long to generate load, excluding warmup.
    #[serde(with = "humantime_serde")]
    pub duration: Duration,
    /// Optional warmup: same load, metrics discarded.
    #[serde(default, with = "humantime_serde")]
    pub warmup: Option<Duration>,
    /// Per-request timeout.
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub timeout: Duration,
    /// Upper bound on concurrent requests; the pacer stalls when reached.
    #[serde(default = "default_in_flight")]
    pub max_in_flight: usize,
    /// How the rate changes over `duration`.
    #[serde(default)]
    pub pattern: Pattern,
    /// Weighted mix of operations. Defaults to REST evaluate only.
    #[serde(default = "default_operations")]
    pub operations: Vec<OperationWeight>,
    /// Seed for the operations that generate data (`ledger_fuzz`, the
    /// ledger subject pool), so a run is reproducible.
    #[serde(default)]
    pub seed: u64,
    /// Subjects the ledger operations spread over.
    #[serde(default = "default_subjects")]
    pub subjects: u32,
}

fn default_subjects() -> u32 {
    100
}

fn default_rate() -> f64 {
    50.0
}

fn default_timeout() -> Duration {
    Duration::from_secs(5)
}

fn default_in_flight() -> usize {
    256
}

fn default_operations() -> Vec<OperationWeight> {
    vec![OperationWeight {
        op: OpKind::RestEvaluate,
        weight: 1,
    }]
}

/// One entry in the operation mix.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct OperationWeight {
    /// Which operation.
    pub op: OpKind,
    /// Relative weight.
    #[serde(default = "one")]
    pub weight: u32,
}

fn one() -> u32 {
    1
}

/// Rate over time.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum Pattern {
    /// `rate` for the whole duration.
    #[default]
    Constant,
    /// Linear from `start_rate` to `end_rate` over the duration.
    Ramp {
        /// Rate at the start.
        start_rate: f64,
        /// Rate at the end.
        end_rate: f64,
    },
}

impl Pattern {
    /// Requests per second at `elapsed` into a run of `total` length.
    pub fn rate_at(&self, base: f64, elapsed: Duration, total: Duration) -> f64 {
        match self {
            Self::Constant => base,
            Self::Ramp {
                start_rate,
                end_rate,
            } => {
                let t = if total.is_zero() {
                    1.0
                } else {
                    (elapsed.as_secs_f64() / total.as_secs_f64()).clamp(0.0, 1.0)
                };
                start_rate + (end_rate - start_rate) * t
            }
        }
    }
}

impl LoadConfig {
    /// The target kinds the configured operations need, deduplicated.
    #[must_use]
    pub fn target_kinds(&self) -> Vec<&'static str> {
        let mut kinds: Vec<&'static str> = self
            .operations
            .iter()
            .filter(|o| o.weight > 0)
            .map(|o| o.op.target_kind())
            .collect();
        kinds.sort_unstable();
        kinds.dedup();
        kinds
    }

    /// Structural checks that do not need a running stack.
    pub fn check(&self) -> Result<(), String> {
        if self.duration.is_zero() {
            return Err("load.duration must be > 0".into());
        }
        if self.operations.is_empty() {
            return Err("load.operations must not be empty".into());
        }
        if self.operations.iter().all(|o| o.weight == 0) {
            return Err("at least one operation needs weight > 0".into());
        }
        match self.pattern {
            Pattern::Constant if self.rate <= 0.0 => Err("load.rate must be > 0".into()),
            Pattern::Ramp {
                start_rate,
                end_rate,
            } if start_rate < 0.0 || end_rate < 0.0 => Err("ramp rates must be >= 0".into()),
            _ => Ok(()),
        }
    }
}
