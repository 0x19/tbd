//! The scenario file. Every table rejects unknown keys.

use std::path::Path;

use serde::Deserialize;

use crate::{load::LoadConfig, topology::StackConfig};

use super::{assertions::Assertions, timeline::TimelineEvent};

/// A whole scenario file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ScenarioFile {
    /// `[scenario]`
    pub scenario: Meta,
    /// `[stack]`
    pub stack: StackConfig,
    /// `[load]`, optional: a scenario may only exercise the timeline.
    #[serde(default)]
    pub load: Option<LoadConfig>,
    /// `[[timeline]]`
    #[serde(default)]
    pub timeline: Vec<TimelineEvent>,
    /// `[assertions]`
    #[serde(default)]
    pub assertions: Assertions,
}

/// `[scenario]`
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Meta {
    /// Name shown in reports.
    pub name: String,
    /// What the scenario proves.
    #[serde(default)]
    pub description: String,
    /// Skip when running a directory. For scenarios that need something
    /// the CI box lacks.
    #[serde(default)]
    pub skip: bool,
}

impl ScenarioFile {
    /// Parse a file.
    pub fn from_path(path: &Path) -> anyhow::Result<Self> {
        let text = std::fs::read_to_string(path)
            .map_err(|e| anyhow::anyhow!("read {}: {e}", path.display()))?;
        let file: Self =
            toml::from_str(&text).map_err(|e| anyhow::anyhow!("parse {}: {e}", path.display()))?;
        file.check()
            .map_err(|e| anyhow::anyhow!("{}: {e}", path.display()))?;
        Ok(file)
    }

    /// Every check that does not need a running stack.
    pub fn check(&self) -> Result<(), String> {
        self.stack.check()?;
        if let Some(load) = &self.load {
            load.check()?;
            if self.stack.protocols.is_empty() {
                return Err("load needs at least one protocol instance to target".into());
            }
        }
        for event in &self.timeline {
            if let Some(service) = event.service()
                && !self.stack.engines.contains_key(service)
                && !self.stack.protocols.contains_key(service)
            {
                return Err(format!("timeline references unknown service {service:?}"));
            }
            if matches!(event, TimelineEvent::SetBehavior { service, .. } if !self.stack.engines.contains_key(service))
            {
                return Err("set_behavior targets an instance without fault injection".into());
            }
        }
        for name in self.assertions.services.keys() {
            if !self.stack.engines.contains_key(name) && !self.stack.protocols.contains_key(name) {
                return Err(format!("assertions reference unknown service {name:?}"));
            }
        }
        if self.load.is_none() && self.assertions.needs_load() {
            return Err("load assertions given but no [load] section".into());
        }
        Ok(())
    }
}
