//! Configuration: `configs/sandboxd/base.toml` plus one environment file, with
//! flags and `SANDBOXD_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/sandboxd";

/// The whole configuration. Every key lives in `base.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// `[server]`
    pub server: Server,
    /// `[limits]`
    pub limits: Limits,
    /// `[images]`
    pub images: Images,
}

/// `[server]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    /// Where the API listens. Only the address the platform's cluster reaches
    /// the machine on; never a public one.
    pub listen: SocketAddr,
    /// Prometheus `/metrics` listener; `None` turns it off.
    #[serde(default)]
    pub metrics: Option<SocketAddr>,
}

/// `[limits]`: every bound of a run. The container's own limits are here too,
/// so the recipe has no number of its own.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Limits {
    /// Largest source, bytes.
    pub max_source_bytes: usize,
    /// Largest input, bytes.
    pub max_stdin_bytes: usize,
    /// Output kept per stream, bytes; past it the run is stopped.
    pub max_output_bytes: usize,
    /// The compiler's deadline.
    #[serde(with = "humantime_serde")]
    pub compile_timeout: Duration,
    /// The program's deadline.
    #[serde(with = "humantime_serde")]
    pub run_timeout: Duration,
    /// Runs at once; one more is refused as busy.
    pub max_concurrent: usize,
    /// The container's memory, swap included (`docker --memory`), e.g. `512m`.
    pub memory: String,
    /// The container's processors (`docker --cpus`).
    pub cpus: String,
    /// Processes and threads in the container.
    pub pids: u32,
    /// The scratch space's size (`/work`, in memory), e.g. `256m`.
    pub scratch: String,
    /// Largest file the program may write, bytes.
    pub max_file_bytes: u64,
}

/// `[images]`: the toolchain image per language, built by `mise run
/// sandbox:images` and never pulled at run time.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Images {
    /// Go.
    pub go: String,
    /// Rust.
    pub rust: String,
    /// The Docker runtime every sandbox runs under.
    pub runtime: String,
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
    #[arg(long, env = "SANDBOXD_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Where the API listens. Default: `[server] listen`.
    #[arg(long, env = "SANDBOXD_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// A file holding the token callers must present (systemd's credential
    /// directory in deployment). Required to serve.
    #[arg(long, env = "SANDBOXD_TOKEN_FILE")]
    pub token_file: Option<PathBuf>,
}

impl Overrides {
    /// Apply the flags that were given.
    pub fn apply(&self, config: &mut Config) {
        if let Some(v) = self.listen_addr {
            config.server.listen = v;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_shipped_env_file_loads() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/sandboxd");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.files.len(), 2);
            assert!(config.limits.run_timeout < config.limits.compile_timeout);
            assert_eq!(
                config.images.runtime, "runsc",
                "{env}: the sandbox runs under gVisor"
            );
        }
    }
}
