//! Configuration: `configs/arena/base.toml` plus one environment file, with
//! flags and `ARENA_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/arena";

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
    /// `[sources]`
    #[serde(default)]
    pub sources: Sources,
    /// `[collect]`
    #[serde(default)]
    pub collect: Collect,
    /// `[watch]`
    #[serde(default)]
    pub watch: Watch,
}

/// `[sources]`: where the snapshot's figures come from. An empty URL turns
/// that source off, and the figures it would give are absent and say so.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sources {
    /// The model service (gRPC), through Envoy's internal listener when deployed.
    pub llm_url: String,
    /// The metrics store's Prometheus query API (`/api/v1/query`), for rates
    /// and percentiles. Observability, dialled directly like a database.
    pub metrics_url: String,
    /// The chaos tool's HTTP API root (`.../api/chaos/v1`), for its runs and
    /// its last end-to-end check. The operator's tool, dialled directly.
    pub chaos_url: String,
    /// A chaos schedule, created once if missing, that checks every way in
    /// end to end without posting to Slack (six-field cron, UTC). Empty: the
    /// arena only reads whatever check ran last.
    pub validate_cron: String,
}

impl Default for Sources {
    fn default() -> Self {
        Self {
            llm_url: "http://127.0.0.1:50057".to_owned(),
            metrics_url: String::new(),
            chaos_url: String::new(),
            validate_cron: "0 */2 * * * *".to_owned(),
        }
    }
}

/// `[collect]`: how often each source is read.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Collect {
    /// The model service's tiers.
    #[serde(with = "humantime_serde")]
    pub llm_every: Duration,
    /// The metrics store's rates.
    #[serde(with = "humantime_serde")]
    pub metrics_every: Duration,
    /// The chaos tool's overview (a running run is then followed live).
    #[serde(with = "humantime_serde")]
    pub chaos_every: Duration,
    /// One read's deadline, for every source.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}

impl Default for Collect {
    fn default() -> Self {
        Self {
            llm_every: Duration::from_secs(1),
            metrics_every: Duration::from_secs(5),
            chaos_every: Duration::from_secs(5),
            timeout: Duration::from_secs(3),
        }
    }
}

/// `[watch]`: who may watch, and how many at once.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Watch {
    /// A snapshot a second.
    #[serde(with = "humantime_serde")]
    pub tick: Duration,
    /// Streams open at once; one more is `RESOURCE_EXHAUSTED`.
    pub max_viewers: usize,
    /// The role a caller needs to read the arena. `admin` while the lab is
    /// private; empty opens it to any caller at publication (RFC 0006).
    pub require_role: String,
}

impl Default for Watch {
    fn default() -> Self {
        Self {
            tick: Duration::from_secs(1),
            max_viewers: 64,
            require_role: "admin".to_owned(),
        }
    }
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
}

/// Flags that override the loaded configuration. Every one has an environment
/// variable.
#[derive(Args, Debug, Clone)]
pub struct Overrides {
    /// Environment: picks `<config-dir>/<env>.toml` to merge over `base.toml`.
    #[arg(long, env = tbd_common::config::ENV_VAR, default_value = tbd_common::config::DEFAULT_ENV, global = true)]
    pub env: String,
    /// Directory holding `base.toml` and one file per environment.
    #[arg(long, env = "ARENA_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "ARENA_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "ARENA_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// The model service. Default: `[sources] llm_url`.
    #[arg(long, env = "ARENA_LLM_URL")]
    pub llm_url: Option<String>,
    /// The metrics store's query API. Default: `[sources] metrics_url`.
    #[arg(long, env = "ARENA_METRICS_URL")]
    pub metrics_url: Option<String>,
    /// The chaos tool's API root. Default: `[sources] chaos_url`.
    #[arg(long, env = "ARENA_CHAOS_URL")]
    pub chaos_url: Option<String>,
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
        if let Some(v) = &self.llm_url {
            config.sources.llm_url.clone_from(v);
        }
        if let Some(v) = &self.metrics_url {
            config.sources.metrics_url.clone_from(v);
        }
        if let Some(v) = &self.chaos_url {
            config.sources.chaos_url.clone_from(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shipped environment file loads over `base.toml`; catches a key
    /// that `deny_unknown_fields` would reject at start.
    #[test]
    fn every_shipped_env_file_loads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/arena");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
        }
    }
}
