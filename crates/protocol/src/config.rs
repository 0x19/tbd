//! Configuration: `configs/protocol/base.toml` plus one environment file, with
//! flags and `PROTOCOL_*` environment variables applied over the result.
//!
//! The `[services]` table is the registry of backends this gateway forwards
//! to. Its keys are service names (`engine`, `ledger`, a scaffolded one); every
//! name has an environment override `PROTOCOL_<NAME>_URL`, read generically so
//! `tbd new service` adds a table and nothing else.

use std::{
    collections::BTreeMap,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/protocol";

/// The backend the typed engine client talks to; must be registered.
pub const ENGINE: &str = "engine";

/// The whole configuration. Every key lives in `base.toml`.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// `[server]`
    pub server: Server,
    /// `[metrics]`
    #[serde(default)]
    pub metrics: Metrics,
    /// `[services.<name>]`: the backends, by name.
    #[serde(default)]
    pub services: BTreeMap<String, ServiceConfig>,
    /// `[health]`
    #[serde(default)]
    pub health: Health,
    /// `[principals]`
    #[serde(default)]
    pub principals: Principals,
}

/// `[server]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Server {
    /// Address the protocol binds: HTTP/1.1 and h2c (gRPC) on one port.
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

/// One `[services.<name>]` table: a backend the gateway forwards to.
#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceConfig {
    /// gRPC endpoint, `http://host:port`. Envoy's internal listener in every
    /// deployed environment; `PROTOCOL_<NAME>_URL` overrides it.
    pub url: String,
    /// The `grpc.health.v1` service name probed on that endpoint and reported
    /// on the protocol's own health service.
    pub service: String,
    /// Whether `/readyz` fails while this backend is not `SERVING`.
    #[serde(default)]
    pub required: bool,
}

/// `[health]`: the backend probes.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Health {
    /// How often every backend is probed for the gRPC health service.
    #[serde(with = "humantime_serde")]
    pub probe_interval: Duration,
    /// How long one probe (and one `/readyz` check) may take before the
    /// backend counts as not serving.
    #[serde(with = "humantime_serde")]
    pub probe_timeout: Duration,
}

impl Default for Health {
    fn default() -> Self {
        Self {
            probe_interval: Duration::from_secs(5),
            probe_timeout: Duration::from_secs(1),
        }
    }
}

/// `[principals]`: how verified callers are classified.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields, default)]
pub struct Principals {
    /// Token subjects that are our own services.
    pub services: Vec<String>,
}

/// Where a loaded configuration came from.
#[derive(Debug, Clone)]
pub struct Source {
    /// Environment name.
    pub env: String,
    /// Files merged, in order.
    pub files: Vec<PathBuf>,
}

/// `ledger` → `tbd.ledger.v1.LedgerService`: the health name a service
/// scaffolded by `tbd new service` has, used when an embedder registers a
/// backend by name only.
#[must_use]
pub fn grpc_service_name(name: &str) -> String {
    let mut pascal = String::new();
    let mut upper = true;
    for c in name.chars() {
        if c == '_' || c == '-' {
            upper = true;
        } else if upper {
            pascal.extend(c.to_uppercase());
            upper = false;
        } else {
            pascal.push(c);
        }
    }
    format!("tbd.{name}.v1.{pascal}Service")
}

/// The environment variable that overrides `[services.<name>] url`.
#[must_use]
pub fn service_url_var(name: &str) -> String {
    format!("PROTOCOL_{}_URL", name.to_ascii_uppercase())
}

fn valid_service_name(name: &str) -> bool {
    let mut chars = name.chars();
    let len = name.len();
    (2..=24).contains(&len)
        && chars.next().is_some_and(|c| c.is_ascii_lowercase())
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
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

    /// An embedded protocol: the given backends by `(name, url)`, all
    /// required, health names by convention ([`grpc_service_name`]), no
    /// metrics listener. What chaos stacks and tests run.
    #[must_use]
    pub fn embedded(
        listen: SocketAddr,
        services: impl IntoIterator<Item = (String, String)>,
    ) -> Self {
        let services = services
            .into_iter()
            .map(|(name, url)| {
                let service = grpc_service_name(&name);
                (
                    name,
                    ServiceConfig {
                        url,
                        service,
                        required: true,
                    },
                )
            })
            .collect();
        Self {
            server: Server { listen },
            metrics: Metrics { listen: None },
            services,
            health: Health::default(),
            principals: Principals::default(),
        }
    }

    /// The engine's registration, if present.
    #[must_use]
    pub fn engine(&self) -> Option<&ServiceConfig> {
        self.services.get(ENGINE)
    }

    /// Cross-field checks after flags were applied.
    ///
    /// # Errors
    /// No engine, a name that cannot be an environment variable, a URL that
    /// does not parse, an empty health name, or a zero duration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        let invalid = |msg: String| ConfigError::Invalid(msg);
        if self.engine().is_none() {
            return Err(invalid(format!(
                "[services.{ENGINE}] is required: the typed engine client needs it"
            )));
        }
        for (name, service) in &self.services {
            if !valid_service_name(name) {
                return Err(invalid(format!(
                    "services.{name}: a service name is 2 to 24 lowercase letters or digits, starting with a letter"
                )));
            }
            if service.service.is_empty() {
                return Err(invalid(format!(
                    "services.{name}.service must name the grpc.health.v1 service"
                )));
            }
            tonic::transport::Endpoint::from_shared(service.url.clone())
                .map_err(|e| invalid(format!("services.{name}.url {:?}: {e}", service.url)))?;
        }
        if self.health.probe_interval.is_zero() || self.health.probe_timeout.is_zero() {
            return Err(invalid(
                "health.probe_interval and probe_timeout must be > 0".to_owned(),
            ));
        }
        Ok(())
    }
}

/// Flags that override the loaded configuration. Every one has an environment
/// variable; the backend URLs have one per registered name,
/// `PROTOCOL_<NAME>_URL`, read by [`Overrides::apply`].
#[derive(Args, Debug, Clone)]
pub struct Overrides {
    /// Environment: picks `<config-dir>/<env>.toml` to merge over `base.toml`.
    #[arg(long, env = tbd_common::config::ENV_VAR, default_value = tbd_common::config::DEFAULT_ENV, global = true)]
    pub env: String,
    /// Directory holding `base.toml` and one file per environment.
    #[arg(long, env = "PROTOCOL_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the protocol binds. Default: `[server] listen`.
    #[arg(long, env = "PROTOCOL_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "PROTOCOL_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// A backend URL, `name=URL`, repeatable; the flag form of
    /// `PROTOCOL_<NAME>_URL` and it wins over the variable.
    #[arg(long = "service-url", value_name = "NAME=URL", value_parser = parse_service_url)]
    pub service_urls: Vec<(String, String)>,
}

fn parse_service_url(s: &str) -> Result<(String, String), String> {
    let (name, url) = s
        .split_once('=')
        .ok_or_else(|| format!("{s:?}: expected name=URL"))?;
    if !valid_service_name(name) {
        return Err(format!("{name:?}: not a service name"));
    }
    if url.is_empty() {
        return Err(format!("{name}: empty URL"));
    }
    Ok((name.to_owned(), url.to_owned()))
}

impl Overrides {
    /// Apply the flags that were given and every `PROTOCOL_<NAME>_URL` set
    /// in the process environment for a registered service.
    pub fn apply(&self, config: &mut Config) {
        self.apply_with(config, |var| std::env::var(var).ok());
    }

    /// [`Overrides::apply`] with the environment supplied by `lookup`; a
    /// `--service-url` flag wins over the variable.
    pub fn apply_with(&self, config: &mut Config, lookup: impl Fn(&str) -> Option<String>) {
        if let Some(v) = self.listen_addr {
            config.server.listen = v;
        }
        if let Some(v) = self.metrics_addr {
            config.metrics.listen = Some(v);
        }
        for (name, service) in &mut config.services {
            if let Some(url) = lookup(&service_url_var(name)).filter(|u| !u.is_empty()) {
                service.url = url;
            }
        }
        for (name, url) in &self.service_urls {
            if let Some(service) = config.services.get_mut(name) {
                service.url.clone_from(url);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dir() -> PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/protocol")
    }

    /// Every shipped environment file loads over `base.toml` and validates;
    /// catches a key that `deny_unknown_fields` would reject at start.
    #[test]
    fn every_shipped_env_file_loads_and_registers_the_engine() {
        for env in ["local", "dev", "production"] {
            let (config, source) =
                Config::load(&dir(), env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            config.validate().unwrap_or_else(|e| panic!("{env}: {e}"));
            let engine = config
                .engine()
                .unwrap_or_else(|| panic!("{env}: no engine"));
            assert!(engine.required);
            assert_eq!(engine.service, "tbd.engine.v1.EngineService");
            for name in ["humans", "ledger"] {
                let s = &config.services[name];
                assert!(!s.required, "{env}: {name} must not gate readiness yet");
                assert_eq!(s.service, grpc_service_name(name));
            }
        }
    }

    #[test]
    fn env_overrides_service_urls_generically_and_flags_win() {
        let (mut config, _) = Config::load(&dir(), "local").unwrap_or_else(|e| panic!("{e}"));
        let overrides = Overrides {
            env: "local".into(),
            config_dir: dir(),
            listen_addr: None,
            metrics_addr: None,
            service_urls: vec![("ledger".into(), "http://flag:1".into())],
        };
        overrides.apply_with(&mut config, |var| match var {
            "PROTOCOL_ENGINE_URL" => Some("http://envoy:50051".into()),
            "PROTOCOL_LEDGER_URL" => Some("http://env:2".into()),
            "PROTOCOL_HUMANS_URL" => Some(String::new()),
            _ => None,
        });
        assert_eq!(config.services["engine"].url, "http://envoy:50051");
        assert_eq!(
            config.services["ledger"].url, "http://flag:1",
            "the flag wins"
        );
        assert_eq!(
            config.services["humans"].url, "http://127.0.0.1:50053",
            "an empty variable is unset"
        );
        assert_eq!(service_url_var("ledger"), "PROTOCOL_LEDGER_URL");
    }

    #[test]
    fn embedded_names_services_by_convention() {
        let listen: SocketAddr = "127.0.0.1:0".parse().unwrap_or_else(|_| unreachable!());
        let config = Config::embedded(
            listen,
            [("engine".to_owned(), "http://127.0.0.1:1".to_owned())],
        );
        assert!(config.metrics.listen.is_none());
        assert!(config.validate().is_ok());
        assert_eq!(
            config.services["engine"].service,
            "tbd.engine.v1.EngineService"
        );
        assert_eq!(grpc_service_name("ledger"), "tbd.ledger.v1.LedgerService");
        assert_eq!(grpc_service_name("humans"), "tbd.humans.v1.HumansService");
    }

    #[test]
    fn validate_rejects_a_missing_engine_a_bad_url_and_a_bad_name() {
        let listen: SocketAddr = "127.0.0.1:0".parse().unwrap_or_else(|_| unreachable!());
        let no_engine = Config::embedded(
            listen,
            [("ledger".to_owned(), "http://127.0.0.1:1".to_owned())],
        );
        assert!(no_engine.validate().is_err());
        let bad_url = Config::embedded(listen, [("engine".to_owned(), "not a url".to_owned())]);
        assert!(bad_url.validate().is_err());
        let mut bad_name = Config::embedded(
            listen,
            [("engine".to_owned(), "http://127.0.0.1:1".to_owned())],
        );
        bad_name.services.insert(
            "Bad-Name".into(),
            ServiceConfig {
                url: "http://127.0.0.1:2".into(),
                service: "x".into(),
                required: false,
            },
        );
        assert!(bad_name.validate().is_err());
        assert!(parse_service_url("ledger=http://x").is_ok());
        assert!(parse_service_url("ledger").is_err());
    }
}
