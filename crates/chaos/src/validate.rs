//! `chaos validate`: hit every surface of a running stack and report per check.
//!
//! Checks belong to the kinds ([`crate::kinds`]): each kind with a target
//! lists its checks, and [`checks`] walks the registry in order. Checks run
//! concurrently, each with its own timeout, each against the [`Endpoint`] of
//! its kind.

use std::{
    collections::BTreeMap,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::{
    config::ChaosConfig,
    kinds::{self, Kind},
    stack::Stack,
    tls::{Grpc, Trust, Ws},
};

/// What one check receives: the target URL of its kind and the trust to use.
#[derive(Debug, Clone)]
pub struct Endpoint {
    /// `http://host:port` or `https://...`.
    pub url: String,
    /// Roots and bearer token.
    pub trust: Trust,
}

impl Endpoint {
    /// An HTTP client.
    #[must_use]
    pub fn http(&self) -> reqwest::Client {
        self.trust.http(None)
    }

    /// A gRPC channel to the URL.
    ///
    /// # Errors
    /// The URL is invalid.
    pub fn grpc(&self) -> Result<Grpc, String> {
        self.trust.grpc(&self.url, None).map_err(|e| e.to_string())
    }

    /// `ws://host:port<path>`.
    #[must_use]
    pub fn ws_url(&self, path: &str) -> String {
        self.url.replacen("http", "ws", 1) + path
    }

    /// A WebSocket at `path`.
    ///
    /// # Errors
    /// The handshake fails.
    pub async fn connect_ws(&self, path: &str) -> Result<Ws, String> {
        self.trust
            .connect_ws(&self.ws_url(path))
            .await
            .map_err(|e| e.to_string())
    }
}

/// The body of a check.
pub type CheckFn = fn(Endpoint) -> futures::future::BoxFuture<'static, Result<String, String>>;

/// One check of a kind.
pub struct Check {
    /// Name, unique across kinds.
    pub name: &'static str,
    /// Surface it exercises.
    pub surface: &'static str,
    /// When it passes, for the docs.
    pub doc: &'static str,
    /// The check.
    pub run: CheckFn,
}

/// Where to point the checks: one URL per kind with a target.
#[derive(Debug, Clone)]
pub struct Targets {
    /// URL by kind name.
    pub urls: BTreeMap<String, String>,
    /// Per-check timeout.
    pub timeout: Duration,
    /// Roots and bearer token for `https://` / `wss://` targets.
    pub trust: Trust,
}

impl Targets {
    /// The URL for a kind.
    #[must_use]
    pub fn url(&self, kind: &Kind) -> Option<&str> {
        self.urls.get(kind.name).map(String::as_str)
    }

    /// The first running instance of every kind with a target, default trust.
    /// For in-process stacks and tests.
    #[must_use]
    pub fn of_stack(stack: &Stack, timeout: Duration) -> Self {
        let mut urls = BTreeMap::new();
        for (kind, _) in kinds::with_target() {
            if let Some(i) = stack.of_kind(kind.name).first() {
                urls.insert(kind.name.to_owned(), i.http_url());
            }
        }
        Self {
            urls,
            timeout,
            trust: Trust::default(),
        }
    }

    /// `[targets]` with per-kind overrides (`POST /validate`, the CLI).
    ///
    /// # Errors
    /// An override names a kind without a target, a URL does not parse, or
    /// the trust cannot be built.
    pub fn from_config(
        config: &ChaosConfig,
        overrides: &BTreeMap<String, Option<String>>,
        timeout: Option<Duration>,
    ) -> anyhow::Result<Self> {
        let mut urls: BTreeMap<String, String> = config
            .targets
            .resolved()
            .into_iter()
            .map(|(k, v)| (k.to_owned(), v))
            .collect();
        for (kind, url) in overrides {
            let Some(url) = url else { continue };
            anyhow::ensure!(
                kinds::by_name(kind).is_some_and(|k| k.target.is_some()),
                "{kind}: not a kind with a validate target; kinds with a target: {}",
                kinds::with_target()
                    .map(|(k, _)| k.name)
                    .collect::<Vec<_>>()
                    .join(", ")
            );
            url::Url::parse(url).map_err(|e| anyhow::anyhow!("{kind}: {e}"))?;
            urls.insert(kind.clone(), url.clone());
        }
        Ok(Self {
            urls,
            timeout: timeout.unwrap_or(config.validate.timeout),
            trust: config.trust()?,
        })
    }
}

/// Outcome of one check.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CheckResult {
    /// Check name.
    pub name: String,
    /// Surface it exercises.
    pub surface: String,
    /// Passed.
    pub passed: bool,
    /// Wall time.
    pub latency_ms: f64,
    /// What was observed, or the error.
    pub detail: String,
}

/// Whole-run report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Report {
    /// Every check.
    pub checks: Vec<CheckResult>,
    /// Count of passed checks.
    pub passed: usize,
    /// Count of failed checks.
    pub failed: usize,
}

impl Report {
    /// True when nothing failed.
    #[must_use]
    pub fn ok(&self) -> bool {
        self.failed == 0
    }

    /// Human-readable rendering.
    #[must_use]
    pub fn render(&self) -> String {
        use std::fmt::Write as _;
        let mut out = String::new();
        for c in &self.checks {
            let mark = if c.passed { "PASS" } else { "FAIL" };
            let _ = writeln!(
                out,
                "{mark}  {:<22} {:<8} {:>8.1} ms  {}",
                c.name, c.surface, c.latency_ms, c.detail
            );
        }
        let _ = writeln!(out, "\n{} passed, {} failed", self.passed, self.failed);
        out
    }
}

/// Every check with its kind, registry order then declaration order.
pub fn checks() -> impl Iterator<Item = (&'static Kind, &'static Check)> {
    kinds::ALL
        .iter()
        .copied()
        .flat_map(|k| k.checks.iter().map(move |c| (k, c)))
}

/// A check as the API sees it.
#[derive(Debug, Clone, Serialize)]
pub struct CheckInfo {
    /// Name.
    pub name: &'static str,
    /// Surface.
    pub surface: &'static str,
    /// The kind it targets.
    pub kind: &'static str,
}

/// The catalogue of checks.
#[must_use]
pub fn catalogue() -> Vec<CheckInfo> {
    checks()
        .map(|(k, c)| CheckInfo {
            name: c.name,
            surface: c.surface,
            kind: k.name,
        })
        .collect()
}

/// Run every check concurrently.
pub async fn run(mut targets: Targets) -> Report {
    // One token for the whole run; a failure to get it fails everything at once.
    match targets.trust.snapshot().await {
        Ok(trust) => targets.trust = trust,
        Err(error) => {
            return Report {
                checks: vec![CheckResult {
                    name: "auth_token".into(),
                    surface: "auth".into(),
                    passed: false,
                    latency_ms: 0.0,
                    detail: format!("{error:#}"),
                }],
                passed: 0,
                failed: 1,
            };
        }
    }
    let mut set = tokio::task::JoinSet::new();
    let mut results: Vec<(usize, CheckResult)> = Vec::new();
    for (idx, (kind, check)) in checks().enumerate() {
        let Some(url) = targets.url(kind) else {
            results.push((
                idx,
                CheckResult {
                    name: check.name.to_owned(),
                    surface: check.surface.to_owned(),
                    passed: false,
                    latency_ms: 0.0,
                    detail: format!("no target URL for kind `{}`", kind.name),
                },
            ));
            continue;
        };
        let endpoint = Endpoint {
            url: url.to_owned(),
            trust: targets.trust.clone(),
        };
        let timeout = targets.timeout;
        let run = check.run;
        let (name, surface) = (check.name, check.surface);
        set.spawn(async move {
            let started = Instant::now();
            let outcome = tokio::time::timeout(timeout, run(endpoint)).await;
            let latency_ms = started.elapsed().as_secs_f64() * 1000.0;
            let (passed, detail) = match outcome {
                Ok(Ok(detail)) => (true, detail),
                Ok(Err(detail)) => (false, detail),
                Err(_) => (false, format!("timed out after {timeout:?}")),
            };
            (
                idx,
                CheckResult {
                    name: name.to_owned(),
                    surface: surface.to_owned(),
                    passed,
                    latency_ms,
                    detail,
                },
            )
        });
    }
    while let Some(joined) = set.join_next().await {
        match joined {
            Ok(r) => results.push(r),
            Err(error) => tracing::error!(%error, "check task panicked"),
        }
    }
    results.sort_by_key(|(i, _)| *i);
    let checks: Vec<CheckResult> = results.into_iter().map(|(_, c)| c).collect();
    let passed = checks.iter().filter(|c| c.passed).count();
    let failed = checks.len() - passed;
    Report {
        checks,
        passed,
        failed,
    }
}
