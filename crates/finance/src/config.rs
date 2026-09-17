//! Configuration: `configs/finance/base.toml` plus one environment file, with
//! flags and `FINANCE_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/finance";

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
}

/// `[store]`
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    /// Connection URL. Environment only (`FINANCE_DATABASE_URL`); never in a
    /// file and never printed. Empty means no store: the service serves `Ping`
    /// and answers every data RPC `UNAVAILABLE`, which is what a scaffolded
    /// deployment does before its database exists.
    #[serde(skip_serializing)]
    #[serde(default)]
    pub url: String,
    /// Pool size.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

const fn default_max_connections() -> u32 {
    10
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
    #[arg(long, env = "FINANCE_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "FINANCE_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "FINANCE_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// Postgres URL. A credential, so it is never echoed.
    #[arg(long, env = "FINANCE_DATABASE_URL", hide_env_values = true)]
    pub database_url: Option<String>,
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shipped environment file loads over `base.toml`; catches a key
    /// that `deny_unknown_fields` would reject at start.
    #[test]
    fn every_shipped_env_file_loads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/finance");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
        }
    }
}
