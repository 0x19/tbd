//! The claim the whole game rests on: **one fault is survivable, two are not.**
//!
//! These boot the real sandbox — four services, a balancer and the real load
//! generator — and break it the way a visitor can, through the same
//! [`World::inject`] the RPC calls. If a single move ever breaches on its own
//! again, the playground is back to being one click long and these fail.
//!
//! They run the **shipped** rate and window on purpose. The margin between a
//! blip and a breach is a fraction of a percent, and it depends on the ratio
//! of the balancer's reaction time to the window — change either and the
//! claim is about a different game. That makes these slow; they are the
//! acceptance test, and they run alongside each other.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{path::Path, time::Duration};

use tbd_playground::{Config, Sandbox};
use tbd_proto::playground::v1::{Health, Move, World};

/// Start the real sandbox on the shipped configuration, with the scoreboard
/// kept out of the tree.
async fn sandbox(name: &str) -> Sandbox {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../configs/playground");
    let (mut config, _) = Config::load(&dir, "local").unwrap();
    config.sandbox.scores = std::env::temp_dir().join(format!("playground-{name}.json"));
    Sandbox::start(&config).await.expect("the sandbox starts")
}

async fn world(sandbox: &Sandbox) -> World {
    sandbox.world.snapshot().await.world.expect("a world")
}

/// Let the window fill with traffic so the objective means something.
async fn settle(sandbox: &Sandbox) {
    let window = world(sandbox)
        .await
        .objective
        .expect("objective")
        .window_seconds;
    tokio::time::sleep(Duration::from_secs(u64::from(window) + 2)).await;
}

/// Watch the world for `secs`, and say whether it was ever breached.
async fn ever_breached(sandbox: &Sandbox, secs: u64) -> bool {
    let mut breached = false;
    for _ in 0..secs * 2 {
        tokio::time::sleep(Duration::from_millis(500)).await;
        breached |= world(sandbox).await.health == Health::Breached as i32;
    }
    breached
}

#[tokio::test(flavor = "multi_thread")]
async fn a_single_fault_is_survivable() {
    let sandbox = sandbox("single").await;
    settle(&sandbox).await;
    assert_eq!(
        world(&sandbox).await.health,
        Health::Healed as i32,
        "the sandbox holds its objective when nothing is wrong"
    );

    // An engine that keeps answering and keeps failing: the case no health
    // check catches, and the one outlier ejection exists for. Of the four
    // moves this is the one measured closest to the budget (0.86% of 1%), so
    // it is the one that guards the claim.
    sandbox
        .world
        .inject(Move::Errors, Some("engine-1"), "tester")
        .await
        .expect("errors is a legal move on engine-1");

    // Watch it through the ejection and one readmission.
    let breached = ever_breached(&sandbox, 20).await;
    let w = world(&sandbox).await;
    let objective = w.objective.expect("objective");
    assert!(
        !breached,
        "one fault must not breach: success now {:.4}, p99 {:.1}ms",
        objective.current, objective.latency_current_ms
    );

    let engine_1 = w
        .instances
        .iter()
        .find(|i| i.name == "engine-1")
        .expect("engine-1 is in the world");
    assert!(engine_1.balanced, "engines sit behind a balancer");
    assert!(
        engine_1.running,
        "ejected is not stopped: it is up, healthy, and lying"
    );

    sandbox.shutdown().await;
}

#[tokio::test(flavor = "multi_thread")]
async fn breaking_both_replicas_of_a_kind_breaches() {
    let sandbox = sandbox("both").await;
    settle(&sandbox).await;
    assert_eq!(world(&sandbox).await.health, Health::Healed as i32);

    // There is nowhere left for the balancer to send evaluate traffic.
    sandbox
        .world
        .inject(Move::Kill, Some("engine-1"), "tester")
        .await
        .expect("kill is legal while another engine runs");
    sandbox
        .world
        .inject(Move::Errors, Some("engine-2"), "tester")
        .await
        .expect("errors is legal on the survivor");

    let breached = ever_breached(&sandbox, 10).await;
    let objective = world(&sandbox).await.objective.expect("objective");
    assert!(
        breached,
        "with both engines gone the objective must break: success {:.4}",
        objective.current
    );

    sandbox.shutdown().await;
}
