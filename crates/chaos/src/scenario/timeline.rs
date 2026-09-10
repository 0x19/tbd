//! Things that happen at a point in time while load runs.

use std::time::Duration;

use serde::Deserialize;
use tbd_common::fault::Behavior;

use crate::stack::Stack;

/// One `[[timeline]]` entry. `at` is measured from when load starts.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case", deny_unknown_fields)]
pub enum TimelineEvent {
    /// Change an engine's fault behaviour.
    SetBehavior {
        /// Offset from load start.
        #[serde(with = "humantime_serde")]
        at: Duration,
        /// Engine instance name.
        service: String,
        /// New behaviour.
        behavior: Behavior,
    },
    /// Stop an instance. Its port stays reserved.
    Stop {
        /// Offset from load start.
        #[serde(with = "humantime_serde")]
        at: Duration,
        /// Instance name.
        service: String,
    },
    /// Start a stopped instance on its previous port.
    Start {
        /// Offset from load start.
        #[serde(with = "humantime_serde")]
        at: Duration,
        /// Instance name.
        service: String,
    },
    /// Write a marker into the log and the report.
    Log {
        /// Offset from load start.
        #[serde(with = "humantime_serde")]
        at: Duration,
        /// Text.
        message: String,
    },
}

impl TimelineEvent {
    /// When it fires.
    pub fn at(&self) -> Duration {
        match self {
            Self::SetBehavior { at, .. }
            | Self::Stop { at, .. }
            | Self::Start { at, .. }
            | Self::Log { at, .. } => *at,
        }
    }

    /// The instance it targets, if any.
    pub fn service(&self) -> Option<&str> {
        match self {
            Self::SetBehavior { service, .. }
            | Self::Stop { service, .. }
            | Self::Start { service, .. } => Some(service),
            Self::Log { .. } => None,
        }
    }

    /// Short description for reports.
    pub fn describe(&self) -> String {
        match self {
            Self::SetBehavior {
                service, behavior, ..
            } => format!("set_behavior {service} {behavior:?}"),
            Self::Stop { service, .. } => format!("stop {service}"),
            Self::Start { service, .. } => format!("start {service}"),
            Self::Log { message, .. } => format!("log {message}"),
        }
    }

    /// Apply to a running stack.
    pub async fn apply(&self, stack: &mut Stack) -> Result<(), String> {
        match self {
            Self::SetBehavior {
                service, behavior, ..
            } => {
                let instance = stack
                    .get(service)
                    .ok_or_else(|| format!("{service} is not running"))?;
                let fault = instance
                    .fault()
                    .ok_or_else(|| format!("{service} has no fault injection"))?;
                fault.set(behavior.clone());
                Ok(())
            }
            Self::Stop { service, .. } => stack
                .stop_instance(service)
                .await
                .map_err(|e| e.to_string()),
            Self::Start { service, .. } => stack
                .start_instance(service)
                .await
                .map(|_| ())
                .map_err(|e| e.to_string()),
            Self::Log { .. } => Ok(()),
        }
    }
}
