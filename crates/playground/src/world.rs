//! The sandbox everyone shares, and the rules of the game.
//!
//! One [`World`] owns a chaos [`Stack`] of real services, the traffic running
//! against it, the faults people have injected, and the rolling objective they
//! are trying to break. It is a single shared world on purpose: whoever
//! arrives sees whatever the last visitor did, and the world heals itself as
//! each fault expires, so it is always playable and never needs a reset.

use std::{
    collections::{BTreeMap, VecDeque},
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};

use tbd_lab::{enginelb::Balancer, kind, kinds::CORE, stack::Stack};
use tbd_proto::playground::v1::{
    ActiveFault, Budget, GetWorldResponse, Health, Instance, Move, Objective, Traffic,
};
use tokio::sync::{RwLock, broadcast};

use crate::{
    config::Game,
    faults::{self, Play},
    score::Scores,
};

/// A fault somebody injected, and when it lapses.
#[derive(Debug, Clone)]
struct Live {
    id: u64,
    mv: Move,
    instance: String,
    actor: String,
    until: Instant,
}

/// One second of traffic, kept only long enough to fill the window.
///
/// The p99 is this second's, not the run's: percentiles cannot be subtracted,
/// so the generator's histogram is drained every second and what comes out is
/// a slice rather than a running total.
#[derive(Debug, Clone, Copy)]
struct Sample {
    requests: u64,
    failures: u64,
    p99_ms: f64,
}

/// What sits in front of a kind's replicas, where a deployed environment has
/// Envoy. A kind with no entry here is addressed directly.
#[derive(Debug, Default)]
pub struct Balancers {
    inner: BTreeMap<&'static str, Fronted>,
}

/// One balancer and the address load should use instead of an instance's.
#[derive(Debug)]
struct Fronted {
    url: String,
    balancer: Arc<Balancer>,
}

impl Balancers {
    /// Put `balancer`, reachable at `url`, in front of every instance of `kind`.
    pub fn insert(&mut self, kind: &'static str, url: String, balancer: Arc<Balancer>) {
        self.inner.insert(kind, Fronted { url, balancer });
    }

    /// Whether anything fronts this kind.
    fn fronts(&self, kind: &str) -> bool {
        self.inner.contains_key(kind)
    }
}

/// Everything the game mutates, under one lock.
#[derive(Debug)]
struct State {
    faults: Vec<Live>,
    window: VecDeque<Sample>,
    tokens: u32,
    last_refill: Instant,
    health: Health,
    since: Instant,
    /// The last moment nothing was injected — a breach is timed from here.
    healed_at: Instant,
    traffic: Traffic,
}

/// What the window says, worked out once.
///
/// Both the game's verdict and the page's numbers come from here, so the bar a
/// player is watching is by construction the one being judged.
#[derive(Debug, Clone, Copy)]
struct Verdict {
    /// Share of them that succeeded, 0 to 1.
    success: f64,
    /// Mean of the per-second p99s, over the seconds that carried traffic.
    /// A mean and not a maximum: one slow second is not a breach.
    p99_ms: f64,
    /// Seconds until the last offending second leaves the window — which is how
    /// long a healed world still reads as broken.
    healing_seconds: u32,
    /// Whether both clauses hold.
    holds: bool,
}

/// The shared sandbox.
#[derive(Debug)]
pub struct World {
    game: Game,
    stack: RwLock<Stack>,
    state: RwLock<State>,
    events: broadcast::Sender<GetWorldResponse>,
    scores: Scores,
    next_id: AtomicU64,
    balancers: Balancers,
}

/// Why a move was refused. Each one is a sentence a player can act on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// The move is not on the list.
    UnknownMove,
    /// Not enough budget left.
    NoBudget {
        /// What the move costs.
        need: u32,
        /// What is left.
        have: u32,
    },
    /// No instance suits the move, or the named one does not exist.
    NoTarget(String),
    /// Taking this one down would leave nothing serving.
    LastOneStanding,
    /// That instance already has this exact fault.
    AlreadyFaulted,
}

impl Refusal {
    /// The reason, as the wire carries it.
    #[must_use]
    pub fn reason(&self) -> String {
        match self {
            Self::UnknownMove => "that is not a move".to_owned(),
            Self::NoBudget { need, have } => {
                format!("that costs {need} and the budget is down to {have}")
            }
            Self::NoTarget(what) => format!("nothing to aim at: {what}"),
            Self::LastOneStanding => {
                "that would leave nothing running, and then there is no game".to_owned()
            }
            Self::AlreadyFaulted => "it is already suffering that".to_owned(),
        }
    }
}

impl World {
    /// Build the world around a started stack, with nothing balanced.
    pub fn new(game: Game, stack: Stack, scores: Scores) -> Arc<Self> {
        Self::fronted(game, stack, scores, Balancers::default())
    }

    /// Build the world with balancers in front of some of its kinds.
    pub fn fronted(game: Game, stack: Stack, scores: Scores, balancers: Balancers) -> Arc<Self> {
        let (events, _) = broadcast::channel(32);
        let now = Instant::now();
        Arc::new(Self {
            game,
            stack: RwLock::new(stack),
            state: RwLock::new(State {
                faults: Vec::new(),
                window: VecDeque::new(),
                tokens: game.max_tokens,
                last_refill: now,
                health: Health::Healed,
                since: now,
                healed_at: now,
                traffic: Traffic::default(),
            }),
            events,
            scores,
            next_id: AtomicU64::new(1),
            balancers,
        })
    }

    /// Subscribe to the tick. A late subscriber gets the next tick, not the
    /// backlog; the payload is a whole world, so one frame is enough to catch up.
    pub fn watch(&self) -> broadcast::Receiver<GetWorldResponse> {
        self.events.subscribe()
    }

    /// The scoreboard.
    pub fn scores(&self) -> &Scores {
        &self.scores
    }

    /// Targets for the load generator, derived from the stack and nowhere else.
    /// No caller-supplied address can reach it.
    ///
    /// A balanced kind is addressed once, at its balancer, instead of once per
    /// replica — otherwise load would pin itself to instances and route around
    /// nothing, which is the whole point of having the balancer there.
    pub async fn targets(&self) -> Vec<tbd_lab::load::Target> {
        let mut targets: Vec<tbd_lab::load::Target> =
            kind::load_targets(&*self.stack.read().await, CORE)
                .into_iter()
                .filter(|t| !self.balancers.fronts(&t.kind))
                .collect();
        for (kind, fronted) in &self.balancers.inner {
            targets.push(tbd_lab::load::Target {
                name: format!("{kind}-lb"),
                http_url: fronted.url.clone(),
                kind: (*kind).to_owned(),
            });
        }
        targets
    }

    /// Record a second of traffic, as the load generator measured it.
    pub async fn observe(&self, requests: u64, failures: u64, traffic: Traffic) {
        let mut state = self.state.write().await;
        state.window.push_back(Sample {
            requests,
            failures,
            p99_ms: traffic.p99_ms,
        });
        while state.window.len() > self.game.window_seconds as usize {
            state.window.pop_front();
        }
        state.traffic = traffic;
    }

    /// One tick: expire what has lapsed, refill the budget, judge the world,
    /// and tell everyone watching.
    pub async fn tick(&self) {
        let now = Instant::now();
        self.expire(now).await;
        self.refill(now).await;
        let breach = self.judge(now).await;
        if let Some((actor, seconds, faults)) = breach {
            self.scores.record(&actor, seconds, faults).await;
        }
        let snapshot = self.snapshot().await;
        // No receivers is the normal case: nobody is watching right now.
        let _ = self.events.send(snapshot);
    }

    /// Drop faults whose time is up, healing whatever they were doing.
    async fn expire(&self, now: Instant) {
        let lapsed: Vec<Live> = {
            let mut state = self.state.write().await;
            let mut lapsed = Vec::new();
            state.faults.retain(|fault| {
                if fault.until <= now {
                    lapsed.push(fault.clone());
                    false
                } else {
                    true
                }
            });
            lapsed
        };
        if lapsed.is_empty() {
            return;
        }
        let mut stack = self.stack.write().await;
        for fault in lapsed {
            let outcome: Result<(), String> = match fault.mv {
                Move::Kill => stack
                    .start_instance(&fault.instance)
                    .await
                    .map(|_| ())
                    .map_err(|error| error.to_string()),
                Move::StallStore => stack
                    .set_store_behavior(&fault.instance, tbd_common::fault::Behavior::Healthy)
                    .map_err(|error| error.to_string()),
                _ => stack
                    .set_behavior(&fault.instance, tbd_common::fault::Behavior::Healthy)
                    .map_err(|error| error.to_string()),
            };
            if let Err(error) = outcome {
                tracing::warn!(instance = %fault.instance, %error, "could not heal an instance");
            }
        }
    }

    /// One token per `refill_seconds`, never above the cap.
    async fn refill(&self, now: Instant) {
        let mut state = self.state.write().await;
        let per = Duration::from_secs(u64::from(self.game.refill_seconds));
        if per.is_zero() || state.tokens >= self.game.max_tokens {
            state.last_refill = now;
            return;
        }
        let elapsed = now.saturating_duration_since(state.last_refill);
        let earned = u32::try_from(elapsed.as_secs() / per.as_secs()).unwrap_or(u32::MAX);
        if earned > 0 {
            state.tokens = (state.tokens + earned).min(self.game.max_tokens);
            state.last_refill = now;
        }
    }

    /// Decide how the world is doing. Returns the score to record when this
    /// tick is the moment it broke.
    async fn judge(&self, now: Instant) -> Option<(String, f64, u32)> {
        let mut state = self.state.write().await;
        // With no traffic yet there is nothing to judge; hold the last verdict.
        let verdict = self.verdict(&state);

        let health = if verdict.holds {
            if state.faults.is_empty() {
                Health::Healed
            } else {
                Health::Degraded
            }
        } else {
            Health::Breached
        };

        if health == state.health {
            return None;
        }

        let was = state.health;
        state.health = health;
        state.since = now;
        if health == Health::Healed {
            state.healed_at = now;
        }

        // Credit the breach to whoever injected the most recent fault.
        if health == Health::Breached && was != Health::Breached {
            let actor = state
                .faults
                .last()
                .map_or_else(|| "anonymous".to_owned(), |f| f.actor.clone());
            let seconds = now.saturating_duration_since(state.healed_at).as_secs_f64();
            let used = u32::try_from(state.faults.len()).unwrap_or(u32::MAX);
            tracing::info!(%actor, seconds, used, "the objective broke");
            return Some((actor, seconds, used));
        }
        None
    }

    /// Judge the window against both clauses of the objective.
    #[allow(clippy::cast_precision_loss)]
    fn verdict(&self, state: &State) -> Verdict {
        let requests: u64 = state.window.iter().map(|s| s.requests).sum();
        let failures: u64 = state.window.iter().map(|s| s.failures).sum();
        let success = if requests == 0 {
            1.0
        } else {
            (requests - failures) as f64 / requests as f64
        };

        // Only seconds that carried traffic have a meaningful percentile; a
        // quiet second reports 0 ms and would drag the mean down.
        let busy: Vec<f64> = state
            .window
            .iter()
            .filter(|s| s.requests > 0)
            .map(|s| s.p99_ms)
            .collect();
        let p99_ms = if busy.is_empty() {
            0.0
        } else {
            busy.iter().sum::<f64>() / busy.len() as f64
        };

        // The newest second that broke either clause: everything older than it
        // leaves the window first, so this is what the world is waiting on.
        //
        // An entry at `index` is dropped once that many pushes have gone past
        // it — plus however many the window still needs to reach its capacity,
        // because nothing is evicted until it is full. A window that is still
        // filling really does take longer to forget, and saying otherwise would
        // count down to a moment that had not arrived.
        let offending = state.window.iter().enumerate().filter(|(_, s)| {
            s.failures > 0 || (s.requests > 0 && s.p99_ms > self.game.latency_target_ms)
        });
        let room = (self.game.window_seconds as usize).saturating_sub(state.window.len());
        let healing_seconds = offending.last().map_or(0, |(index, _)| {
            u32::try_from(index + 1 + room).unwrap_or(u32::MAX)
        });

        let holds = requests == 0
            || (success >= self.game.target
                && (busy.is_empty() || p99_ms <= self.game.latency_target_ms));

        Verdict {
            success,
            p99_ms,
            healing_seconds,
            holds,
        }
    }

    /// Spend budget on a move and apply it.
    ///
    /// # Errors
    /// A [`Refusal`] the caller can act on; the world is unchanged.
    pub async fn inject(
        &self,
        mv: Move,
        instance: Option<&str>,
        actor: &str,
    ) -> Result<(u32, GetWorldResponse), Refusal> {
        let play = faults::play(mv).ok_or(Refusal::UnknownMove)?;
        let target = self.choose(mv, play, instance).await?;

        {
            let mut state = self.state.write().await;
            if state.tokens < play.cost {
                return Err(Refusal::NoBudget {
                    need: play.cost,
                    have: state.tokens,
                });
            }
            if state
                .faults
                .iter()
                .any(|f| f.instance == target && f.mv == mv)
            {
                return Err(Refusal::AlreadyFaulted);
            }
            state.tokens -= play.cost;
        }

        let applied: Result<(), String> = {
            let mut stack = self.stack.write().await;
            match mv {
                Move::Kill => stack
                    .stop_instance(&target)
                    .await
                    .map_err(|error| error.to_string()),
                Move::StallStore => faults::behavior(mv)
                    .ok_or_else(|| "no behaviour".to_owned())
                    .and_then(|b| {
                        stack
                            .set_store_behavior(&target, b)
                            .map_err(|error| error.to_string())
                    }),
                _ => faults::behavior(mv)
                    .ok_or_else(|| "no behaviour".to_owned())
                    .and_then(|b| {
                        stack
                            .set_behavior(&target, b)
                            .map_err(|error| error.to_string())
                    }),
            }
        };

        if let Err(error) = applied {
            // Give the tokens back: nothing happened to the world.
            let mut state = self.state.write().await;
            state.tokens = (state.tokens + play.cost).min(self.game.max_tokens);
            tracing::warn!(instance = %target, %error, "could not apply a move");
            return Err(Refusal::NoTarget(target));
        }

        {
            let mut state = self.state.write().await;
            state.faults.push(Live {
                id: self.next_id.fetch_add(1, Ordering::Relaxed),
                mv,
                instance: target,
                actor: actor.to_owned(),
                until: Instant::now() + play.ttl,
            });
        }

        let snapshot = self.snapshot().await;
        let _ = self.events.send(snapshot.clone());
        Ok((play.cost, snapshot))
    }

    /// Pick what to hit: the named instance if it suits, else one that does.
    async fn choose(&self, mv: Move, play: Play, wanted: Option<&str>) -> Result<String, Refusal> {
        let stack = self.stack.read().await;
        let described = stack.describe();
        let suitable = |i: &tbd_lab::stack::InstanceInfo| {
            if let Some(kind) = play.kind
                && i.kind != kind
            {
                return false;
            }
            match mv {
                Move::Kill => i.running,
                Move::StallStore => i.running && i.store_behavior.is_some(),
                _ => i.running && i.behavior.is_some(),
            }
        };

        if mv == Move::Kill {
            let running = described.iter().filter(|i| i.running).count();
            if running <= 1 {
                return Err(Refusal::LastOneStanding);
            }
        }

        if let Some(name) = wanted.filter(|n| !n.is_empty()) {
            return described
                .iter()
                .find(|i| i.name == name && suitable(i))
                .map(|i| i.name.clone())
                .ok_or_else(|| Refusal::NoTarget(name.to_owned()));
        }

        described
            .iter()
            .find(|i| suitable(i))
            .map(|i| i.name.clone())
            .ok_or_else(|| Refusal::NoTarget(faults::describe(mv).to_owned()))
    }

    /// The whole world, as the wire carries it.
    pub async fn snapshot(&self) -> GetWorldResponse {
        let described = self.stack.read().await.describe();
        // Every balancer's view of its replicas, gathered before the state lock
        // so a slow one cannot hold up the tick.
        let mut rotation: BTreeMap<String, bool> = BTreeMap::new();
        let mut ejected: BTreeMap<String, bool> = BTreeMap::new();
        for fronted in self.balancers.inner.values() {
            for endpoint in fronted.balancer.status().await {
                rotation.insert(endpoint.name.clone(), endpoint.in_rotation);
                ejected.insert(endpoint.name, endpoint.ejected);
            }
        }
        let state = self.state.read().await;
        let now = Instant::now();

        let ailing: BTreeMap<&str, &Live> = state
            .faults
            .iter()
            .map(|f| (f.instance.as_str(), f))
            .collect();

        let instances = described
            .iter()
            .map(|i| Instance {
                name: i.name.clone(),
                kind: i.kind.clone(),
                running: i.running,
                ailment: ailing
                    .get(i.name.as_str())
                    .map_or_else(String::new, |f| faults::describe(f.mv).to_owned()),
                requests_total: i.requests.map_or(0, |r| r.total),
                requests_failed: i.requests.map_or(0, |r| r.failed),
                balanced: self.balancers.fronts(&i.kind),
                in_rotation: rotation.get(&i.name).copied().unwrap_or(false),
                ejected: ejected.get(&i.name).copied().unwrap_or(false),
            })
            .collect();

        let faults = state
            .faults
            .iter()
            .map(|f| ActiveFault {
                id: f.id.to_string(),
                r#move: f.mv as i32,
                instance: f.instance.clone(),
                description: faults::describe(f.mv).to_owned(),
                expires_in_seconds: u32::try_from(f.until.saturating_duration_since(now).as_secs())
                    .unwrap_or(u32::MAX),
                actor: f.actor.clone(),
            })
            .collect();

        let verdict = self.verdict(&state);

        let refill_in = if state.tokens >= self.game.max_tokens {
            0
        } else {
            let per = Duration::from_secs(u64::from(self.game.refill_seconds));
            let waited = now.saturating_duration_since(state.last_refill);
            u32::try_from(per.saturating_sub(waited).as_secs()).unwrap_or(u32::MAX)
        };

        GetWorldResponse {
            world: Some(tbd_proto::playground::v1::World {
                health: state.health as i32,
                objective: Some(Objective {
                    target: self.game.target,
                    current: verdict.success,
                    window_seconds: self.game.window_seconds,
                    latency_target_ms: self.game.latency_target_ms,
                    latency_current_ms: verdict.p99_ms,
                }),
                budget: Some(Budget {
                    tokens: state.tokens,
                    max_tokens: self.game.max_tokens,
                    refill_in_seconds: refill_in,
                }),
                traffic: Some(state.traffic),
                instances,
                faults,
                held_for_seconds: u32::try_from(now.duration_since(state.since).as_secs())
                    .unwrap_or(u32::MAX),
                healing_seconds: verdict.healing_seconds,
                now: Some(std::time::SystemTime::now().into()),
            }),
        }
    }

    /// Stop everything. Used on shutdown.
    pub async fn shutdown(self: Arc<Self>) {
        if let Ok(world) = Arc::try_unwrap(self) {
            world.stack.into_inner().shutdown().await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn game() -> Game {
        Game {
            target: 0.99,
            window_seconds: 60,
            latency_target_ms: 250.0,
            max_tokens: 10,
            refill_seconds: 3,
            rate: 20.0,
        }
    }

    /// A world with no stack behind it: enough to exercise the arithmetic.
    async fn bare() -> Arc<World> {
        let stack = Stack::start(BTreeMap::new())
            .await
            .expect("an empty stack starts");
        World::new(game(), stack, Scores::in_memory(5))
    }

    #[tokio::test]
    async fn an_untouched_world_is_healed_and_full() {
        let world = bare().await;
        world.tick().await;
        let snap = world.snapshot().await;
        let w = snap.world.expect("a world");
        assert_eq!(w.health, Health::Healed as i32);
        assert_eq!(w.budget.expect("budget").tokens, 10);
        assert!((w.objective.expect("objective").current - 1.0).abs() < f64::EPSILON);
    }

    #[tokio::test]
    async fn the_objective_breaks_when_enough_requests_fail() {
        let world = bare().await;
        // Well under the 99% target.
        world.observe(100, 40, Traffic::default()).await;
        world.tick().await;
        let w = world.snapshot().await.world.expect("a world");
        assert_eq!(
            w.health,
            Health::Breached as i32,
            "40% failures must breach"
        );
    }

    #[tokio::test]
    async fn a_quiet_world_is_not_a_broken_one() {
        let world = bare().await;
        // No requests at all must never read as a breach.
        world.observe(0, 0, Traffic::default()).await;
        world.tick().await;
        let w = world.snapshot().await.world.expect("a world");
        assert_eq!(w.health, Health::Healed as i32);
    }

    #[tokio::test]
    async fn an_unknown_move_is_refused_before_anything_is_spent() {
        let world = bare().await;
        let err = world
            .inject(Move::Unspecified, None, "tester")
            .await
            .expect_err("unspecified is not a move");
        assert_eq!(err, Refusal::UnknownMove);
        let w = world.snapshot().await.world.expect("a world");
        assert_eq!(w.budget.expect("budget").tokens, 10, "nothing was spent");
    }

    #[tokio::test]
    async fn with_nothing_running_there_is_nothing_to_aim_at() {
        let world = bare().await;
        let err = world
            .inject(Move::Latency, None, "tester")
            .await
            .expect_err("an empty stack has no targets");
        assert!(matches!(err, Refusal::NoTarget(_)), "{err:?}");
    }

    #[tokio::test]
    async fn the_budget_refills_but_never_past_the_cap() {
        let world = bare().await;
        {
            let mut state = world.state.write().await;
            state.tokens = 0;
            state.last_refill = Instant::now()
                .checked_sub(Duration::from_secs(30))
                .expect("the clock started more than 30s ago");
        }
        world.refill(Instant::now()).await;
        let tokens = world.state.read().await.tokens;
        assert_eq!(tokens, 10, "30s at one per 3s tops out at the cap");
    }

    /// One second of traffic at a given p99, repeated.
    async fn seconds(world: &World, count: usize, requests: u64, failures: u64, p99_ms: f64) {
        for _ in 0..count {
            world
                .observe(
                    requests,
                    failures,
                    Traffic {
                        p99_ms,
                        ..Traffic::default()
                    },
                )
                .await;
        }
    }

    #[tokio::test]
    async fn slow_but_succeeding_still_breaks_the_objective() {
        let world = bare().await;
        // Not one failed request, and well past the latency clause.
        seconds(&world, 30, 100, 0, 600.0).await;
        world.tick().await;
        let w = world.snapshot().await.world.expect("a world");
        assert_eq!(
            w.health,
            Health::Breached as i32,
            "latency is half the objective"
        );
        let objective = w.objective.expect("objective");
        assert!(
            (objective.current - 1.0).abs() < f64::EPSILON,
            "every request succeeded"
        );
        assert!(objective.latency_current_ms > objective.latency_target_ms);
    }

    #[tokio::test]
    async fn latency_under_the_target_is_not_a_breach() {
        let world = bare().await;
        seconds(&world, 30, 100, 0, 200.0).await;
        world.tick().await;
        let w = world.snapshot().await.world.expect("a world");
        assert_eq!(w.health, Health::Healed as i32);
    }

    #[tokio::test]
    async fn one_slow_second_does_not_breach_a_healthy_window() {
        let world = bare().await;
        seconds(&world, 29, 100, 0, 10.0).await;
        // A single spike, far over the target.
        seconds(&world, 1, 100, 0, 5_000.0).await;
        world.tick().await;
        let w = world.snapshot().await.world.expect("a world");
        assert_eq!(
            w.health,
            Health::Healed as i32,
            "the clause is a mean, so one spike is absorbed"
        );
    }

    #[tokio::test]
    async fn healing_counts_down_as_failures_age_out() {
        let world = bare().await;
        seconds(&world, 5, 100, 40, 10.0).await;
        world.tick().await;
        let before = world.snapshot().await.world.expect("a world");
        assert_eq!(before.health, Health::Breached as i32);
        let started = before.healing_seconds;
        assert!(started > 0, "a breached window is waiting on something");

        // Clean traffic pushes the failures towards the back of the window.
        seconds(&world, 2, 100, 0, 10.0).await;
        let after = world.snapshot().await.world.expect("a world");
        assert!(
            after.healing_seconds < started,
            "healing shortens as failures age out: {} then {}",
            started,
            after.healing_seconds
        );
    }

    #[tokio::test]
    async fn a_world_with_nothing_wrong_is_not_healing() {
        let world = bare().await;
        seconds(&world, 10, 100, 0, 10.0).await;
        world.tick().await;
        let w = world.snapshot().await.world.expect("a world");
        assert_eq!(w.healing_seconds, 0);
    }

    #[tokio::test]
    async fn an_unbalanced_world_reports_no_rotation() {
        let world = bare().await;
        let w = world.snapshot().await.world.expect("a world");
        assert!(
            w.instances.iter().all(|i| !i.balanced),
            "nothing fronts a bare stack, and the page must be able to tell"
        );
    }

    #[test]
    fn every_refusal_says_something_useful() {
        for refusal in [
            Refusal::UnknownMove,
            Refusal::NoBudget { need: 3, have: 1 },
            Refusal::NoTarget("ledger-1".to_owned()),
            Refusal::LastOneStanding,
            Refusal::AlreadyFaulted,
        ] {
            let reason = refusal.reason();
            assert!(reason.len() > 10, "{refusal:?} -> {reason:?}");
            assert!(!reason.ends_with('.'), "reasons are phrases, not sentences");
        }
    }
}
