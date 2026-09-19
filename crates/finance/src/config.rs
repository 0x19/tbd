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
    /// `[provider]`
    #[serde(default)]
    pub provider: Provider,
    /// `[sync]`
    #[serde(default)]
    pub sync: Sync,
    /// `[connectors]`
    #[serde(default)]
    pub connectors: Connectors,
    /// `[mail]`
    #[serde(default)]
    pub mail: Mail,
}

/// `[mail]`: sending from a linked mailbox.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Mail {
    /// Addresses a mail may go to. Empty means anyone. Set in an environment
    /// where a test send must never reach a real recipient; a send to anyone
    /// else is refused before the provider is asked.
    #[serde(default)]
    pub allow_to: Vec<String>,
}

impl Mail {
    /// Whether `address` may be written to under this configuration.
    #[must_use]
    pub fn allows(&self, address: &str) -> bool {
        self.allow_to.is_empty()
            || self
                .allow_to
                .iter()
                .any(|a| a.eq_ignore_ascii_case(address.trim()))
    }
}

/// `[connectors]`: linked external accounts (mailboxes, vendor portals).
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Connectors {
    /// Base64 of 32 bytes; seals every stored credential. Environment only
    /// (`FINANCE_CONNECTOR_KEY`). Empty means connectors cannot be linked.
    #[serde(skip_serializing, default)]
    pub key: String,
    /// Google OAuth client for the Gmail kind. Environment only.
    #[serde(skip_serializing, default)]
    pub google_client_id: String,
    /// Its secret. Environment only.
    #[serde(skip_serializing, default)]
    pub google_client_secret: String,
    /// Where a provider sends the browser back: the finance UI host's
    /// `/connectors/callback/`, registered with each provider.
    #[serde(default)]
    pub redirect_url: String,
}

impl Connectors {
    /// Whether credentials can be sealed at all.
    #[must_use]
    pub fn configured(&self) -> bool {
        !self.key.is_empty()
    }
}

/// `[provider]`: the open-banking API.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Provider {
    /// Base URL. Tests point it at a fixture server.
    #[serde(default = "default_base_url")]
    pub base_url: String,
    /// The Enable Banking application id; also the JWT `kid`. Environment
    /// only (`FINANCE_EB_APPLICATION_ID`). Empty means no provider: the
    /// syncer does not start and `session` refuses.
    #[serde(skip_serializing, default)]
    pub application_id: String,
    /// PKCS#8 PEM of the application's RSA key. A credential: environment
    /// only (`FINANCE_EB_PRIVATE_KEY`), never in a file, never printed.
    /// Takes precedence over `private_key_path` when both are set.
    #[serde(skip_serializing, default)]
    pub private_key: String,
    /// Path to the same PEM, for a mounted Secret or a developer's
    /// `~/.config/enablebanking/private.key`.
    #[serde(skip_serializing, default)]
    pub private_key_path: String,
    /// Per-request timeout, seconds. Erste has answered a 51-page history in
    /// well under this; the timeout exists for the day it does not answer.
    #[serde(default = "default_provider_timeout")]
    pub timeout_secs: u64,
    /// Where the bank sends the browser after authorization. Must be one of
    /// the URLs registered with the provider, and must be on the finance UI
    /// host so the page can complete the connection as the signed-in person.
    #[serde(default)]
    pub redirect_url: String,
}

fn default_base_url() -> String {
    crate::banking::client::DEFAULT_BASE_URL.to_owned()
}

const fn default_provider_timeout() -> u64 {
    60
}

impl Default for Provider {
    fn default() -> Self {
        Self {
            base_url: default_base_url(),
            application_id: String::new(),
            private_key: String::new(),
            private_key_path: String::new(),
            timeout_secs: default_provider_timeout(),
            redirect_url: String::new(),
        }
    }
}

impl Provider {
    /// Whether enough is configured to talk to a bank.
    #[must_use]
    pub fn configured(&self) -> bool {
        !self.application_id.is_empty()
            && (!self.private_key.is_empty() || !self.private_key_path.is_empty())
    }
}

/// `[sync]`: how often, and how much, the syncer asks the bank.
///
/// PSD2 lets a bank cap unattended fetches at about four per account per
/// day, and Erste does: the fifth fetch of a day is a 429 with no
/// Retry-After. Three fetches a day are the scheduler's, eight hours apart;
/// the fourth is a person's.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sync {
    /// How often the loop wakes to look for due accounts. Not how often a
    /// bank is called; that is `min_interval` and the budget.
    #[serde(default = "default_sync_interval")]
    pub interval_secs: u64,
    /// The least time between two scheduled fetches of one account.
    #[serde(default = "default_min_interval")]
    pub min_interval_secs: u64,
    /// Fetches per account per day, counting manual refreshes.
    #[serde(default = "default_budget")]
    pub budget_per_day: u32,
    /// How many of those the scheduler may spend. The rest is reserved for a
    /// person pressing Refresh -- which must not answer "come back tomorrow"
    /// because a timer spent the quota.
    #[serde(default = "default_scheduled_budget")]
    pub scheduled_budget: u32,
    /// How far back a routine fetch reaches past the last booked date, so a
    /// late-booked transaction is not missed. Dedup makes the overlap free.
    #[serde(default = "default_overlap_days")]
    pub overlap_days: u32,
    /// The least a first fetch reaches back; it always reaches at least to
    /// 1 January of the current year. Unattended, Erste answers 90 days
    /// whatever is asked (measured); the full year is the attended pull
    /// right after authorization.
    #[serde(default = "default_initial_history_days")]
    pub initial_history_days: u32,
    /// How long before a consent lapses the service starts saying so.
    #[serde(default = "default_reconsent_lead_days")]
    pub reconsent_lead_days: u32,
    /// What to wait after a 429 that carries no `Retry-After`. Enable
    /// Banking documents six hours.
    #[serde(default = "default_backoff")]
    pub default_backoff_secs: u64,
}

const fn default_sync_interval() -> u64 {
    15 * 60
}
const fn default_min_interval() -> u64 {
    8 * 60 * 60
}
const fn default_budget() -> u32 {
    4
}
const fn default_scheduled_budget() -> u32 {
    3
}
const fn default_overlap_days() -> u32 {
    7
}
const fn default_initial_history_days() -> u32 {
    90
}
const fn default_reconsent_lead_days() -> u32 {
    14
}
const fn default_backoff() -> u64 {
    6 * 60 * 60
}

impl Default for Sync {
    fn default() -> Self {
        Self {
            interval_secs: default_sync_interval(),
            min_interval_secs: default_min_interval(),
            budget_per_day: default_budget(),
            scheduled_budget: default_scheduled_budget(),
            overlap_days: default_overlap_days(),
            initial_history_days: default_initial_history_days(),
            reconsent_lead_days: default_reconsent_lead_days(),
            default_backoff_secs: default_backoff(),
        }
    }
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
    /// Enable Banking application id. Default: `[provider] application_id`.
    #[arg(long, env = "FINANCE_EB_APPLICATION_ID")]
    pub eb_application_id: Option<String>,
    /// PKCS#8 PEM of the application's RSA key. A credential, never echoed.
    #[arg(long, env = "FINANCE_EB_PRIVATE_KEY", hide_env_values = true)]
    pub eb_private_key: Option<String>,
    /// Path to that PEM instead. Default: `[provider] private_key_path`.
    #[arg(long, env = "FINANCE_EB_PRIVATE_KEY_PATH")]
    pub eb_private_key_path: Option<String>,
    /// Provider base URL. Default: `[provider] base_url`.
    #[arg(long, env = "FINANCE_EB_BASE_URL")]
    pub eb_base_url: Option<String>,
    /// Key sealing stored connector credentials, base64 of 32 bytes. Never echoed.
    #[arg(long, env = "FINANCE_CONNECTOR_KEY", hide_env_values = true)]
    pub connector_key: Option<String>,
    /// Google OAuth client id for the Gmail connector.
    #[arg(long, env = "FINANCE_GOOGLE_CLIENT_ID")]
    pub google_client_id: Option<String>,
    /// Google OAuth client secret. Never echoed.
    #[arg(long, env = "FINANCE_GOOGLE_CLIENT_SECRET", hide_env_values = true)]
    pub google_client_secret: Option<String>,
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
        if let Some(v) = &self.eb_application_id {
            config.provider.application_id.clone_from(v);
        }
        if let Some(v) = &self.eb_private_key {
            config.provider.private_key.clone_from(v);
        }
        if let Some(v) = &self.eb_private_key_path {
            config.provider.private_key_path.clone_from(v);
        }
        if let Some(v) = &self.eb_base_url {
            config.provider.base_url.clone_from(v);
        }
        if let Some(v) = &self.connector_key {
            config.connectors.key.clone_from(v);
        }
        if let Some(v) = &self.google_client_id {
            config.connectors.google_client_id.clone_from(v);
        }
        if let Some(v) = &self.google_client_secret {
            config.connectors.google_client_secret.clone_from(v);
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
