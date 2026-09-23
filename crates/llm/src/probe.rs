//! Each tier's engine is asked, every `[engines] probe_interval`, whether it
//! is up. The answer is a gauge (`tbd_llm_engine_up`) and the `up` flag
//! `ListModels` reports; it never drives the gRPC health status, because an
//! engine restart must not read as an outage of the service (`Ping`,
//! `ListModels` and `GetBudget` still answer).

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use tbd_common::metrics::names;

use crate::{config::Tier, engine::Engine};

/// The last probe's verdict per tier.
#[derive(Debug, Clone, Default)]
pub struct Status {
    up: Arc<RwLock<BTreeMap<Tier, bool>>>,
}

impl Status {
    /// Whether the tier answered its last probe. Unknown (no probe yet) is down.
    #[must_use]
    pub fn up(&self, tier: Tier) -> bool {
        self.up
            .read()
            .is_ok_and(|m| m.get(&tier).copied().unwrap_or(false))
    }

    fn set(&self, tier: Tier, up: bool) {
        if let Ok(mut m) = self.up.write() {
            m.insert(tier, up);
        }
    }
}

/// Probe every tier once, now, and then every `interval`, until aborted.
pub async fn run(
    engines: BTreeMap<Tier, Arc<dyn Engine>>,
    status: Status,
    interval: Duration,
    timeout: Duration,
) {
    let mut first = true;
    loop {
        if !first {
            tokio::time::sleep(interval).await;
        }
        first = false;
        for (tier, engine) in &engines {
            let before = status.up(*tier);
            let now = match tokio::time::timeout(timeout, engine.health()).await {
                Ok(Ok(models)) => {
                    if !models.iter().any(|m| m == engine.model()) {
                        tracing::warn!(%tier, engine = engine.kind().as_str(), model = engine.model(), served = ?models, "the engine does not list the configured model");
                    }
                    Ok(())
                }
                Ok(Err(e)) => Err(e.to_string()),
                Err(_) => Err(format!("no answer within {timeout:?}")),
            };
            match (&now, before) {
                (Ok(()), false) => {
                    tracing::info!(%tier, engine = engine.kind().as_str(), model = engine.model(), "engine up");
                }
                (Err(error), true) => {
                    tracing::warn!(%tier, engine = engine.kind().as_str(), %error, "engine down");
                }
                _ => {}
            }
            let up = now.is_ok();
            status.set(*tier, up);
            metrics::gauge!(names::LLM_ENGINE_UP, "tier" => tier.as_str(), "engine" => engine.kind().as_str())
                .set(if up { 1.0 } else { 0.0 });
        }
    }
}
