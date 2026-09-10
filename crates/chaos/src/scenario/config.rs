//! The scenario file. Every table rejects unknown keys.

use std::path::Path;

use serde::{Deserialize, Serialize};

use crate::{load::LoadConfig, topology::StackConfig};

use super::{assertions::Assertions, timeline::TimelineEvent};

/// A whole scenario file.
#[derive(Debug, Clone, Deserialize, Serialize)]
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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
            if !self.stack.instances.values().any(|i| i.kind.load_target) {
                return Err(format!(
                    "load needs at least one instance load can target ({})",
                    crate::kinds::ALL
                        .iter()
                        .filter(|k| k.load_target)
                        .map(|k| k.name)
                        .collect::<Vec<_>>()
                        .join(", ")
                ));
            }
        }
        for event in &self.timeline {
            if let Some(service) = event.service()
                && self.stack.get(service).is_none()
            {
                return Err(format!("timeline references unknown service {service:?}"));
            }
            if matches!(event, TimelineEvent::SetBehavior { service, .. } if !self.stack.get(service).is_some_and(|i| i.kind.fault))
            {
                return Err("set_behavior targets an instance without fault injection".into());
            }
        }
        for name in self.assertions.services.keys() {
            if self.stack.get(name).is_none() {
                return Err(format!("assertions reference unknown service {name:?}"));
            }
        }
        if self.load.is_none() && self.assertions.needs_load() {
            return Err("load assertions given but no [load] section".into());
        }
        Ok(())
    }
}
