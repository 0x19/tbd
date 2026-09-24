//! Configuration: `configs/llm/base.toml` plus one environment file, with
//! flags and `LLM_*` environment variables applied over the result.
//!
//! The `[engines]` table is the one place an engine (the L1 that runs the
//! weights) is named. Everything above it is engine-neutral, and there is
//! deliberately no flag or variable that can select an engine *kind*: a
//! deployment picks its engines in a file that is reviewed, and only the
//! addresses and model ids vary by environment.

use std::{
    collections::BTreeMap,
    fmt,
    net::SocketAddr,
    path::{Path, PathBuf},
    time::Duration,
};

use clap::Args;
use serde::{Deserialize, Serialize};
use tbd_common::config::{ConfigError, Loaded};

/// Default directory holding `base.toml` and one file per environment.
pub const DEFAULT_DIR: &str = "configs/llm";

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
    /// `[engines]`
    #[serde(default)]
    pub engines: Engines,
    /// `[generate]`
    #[serde(default)]
    pub generate: Generate,
    /// `[budget]`
    #[serde(default)]
    pub budget: Budget,
    /// `[store]`
    #[serde(default)]
    pub store: Store,
    /// `[agents]`
    #[serde(default)]
    pub agents: AgentsConfig,
}

/// `[agents]` (RFC 0011): where the agent files are. An empty or absent
/// directory is no agents.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AgentsConfig {
    /// One `<id>.toml` per agent, and the knowledge files they name.
    pub dir: PathBuf,
}

impl Default for AgentsConfig {
    fn default() -> Self {
        Self {
            dir: PathBuf::from("configs/llm/agents"),
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

/// A tier: which engine a request is routed to.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Tier {
    /// The model that fits the GPU and answers a visitor.
    Fast,
    /// The large model that runs from memory, for quality.
    Deep,
}

impl Tier {
    /// Every tier, in routing order.
    pub const ALL: [Tier; 2] = [Tier::Fast, Tier::Deep];

    /// The lower-case name used in config, metrics and the store.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            Tier::Fast => "fast",
            Tier::Deep => "deep",
        }
    }
}

impl fmt::Display for Tier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// What runs the weights behind a tier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum EngineKind {
    /// Ollama's HTTP API (`/api/chat`, NDJSON).
    Ollama,
    /// llama.cpp's `llama-server` (OpenAI-compatible, SSE).
    Llamacpp,
    /// The in-process test engine. Labelled `stub` on every surface; refused
    /// in production.
    Stub,
}

impl EngineKind {
    /// The name on the wire and in metrics.
    #[must_use]
    pub fn as_str(self) -> &'static str {
        match self {
            EngineKind::Ollama => "ollama",
            EngineKind::Llamacpp => "llamacpp",
            EngineKind::Stub => "stub",
        }
    }
}

/// `[engines]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Engines {
    /// The tier a request without one goes to.
    pub default_tier: Tier,
    /// How often each engine is asked whether it is up.
    #[serde(with = "humantime_serde")]
    pub probe_interval: Duration,
    /// How long one probe may take before it counts as down.
    #[serde(with = "humantime_serde")]
    pub probe_timeout: Duration,
    /// `[engines.fast]`
    pub fast: EngineConfig,
    /// `[engines.deep]`
    pub deep: EngineConfig,
}

impl Default for Engines {
    fn default() -> Self {
        Self {
            default_tier: Tier::Fast,
            probe_interval: Duration::from_secs(15),
            probe_timeout: Duration::from_secs(3),
            fast: EngineConfig {
                kind: EngineKind::Llamacpp,
                url: "http://127.0.0.1:8082".to_owned(),
                model: "gpt-oss-20b".to_owned(),
                timeout_secs: 120,
                embed_model: String::new(),
                max_in_flight: 4,
                max_queued: 8,
                queue_timeout: Duration::from_secs(30),
                reasoning_on: effort("medium"),
                reasoning_off: effort("low"),
            },
            deep: EngineConfig {
                kind: EngineKind::Llamacpp,
                url: "http://127.0.0.1:8081".to_owned(),
                model: "gpt-oss-120b".to_owned(),
                timeout_secs: 900,
                embed_model: String::new(),
                max_in_flight: 1,
                max_queued: 2,
                queue_timeout: Duration::from_secs(120),
                reasoning_on: effort("medium"),
                reasoning_off: effort("low"),
            },
        }
    }
}

impl Engines {
    /// The configuration of one tier.
    #[must_use]
    pub fn tier(&self, tier: Tier) -> &EngineConfig {
        match tier {
            Tier::Fast => &self.fast,
            Tier::Deep => &self.deep,
        }
    }

    fn tier_mut(&mut self, tier: Tier) -> &mut EngineConfig {
        match tier {
            Tier::Fast => &mut self.fast,
            Tier::Deep => &mut self.deep,
        }
    }
}

/// `[engines.<tier>]`: the same keys for every kind, only the values differ.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EngineConfig {
    /// What answers.
    pub kind: EngineKind,
    /// Its base URL. Engines are dialled directly, like databases.
    pub url: String,
    /// The model the tier serves, as the engine names it.
    pub model: String,
    /// Whole-request deadline for a generation or an embedding.
    pub timeout_secs: u64,
    /// The embedding model, when the tier embeds at all. Empty: the tier does
    /// not embed, and `Embed` on it is refused before any engine is asked.
    pub embed_model: String,
    /// Requests the tier runs at once: what the engine really runs in
    /// parallel, so its own hidden queue stays empty. At least 1.
    pub max_in_flight: usize,
    /// Requests that may wait for a slot; one more is refused at once. 0: no
    /// line, a busy tier refuses immediately.
    pub max_queued: usize,
    /// How long a request waits in line before it is refused.
    #[serde(with = "humantime_serde")]
    pub queue_timeout: Duration,
    /// The chat template's arguments when a request asks the model to reason
    /// (llama.cpp's `chat_template_kwargs`). A model's switch is its template's:
    /// Qwen-style templates take `enable_thinking`, gpt-oss takes
    /// `reasoning_effort` and ignores `enable_thinking` (and, told `false`,
    /// reasons longer). Data, never a match on the model's name.
    #[serde(default = "thinking_on")]
    pub reasoning_on: TemplateArgs,
    /// The same when a request asks it not to reason.
    #[serde(default = "thinking_off")]
    pub reasoning_off: TemplateArgs,
}

/// A chat template's arguments, as the engine is sent them.
pub type TemplateArgs = BTreeMap<String, serde_json::Value>;

fn thinking(on: bool) -> TemplateArgs {
    BTreeMap::from([("enable_thinking".to_owned(), serde_json::Value::Bool(on))])
}

fn thinking_on() -> TemplateArgs {
    thinking(true)
}

fn thinking_off() -> TemplateArgs {
    thinking(false)
}

fn effort(level: &str) -> TemplateArgs {
    BTreeMap::from([(
        "reasoning_effort".to_owned(),
        serde_json::Value::String(level.to_owned()),
    )])
}

impl EngineConfig {
    /// The deadline as a duration.
    #[must_use]
    pub fn timeout(&self) -> Duration {
        Duration::from_secs(self.timeout_secs)
    }

    /// The model used for embeddings.
    #[must_use]
    pub fn embed_model(&self) -> &str {
        if self.embed_model.is_empty() {
            &self.model
        } else {
            &self.embed_model
        }
    }
}

/// `[generate]`: the bounds on one request.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Generate {
    /// Most messages in one request.
    pub max_messages: usize,
    /// Longest message content, in bytes.
    pub max_message_len: usize,
    /// The cap `max_tokens` is clamped to, and the default when absent.
    pub max_tokens_cap: u32,
    /// Most inputs in one `Embed`.
    pub max_embed_inputs: usize,
}

impl Default for Generate {
    fn default() -> Self {
        Self {
            max_messages: 64,
            max_message_len: 32_768,
            max_tokens_cap: 4096,
            max_embed_inputs: 64,
        }
    }
}

/// `[budget]`
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Budget {
    /// Tokens (prompt plus completion) one caller may spend per UTC day;
    /// 0 is no limit. Enforced only when generations are recorded.
    pub tokens_per_day: u64,
    /// Subjects the limit does not apply to: the platform's own instruments
    /// (the chaos tool's load subject), never a person. Their generations are
    /// still recorded.
    #[serde(default)]
    pub unlimited_subjects: Vec<String>,
}

impl Default for Budget {
    fn default() -> Self {
        Self {
            tokens_per_day: 200_000,
            unlimited_subjects: vec!["chaos-load".to_owned()],
        }
    }
}

/// `[store]`: the Postgres schema `llm` holds sessions and generations.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Store {
    /// Connection URL; a credential, so it comes from `LLM_DATABASE_URL` and
    /// is never written to a file or echoed. Empty: no store.
    #[serde(default, skip_serializing)]
    pub url: String,
    /// Pool size.
    pub max_connections: u32,
}

impl Default for Store {
    fn default() -> Self {
        Self {
            url: String::new(),
            max_connections: 5,
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

/// A configuration that cannot be served from.
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    /// A tier's URL does not parse.
    #[error("[engines.{tier}] url {url:?}: {reason}")]
    Url {
        /// The tier.
        tier: Tier,
        /// The value.
        url: String,
        /// Why.
        reason: String,
    },
    /// A tier's timeout or model is empty.
    #[error("[engines.{tier}] {what} must be set")]
    Empty {
        /// The tier.
        tier: Tier,
        /// The key.
        what: &'static str,
    },
    /// A stub engine in an environment that must not have one.
    #[error("[engines.{tier}] kind = \"stub\" is not allowed in environment {env:?}")]
    StubInProduction {
        /// The tier.
        tier: Tier,
        /// The environment.
        env: String,
    },
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

    /// Both tiers on the in-process stub, no store, no metrics listener: what
    /// tests and the chaos tool run, never a deployment.
    #[must_use]
    pub fn stub(listen: SocketAddr) -> Self {
        let mut engines = Engines::default();
        for tier in Tier::ALL {
            let e = engines.tier_mut(tier);
            e.kind = EngineKind::Stub;
            e.url = String::new();
            "stub-model".clone_into(&mut e.model);
            "stub-embed".clone_into(&mut e.embed_model);
            e.timeout_secs = 30;
            // The stub answers at once; tests that exercise admission set
            // their own bounds.
            e.max_in_flight = 64;
            e.max_queued = 64;
        }
        Self {
            server: Server { listen },
            metrics: Metrics { listen: None },
            ping: Ping::default(),
            engines,
            generate: Generate::default(),
            budget: Budget::default(),
            store: Store::default(),
            // No agents unless a test points here at some.
            agents: AgentsConfig {
                dir: PathBuf::new(),
            },
        }
    }

    /// The tier's engine configuration.
    #[must_use]
    pub fn engine(&self, tier: Tier) -> &EngineConfig {
        self.engines.tier(tier)
    }

    /// The stub engine is for tests and the chaos tool; a production file
    /// that names it is a mistake.
    ///
    /// # Errors
    /// A tier is the stub and `env` is `production`.
    pub fn refuse_stub(&self, env: &str) -> Result<(), ValidationError> {
        if env != "production" {
            return Ok(());
        }
        for tier in Tier::ALL {
            if self.engine(tier).kind == EngineKind::Stub {
                return Err(ValidationError::StubInProduction {
                    tier,
                    env: env.to_owned(),
                });
            }
        }
        Ok(())
    }

    /// Refuse a configuration that cannot serve: an engine URL that does not
    /// parse, an empty model, a zero timeout.
    ///
    /// # Errors
    /// The first problem found.
    pub fn validate(&self) -> Result<(), ValidationError> {
        for tier in Tier::ALL {
            let e = self.engine(tier);
            if e.model.is_empty() {
                return Err(ValidationError::Empty {
                    tier,
                    what: "model",
                });
            }
            if e.timeout_secs == 0 {
                return Err(ValidationError::Empty {
                    tier,
                    what: "timeout_secs",
                });
            }
            if e.max_in_flight == 0 {
                return Err(ValidationError::Empty {
                    tier,
                    what: "max_in_flight",
                });
            }
            match e.kind {
                EngineKind::Stub => {}
                EngineKind::Ollama | EngineKind::Llamacpp => {
                    url::Url::parse(&e.url).map_err(|err| ValidationError::Url {
                        tier,
                        url: e.url.clone(),
                        reason: err.to_string(),
                    })?;
                }
            }
        }
        Ok(())
    }
}

/// Flags that override the loaded configuration. Every one has an environment
/// variable. There is no flag for an engine's *kind* on purpose.
#[derive(Args, Debug, Clone)]
pub struct Overrides {
    /// Environment: picks `<config-dir>/<env>.toml` to merge over `base.toml`.
    #[arg(long, env = tbd_common::config::ENV_VAR, default_value = tbd_common::config::DEFAULT_ENV, global = true)]
    pub env: String,
    /// Directory holding `base.toml` and one file per environment.
    #[arg(long, env = "LLM_CONFIG_DIR", default_value = DEFAULT_DIR, global = true)]
    pub config_dir: PathBuf,
    /// Address the gRPC server binds. Default: `[server] listen`.
    #[arg(long, env = "LLM_LISTEN_ADDR")]
    pub listen_addr: Option<SocketAddr>,
    /// Prometheus `/metrics` listener. Default: `[metrics] listen`.
    #[arg(long, env = "LLM_METRICS_ADDR")]
    pub metrics_addr: Option<SocketAddr>,
    /// Postgres URL. A credential, so it is never echoed.
    #[arg(long, env = "LLM_DATABASE_URL", hide_env_values = true)]
    pub database_url: Option<String>,
    /// The fast tier's engine URL. Default: `[engines.fast] url`.
    #[arg(long, env = "LLM_FAST_URL")]
    pub fast_url: Option<String>,
    /// The fast tier's model. Default: `[engines.fast] model`.
    #[arg(long, env = "LLM_FAST_MODEL")]
    pub fast_model: Option<String>,
    /// The deep tier's engine URL. Default: `[engines.deep] url`.
    #[arg(long, env = "LLM_DEEP_URL")]
    pub deep_url: Option<String>,
    /// The deep tier's model. Default: `[engines.deep] model`.
    #[arg(long, env = "LLM_DEEP_MODEL")]
    pub deep_model: Option<String>,
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
        if let Some(v) = &self.fast_url {
            config.engines.fast.url.clone_from(v);
        }
        if let Some(v) = &self.fast_model {
            config.engines.fast.model.clone_from(v);
        }
        if let Some(v) = &self.deep_url {
            config.engines.deep.url.clone_from(v);
        }
        if let Some(v) = &self.deep_model {
            config.engines.deep.model.clone_from(v);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Every shipped environment file loads over `base.toml`, validates, and
    /// none of them selects the stub engine: that one is for tests only.
    #[test]
    fn every_shipped_env_file_loads_and_names_real_engines() {
        let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/llm");
        for env in ["local", "dev", "production"] {
            let (config, source) = Config::load(&dir, env).unwrap_or_else(|e| panic!("{env}: {e}"));
            assert_eq!(source.env, env);
            assert_eq!(source.files.len(), 2);
            assert!(config.ping.max_message_len > 0);
            config.validate().unwrap_or_else(|e| panic!("{env}: {e}"));
            config
                .refuse_stub(env)
                .unwrap_or_else(|e| panic!("{env}: {e}"));
            for tier in Tier::ALL {
                assert_ne!(
                    config.engine(tier).kind,
                    EngineKind::Stub,
                    "{env}: [engines.{tier}] must not be the stub"
                );
            }
        }
    }

    #[test]
    fn the_stub_configuration_is_refused_in_production_only() {
        let config = Config::stub("127.0.0.1:0".parse().unwrap_or_else(|_| unreachable!()));
        assert!(config.validate().is_ok());
        assert!(config.refuse_stub("local").is_ok());
        assert!(matches!(
            config.refuse_stub("production"),
            Err(ValidationError::StubInProduction { .. })
        ));
    }

    #[test]
    fn a_bad_engine_url_is_refused() {
        let mut config = Config::default_for_test();
        config.engines.fast.url = "not a url".to_owned();
        assert!(matches!(
            config.validate(),
            Err(ValidationError::Url { .. })
        ));
    }

    impl Config {
        fn default_for_test() -> Self {
            Self {
                server: Server {
                    listen: "127.0.0.1:0".parse().unwrap_or_else(|_| unreachable!()),
                },
                metrics: Metrics::default(),
                ping: Ping::default(),
                engines: Engines::default(),
                generate: Generate::default(),
                budget: Budget::default(),
                store: Store::default(),
                agents: AgentsConfig::default(),
            }
        }
    }
}
