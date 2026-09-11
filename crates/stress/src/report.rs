//! What a campaign produces: a per-second snapshot while it runs, a result at
//! the end, and the text a terminal shows. JSON is `serde_json` on the same
//! data.

use std::{collections::BTreeMap, fmt::Write as _};

use serde::{Deserialize, Serialize};

use crate::{finding::Finding, metrics::LoadSnapshot};

/// How often an invariant was evaluated and how often it broke.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckCount {
    /// Evaluations that held.
    pub passed: u64,
    /// Evaluations that broke.
    pub violated: u64,
}

/// A point-in-time view of the checks, once a second.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct StressSnapshot {
    /// Seconds since the measured phase began.
    pub elapsed_s: f64,
    /// `warmup`, `run`, `shrink`, `done`.
    pub phase: String,
    /// Requests sent.
    pub ops_total: u64,
    /// Requests that failed.
    pub ops_failed: u64,
    /// Failures the campaign tolerates (faults being injected).
    pub tolerated: u64,
    /// Writes re-driven to a known outcome after a tolerated failure.
    pub redriven: u64,
    /// Per invariant.
    pub checks: BTreeMap<String, CheckCount>,
    /// Findings so far.
    pub findings: u64,
    /// Subjects the workers own.
    pub subjects: u64,
    /// Workers per class.
    pub workers: BTreeMap<String, u32>,
}

/// The result of one campaign.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CampaignResult {
    /// From `[campaign] name`.
    pub name: String,
    /// Source file, when run from one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    /// No findings, no error.
    pub passed: bool,
    /// `skip = true`; nothing ran.
    pub skipped: bool,
    /// Wall time.
    pub duration_s: f64,
    /// The store the targets reported on ping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub store: Option<String>,
    /// Target names.
    pub targets: Vec<String>,
    /// The load numbers of the measured phase.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub load: Option<LoadSnapshot>,
    /// Per invariant.
    pub checks: BTreeMap<String, CheckCount>,
    /// Failures the campaign tolerated.
    pub tolerated: u64,
    /// Writes re-driven.
    pub redriven: u64,
    /// Every finding.
    pub findings: Vec<Finding>,
    /// `[stop] max_findings` was reached.
    pub stopped_early: bool,
    /// Failure outside the checks: a target unreachable, a task that panicked.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl CampaignResult {
    /// Total checks evaluated.
    #[must_use]
    pub fn checks_total(&self) -> u64 {
        self.checks.values().map(|c| c.passed + c.violated).sum()
    }
}

/// Render one result.
#[must_use]
pub fn render(r: &CampaignResult) -> String {
    let mut out = String::new();
    let status = if r.skipped {
        "SKIP"
    } else if r.passed {
        "PASS"
    } else {
        "FAIL"
    };
    let _ = writeln!(out, "{status}  {}  ({:.1}s)", r.name, r.duration_s);
    if let Some(f) = &r.file {
        let _ = writeln!(out, "      file      {f}");
    }
    if let Some(e) = &r.error {
        let _ = writeln!(out, "      error     {e}");
    }
    if !r.targets.is_empty() {
        let _ = writeln!(
            out,
            "      target    {}{}",
            r.targets.join(", "),
            r.store
                .as_deref()
                .map(|s| format!("  (store {s})"))
                .unwrap_or_default()
        );
    }
    if let Some(l) = &r.load {
        let _ = writeln!(
            out,
            "      load      {} req, {:.2}% errors, {:.1} rps, p50 {:.1} ms, p99 {:.1} ms, max {:.1} ms",
            l.requests_total,
            l.error_rate * 100.0,
            l.throughput_rps,
            l.latency.p50_ms,
            l.latency.p99_ms,
            l.latency.max_ms
        );
        for (op, c) in &l.per_op {
            let _ = writeln!(
                out,
                "      op        {op:<18} {:>7} sent {:>6} failed   p50 {:>6.1} ms  p99 {:>6.1} ms  max {:>6.1} ms",
                c.total, c.failed, c.latency.p50_ms, c.latency.p99_ms, c.latency.max_ms
            );
        }
        for (class, n) in &l.errors {
            let _ = writeln!(out, "      error     {class:<18} {n:>7}");
        }
    }
    if r.tolerated > 0 || r.redriven > 0 {
        let _ = writeln!(
            out,
            "      faults    {} tolerated, {} writes re-driven",
            r.tolerated, r.redriven
        );
    }
    for (name, c) in &r.checks {
        let mark = if c.violated > 0 { "!" } else { " " };
        let _ = writeln!(
            out,
            "      check  {mark}  {name:<32} {:>8} passed {:>6} violated",
            c.passed, c.violated
        );
    }
    for f in &r.findings {
        let _ = writeln!(
            out,
            "      finding   {}  {}  ({} steps{}) subject {}",
            f.invariant,
            f.message,
            f.trace.len(),
            if f.shrunk { ", shrunk" } else { "" },
            f.subject
        );
    }
    if r.stopped_early {
        let _ = writeln!(out, "      stopped   early: [stop] max_findings reached");
    }
    out
}

/// One line for a directory run.
#[must_use]
pub fn summary(results: &[CampaignResult]) -> String {
    let passed = results.iter().filter(|r| r.passed && !r.skipped).count();
    let skipped = results.iter().filter(|r| r.skipped).count();
    let failed = results.len() - passed - skipped;
    let findings: usize = results.iter().map(|r| r.findings.len()).sum();
    format!("{passed} passed, {failed} failed, {skipped} skipped, {findings} findings")
}
