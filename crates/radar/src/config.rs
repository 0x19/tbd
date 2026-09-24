//! Configuration: `configs/radar/base.toml` plus one environment file, with
//! flags and `RADAR_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/radar";

/// The whole configuration. Every key lives in `base.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// `[server]`
    pub server: Server,
    /// `[metrics]`
    #[serde(default)]
    pub metrics: Metrics,
    /// `[ping]`
    #[serde(default)]
    pub ping: Ping,
    /// `[store]`
    #[serde(default)]
    pub store: Store,
    /// `[llm]`
    #[serde(default)]
    pub llm: Llm,
    /// `[fetch]`
    #[serde(default)]
    pub fetch: Fetch,
    /// `[digest]`
    #[serde(default)]
    pub digest: Digest,
    /// `[[sources]]`
    #[serde(default)]
    pub sources: Vec<SourceSpec>,
}

/// `[server]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    /// Address the gRPC server binds.
    pub listen: SocketAddr,
}

/// `[metrics]`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Metrics {
    /// Prometheus `/metrics` listener. `None` in embedded use.
    #[serde(default)]
    pub listen: Option<SocketAddr>,
}

/// `[ping]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ping {
    /// Longest message `Ping` echoes; longer is `INVALID_ARGUMENT`.
    pub max_message_len: usize,
}

impl Default for Ping {
    fn default() -> Self {
        Self {
            max_message_len: 1024,
        }
    }
}

/// `[store]`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    /// Connection URL. Environment only (`RADAR_DATABASE_URL`); never in a file
    /// and never printed. Empty means no store: `Ping` answers, every other RPC
    /// is `UNAVAILABLE`, and the timers do nothing.
    #[serde(skip_serializing, default)]
    pub url: String,
    /// Pool size.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

const fn default_max_connections() -> u32 {
    3
}

/// `[llm]`: the model service that writes the digests, reached through
/// Envoy's internal listener in a deployment. Empty `url`: no digests.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Llm {
    /// gRPC URL of the llm service.
    #[serde(default)]
    pub url: String,
    /// `"fast"` or `"deep"`.
    pub tier: String,
    /// Longest answer asked for, in tokens.
    pub max_tokens: u32,
    /// Sampling temperature.
    pub temperature: f32,
    /// How long one digest may take, seconds.
    pub timeout_secs: u64,
}

impl Default for Llm {
    fn default() -> Self {
        Self {
            url: String::new(),
            tier: "deep".to_owned(),
            max_tokens: 2048,
            temperature: 0.3,
            timeout_secs: 900,
        }
    }
}

/// `[fetch]`: how often the sources are read.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Fetch {
    /// Seconds between two reads of every source. 0 turns the timer off
    /// (Refresh still works).
    pub interval_secs: u64,
    /// Seconds to wait after start before the first read.
    pub initial_delay_secs: u64,
    /// Whole-request deadline for one source, seconds.
    pub timeout_secs: u64,
    /// Items kept from one source per read, newest first.
    pub max_items_per_source: usize,
    /// Longest summary kept, in characters.
    pub max_summary_chars: usize,
    /// GitHub token for the search sources, from `RADAR_GITHUB_TOKEN` only;
    /// empty reads unauthenticated (60 requests an hour, plenty for a timer).
    #[serde(skip_serializing, default)]
    pub github_token: String,
}

impl Default for Fetch {
    fn default() -> Self {
        Self {
            interval_secs: 6 * 3600,
            initial_delay_secs: 60,
            timeout_secs: 30,
            max_items_per_source: 30,
            max_summary_chars: 600,
            github_token: String::new(),
        }
    }
}

/// `[digest]`: when the weekly digests are written.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Digest {
    /// Write on this weekday (ISO: 1 Monday … 7 Sunday), UTC.
    pub weekday: u32,
    /// Write at or after this hour, UTC.
    pub hour: u32,
    /// Seconds between two checks of whether it is time. 0 turns the schedule
    /// off (`RunDigest` still works).
    pub check_secs: u64,
    /// The reader languages written, e.g. `["en", "hr"]`.
    pub langs: Vec<String>,
    /// Most items given to the model for one digest, newest first.
    pub max_items: usize,
    /// Default page size for the list RPCs.
    pub page_size: u32,
}

impl Default for Digest {
    fn default() -> Self {
        Self {
            weekday: 1,
            hour: 6,
            check_secs: 900,
            langs: vec!["en".to_owned(), "hr".to_owned()],
            max_items: 40,
            page_size: 20,
        }
    }
}

/// One `[[sources]]` entry.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct SourceSpec {
    /// Stable name, stored with every item, e.g. `"go-blog"`.
    pub name: String,
    /// How to read it.
    pub kind: SourceKind,
    /// `"go"` or `"rust"`.
    pub language: String,
    /// Feed URL (`atom`, `rss`) or the GitHub search API URL (`github`).
    pub url: String,
    /// `github` only: the search query, where `{since}` becomes the date
    /// `lookback_days` ago (YYYY-MM-DD).
    #[serde(default)]
    pub query: String,
    /// `github` only: how far back the query looks.
    #[serde(default = "default_lookback")]
    pub lookback_days: u32,
}

const fn default_lookback() -> u32 {
    14
}

/// How a source is read.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum SourceKind {
    /// An Atom feed.
    Atom,
    /// An RSS 2.0 feed.
    Rss,
    /// GitHub's issue and pull-request search.
    Github,
}

/// Where a loaded configuration came from.
#[derive(Debug, Clone)]
pub struct Source {
    /// Environment name.
    pub env: String,
    /// Files merged, in order.
    pub files: Vec<PathBuf>,
}

impl Config {
    /// Load `dir/base.toml` + `dir/<env>.toml`.
    ///
    /// # Errors
    /// A file is missing or malformed, or a key is unknown.
    pub fn load(dir: &Path, env: &str) -> Result<(Self, Source), ConfigError> {
        let Loaded { value, env, files } = tbd_common::config::load::<Self>(dir, env)?;
        Ok((value, Source { env, files }))
    }

    /// Whether the configuration makes sense beyond parsing.
    ///
    /// # Errors
    /// A human-readable reason.
    pub fn validate(&self) -> Result<(), String> {
        if !matches!(self.llm.tier.as_str(), "fast" | "deep") {
            return Err(format!(
                "[llm] tier must be fast or deep, not {:?}",
                self.llm.tier
            ));
        }
        if !(1..=7).contains(&self.digest.weekday) || self.digest.hour > 23 {
            return Err("[digest] weekday is 1..=7 and hour 0..=23".to_owned());
        }
        for lang in &self.digest.langs {
            if !matches!(lang.as_str(), "en" | "hr") {
                return Err(format!("[digest] langs: {lang:?} is not en or hr"));
            }
        }
        let mut names = std::collections::HashSet::new();
        for s in &self.sources {
            if !names.insert(s.name.as_str()) {
                return Err(format!("[[sources]] name {:?} appears twice", s.name));
            }
            if !matches!(s.language.as_str(), "go" | "rust") {
                return Err(format!("source {}: language must be go or rust", s.name));
            }
            if s.kind == SourceKind::Github && s.query.is_empty() {
                return Err(format!("source {}: a github source needs a query", s.name));
            }
        }
        Ok(())
    }
}

/// Flags that override the loaded configuration. Every one has an environment
/// variable.
#[derive(Args, Debug, Clone)]
pub struct Overrides {
    /// Environment: picks `<config-dir>/<env>.toml` to merge over `base.toml`.
    #[arg(long, env = tbd_common::config::ENV_VAR, default_value = tbd_common::config::DEFAULT_ENV, global = true)]
    pub env: String,
    /// Directory holding `base.toml` and one file per environment.
    #[arg(long, env = "RADAR_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "RADAR_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "RADAR_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// The store's connection URL (the radar-db Secret in a deployment).
    #[arg(long, env = "RADAR_DATABASE_URL", hide_env_values = true)]
    pub database_url: Option<String>,
    /// The llm service's gRPC URL. Default: `[llm] url`.
    #[arg(long, env = "RADAR_LLM_URL")]
    pub llm_url: Option<String>,
    /// GitHub token for the search sources (the optional radar-github Secret).
    #[arg(long, env = "RADAR_GITHUB_TOKEN", hide_env_values = true)]
    pub github_token: Option<String>,
}

impl Overrides {
    /// Apply the flags that were given.
    pub fn apply(&self, config: &mut Config) {
        if let Some(v) = self.listen_addr {
            config.server.listen = v;
        }
        if let Some(v) = self.metrics_addr {
            config.metrics.listen = Some(v);
        }
        if let Some(v) = &self.database_url {
            config.store.url.clone_from(v);
        }
        if let Some(v) = &self.llm_url {
            config.llm.url.clone_from(v);
        }
        if let Some(v) = &self.github_token {
            config.fetch.github_token.clone_from(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shipped environment file loads over `base.toml` and validates;
    /// catches a key that `deny_unknown_fields` would reject at start.
    #[test]
    fn every_shipped_env_file_loads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/radar");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
            config.validate().unwrap_or_else(|e| panic!("{env}: {e}"));
            assert!(!config.sources.is_empty(), "{env}: no sources");
        }
    }
}
