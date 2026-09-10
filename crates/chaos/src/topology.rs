//! This project's topology: the TOML tables that describe which engines,
//! protocols and ledgers to run. A different project replaces this file and
//! keeps the rest.

use std::{collections::BTreeMap, net::SocketAddr, sync::Arc, time::Duration};

use serde::{Deserialize, Serialize};
use tbd_common::fault::Behavior;

use crate::{
    service::{engine::Engine, ledger::Ledger, protocol::Protocol},
    stack::{Launcher, Stack, ephemeral},
};

/// `[stack]` in a scenario or topology file.
#[derive(Debug, Clone, Default, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct StackConfig {
    /// `[stack.engines.<name>]`
    #[serde(default)]
    pub engines: BTreeMap<String, EngineSpec>,
    /// `[stack.protocols.<name>]`
    #[serde(default)]
    pub protocols: BTreeMap<String, ProtocolSpec>,
    /// `[stack.ledgers.<name>]`
    #[serde(default)]
    pub ledgers: BTreeMap<String, LedgerSpec>,
}

/// One ledger instance.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct LedgerSpec {
    /// Bind address. Omit for any free loopback port.
    #[serde(default)]
    pub listen: Option<SocketAddr>,
    /// Initial fault behaviour.
    #[serde(default)]
    pub behavior: Behavior,
}

/// One engine instance.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct EngineSpec {
    /// Bind address. Omit for any free loopback port.
    #[serde(default)]
    pub listen: Option<SocketAddr>,
    /// Initial fault behaviour.
    #[serde(default)]
    pub behavior: Behavior,
    /// Heartbeat interval on streams.
    #[serde(default = "default_heartbeat", with = "humantime_serde")]
    pub heartbeat: Duration,
}

fn default_heartbeat() -> Duration {
    Duration::from_secs(1)
}

/// One protocol instance.
#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolSpec {
    /// Bind address. Omit for any free loopback port.
    #[serde(default)]
    pub listen: Option<SocketAddr>,
    /// Engine instance to forward to.
    pub engine: String,
}

impl StackConfig {
    /// Turn the config into launchers the generic [`Stack`] can run.
    pub fn launchers(&self) -> BTreeMap<String, Launcher> {
        let mut out = BTreeMap::new();
        for (name, spec) in &self.engines {
            out.insert(
                name.clone(),
                Launcher {
                    service: Arc::new(Engine {
                        behavior: spec.behavior.clone(),
                        heartbeat: spec.heartbeat,
                    }),
                    listen: spec.listen.unwrap_or_else(ephemeral),
                },
            );
        }
        for (name, spec) in &self.protocols {
            out.insert(
                name.clone(),
                Launcher {
                    service: Arc::new(Protocol {
                        engine: spec.engine.clone(),
                    }),
                    listen: spec.listen.unwrap_or_else(ephemeral),
                },
            );
        }
        for (name, spec) in &self.ledgers {
            out.insert(
                name.clone(),
                Launcher {
                    service: Arc::new(Ledger {
                        behavior: spec.behavior.clone(),
                    }),
                    listen: spec.listen.unwrap_or_else(ephemeral),
                },
            );
        }
        out
    }

    /// Start the whole stack.
    pub async fn start(&self) -> Result<Stack, crate::stack::StackError> {
        Stack::start(self.launchers()).await
    }

    /// Referential check without starting anything.
    pub fn check(&self) -> Result<(), String> {
        for (name, p) in &self.protocols {
            if !self.engines.contains_key(&p.engine) {
                return Err(format!(
                    "protocol {name:?} references unknown engine {:?}",
                    p.engine
                ));
            }
            if self.engines.contains_key(name) {
                return Err(format!("{name:?} is both an engine and a protocol"));
            }
        }
        for name in self.ledgers.keys() {
            if self.engines.contains_key(name) || self.protocols.contains_key(name) {
                return Err(format!("{name:?} is a ledger and another kind"));
            }
        }
        if self.protocols.is_empty() && self.engines.is_empty() && self.ledgers.is_empty() {
            return Err("stack has no instances".to_owned());
        }
        Ok(())
    }
}

/// A file with only a stack in it, for `chaos up`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TopologyFile {
    /// Optional description.
    #[serde(default)]
    pub scenario: Option<toml::Value>,
    /// The stack.
    pub stack: StackConfig,
    /// Ignored here; scenarios use them.
    #[serde(default)]
    pub load: Option<toml::Value>,
    /// Ignored here; scenarios use them.
    #[serde(default)]
    pub timeline: Option<toml::Value>,
    /// Ignored here; scenarios use them.
    #[serde(default)]
    pub assertions: Option<toml::Value>,
}
