//! Configuration: `configs/cv/base.toml` plus one environment file, with
//! flags and `CV_*` environment variables applied over the result.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/cv";

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
    /// `[finance]`
    #[serde(default)]
    pub finance: Finance,
    /// `[notify]`
    #[serde(default)]
    pub notify: Notify,
    /// `[private]`
    #[serde(default)]
    pub private: Private,
    /// `[render]`
    #[serde(default)]
    pub render: Render,
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
    /// Connection URL. Environment only (`CV_DATABASE_URL`); never in a file
    /// and never printed. Empty means no store: the service serves `Ping` and
    /// answers every other RPC `UNAVAILABLE`.
    #[serde(skip_serializing)]
    #[serde(default)]
    pub url: String,
    /// Pool size.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,
}

const fn default_max_connections() -> u32 {
    5
}

/// `[finance]`: where the finance service is reached for the mail. Through
/// Envoy's internal listener in a deployment (`http://envoy:50051`); empty
/// means the owner is not told and requests still work.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Finance {
    /// gRPC URL of the finance service, matched by service name at Envoy.
    #[serde(default)]
    pub url: String,
}

/// `[notify]`: the mail that tells the owner of a request.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Notify {
    /// The address of the mailbox linked in the finance app the mail goes out
    /// as; the connector is found by it. Empty turns notifications off.
    #[serde(default)]
    pub from: String,
    /// Where the owner reads: the recipient of every request notice.
    #[serde(default)]
    pub to: String,
    /// The gated site's URL, `https://cv.example.hr/`: the mail links the
    /// owner to its admin page and an approved person to its front page.
    #[serde(default)]
    pub url: String,
}

impl Notify {
    /// Notifications are on when a mailbox and a recipient are named.
    #[must_use]
    pub fn enabled(&self) -> bool {
        !self.from.trim().is_empty() && !self.to.trim().is_empty()
    }

    /// The admin page, where a request is decided.
    #[must_use]
    pub fn admin_url(&self) -> String {
        format!("{}admin/", self.site_url())
    }

    /// The site with one trailing slash.
    #[must_use]
    pub fn site_url(&self) -> String {
        let u = self.url.trim().trim_end_matches('/');
        if u.is_empty() {
            String::new()
        } else {
            format!("{u}/")
        }
    }
}

/// `[private]`: the fields the full CV has and the public one does not.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Private {
    /// Path to a JSON file (`{phone, address, references}`); read at start.
    /// Empty means the full CV carries no private fields at all.
    #[serde(default)]
    pub path: String,
    /// The same JSON inline. Environment only (`CV_PRIVATE_JSON`, from a
    /// Secret); never in a file and never printed.
    #[serde(skip_serializing)]
    #[serde(default)]
    pub json: String,
}

/// `[render]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Render {
    /// Longest a render may take before the download answers
    /// `DEADLINE_EXCEEDED`. Warm renders take well under a second.
    pub timeout_secs: u64,
}

impl Default for Render {
    fn default() -> Self {
        Self { timeout_secs: 20 }
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
    #[arg(long, env = "CV_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "CV_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "CV_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// Postgres URL. A credential, so it is never echoed.
    #[arg(long, env = "CV_DATABASE_URL", hide_env_values = true)]
    pub database_url: Option<String>,
    /// The finance service's gRPC URL, for the mail. Default: `[finance] url`.
    #[arg(long, env = "CV_FINANCE_URL")]
    pub finance_url: Option<String>,
    /// The mailbox the notice goes out as. Default: `[notify] from`.
    #[arg(long, env = "CV_NOTIFY_FROM")]
    pub notify_from: Option<String>,
    /// The owner's address. Default: `[notify] to`.
    #[arg(long, env = "CV_NOTIFY_TO")]
    pub notify_to: Option<String>,
    /// The gated site's URL. Default: `[notify] url`.
    #[arg(long, env = "CV_NOTIFY_URL")]
    pub notify_url: Option<String>,
    /// The private fields as JSON. Never echoed.
    #[arg(long, env = "CV_PRIVATE_JSON", hide_env_values = true)]
    pub private_json: Option<String>,
    /// Path to that JSON instead. Default: `[private] path`.
    #[arg(long, env = "CV_PRIVATE_JSON_PATH")]
    pub private_json_path: Option<String>,
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
        if let Some(v) = &self.finance_url {
            config.finance.url.clone_from(v);
        }
        if let Some(v) = &self.notify_from {
            config.notify.from.clone_from(v);
        }
        if let Some(v) = &self.notify_to {
            config.notify.to.clone_from(v);
        }
        if let Some(v) = &self.notify_url {
            config.notify.url.clone_from(v);
        }
        if let Some(v) = &self.private_json {
            config.private.json.clone_from(v);
        }
        if let Some(v) = &self.private_json_path {
            config.private.path.clone_from(v);
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
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/cv");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
            assert!(
                config.store.url.is_empty(),
                "{env}: a database URL in a file"
            );
            assert!(
                config.private.json.is_empty(),
                "{env}: private fields in a file"
            );
        }
    }

    #[test]
    fn the_notify_urls_derive_from_one_setting() {
        let n = Notify {
            from: "a@b".into(),
            to: "c@d".into(),
            url: "https://cv.example.hr".into(),
        };
        assert!(n.enabled());
        assert_eq!(n.site_url(), "https://cv.example.hr/");
        assert_eq!(n.admin_url(), "https://cv.example.hr/admin/");
        assert!(!Notify::default().enabled());
        assert_eq!(Notify::default().admin_url(), "admin/");
    }
}
