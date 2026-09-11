//! Campaigns against a real ledger, in-process on the memory store. No mocks
//! of our own services: `tbd_ledger::serve_store` on port 0 is the target.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{sync::Arc, time::Duration};

use tbd_stress::{Campaign, Hooks, StressEvent, Target};
use tokio::sync::mpsc;

/// An in-process ledger with a zero grace window and a one-second sweeper.
struct Ledger {
    addr: std::net::SocketAddr,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
}

impl Ledger {
    async fn start() -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut config = tbd_ledger::Config::in_memory(addr);
        config.erasure.grace = Duration::ZERO;
        config.erasure.sweep_interval = Duration::from_millis(200);
        let store: Arc<dyn tbd_ledger::Store> =
            Arc::new(tbd_ledger::Instrumented(tbd_ledger::MemoryStore::new()));
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            tbd_ledger::serve_store(
                listener,
                config,
                tbd_ledger::Runtime::default(),
                store,
                async {
                    let _ = stopped.await;
                },
            )
            .await
            .unwrap();
        });
        Self {
            addr,
            stop: Some(stop),
            task,
        }
    }

    fn target(&self) -> Target {
        Target {
            name: "ledger-it".into(),
            channel: tonic::transport::Endpoint::from_shared(format!("http://{}", self.addr))
                .unwrap()
                .connect_lazy(),
            bearer: None,
        }
    }

    async fn stop(mut self) {
        if let Some(s) = self.stop.take() {
            let _ = s.send(());
        }
        let _ = self.task.await;
    }
}

fn campaign(duration: &str, extra: &str) -> Campaign {
    Campaign::parse(&format!(
        "[campaign]\nname = \"it\"\nduration = \"{duration}\"\nwarmup = \"200ms\"\nseed = 11\n\n[workload]\nmax_in_flight = 16\n\n[workload.owner]\nworkers = 4\nsubjects = 2\n{extra}"
    ))
    .unwrap()
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_short_campaign_passes_and_evaluates_every_invariant() {
    let ledger = Ledger::start().await;
    let (tx, mut rx) = mpsc::unbounded_channel();
    let hooks = Hooks {
        events: Some(tx),
        cancel: tokio_util::sync::CancellationToken::default(),
    };
    // Relations and erasure cycles weighted up, so the cascade runs inside two seconds.
    let c = campaign(
        "2s",
        "[workload.owner.mix]\nappend = 4\ncurrent = 3\nhistory = 3\nhistory_cut = 2\nretract = 1\nidempotent_replay = 1\npair_relation = 3\nerase_cycle = 2\nexpiring = 1\n",
    );
    let result = tbd_stress::run(&c, vec![ledger.target()], &hooks).await;
    drop(hooks);
    let mut phases = Vec::new();
    let mut stress_frames = 0;
    let mut load_frames = 0;
    while let Ok(e) = rx.try_recv() {
        match e {
            StressEvent::Phase { name } => phases.push(name),
            StressEvent::Stress { .. } => stress_frames += 1,
            StressEvent::Load { .. } => load_frames += 1,
            StressEvent::Finding { .. } => {}
        }
    }
    let text = tbd_stress::render(&result);
    assert!(result.passed, "{text}");
    assert!(result.findings.is_empty(), "{text}");
    assert_eq!(result.store.as_deref(), Some("memory"));
    assert_eq!(phases, ["warmup", "run", "done"]);
    assert!(
        stress_frames >= 2 && load_frames >= 2,
        "{stress_frames} {load_frames}"
    );
    let load = result.load.as_ref().unwrap();
    assert!(load.requests_total > 100, "{text}");
    assert_eq!(load.requests_failed, load.errors.values().sum::<u64>());
    // Every enabled invariant was evaluated at least once in two seconds.
    for name in tbd_stress::model::invariants::ALL {
        let c = result.checks.get(*name).copied().unwrap_or_default();
        assert!(c.passed > 0, "{name} never evaluated:\n{text}");
        assert_eq!(c.violated, 0, "{name} violated:\n{text}");
    }
    // The erasure cycle ran the cascade: a fresh ledger has no other executed erasures.
    assert!(result.checks["erasure_executes"].passed > 0);
    ledger.stop().await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancel_ends_the_run_within_a_timeout() {
    let ledger = Ledger::start().await;
    let hooks = Hooks::default();
    let cancel = hooks.cancel.clone();
    let c = campaign("30s", "");
    let target = ledger.target();
    let run = tokio::spawn(async move { tbd_stress::run(&c, vec![target], &hooks).await });
    tokio::time::sleep(Duration::from_millis(600)).await;
    let before = std::time::Instant::now();
    cancel.cancel();
    let result = tokio::time::timeout(Duration::from_secs(6), run)
        .await
        .expect("the run ends after cancel")
        .unwrap();
    assert!(before.elapsed() < Duration::from_secs(6));
    assert_eq!(result.error.as_deref(), Some("cancelled"));
    assert!(!result.passed);
    ledger.stop().await;
}

#[tokio::test]
async fn an_unreachable_target_is_an_error_not_a_hang() {
    let target = Target {
        name: "nowhere".into(),
        channel: tonic::transport::Endpoint::from_shared("http://127.0.0.1:1")
            .unwrap()
            .connect_lazy(),
        bearer: None,
    };
    let c = campaign("1s", "");
    let result = tbd_stress::run(&c, vec![target], &Hooks::default()).await;
    assert!(!result.passed);
    assert!(
        result
            .error
            .as_deref()
            .unwrap_or("")
            .contains("unreachable"),
        "{result:?}"
    );
}
