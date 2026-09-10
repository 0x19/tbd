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

/// Environment variable naming the environment, shared by every binary.
pub const ENV_VAR: &str = "TBD_ENV";
/// Default environment.
pub const DEFAULT_ENV: &str = "local";
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
    /// `[links]`
    pub links: Links,
}

/// `[serve]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Serve {
    /// Listen address.
    pub listen: SocketAddr,
    /// API prefix, e.g. `/api/chaos`.
    pub base_path: String,
    /// Built UI directory; empty for none.
    pub ui_dir: String,
    /// Path the UI is served at.
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

/// `[links]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Links {
    /// Grafana root.
    pub grafana: String,
    /// `VictoriaLogs` UI.
    pub victorialogs: String,
    /// Pyroscope UI.
    pub pyroscope: String,
    /// Envoy admin.
    pub envoy_admin: String,
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
        for (what, p) in [
            ("serve.base_path", &self.serve.base_path),
            ("serve.ui_path", &self.serve.ui_path),
        ] {
            anyhow::ensure!(
                p.starts_with('/') && (p.len() == 1 || !p.ends_with('/')),
                "{what} must start with / and not end with one, got {p:?}"
            );
        }
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
        Ok(())
    }
}
