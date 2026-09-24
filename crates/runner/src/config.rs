//! Configuration: `configs/runner/base.toml` plus one environment file, with
//! flags and `RUNNER_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/runner";

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
    /// `[engine]`
    pub engine: Engine,
    /// `[access]`
    pub access: Access,
    /// `[admission]`
    pub admission: Admission,
    /// `[budget]`
    pub budget: Budget,
    /// `[limits]`
    pub limits: Limits,
    /// The sandbox daemon's token, from `RUNNER_SANDBOX_TOKEN` only. Never
    /// read from a file and never written out: `runner config` cannot print it.
    #[serde(skip)]
    pub sandbox_token: Option<String>,
}

/// What runs the programs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineKind {
    /// The sandbox daemon on the machine (RFC 0010).
    Sandboxd,
    /// In process, runs nothing, says so (`stub: true`): tests and the chaos tool.
    Stub,
}

/// `[engine]`: never overridable from a flag, on purpose: a deployment's engine
/// is chosen in a reviewed file, and production refuses the stub.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Engine {
    /// What runs the programs.
    pub kind: EngineKind,
    /// The sandbox daemon's base URL (`RUNNER_SANDBOX_URL`), dialled directly
    /// like the model service's engines: it runs on the machine, not in the cluster.
    pub url: String,
    /// One run's deadline here, past the sandbox's own compile and run deadlines.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
}

/// `[access]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Access {
    /// The role a caller needs; empty lets any verified caller run (RFC 0006).
    pub require_role: String,
}

/// `[admission]`: runs at once, and a short line behind them.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Admission {
    /// Runs at once; at most the sandbox daemon's own `max_concurrent`.
    pub max_in_flight: usize,
    /// Runs that may wait; one more is refused at once.
    pub max_queued: usize,
    /// How long one waits before it is refused.
    #[serde(with = "humantime_serde")]
    pub queue_timeout: Duration,
}

/// `[budget]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    /// Runs one caller may make per UTC day; 0 is no limit. Counted in memory,
    /// so a restart of the service resets it (RFC 0010 says so).
    pub runs_per_day: u32,
}

/// `[limits]`: refused here, before the sandbox is asked.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Largest source, bytes.
    pub max_source_bytes: usize,
    /// Largest input, bytes.
    pub max_stdin_bytes: usize,
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
    #[arg(long, env = "RUNNER_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "RUNNER_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "RUNNER_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// The sandbox daemon's URL. Default: `[engine] url`.
    #[arg(long, env = "RUNNER_SANDBOX_URL")]
    pub sandbox_url: Option<String>,
    /// The token the sandbox daemon requires; a credential, so it comes only
    /// from the environment (the `runner-sandbox` Secret), never a file here.
    #[arg(long, env = "RUNNER_SANDBOX_TOKEN", hide_env_values = true)]
    pub sandbox_token: Option<String>,
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
        if let Some(v) = &self.sandbox_url {
            config.engine.url.clone_from(v);
        }
        config.sandbox_token.clone_from(&self.sandbox_token);
    }
}

impl Config {
    /// The in-process stub engine, no metrics listener, admins only: what tests
    /// and the chaos tool run, never a deployment.
    #[must_use]
    pub fn stub(listen: SocketAddr) -> Self {
        Self {
            server: Server { listen },
            metrics: Metrics { listen: None },
            ping: Ping::default(),
            engine: Engine {
                kind: EngineKind::Stub,
                url: String::new(),
                timeout: Duration::from_secs(10),
            },
            access: Access {
                require_role: "admin".to_owned(),
            },
            admission: Admission {
                max_in_flight: 4,
                max_queued: 8,
                queue_timeout: Duration::from_secs(10),
            },
            budget: Budget { runs_per_day: 200 },
            limits: Limits {
                max_source_bytes: 65536,
                max_stdin_bytes: 65536,
            },
            sandbox_token: None,
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
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/runner");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
        }
    }
}
