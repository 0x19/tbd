//! Campaigns against a real ledger, in-process on the memory store. No mocks
//! of our own services: `tbd_ledger::serve_store` on port 0 is the target.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::{sync::Arc, time::Duration};

use tbd_stress::{
    Campaign, GrpcLedger, Hooks, LedgerClient, StressEvent, Target,
    model::invariants::{CONTENTION, FUZZ, OWNER},
};
use tokio::sync::mpsc;

/// An in-process ledger with a zero grace window and a one-second sweeper.
struct Ledger {
    addr: std::net::SocketAddr,
    stop: Option<tokio::sync::oneshot::Sender<()>>,
    task: tokio::task::JoinHandle<()>,
}

impl Ledger {
    async fn start() -> Self {
        Self::start_with(tbd_ledger::Runtime::default()).await
    }

    async fn start_with(runtime: tbd_ledger::Runtime) -> Self {
        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        let mut config = tbd_ledger::Config::in_memory(addr);
        config.erasure.grace = Duration::ZERO;
        config.erasure.sweep_interval = Duration::from_millis(200);
        let store: Arc<dyn tbd_ledger::Store> =
            Arc::new(tbd_ledger::Instrumented(tbd_ledger::MemoryStore::new()));
        let (stop, stopped) = tokio::sync::oneshot::channel::<()>();
        let task = tokio::spawn(async move {
            tbd_ledger::serve_store(listener, config, runtime, store, async {
                let _ = stopped.await;
            })
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

/// The three worker classes together.
const ALL_CLASSES: &str = "[workload.contention]\nworkers = 3\nsubjects = 2\n[workload.fuzz]\nworkers = 1\nsubjects = 1\n";

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
        &format!(
            "{ALL_CLASSES}[workload.owner.mix]\nappend = 4\ncurrent = 3\nhistory = 3\nhistory_cut = 2\nretract = 1\nidempotent_replay = 1\npair_relation = 3\nerase_cycle = 2\nexpiring = 1\n"
        ),
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
    // Every rule of every class was evaluated at least once in two seconds;
    // `durability` needs a fault and is covered below.
    for name in OWNER.iter().chain(CONTENTION).chain(FUZZ) {
        let c = result.checks.get(*name).copied().unwrap_or_default();
        assert!(c.passed > 0, "{name} never evaluated:\n{text}");
        assert_eq!(c.violated, 0, "{name} violated:\n{text}");
    }
    for name in tbd_stress::model::invariants::ALL {
        let c = result.checks.get(*name).copied().unwrap_or_default();
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

/// A client that drops the last fact of every history page: the kind of
/// bug the harness exists for, wrapped around the honest gRPC client so the
/// ledger itself stays real.
struct Lying(GrpcLedger);

#[async_trait::async_trait]
impl LedgerClient for Lying {
    async fn ping(
        &self,
        req: tbd_proto::ledger::v1::PingRequest,
    ) -> Result<tbd_proto::ledger::v1::PingResponse, tbd_stress::CallError> {
        self.0.ping(req).await
    }
    async fn append(
        &self,
        req: tbd_proto::ledger::v1::AppendRequest,
    ) -> Result<tbd_proto::ledger::v1::AppendResponse, tbd_stress::CallError> {
        self.0.append(req).await
    }
    async fn current(
        &self,
        req: tbd_proto::ledger::v1::CurrentRequest,
    ) -> Result<tbd_proto::ledger::v1::CurrentResponse, tbd_stress::CallError> {
        self.0.current(req).await
    }
    async fn history(
        &self,
        req: tbd_proto::ledger::v1::HistoryRequest,
    ) -> Result<tbd_proto::ledger::v1::HistoryResponse, tbd_stress::CallError> {
        let mut r = self.0.history(req).await?;
        if r.next.is_empty() {
            r.facts.pop();
        }
        Ok(r)
    }
    async fn retract(
        &self,
        req: tbd_proto::ledger::v1::RetractRequest,
    ) -> Result<tbd_proto::ledger::v1::RetractResponse, tbd_stress::CallError> {
        self.0.retract(req).await
    }
    async fn erase(
        &self,
        req: tbd_proto::ledger::v1::EraseRequest,
    ) -> Result<tbd_proto::ledger::v1::EraseResponse, tbd_stress::CallError> {
        self.0.erase(req).await
    }
    async fn restore(
        &self,
        req: tbd_proto::ledger::v1::RestoreRequest,
    ) -> Result<tbd_proto::ledger::v1::RestoreResponse, tbd_stress::CallError> {
        self.0.restore(req).await
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn a_lying_client_produces_a_shrunk_finding_that_replays() {
    let ledger = Ledger::start().await;
    let c = campaign(
        "1500ms",
        "[workload.owner.mix]\nappend = 4\nhistory = 4\n\n[stop]\nmax_findings = 3\nshrink_attempts = 300\nshrink_timeout = \"30s\"\n",
    );
    let lying: Arc<dyn LedgerClient> = Arc::new(Lying(GrpcLedger::new(
        &ledger.target(),
        Duration::from_secs(5),
    )));
    let result = tbd_stress::run_with_clients(
        &c,
        vec!["ledger-it".into()],
        vec![lying.clone()],
        &Hooks::default(),
    )
    .await;
    let text = tbd_stress::render(&result);
    assert!(!result.passed, "{text}");
    assert!(result.stopped_early, "{text}");
    assert!(!result.findings.is_empty(), "{text}");
    for f in &result.findings {
        assert_eq!(f.invariant, "history_is_everything", "{text}");
        assert!(
            f.shrunk,
            "{}: {text}",
            f.shrink_note.as_deref().unwrap_or("")
        );
        // One append and one history read are all it takes.
        assert!(f.trace.len() <= 3, "{} steps: {text}", f.trace.len());
    }
    assert!(
        result
            .findings
            .iter()
            .any(|f| f.trace.len() < f.original_len),
        "no trace got shorter: {text}"
    );
    // The shrunk trace reproduces through the lying client and not through the honest one.
    let f = &result.findings[0];
    let honest: Arc<dyn LedgerClient> =
        Arc::new(GrpcLedger::new(&ledger.target(), Duration::from_secs(5)));
    let through_liar =
        tbd_stress::replay(&f.trace, lying.as_ref(), Duration::from_millis(500), &[]).await;
    assert!(
        through_liar.reproduces(&f.invariant, &f.signature),
        "{:?}",
        through_liar.violations
    );
    let through_honest =
        tbd_stress::replay(&f.trace, honest.as_ref(), Duration::from_millis(500), &[]).await;
    assert!(
        !through_honest.reproduces(&f.invariant, &f.signature),
        "{:?}",
        through_honest.violations
    );
    assert!(
        through_honest.violations.is_empty(),
        "{:?}",
        through_honest.violations
    );
    ledger.stop().await;
}

/// A store that fails on command under the workers: writes lose their
/// acknowledgement and reads fail. With the class tolerated, every write is
/// re-driven to a known outcome, `durability` is judged on the next history
/// walk, and nothing is a finding.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn store_faults_are_tolerated_redriven_and_durable() {
    let store_fault = tbd_ledger::FaultHandle::default();
    let ledger = Ledger::start_with(tbd_ledger::Runtime {
        store_fault: store_fault.clone(),
        ..Default::default()
    })
    .await;
    let c = campaign(
        "3s",
        &format!(
            "{ALL_CLASSES}[faults]\ntolerate = [\"unavailable\"]\n[workload.owner.mix]\nappend = 6\nhistory = 4\ncurrent = 2\nretract = 1\nidempotent_replay = 1\n"
        ),
    );
    let flipper = {
        let handle = store_fault.clone();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(600)).await;
            let outage: tbd_ledger::Behavior = serde_json::from_value(serde_json::json!({
                "type": "error", "kind": "unavailable", "rate": 0.4, "message": "db down"
            }))
            .unwrap();
            handle.set(outage);
            tokio::time::sleep(Duration::from_millis(900)).await;
            handle.set(tbd_ledger::Behavior::Healthy);
        })
    };
    let result = tbd_stress::run(&c, vec![ledger.target()], &Hooks::default()).await;
    let _ = flipper.await;
    let text = tbd_stress::render(&result);
    assert!(result.passed, "{text}");
    assert!(result.tolerated > 0, "no fault was drawn:\n{text}");
    assert!(result.redriven > 0, "no write was re-driven:\n{text}");
    let durability = result.checks.get("durability").copied().unwrap_or_default();
    assert!(durability.passed > 0, "durability never judged:\n{text}");
    assert_eq!(durability.violated, 0, "{text}");
    ledger.stop().await;
}
