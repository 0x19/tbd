//! Assertions evaluated over an immutable snapshot after load finishes.

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::{load::LoadSnapshot, service::RequestCounts};

/// `[assertions]`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Assertions {
    /// `failed / total` across all load, inclusive bound.
    #[serde(default)]
    pub max_error_rate: Option<f64>,
    /// Median latency of successful requests.
    #[serde(default)]
    pub max_p50_ms: Option<f64>,
    /// 99th percentile latency of successful requests.
    #[serde(default)]
    pub max_p99_ms: Option<f64>,
    /// Requests sent during the measured window.
    #[serde(default)]
    pub min_requests: Option<u64>,
    /// Requests per second over the measured window.
    #[serde(default)]
    pub min_throughput: Option<f64>,
    /// Per service instance, keyed by name.
    #[serde(default)]
    pub services: BTreeMap<String, ServiceAssertions>,
}

/// Assertions on one instance. Engines are checked against their own counters;
/// protocols against what the load generator sent them.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceAssertions {
    /// Requests the instance handled, at least.
    #[serde(default)]
    pub min_requests: Option<u64>,
    /// Requests the instance handled, at most.
    #[serde(default)]
    pub max_requests: Option<u64>,
    /// Failed requests, at most.
    #[serde(default)]
    pub max_failed: Option<u64>,
}

/// One evaluated assertion.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssertionResult {
    /// Which assertion.
    pub name: String,
    /// Passed.
    pub passed: bool,
    /// Bound.
    pub expected: String,
    /// Observed.
    pub actual: String,
}

/// What assertions look at.
pub struct Snapshot<'a> {
    /// Load metrics, if load ran.
    pub load: Option<&'a LoadSnapshot>,
    /// Engine-side counters by instance name.
    pub engines: &'a BTreeMap<String, RequestCounts>,
}

impl Assertions {
    /// True when any load-level assertion is set.
    pub fn needs_load(&self) -> bool {
        self.max_error_rate.is_some()
            || self.max_p50_ms.is_some()
            || self.max_p99_ms.is_some()
            || self.min_requests.is_some()
            || self.min_throughput.is_some()
    }

    /// Evaluate everything.
    pub fn evaluate(&self, snap: &Snapshot<'_>) -> Vec<AssertionResult> {
        let mut out = Vec::new();
        let load = snap.load.cloned().unwrap_or_default();

        if let Some(max) = self.max_error_rate {
            out.push(check(
                "max_error_rate",
                load.error_rate <= max,
                format!("<= {:.2}%", max * 100.0),
                format!("{:.2}%", load.error_rate * 100.0),
            ));
        }
        if let Some(max) = self.max_p50_ms {
            out.push(check(
                "max_p50_ms",
                load.latency.p50_ms <= max,
                format!("<= {max} ms"),
                format!("{:.1} ms", load.latency.p50_ms),
            ));
        }
        if let Some(max) = self.max_p99_ms {
            out.push(check(
                "max_p99_ms",
                load.latency.p99_ms <= max,
                format!("<= {max} ms"),
                format!("{:.1} ms", load.latency.p99_ms),
            ));
        }
        if let Some(min) = self.min_requests {
            out.push(check(
                "min_requests",
                load.requests_total >= min,
                format!(">= {min}"),
                load.requests_total.to_string(),
            ));
        }
        if let Some(min) = self.min_throughput {
            out.push(check(
                "min_throughput",
                load.throughput_rps >= min,
                format!(">= {min} rps"),
                format!("{:.1} rps", load.throughput_rps),
            ));
        }

        for (name, a) in &self.services {
            let counts = snap.engines.get(name).copied().or_else(|| {
                load.per_target.get(name).map(|t| RequestCounts {
                    total: t.total,
                    failed: t.failed,
                })
            });
            let Some(counts) = counts else {
                out.push(check(
                    &format!("{name}.exists"),
                    false,
                    "counters available".into(),
                    "no counters".into(),
                ));
                continue;
            };
            if let Some(min) = a.min_requests {
                out.push(check(
                    &format!("{name}.min_requests"),
                    counts.total >= min,
                    format!(">= {min}"),
                    counts.total.to_string(),
                ));
            }
            if let Some(max) = a.max_requests {
                out.push(check(
                    &format!("{name}.max_requests"),
                    counts.total <= max,
                    format!("<= {max}"),
                    counts.total.to_string(),
                ));
            }
            if let Some(max) = a.max_failed {
                out.push(check(
                    &format!("{name}.max_failed"),
                    counts.failed <= max,
                    format!("<= {max}"),
                    counts.failed.to_string(),
                ));
            }
        }
        out
    }
}

fn check(name: &str, passed: bool, expected: String, actual: String) -> AssertionResult {
    AssertionResult {
        name: name.to_owned(),
        passed,
        expected,
        actual,
    }
}
