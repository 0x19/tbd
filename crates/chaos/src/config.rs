//! `configs/chaos/`: the tool's own configuration, layered per environment.
//!
//! Loaded through [`tbd_common::config`]: `base.toml` merged with
//! `<env>.toml`. Flags override fields after loading; see `main.rs`.

use std::{
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use serde::{Deserialize, Serialize};

pub use tbd_common::config::{DEFAULT_ENV, ENV_VAR};

/// Default directory.
pub const DEFAULT_DIR: &str = "configs/chaos";

/// The whole configuration.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ChaosConfig {
    /// `[serve]`
    pub serve: Serve,
    /// `[paths]`
    pub paths: Paths,
    /// `[targets]`
    pub targets: Targets,
    /// `[validate]`
    pub validate: Validate,
    /// `[auth]`
    #[serde(default)]
    pub auth: AuthConfig,
    /// `[links]`
    pub links: Links,
}

/// `[auth]`: the bearer token `validate` and load runs send to a deployed
/// stack, where Envoy requires one on every API route except health.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct AuthConfig {
    /// A fixed token. Wins over the client-credentials fields.
    #[serde(skip_serializing)]
    pub token: String,
    /// `OAuth2` token endpoint, e.g. `https://auth.example.com/oauth2/token`. Empty: no auth.
    pub token_url: String,
    /// Client id for the client-credentials grant.
    pub client_id: String,
    /// Its secret. Comes from the environment, never from a file.
    #[serde(skip_serializing)]
    pub client_secret: String,
    /// Requested scope.
    pub scope: String,
    /// Requested audience.
    pub audience: String,
}

impl AuthConfig {
    /// The configured token source, if any.
    #[must_use]
    pub fn auth(&self) -> Option<crate::auth::Auth> {
        if !self.token.is_empty() {
            return Some(crate::auth::Auth::token(self.token.clone()));
        }
        if self.token_url.is_empty() {
            return None;
        }
        Some(crate::auth::Auth::client_credentials(
            self.token_url.clone(),
            self.client_id.clone(),
            self.client_secret.clone(),
            self.scope.clone(),
            self.audience.clone(),
        ))
    }
}

/// `[serve]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Serve {
    /// Listen address.
    pub listen: SocketAddr,
    /// API prefix, e.g. `/api/chaos/v1`.
    pub base_path: String,
    /// Built UI directory; empty for none.
    pub ui_dir: String,
    /// Path the UI is served at; `""` is the root of the host.
    pub ui_path: String,
    /// Start the topology stack on boot.
    pub start_stack: bool,
}

/// `[paths]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Paths {
    /// Topology file.
    pub topology: PathBuf,
    /// Scenario directory.
    pub scenarios: PathBuf,
    /// Copied into `scenarios` on `serve` start when that directory is missing
    /// or empty. Empty string: no seeding. Lets a read-only image ship its
    /// scenarios while the UI edits a writable copy.
    #[serde(default)]
    pub scenarios_seed: PathBuf,
    /// Run records.
    pub results: PathBuf,
    /// Schedules file (`chaos serve` cron jobs). Missing: no schedules yet.
    #[serde(default = "default_schedules")]
    pub schedules: PathBuf,
}

fn default_schedules() -> PathBuf {
    PathBuf::from(".chaos/schedules.json")
}

/// `[targets]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Targets {
    /// Protocol base URL.
    pub protocol: String,
    /// Engine gRPC URL.
    pub engine: String,
}

/// `[validate]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Validate {
    /// Per-check timeout.
    #[serde(with = "humantime_serde")]
    pub timeout: Duration,
    /// Extra PEM root for `https://` / `wss://` targets. Empty: public roots only.
    #[serde(default)]
    pub ca_cert: PathBuf,
}

impl Validate {
    /// TLS trust for the targets: public roots plus `ca_cert` when set.
    pub fn trust(&self) -> anyhow::Result<crate::tls::Trust> {
        if self.ca_cert.as_os_str().is_empty() {
            Ok(crate::tls::Trust::default())
        } else {
            crate::tls::Trust::from_pem_file(&self.ca_cert)
        }
    }
}

/// `[links]`: where the UI sends people. Either a public `domain`, from which
/// the edge's fixed host names are derived (`grafana.<domain>`,
/// `logs.<domain>`, `profiles.<domain>`, `metrics.<domain>`, see
/// `devops/edge/Caddyfile`), or explicit URLs for environments without one.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Links {
    /// Public base domain. When set, every link below is derived from it and
    /// the explicit values are ignored; `envoy_admin` is cleared because the
    /// edge never exposes it.
    #[serde(default)]
    pub domain: String,
    /// Grafana root.
    pub grafana: String,
    /// `VictoriaLogs` UI.
    pub victorialogs: String,
    /// `VictoriaMetrics` UI.
    #[serde(default)]
    pub metrics: String,
    /// Pyroscope UI.
    pub pyroscope: String,
    /// Envoy admin.
    pub envoy_admin: String,
}

impl Links {
    /// The links as the UI should show them: derived from `domain` when set.
    #[must_use]
    pub fn resolved(&self) -> Self {
        let d = self.domain.trim().trim_matches('.');
        if d.is_empty() {
            return self.clone();
        }
        Self {
            domain: d.to_owned(),
            grafana: format!("https://grafana.{d}"),
            victorialogs: format!("https://logs.{d}/select/vmui/"),
            metrics: format!("https://metrics.{d}/vmui/"),
            pyroscope: format!("https://profiles.{d}"),
            envoy_admin: String::new(),
        }
    }
}

/// Where the config came from, for the overview.
#[derive(Debug, Clone, Serialize)]
pub struct Source {
    /// Environment name.
    pub env: String,
    /// Files merged, in order.
    pub files: Vec<PathBuf>,
}

impl ChaosConfig {
    /// TLS trust plus the bearer token source for the configured targets.
    ///
    /// # Errors
    /// The extra CA file cannot be read.
    pub fn trust(&self) -> anyhow::Result<crate::tls::Trust> {
        Ok(self.validate.trust()?.with_auth(self.auth.auth()))
    }

    /// Load `dir/base.toml` + `dir/<env>.toml`.
    pub fn load(dir: &Path, env: &str) -> anyhow::Result<(Self, Source)> {
        let loaded = tbd_common::config::load::<Self>(dir, env)?;
        loaded.value.check()?;
        Ok((
            loaded.value,
            Source {
                env: loaded.env,
                files: loaded.files,
            },
        ))
    }

    /// Structural checks.
    pub fn check(&self) -> anyhow::Result<()> {
        let p = &self.serve.base_path;
        anyhow::ensure!(
            p.starts_with('/') && p.len() > 1 && !p.ends_with('/'),
            "serve.base_path must start with / and not end with one, got {p:?}"
        );
        let p = &self.serve.ui_path;
        anyhow::ensure!(
            p.is_empty() || (p.starts_with('/') && p.len() > 1 && !p.ends_with('/')),
            "serve.ui_path must be empty (the root) or start with / and not end with one, got {p:?}"
        );
        anyhow::ensure!(
            self.serve.base_path != self.serve.ui_path,
            "serve.base_path and serve.ui_path must differ"
        );
        for (what, u) in [
            ("targets.protocol", &self.targets.protocol),
            ("targets.engine", &self.targets.engine),
        ] {
            url::Url::parse(u).map_err(|e| anyhow::anyhow!("{what}: {e}"))?;
        }
        if !self.auth.token_url.is_empty() {
            url::Url::parse(&self.auth.token_url)
                .map_err(|e| anyhow::anyhow!("auth.token_url: {e}"))?;
            anyhow::ensure!(
                !self.auth.client_id.is_empty() && !self.auth.client_secret.is_empty(),
                "auth.token_url is set but auth.client_id / auth.client_secret (CHAOS_AUTH_CLIENT_SECRET) are not"
            );
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::Links;

    #[test]
    fn a_domain_derives_the_edge_hosts_and_drops_envoy_admin() {
        let links = Links {
            domain: "example.org.".into(),
            grafana: "http://localhost:3000".into(),
            victorialogs: String::new(),
            metrics: String::new(),
            pyroscope: String::new(),
            envoy_admin: "http://localhost:9901".into(),
        }
        .resolved();
        assert_eq!(links.grafana, "https://grafana.example.org");
        assert_eq!(links.victorialogs, "https://logs.example.org/select/vmui/");
        assert_eq!(links.metrics, "https://metrics.example.org/vmui/");
        assert_eq!(links.pyroscope, "https://profiles.example.org");
        assert_eq!(links.envoy_admin, "");
    }

    #[test]
    fn no_domain_keeps_explicit_links() {
        let links = Links {
            domain: String::new(),
            grafana: "http://localhost:3000".into(),
            victorialogs: String::new(),
            metrics: String::new(),
            pyroscope: String::new(),
            envoy_admin: "http://localhost:9901".into(),
        };
        assert_eq!(links.resolved().envoy_admin, "http://localhost:9901");
    }
}
