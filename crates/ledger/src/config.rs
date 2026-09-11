//! Configuration: `configs/ledger/base.toml` plus one environment file, with
//! flags and `LEDGER_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

pub use crate::store::StoreKind;

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/ledger";

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
    pub store: StoreConfig,
    /// `[analytics]`
    #[serde(default)]
    pub analytics: Analytics,
    /// `[erasure]`
    #[serde(default)]
    pub erasure: ErasureConfig,
    /// `[idempotency]`
    #[serde(default)]
    pub idempotency: Idempotency,
    /// `[health]`
    #[serde(default)]
    pub health: Health,
}

/// `[store]`: which backend and how to reach it.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct StoreConfig {
    /// `postgres` (the real store) or `memory` (tests, chaos stacks, host runs
    /// without a database).
    pub kind: StoreKind,
    /// Connection URL. Environment only (`LEDGER_DATABASE_URL`); never in a
    /// file and never printed.
    #[serde(skip_serializing)]
    pub url: String,
    /// Pool size.
    pub max_connections: u32,
    /// How long a request waits for a pooled connection.
    #[serde(with = "humantime_serde")]
    pub acquire_timeout: Duration,
    /// Run the embedded migrations at start, under an advisory lock.
    /// `ledger migrate` runs them alone.
    pub migrate_on_start: bool,
}

impl Default for StoreConfig {
    fn default() -> Self {
        Self {
            kind: StoreKind::Memory,
            url: String::new(),
            max_connections: 16,
            acquire_timeout: Duration::from_secs(5),
            migrate_on_start: true,
        }
    }
}

/// `[analytics]`: the outbox drain into `ClickHouse`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Analytics {
    /// `ClickHouse` HTTP URL the outbox drainer ships events to. Environment
    /// only (`LEDGER_CLICKHOUSE_URL`, it carries a credential); empty = off:
    /// events are acked and counted, nothing is shipped.
    #[serde(skip_serializing)]
    pub clickhouse_url: String,
    /// Events per batch.
    pub batch: u32,
    /// Poll interval when the outbox is empty.
    #[serde(with = "humantime_serde")]
    pub period: Duration,
    /// How long a claimed batch stays reserved before another drainer may retry it.
    #[serde(with = "humantime_serde")]
    pub lease: Duration,
}

impl Default for Analytics {
    fn default() -> Self {
        Self {
            clickhouse_url: String::new(),
            batch: 500,
            period: Duration::from_secs(1),
            lease: Duration::from_secs(30),
        }
    }
}

/// `[erasure]`: the grace window and the sweeper.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct ErasureConfig {
    /// Time between the request and the cascade, during which a restore
    /// reopens the subject.
    #[serde(with = "humantime_serde")]
    pub grace: Duration,
    /// How often due erasures are executed and stale idempotency keys purged.
    #[serde(with = "humantime_serde")]
    pub sweep_interval: Duration,
    /// Erasures executed per sweep.
    pub batch: u32,
}

impl Default for ErasureConfig {
    fn default() -> Self {
        Self {
            grace: Duration::from_hours(7 * 24),
            sweep_interval: Duration::from_hours(1),
            batch: 100,
        }
    }
}

/// `[idempotency]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Idempotency {
    /// How long an idempotency key is remembered.
    #[serde(with = "humantime_serde")]
    pub ttl: Duration,
}

impl Default for Idempotency {
    fn default() -> Self {
        Self {
            ttl: Duration::from_hours(24),
        }
    }
}

/// `[health]`: the readiness probe against the store.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Health {
    /// How often the store is probed.
    #[serde(with = "humantime_serde")]
    pub probe_interval: Duration,
    /// How long one probe may take before it counts as down.
    #[serde(with = "humantime_serde")]
    pub probe_timeout: Duration,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            probe_interval: Duration::from_secs(5),
            probe_timeout: Duration::from_secs(2),
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

    /// An embedded ledger: the in-memory store, no metrics listener, no
    /// analytics. What chaos stacks and tests run.
    #[must_use]
    pub fn in_memory(listen: SocketAddr) -> Self {
        Self {
            server: Server { listen },
            metrics: Metrics { listen: None },
            ping: Ping::default(),
            store: StoreConfig {
                kind: StoreKind::Memory,
                ..StoreConfig::default()
            },
            analytics: Analytics::default(),
            erasure: ErasureConfig::default(),
            idempotency: Idempotency::default(),
            health: Health::default(),
        }
    }

    /// Cross-field checks after flags were applied.
    ///
    /// # Errors
    /// The Postgres store has no URL, or a bound is zero.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let invalid = |msg: &str| ConfigError::Invalid(msg.to_owned());
        if self.store.kind == StoreKind::Postgres && self.store.url.is_empty() {
            return Err(invalid(
                "store.kind = \"postgres\" but LEDGER_DATABASE_URL is not set",
            ));
        }
        if self.store.max_connections == 0 {
            return Err(invalid("store.max_connections must be > 0"));
        }
        if self.analytics.batch == 0 || self.erasure.batch == 0 {
            return Err(invalid("analytics.batch and erasure.batch must be > 0"));
        }
        if self.analytics.period.is_zero() || self.erasure.sweep_interval.is_zero() {
            return Err(invalid(
                "analytics.period and erasure.sweep_interval must be > 0",
            ));
        }
        if self.health.probe_interval.is_zero() || self.health.probe_timeout.is_zero() {
            return Err(invalid(
                "health.probe_interval and probe_timeout must be > 0",
            ));
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
    #[arg(long, env = "LEDGER_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "LEDGER_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "LEDGER_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// Store backend: `postgres` or `memory`. Default: `[store] kind`.
    #[arg(long, env = "LEDGER_STORE_KIND", value_parser = parse_store_kind)]
    pub store_kind: Option<StoreKind>,
    /// Postgres connection URL. Required when the store is `postgres`.
    #[arg(long, env = "LEDGER_DATABASE_URL", hide_env_values = true)]
    pub database_url: Option<String>,
    /// `ClickHouse` HTTP URL for the outbox drain; empty turns analytics off.
    /// Default: `[analytics] clickhouse_url`.
    #[arg(long, env = "LEDGER_CLICKHOUSE_URL", hide_env_values = true)]
    pub clickhouse_url: Option<String>,
}

fn parse_store_kind(s: &str) -> Result<StoreKind, String> {
    match s {
        "postgres" => Ok(StoreKind::Postgres),
        "memory" => Ok(StoreKind::Memory),
        other => Err(format!("{other:?}: postgres or memory")),
    }
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
        if let Some(v) = self.store_kind {
            config.store.kind = v;
        }
        if let Some(v) = &self.database_url {
            config.store.url.clone_from(v);
        }
        if let Some(v) = &self.clickhouse_url {
            config.analytics.clickhouse_url.clone_from(v);
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
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/ledger");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
            assert!(config.validate().is_ok() || config.store.kind == StoreKind::Postgres);
        }
    }

    #[test]
    fn postgres_without_a_url_is_rejected_and_the_url_never_prints() {
        let mut config =
            Config::in_memory("127.0.0.1:0".parse().unwrap_or_else(|_| unreachable!()));
        config.store.kind = StoreKind::Postgres;
        assert!(config.validate().is_err());
        config.store.url = "postgres://u:secret@h/db".into();
        assert!(config.validate().is_ok());
        let printed = toml::to_string(&config).unwrap_or_default();
        assert!(!printed.contains("secret"));
    }
}
