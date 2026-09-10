//! Open-loop, absolute-deadline pacer. Requests are spawned on schedule
//! regardless of how long earlier ones take, bounded by `max_in_flight`.

use std::{sync::Arc, time::Duration};

use tokio::{sync::Semaphore, task::JoinSet, time::Instant};

use super::{
    LoadConfig, Metrics,
    ops::{Clients, Operation, Target},
};

/// Run the configured load against `targets`. Returns the final snapshot.
///
/// A warmup phase, if configured, runs first with the same shape and its
/// metrics are discarded.
pub async fn run(
    config: &LoadConfig,
    targets: &[Target],
    metrics: Arc<Metrics>,
) -> super::LoadSnapshot {
    let clients = Arc::new(Clients::new(config.timeout));
    let ops: Vec<(Arc<dyn Operation>, u32)> = config
        .operations
        .iter()
        .filter(|o| o.weight > 0)
        .map(|o| (o.op.build(), o.weight))
        .collect();

    if let Some(warmup) = config.warmup
        && !warmup.is_zero()
    {
        tracing::info!(?warmup, "warmup");
        phase(config, targets, &ops, &clients, &metrics, warmup).await;
        metrics.reset();
    }
    tracing::info!(duration = ?config.duration, rate = config.rate, pattern = ?config.pattern, "load");
    phase(config, targets, &ops, &clients, &metrics, config.duration).await;
    metrics.snapshot()
}

async fn phase(
    config: &LoadConfig,
    targets: &[Target],
    ops: &[(Arc<dyn Operation>, u32)],
    clients: &Arc<Clients>,
    metrics: &Arc<Metrics>,
    duration: Duration,
) {
    if targets.is_empty() || ops.is_empty() {
        return;
    }
    let total_weight: u32 = ops.iter().map(|(_, w)| w).sum();
    let permits = Arc::new(Semaphore::new(config.max_in_flight));
    let mut tasks = JoinSet::new();
    let start = Instant::now();
    let deadline = start + duration;
    let mut next = start;
    let mut rr = 0_usize;

    while Instant::now() < deadline {
        let rate = config
            .pattern
            .rate_at(config.rate, start.elapsed(), duration);
        if rate <= 0.0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
            next = Instant::now();
            continue;
        }
        next += Duration::from_secs_f64(1.0 / rate);
        tokio::time::sleep_until(next).await;
        if Instant::now() >= deadline {
            break;
        }
        let Ok(permit) = Arc::clone(&permits).acquire_owned().await else {
            break;
        };

        let target = targets[rr % targets.len()].clone();
        rr = rr.wrapping_add(1);
        let op = pick(ops, total_weight);
        let clients = Arc::clone(clients);
        let metrics = Arc::clone(metrics);
        let timeout = config.timeout;
        tasks.spawn(async move {
            let _permit = permit;
            let started = Instant::now();
            let outcome = match tokio::time::timeout(timeout, op.run(&clients, &target)).await {
                Ok(Ok(())) => Ok(()),
                Ok(Err(e)) => Err(e.class()),
                Err(_) => Err("timeout".to_owned()),
            };
            metrics.record(&target.name, op.name(), outcome, started.elapsed());
        });
        // Reap finished tasks so the set does not grow without bound.
        while let Some(Some(_)) = tasks.try_join_next().map(Some) {}
    }

    // Drain in-flight requests, but never wait longer than one timeout.
    let drain = tokio::time::timeout(config.timeout + Duration::from_millis(100), async {
        while tasks.join_next().await.is_some() {}
    });
    if drain.await.is_err() {
        tracing::warn!("aborting requests still in flight after the drain window");
        tasks.abort_all();
    }
}

fn pick(ops: &[(Arc<dyn Operation>, u32)], total_weight: u32) -> Arc<dyn Operation> {
    let mut roll = rand::random_range(0..total_weight);
    for (op, weight) in ops {
        if roll < *weight {
            return Arc::clone(op);
        }
        roll -= weight;
    }
    Arc::clone(&ops[ops.len() - 1].0)
}
