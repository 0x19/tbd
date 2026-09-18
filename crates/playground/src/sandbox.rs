//! Starting the world: the stack, the traffic and the tick that drives them.
//!
//! Kept apart from [`crate::world`] so the game's rules can be tested without
//! starting four services, and so `serve_on` stays the plain gRPC server the
//! scaffolder wrote.
//!
//! The composition is fixed here rather than read from a topology file. It is
//! not a knob, and every part of it earns its place:
//!
//! - **Two engines behind a balancer**, because a single fault has to be
//!   survivable. Without the second replica every move is an instant breach and
//!   the game is one click long.
//! - **One ledger, unbalanced**, because a second one would not be redundancy —
//!   see `LEDGER` below. It is the sandbox's single point of failure on purpose.
//! - **A balancer per kind** ([`tbd_lab::enginelb`]), doing what Envoy does in
//!   a deployed environment: spreading requests, health checking, and ejecting
//!   a replica that keeps answering but starts failing. That last part is what
//!   makes an injected error rate survivable — a lying backend passes every
//!   health check there is.
//! - **A protocol instance** for the REST operations to drive, pointed at the
//!   engine balancer rather than at an engine.
//!
//! Every port is ephemeral — nothing outside this process addresses these
//! instances, and no caller ever names an address.

use std::{sync::Arc, time::Duration};

use tbd_lab::{
    enginelb::{self, Policy},
    kind::Kind,
    kinds::{CORE, engine, ledger, protocol},
    stack::{Launcher, Stack},
};
use tokio_util::sync::CancellationToken;

use crate::{
    Scores,
    config::Config,
    traffic,
    world::{Balancers, World},
};

/// How often the world expires faults, refills the budget and publishes.
const TICK: Duration = Duration::from_secs(1);

/// The engines: two, behind a balancer. One to take down, one to carry the
/// traffic while it is down.
const ENGINES: [&str; 2] = ["engine-1", "engine-2"];

/// The ledger: one, and deliberately not balanced.
///
/// Two would break the game rather than harden it. A ledger with no database
/// behind it keeps its facts in memory, so round-robin would append to one
/// store and read from the other, and `Current` would miss what `Append` had
/// just written. The sandbox measured this: a second ledger failed about 1.7%
/// of requests with nothing injected at all, which is a breach by itself.
///
/// Leaving it single is the honest version, and it is the more interesting
/// exhibit: **stateless replicas are cheap and stateful ones are not.** It is
/// the one part of the sandbox with no redundancy to hide behind, which is why
/// `stall store` is the move whose damage no balancer can absorb.
const LEDGER: &str = "ledger-1";

/// How quickly the balancer routes around a broken replica.
///
/// Tighter than the library's default, because here the cost of being slow is
/// measured: every request that fails before the balancer reacts counts against
/// the objective, and the objective allows 1% of a 30 s window. At 20 rps a
/// quarter of the traffic reaches any one engine, so a 250 ms probe interval
/// puts a stopped instance's blip near one request, and six requests of
/// evidence catches a lying one inside a second and a half.
///
/// Measured against the shipped configuration, one move at a time, worst
/// point of a 30 s window: kill 0.00% failed, latency 0.00% (p99 peaked at
/// 52 ms against a 250 ms target), stall store 0.52%, errors 0.86%. All four
/// under the budget, none a breach — which is what makes a single fault
/// survivable rather than nearly so. `tests/it/game.rs` holds it there.
fn policy() -> Policy {
    Policy {
        probe_interval: Duration::from_millis(250),
        window: Duration::from_secs(5),
        min_requests: 6,
        error_ratio: 0.15,
        cooldown: Duration::from_secs(15),
        // A fault lasts 30 s, so doubling the cooldown means a player pays for
        // one blip when it lands and at most one more before it lapses, rather
        // than a fresh one every ten seconds.
        max_cooldown: Duration::from_secs(120),
        // A replica four times slower than its twin is doing damage the twin is
        // not, which is what makes `latency` a move rather than a decoration:
        // it lands, the balancer routes around it, and the p99 recovers.
        slow_multiple: Some(4.0),
        slow_floor: Duration::from_millis(100),
    }
}

/// Everything the playground runs beside its gRPC server.
#[derive(Debug)]
pub struct Sandbox {
    /// The world the RPCs read and mutate.
    pub world: Arc<World>,
    /// Held, not dropped: a balancer stops serving the moment its handle goes,
    /// and the protocol would be left pointing at a closed port.
    balancers: Vec<enginelb::Running>,
    cancel: CancellationToken,
}

/// Why a sandbox could not start.
#[derive(Debug, thiserror::Error)]
pub enum StartError {
    /// A kind rejected the table built for it, which would be a bug here.
    #[error("{kind}: {source}")]
    Spec {
        /// The kind being built.
        kind: &'static str,
        /// What it objected to.
        source: toml::de::Error,
    },
    /// An instance would not start.
    #[error("stack: {0}")]
    Stack(#[from] tbd_lab::stack::StackError),
    /// A balancer would not bind, or was handed an address it could not parse.
    #[error("balancer: {0}")]
    Balancer(#[source] anyhow::Error),
}

/// The `(name, url)` pairs a balancer wants, for the instances that are up.
fn running(stack: &Stack, names: &[&str]) -> Vec<(String, String)> {
    names
        .iter()
        .filter_map(|name| {
            stack
                .get(name)
                .map(|instance| ((*name).to_owned(), instance.http_url()))
        })
        .collect()
}

/// One instance of `kind`, on any free port.
fn launcher(kind: &'static Kind, table: toml::Table) -> Result<Launcher, StartError> {
    kind.launcher(table, None)
        .map_err(|source| StartError::Spec {
            kind: kind.name,
            source,
        })
}

impl Sandbox {
    /// Start the stack, then set the traffic and the tick going.
    ///
    /// # Errors
    /// An instance refuses to start.
    pub async fn start(config: &Config) -> Result<Self, StartError> {
        // The replicas first. The protocol is not among them: it has to be
        // told where its engines are, and that address does not exist until the
        // balancer in front of them is listening.
        let mut launchers = std::collections::BTreeMap::new();
        for name in ENGINES {
            launchers.insert(
                name.to_owned(),
                launcher(&engine::KIND, toml::Table::new())?,
            );
        }
        launchers.insert(
            LEDGER.to_owned(),
            launcher(&ledger::KIND, toml::Table::new())?,
        );
        let mut stack = Stack::start(launchers).await?;

        // Then the balancers, over the addresses the stack just handed back.
        let engines = running(&stack, &ENGINES);
        let engine_lb = enginelb::start(tbd_lab::stack::ephemeral(), engines, policy())
            .await
            .map_err(StartError::Balancer)?;
        let engine_url = format!("http://{}", engine_lb.addr);

        // And last the protocol, pointed at the engine balancer rather than at
        // an engine, which is what production does through Envoy.
        let mut proto = toml::Table::new();
        proto.insert(
            "engine_url".to_owned(),
            toml::Value::String(engine_url.clone()),
        );
        stack
            .add_instance("protocol-1", launcher(&protocol::KIND, proto)?)
            .await?;

        let mut balancers = Balancers::default();
        balancers.insert(
            engine::KIND.name,
            engine_url,
            Arc::clone(&engine_lb.balancer),
        );

        tracing::info!(
            instances = stack.describe().len(),
            targets = tbd_lab::kind::load_targets(&stack, CORE).len(),
            "the sandbox is up and ready to be broken"
        );

        let scores = Scores::open(config.sandbox.scores.clone(), config.sandbox.keep_scores).await;
        let world = World::fronted(config.game, stack, scores, balancers);
        let cancel = CancellationToken::new();

        // The traffic the objective is measured on.
        tokio::spawn(traffic::run(
            Arc::clone(&world),
            config.game.rate,
            cancel.clone(),
        ));

        // The heartbeat: expire, refill, judge, publish.
        {
            let world = Arc::clone(&world);
            let cancel = cancel.clone();
            tokio::spawn(async move {
                let mut ticker = tokio::time::interval(TICK);
                ticker.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
                loop {
                    tokio::select! {
                        () = cancel.cancelled() => break,
                        _ = ticker.tick() => world.tick().await,
                    }
                }
            });
        }

        Ok(Self {
            world,
            balancers: vec![engine_lb],
            cancel,
        })
    }

    /// Stop the traffic and the tick, then the balancers, then the services.
    pub async fn shutdown(self) {
        self.cancel.cancel();
        for balancer in self.balancers {
            balancer.stop().await;
        }
        Arc::clone(&self.world).shutdown().await;
    }
}
