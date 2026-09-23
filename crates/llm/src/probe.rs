//! Each tier's engine is asked, every `[engines] probe_interval`, whether it
//! is up and what it is: its build and the revision of the weights it serves.
//! The verdict is a gauge (`tbd_llm_engine_up`) and what `ListModels` reports;
//! the identity is what every generation's record names, so a benchmark can
//! be rerun against the same build and the same weights. The probe never
//! drives the gRPC health status, because an engine restart must not read as
//! an outage of the service (`Ping`, `ListModels` and `GetBudget` still answer).

use std::{
    collections::BTreeMap,
    sync::{Arc, RwLock},
    time::Duration,
};

use tbd_common::metrics::names;

use crate::{
    config::Tier,
    engine::{Engine, Identity},
};

/// What the last probe learned about one tier.
#[derive(Debug, Clone, Default)]
struct TierStatus {
    up: bool,
    identity: Identity,
}

/// The last probe's verdict and identity per tier.
#[derive(Debug, Clone, Default)]
pub struct Status {
    tiers: Arc<RwLock<BTreeMap<Tier, TierStatus>>>,
}

impl Status {
    /// Whether the tier answered its last probe. Unknown (no probe yet) is down.
    #[must_use]
    pub fn up(&self, tier: Tier) -> bool {
        self.tiers
            .read()
            .is_ok_and(|m| m.get(&tier).is_some_and(|t| t.up))
    }

    /// The tier's engine build and model revision as last learned; empty
    /// strings until a probe answered.
    #[must_use]
    pub fn identity(&self, tier: Tier) -> Identity {
        self.tiers
            .read()
            .ok()
            .and_then(|m| m.get(&tier).map(|t| t.identity.clone()))
            .unwrap_or_default()
    }

    fn set_up(&self, tier: Tier, up: bool) {
        if let Ok(mut m) = self.tiers.write() {
            m.entry(tier).or_default().up = up;
        }
    }

    fn set_identity(&self, tier: Tier, identity: Identity) {
        if let Ok(mut m) = self.tiers.write() {
            m.entry(tier).or_default().identity = identity;
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
            status.set_up(*tier, up);
            metrics::gauge!(names::LLM_ENGINE_UP, "tier" => tier.as_str(), "engine" => engine.kind().as_str())
                .set(if up { 1.0 } else { 0.0 });
            if up {
                // The identity changes when the engine or its weights do; a
                // probe that cannot read it keeps the last one rather than
                // pretending it went blank.
                match tokio::time::timeout(timeout, engine.identity()).await {
                    Ok(Ok(identity)) => {
                        if identity != status.identity(*tier) {
                            tracing::info!(%tier, engine = engine.kind().as_str(), build = %identity.engine_version, revision = %identity.model_revision, "engine identity");
                        }
                        status.set_identity(*tier, identity);
                    }
                    Ok(Err(error)) => {
                        tracing::warn!(%tier, engine = engine.kind().as_str(), %error, "the engine's identity could not be read");
                    }
                    Err(_) => {
                        tracing::warn!(%tier, engine = engine.kind().as_str(), "the engine's identity did not answer within {timeout:?}");
                    }
                }
            }
        }
    }
}
