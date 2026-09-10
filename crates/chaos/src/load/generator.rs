//! Open-loop, absolute-deadline pacer. Requests are spawned on schedule
//! regardless of how long earlier ones take, bounded by `max_in_flight`.

use std::{sync::Arc, time::Duration};

use tokio::{
    sync::{Semaphore, mpsc},
    task::JoinSet,
    time::Instant,
};
use tokio_util::sync::CancellationToken;

use super::{
    LoadConfig, LoadSnapshot, Metrics,
    ops::{Clients, Operation, Target},
};
use crate::tls::Trust;

/// How often [`Hooks::progress`] receives a snapshot.
pub const PROGRESS_INTERVAL: Duration = Duration::from_secs(1);

/// Optional observation and control of a load run.
#[derive(Debug, Clone, Default)]
pub struct Hooks {
    /// Receives a snapshot every [`PROGRESS_INTERVAL`] while load runs, and
    /// one final snapshot when each phase ends.
    pub progress: Option<mpsc::UnboundedSender<LoadSnapshot>>,
    /// Cancel: the pacer stops scheduling, in-flight requests drain.
    pub cancel: CancellationToken,
}

/// Run the configured load against `targets`. Returns the final snapshot.
///
/// A warmup phase, if configured, runs first with the same shape and its
/// metrics are discarded.
pub async fn run(config: &LoadConfig, targets: &[Target], metrics: Arc<Metrics>) -> LoadSnapshot {
    run_with(
        config,
        targets,
        metrics,
        &Hooks::default(),
        &Trust::default(),
    )
    .await
}

/// [`run`] with progress reporting and cancellation.
pub async fn run_with(
    config: &LoadConfig,
    targets: &[Target],
    metrics: Arc<Metrics>,
    hooks: &Hooks,
    trust: &Trust,
) -> LoadSnapshot {
    // The token is fetched once per run; a failure is logged and the run goes
    // on without it, which shows up as 401s in the error classes.
    let trust = match trust.snapshot().await {
        Ok(t) => t,
        Err(error) => {
            tracing::error!(%error, "no bearer token for the load run");
            trust.clone()
        }
    };
    let clients = Arc::new(Clients::with_trust(config.timeout, trust));
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
        phase(config, targets, &ops, &clients, &metrics, warmup, hooks).await;
        metrics.reset();
    }
    tracing::info!(duration = ?config.duration, rate = config.rate, pattern = ?config.pattern, "load");
    phase(
        config,
        targets,
        &ops,
        &clients,
        &metrics,
        config.duration,
        hooks,
    )
    .await;
    metrics.snapshot()
}

/// Sends a snapshot every [`PROGRESS_INTERVAL`] until aborted.
fn progress_ticker(
    metrics: &Arc<Metrics>,
    progress: Option<&mpsc::UnboundedSender<LoadSnapshot>>,
) -> Option<tokio::task::AbortHandle> {
    let tx = progress?.clone();
    let metrics = Arc::clone(metrics);
    let task = tokio::spawn(async move {
        let mut tick =
            tokio::time::interval_at(Instant::now() + PROGRESS_INTERVAL, PROGRESS_INTERVAL);
        loop {
            tick.tick().await;
            if tx.send(metrics.snapshot()).is_err() {
                break;
            }
        }
    });
    Some(task.abort_handle())
}

#[allow(clippy::too_many_arguments)]
async fn phase(
    config: &LoadConfig,
    targets: &[Target],
    ops: &[(Arc<dyn Operation>, u32)],
    clients: &Arc<Clients>,
    metrics: &Arc<Metrics>,
    duration: Duration,
    hooks: &Hooks,
) {
    if targets.is_empty() || ops.is_empty() {
        return;
    }
    let ticker = progress_ticker(metrics, hooks.progress.as_ref());
    let total_weight: u32 = ops.iter().map(|(_, w)| w).sum();
    let permits = Arc::new(Semaphore::new(config.max_in_flight));
    let mut tasks = JoinSet::new();
    let start = Instant::now();
    let deadline = start + duration;
    let mut next = start;
    let mut rr = 0_usize;

    while Instant::now() < deadline && !hooks.cancel.is_cancelled() {
        let rate = config
            .pattern
            .rate_at(config.rate, start.elapsed(), duration);
        if rate <= 0.0 {
            tokio::time::sleep(Duration::from_millis(10)).await;
            next = Instant::now();
            continue;
        }
        next += Duration::from_secs_f64(1.0 / rate);
        tokio::select! {
            () = tokio::time::sleep_until(next) => {}
            () = hooks.cancel.cancelled() => break,
        }
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
    if let Some(ticker) = ticker {
        ticker.abort();
    }
    if let Some(tx) = &hooks.progress {
        let _ = tx.send(metrics.snapshot());
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
