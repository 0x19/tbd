//! The traffic the sandbox lives under.
//!
//! The real chaos load generator, driving the real instances, in phases that
//! restart so the world is never quiet for long. Its configuration is fixed
//! here and comes from nowhere else: a caller chooses moves, never a rate, a
//! duration or an address, so this loop is not reachable as an outbound
//! request primitive.

use std::{sync::Arc, time::Duration};

use tbd_lab::load::{
    LoadConfig, OperationWeight, Pattern,
    generator::{self, Hooks},
    ops::OpKind,
};
use tbd_proto::playground::v1::Traffic;
use tokio_util::sync::CancellationToken;

use crate::world::World;

/// How long one phase runs before the loop starts another.
const PHASE: Duration = Duration::from_secs(60);

/// How often the world is handed a slice of traffic. One second, because
/// the objective's window is counted in these samples.
const SAMPLE: Duration = Duration::from_secs(1);

/// The load, fixed. Modest on purpose: this shares a home cluster with
/// everything else, and the game is about faults rather than throughput.
fn config(rate: f64) -> LoadConfig {
    LoadConfig {
        rate,
        duration: PHASE,
        warmup: None,
        timeout: Duration::from_secs(2),
        max_in_flight: 64,
        pattern: Pattern::Constant,
        operations: vec![
            OperationWeight {
                op: OpKind::RestEvaluate,
                weight: 3,
            },
            OperationWeight {
                op: OpKind::LedgerAppend,
                weight: 1,
            },
            OperationWeight {
                op: OpKind::LedgerCurrent,
                weight: 2,
            },
        ],
        seed: 1,
        subjects: 32,
    }
}

/// Run load against the world until `cancel` fires, handing it one second of
/// traffic at a time. Each sample is drained, so it describes that second and
/// not the phase so far — which is what lets the objective judge a rolling
/// window and the page show a number that is true right now.
pub async fn run(world: Arc<World>, rate: f64, cancel: CancellationToken) {
    let config = config(rate);
    while !cancel.is_cancelled() {
        let targets = world.targets().await;
        if targets.is_empty() {
            tracing::warn!("no load targets in the sandbox; waiting");
            tokio::select! {
                () = cancel.cancelled() => return,
                () = tokio::time::sleep(Duration::from_secs(5)) => continue,
            }
        }

        let metrics = Arc::new(tbd_stress::metrics::Metrics::new());
        let hooks = Hooks {
            // No progress channel: a generator snapshot is cumulative for the
            // whole phase, so its error rate and percentiles describe the last
            // minute rather than the last second. The world needs "now", and
            // draining on our own clock is the only way to get it.
            progress: None,
            cancel: cancel.clone(),
        };

        let pump = {
            let world = Arc::clone(&world);
            let metrics = Arc::clone(&metrics);
            let cancel = cancel.clone();
            tokio::spawn(async move {
                let mut tick = tokio::time::interval(SAMPLE);
                tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Delay);
                loop {
                    tokio::select! {
                        () = cancel.cancelled() => return,
                        _ = tick.tick() => {}
                    }
                    let s = metrics.drain();
                    world
                        .observe(
                            s.requests_total,
                            s.requests_failed,
                            Traffic {
                                requests_per_second: s.throughput_rps,
                                error_rate: s.error_rate,
                                p50_ms: s.latency.p50_ms,
                                p99_ms: s.latency.p99_ms,
                            },
                        )
                        .await;
                }
            })
        };

        generator::run_with(
            &config,
            &targets,
            Arc::clone(&metrics),
            &hooks,
            &tbd_lab::tls::Trust::default(),
        )
        .await;
        pump.abort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_load_is_fixed_modest_and_bounded() {
        let config = config(20.0);
        assert!(config.rate <= 100.0, "a home cluster runs this");
        assert!(config.max_in_flight <= 128);
        assert!(!config.duration.is_zero());
        assert!(config.timeout <= Duration::from_secs(5));
        assert!(
            config.check().is_ok(),
            "the generator must accept its own configuration"
        );
    }

    #[test]
    fn it_only_runs_operations_the_sandbox_can_serve() {
        let ops: Vec<_> = config(20.0).operations.into_iter().map(|o| o.op).collect();
        assert!(ops.contains(&OpKind::RestEvaluate));
        assert!(
            ops.iter().all(|op| matches!(
                op,
                OpKind::RestEvaluate | OpKind::LedgerAppend | OpKind::LedgerCurrent
            )),
            "no destructive or long-running operation belongs in the sandbox: {ops:?}"
        );
    }
}
