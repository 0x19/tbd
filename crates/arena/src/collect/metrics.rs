//! The metrics store's rates: a handful of `PromQL` instant queries once a
//! `[collect] metrics_every`, answered per tier. A query with no data (no
//! generation in the window, a histogram with nothing in it) is absent, never
//! zero: the page says "no traffic", not "0 tok/s".

use std::{collections::BTreeMap, time::Duration};

use serde::Deserialize;

use crate::world::{Rates, RunnerRates, World};

/// The queries, each answered per `tier`. Windows are wide enough to hold a
/// few generations on a quiet day and short enough to move while one runs.
pub const TOKENS_PER_SECOND: &str =
    r#"sum by (tier) (rate(tbd_llm_tokens_total{kind="completion"}[1m]))"#;
/// Median time to the first token, seconds.
pub const TTFT_P50: &str = "histogram_quantile(0.5, sum by (le, tier) (rate(tbd_llm_time_to_first_token_seconds_bucket[5m])))";
/// 99th percentile time to the first token, seconds.
pub const TTFT_P99: &str = "histogram_quantile(0.99, sum by (le, tier) (rate(tbd_llm_time_to_first_token_seconds_bucket[5m])))";
/// Refusals by admission in the last minute.
pub const REFUSED: &str = "sum by (tier) (increase(tbd_llm_refused_total[1m]))";

/// The sandbox runner's, over five minutes (runs are rarer than tokens):
/// runs a minute.
pub const RUNNER_RUNS: &str = "sum(rate(tbd_runner_runs_total[5m])) * 60";
/// Runs a minute the sandbox did not answer.
pub const RUNNER_UNAVAILABLE: &str =
    r#"sum(rate(tbd_runner_runs_total{outcome="unavailable"}[5m])) * 60"#;
/// A whole run, median, seconds.
pub const RUNNER_P50: &str =
    "histogram_quantile(0.5, sum by (le) (rate(tbd_runner_duration_seconds_bucket[5m])))";
/// A whole run, 99th percentile, seconds.
pub const RUNNER_P99: &str =
    "histogram_quantile(0.99, sum by (le) (rate(tbd_runner_duration_seconds_bucket[5m])))";

#[derive(Debug, Deserialize)]
struct Response {
    status: String,
    #[serde(default)]
    data: Option<Data>,
    #[serde(default)]
    error: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Data {
    #[serde(default)]
    result: Vec<Series>,
}

#[derive(Debug, Deserialize)]
struct Series {
    #[serde(default)]
    metric: BTreeMap<String, String>,
    /// `[unix seconds, "value"]`.
    value: (f64, String),
}

/// Read the rates forever.
pub fn start(
    url: &str,
    every: Duration,
    timeout: Duration,
    world: World,
) -> tokio::task::JoinHandle<()> {
    let base = url.trim_end_matches('/').to_owned();
    let http = reqwest::Client::builder()
        .timeout(timeout)
        .build()
        .unwrap_or_default();
    tokio::spawn(async move {
        let mut tick = tokio::time::interval(every);
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            tick.tick().await;
            match read(&http, &base).await {
                Ok((rates, runner)) => {
                    world.set_rates(rates, runner);
                    world.source_ok("metrics");
                }
                Err(error) => world.source_failed("metrics", error),
            }
        }
    })
}

async fn read(
    http: &reqwest::Client,
    base: &str,
) -> Result<(BTreeMap<String, Rates>, RunnerRates), String> {
    let (tps, p50, p99, refused) = tokio::try_join!(
        query(http, base, TOKENS_PER_SECOND),
        query(http, base, TTFT_P50),
        query(http, base, TTFT_P99),
        query(http, base, REFUSED),
    )?;
    let (runs, unavailable, run_p50, run_p99) = tokio::try_join!(
        series(http, base, RUNNER_RUNS),
        series(http, base, RUNNER_UNAVAILABLE),
        series(http, base, RUNNER_P50),
        series(http, base, RUNNER_P99),
    )?;
    let runner = RunnerRates {
        runs_per_minute: scalar(runs),
        unavailable_per_minute: scalar(unavailable),
        p50_ms: scalar(run_p50).map(|v| v * 1000.0),
        p99_ms: scalar(run_p99).map(|v| v * 1000.0),
    };
    let mut rates: BTreeMap<String, Rates> = BTreeMap::new();
    for (tier, v) in tps {
        rates.entry(tier).or_default().tokens_per_second = Some(v);
    }
    for (tier, v) in p50 {
        rates.entry(tier).or_default().ttft_p50_ms = Some(v * 1000.0);
    }
    for (tier, v) in p99 {
        rates.entry(tier).or_default().ttft_p99_ms = Some(v * 1000.0);
    }
    for (tier, v) in refused {
        rates.entry(tier).or_default().refused_per_minute = Some(v);
    }
    Ok((rates, runner))
}

/// One instant query, as tier to value; a value that is not a finite number
/// (a histogram with nothing in it answers `NaN`) is left out.
async fn query(
    http: &reqwest::Client,
    base: &str,
    promql: &str,
) -> Result<Vec<(String, f64)>, String> {
    Ok(parse(series(http, base, promql).await?))
}

/// One instant query's series, as the store answered them.
async fn series(http: &reqwest::Client, base: &str, promql: &str) -> Result<Vec<Series>, String> {
    let resp = http
        .get(format!("{base}/api/v1/query"))
        .query(&[("query", promql)])
        .send()
        .await
        .map_err(|e| format!("query: {e}"))?;
    let status = resp.status();
    let body: Response = resp
        .json()
        .await
        .map_err(|e| format!("query answered {status}: {e}"))?;
    if body.status != "success" {
        return Err(format!("query: {}", body.error.unwrap_or(body.status)));
    }
    Ok(body.data.map(|d| d.result).unwrap_or_default())
}

/// A query summed to one series: its value, when it is a finite number. No
/// series (nothing in the window) and `NaN` (an empty histogram) are absent.
fn scalar(series: Vec<Series>) -> Option<f64> {
    let s = series.into_iter().next()?;
    let v: f64 = s.value.1.parse().ok()?;
    v.is_finite().then_some(v)
}

fn parse(series: Vec<Series>) -> Vec<(String, f64)> {
    series
        .into_iter()
        .filter_map(|s| {
            let tier = s.metric.get("tier")?.clone();
            let v: f64 = s.value.1.parse().ok()?;
            v.is_finite().then_some((tier, v))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_nan_is_absent_and_a_number_is_kept() {
        let series: Vec<Series> = serde_json::from_value(serde_json::json!([
            {"metric": {"tier": "fast"}, "value": [1.0, "12.5"]},
            {"metric": {"tier": "deep"}, "value": [1.0, "NaN"]},
            {"metric": {}, "value": [1.0, "3"]}
        ]))
        .unwrap();
        assert_eq!(parse(series), vec![("fast".to_owned(), 12.5)]);
    }

    #[test]
    fn a_summed_series_is_its_value_and_nan_or_nothing_is_absent() {
        let one = |v: &str| -> Vec<Series> {
            serde_json::from_value(serde_json::json!([{"metric": {}, "value": [1.0, v]}])).unwrap()
        };
        assert_eq!(scalar(one("2.5")), Some(2.5));
        assert_eq!(scalar(one("NaN")), None);
        assert_eq!(scalar(Vec::new()), None);
    }
}
