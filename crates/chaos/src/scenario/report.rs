//! Text rendering of scenario results. JSON is `serde_json` on the same data.

use std::fmt::Write as _;

use super::ScenarioResult;

/// Render one result.
pub fn render(r: &ScenarioResult) -> String {
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
    for (name, c) in &r.services {
        let _ = writeln!(
            out,
            "      service   {name:<18} {:>7} served {:>6} failed",
            c.total, c.failed
        );
    }
    for e in &r.events {
        let mark = if e.error.is_some() { "!" } else { " " };
        let _ = writeln!(
            out,
            "      event  {mark}  {:>6.2}s  {}{}",
            e.at_s,
            e.action,
            e.error
                .as_ref()
                .map(|x| format!("  ({x})"))
                .unwrap_or_default()
        );
    }
    for a in &r.assertions {
        let mark = if a.passed { "ok " } else { "FAIL" };
        let _ = writeln!(
            out,
            "      {mark} {:<28} expected {:<14} actual {}",
            a.name, a.expected, a.actual
        );
    }
    out
}

/// Render a summary line for many results.
pub fn summary(results: &[ScenarioResult]) -> String {
    let passed = results.iter().filter(|r| r.passed && !r.skipped).count();
    let skipped = results.iter().filter(|r| r.skipped).count();
    let failed = results.len() - passed - skipped;
    format!("{passed} passed, {failed} failed, {skipped} skipped")
}
