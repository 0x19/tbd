//! The world: everything the collectors last read, and the snapshot built
//! from it once a tick for every viewer.
//!
//! Collectors write their part and say whether their source answered; the
//! ticker turns the whole into one [`Snapshot`] and broadcasts it. A figure
//! nobody could read is absent in the snapshot, never zero, and `sources`
//! says which source is behind and why.

use std::{
    collections::BTreeMap,
    sync::{Arc, Mutex, PoisonError},
    time::{Duration, Instant, SystemTime},
};

use tbd_common::metrics::names;
use tbd_proto::arena::v1::{ChaosRun, Snapshot, SourceState, SurfaceState, TierState};
use tokio::sync::broadcast;

/// The sources, in the order `sources` lists them.
pub const SOURCES: [&str; 3] = ["llm", "metrics", "chaos"];

/// One tier's rates from the metrics store.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Rates {
    /// Completion tokens a second over the last minute.
    pub tokens_per_second: Option<f64>,
    /// Time to the first token, median, in milliseconds.
    pub ttft_p50_ms: Option<f64>,
    /// Time to the first token, 99th percentile, in milliseconds.
    pub ttft_p99_ms: Option<f64>,
    /// Requests admission refused in the last minute.
    pub refused_per_minute: Option<f64>,
}

#[derive(Debug, Clone)]
struct Source {
    ok: bool,
    last_ok: Option<Instant>,
    error: String,
}

#[derive(Debug, Default)]
struct State {
    tiers: Vec<TierState>,
    rates: BTreeMap<String, Rates>,
    surfaces: Vec<SurfaceState>,
    surfaces_checked_at: Option<SystemTime>,
    mcp_tools: Option<u32>,
    chaos: Option<ChaosRun>,
    sources: BTreeMap<&'static str, Source>,
}

/// Shared by the collectors, the ticker and the service. Cheap to clone.
#[derive(Debug, Clone)]
pub struct World {
    state: Arc<Mutex<State>>,
    events: broadcast::Sender<Snapshot>,
}

impl World {
    /// An empty world. `configured` names the sources that have a URL; the
    /// rest read "not configured" for as long as the process runs.
    #[must_use]
    pub fn new(configured: &[&'static str]) -> Self {
        let sources = SOURCES
            .into_iter()
            .map(|name| {
                let error = if configured.contains(&name) {
                    "not read yet"
                } else {
                    "not configured"
                };
                (
                    name,
                    Source {
                        ok: false,
                        last_ok: None,
                        error: error.to_owned(),
                    },
                )
            })
            .collect();
        let (events, _) = broadcast::channel(32);
        Self {
            state: Arc::new(Mutex::new(State {
                sources,
                ..State::default()
            })),
            events,
        }
    }

    fn with<T>(&self, f: impl FnOnce(&mut State) -> T) -> T {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        f(&mut state)
    }

    /// A source answered.
    pub fn source_ok(&self, name: &'static str) {
        self.with(|s| {
            s.sources.insert(
                name,
                Source {
                    ok: true,
                    last_ok: Some(Instant::now()),
                    error: String::new(),
                },
            );
        });
    }

    /// A source did not answer; what it gave last is kept, and marked stale.
    pub fn source_failed(&self, name: &'static str, error: impl Into<String>) {
        let error = error.into();
        self.with(|s| {
            let last_ok = s.sources.get(name).and_then(|x| x.last_ok);
            s.sources.insert(
                name,
                Source {
                    ok: false,
                    last_ok,
                    error,
                },
            );
        });
    }

    /// The model service's tiers, as `ListModels` answered.
    pub fn set_tiers(&self, tiers: Vec<TierState>) {
        self.with(|s| s.tiers = tiers);
    }

    /// The metrics store's rates, by tier.
    pub fn set_rates(&self, rates: BTreeMap<String, Rates>) {
        self.with(|s| s.rates = rates);
    }

    /// The last end-to-end check of every way in, and when it ran.
    pub fn set_surfaces(
        &self,
        surfaces: Vec<SurfaceState>,
        at: SystemTime,
        mcp_tools: Option<u32>,
    ) {
        self.with(|s| {
            s.surfaces = surfaces;
            s.surfaces_checked_at = Some(at);
            s.mcp_tools = mcp_tools;
        });
    }

    /// The chaos tool's current run, or `None` when the tool is not there.
    pub fn set_chaos(&self, chaos: Option<ChaosRun>) {
        self.with(|s| s.chaos = chaos);
    }

    /// Change the current run in place (a live frame of the run it follows).
    pub fn update_chaos(&self, f: impl FnOnce(&mut ChaosRun)) {
        self.with(|s| {
            if let Some(run) = s.chaos.as_mut() {
                f(run);
            }
        });
    }

    /// The id of the run being shown, when one is running.
    #[must_use]
    pub fn running_run(&self) -> Option<String> {
        self.with(|s| {
            s.chaos
                .as_ref()
                .filter(|c| c.state == "running")
                .map(|c| c.id.clone())
        })
    }

    /// The snapshot, now.
    #[must_use]
    pub fn snapshot(&self) -> Snapshot {
        let now = SystemTime::now();
        self.with(|s| {
            let tiers = s
                .tiers
                .iter()
                .cloned()
                .map(|mut t| {
                    let r = s.rates.get(&t.tier).copied().unwrap_or_default();
                    t.tokens_per_second = r.tokens_per_second;
                    t.ttft_p50_ms = r.ttft_p50_ms;
                    t.ttft_p99_ms = r.ttft_p99_ms;
                    t.refused_per_minute = r.refused_per_minute;
                    t
                })
                .collect();
            let sources = SOURCES
                .into_iter()
                .filter_map(|name| {
                    let src = s.sources.get(name)?;
                    let age = src.last_ok.map(|at| at.elapsed().as_secs_f64());
                    metrics::gauge!(names::ARENA_SOURCE_OK, "source" => name).set(if src.ok {
                        1.0
                    } else {
                        0.0
                    });
                    if let Some(age) = age {
                        metrics::gauge!(names::ARENA_SOURCE_AGE, "source" => name).set(age);
                    }
                    Some(SourceState {
                        name: name.to_owned(),
                        ok: src.ok,
                        age_s: age,
                        error: src.error.clone(),
                    })
                })
                .collect();
            Snapshot {
                now: Some(now.into()),
                tiers,
                surfaces: s.surfaces.clone(),
                surfaces_checked_at: s.surfaces_checked_at.map(Into::into),
                mcp_tools: s.mcp_tools,
                chaos: Some(s.chaos.clone().unwrap_or_else(|| ChaosRun {
                    state: "absent".to_owned(),
                    ..ChaosRun::default()
                })),
                sources,
            }
        })
    }

    /// A receiver of every snapshot from now on.
    #[must_use]
    pub fn watch(&self) -> broadcast::Receiver<Snapshot> {
        self.events.subscribe()
    }

    /// Build and send one snapshot every `tick`, forever. Nobody listening is
    /// normal and costs one snapshot.
    pub async fn tick(self, tick: Duration) {
        let mut every = tokio::time::interval(tick);
        every.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        loop {
            every.tick().await;
            let _ = self.events.send(self.snapshot());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unconfigured_source_says_so_and_a_missing_rate_is_absent() {
        let world = World::new(&["llm"]);
        world.set_tiers(vec![TierState {
            tier: "fast".into(),
            in_flight: 1,
            ..TierState::default()
        }]);
        let snap = world.snapshot();
        let metrics = snap.sources.iter().find(|s| s.name == "metrics").unwrap();
        assert!(!metrics.ok);
        assert_eq!(metrics.error, "not configured");
        assert_eq!(snap.tiers[0].tokens_per_second, None, "absent, never zero");
        assert_eq!(snap.chaos.unwrap().state, "absent");
    }

    #[test]
    fn a_failed_read_keeps_the_last_figures_and_says_why() {
        let world = World::new(&["llm"]);
        world.source_ok("llm");
        world.source_failed("llm", "connection refused");
        let snap = world.snapshot();
        let llm = snap.sources.iter().find(|s| s.name == "llm").unwrap();
        assert!(!llm.ok);
        assert!(llm.age_s.is_some(), "the last good read is still dated");
        assert_eq!(llm.error, "connection refused");
    }
}
