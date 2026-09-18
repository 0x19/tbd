//! Configuration: `configs/playground/base.toml` plus one environment file, with
//! flags and `PLAYGROUND_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/playground";

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
    /// `[game]`
    #[serde(default)]
    pub game: Game,
    /// `[sandbox]`
    #[serde(default)]
    pub sandbox: Sandbox,
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

/// `[game]`
///
/// The rules. Everything a player can influence is a move; these are the
/// numbers behind the moves, and they are set here rather than on the wire.
#[derive(Debug, Clone, Copy, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Game {
    /// The success rate the sandbox is meant to hold, 0 to 1.
    pub target: f64,
    /// How many seconds of traffic the objective is judged over.
    pub window_seconds: u32,
    /// The p99 the sandbox is meant to stay under, in milliseconds. The other
    /// half of the objective: breaking either clause breaks it.
    pub latency_target_ms: f64,
    /// The most budget the world ever holds.
    pub max_tokens: u32,
    /// Seconds per token earned back.
    pub refill_seconds: u32,
    /// Requests per second of synthetic traffic.
    pub rate: f64,
}

impl Default for Game {
    fn default() -> Self {
        Self {
            target: 0.99,
            window_seconds: 30,
            latency_target_ms: 250.0,
            max_tokens: 10,
            refill_seconds: 3,
            rate: 20.0,
        }
    }
}

/// `[sandbox]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sandbox {
    /// Where the scoreboard is kept.
    pub scores: PathBuf,
    /// How many breaches to remember.
    pub keep_scores: usize,
    /// The most `Watch` streams served at once; beyond it, callers poll.
    pub max_watchers: usize,
}

impl Default for Sandbox {
    fn default() -> Self {
        Self {
            scores: PathBuf::from(".playground/scores.json"),
            keep_scores: 20,
            max_watchers: 200,
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
    #[arg(long, env = "PLAYGROUND_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "PLAYGROUND_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "PLAYGROUND_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// Where the scoreboard is kept. Default: `[sandbox] scores`. A container
    /// points this at its volume, because its root filesystem is read-only.
    #[arg(long, env = "PLAYGROUND_SCORES")]
    pub scores: Option<PathBuf>,
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
        if let Some(v) = &self.scores {
            config.sandbox.scores.clone_from(v);
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
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/playground");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
        }
    }
}
