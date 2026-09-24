//! The balancer, in front of two real engines.
//!
//! The playground's whole game rests on one claim: **a single fault is
//! survivable**. Everything else is tuning. So these tests break a real engine
//! in the three ways a player can — stopping it, making it lie, and making it
//! crawl — and assert that calls through the balancer keep succeeding.

use std::{collections::BTreeMap, time::Duration};

use tbd_common::fault::Behavior;
use tbd_lab::{
    enginelb::{self, Policy},
    kinds::engine,
    stack::{Stack, ephemeral},
};
use tbd_proto::engine::v1::{EvaluateRequest, engine_service_client::EngineServiceClient};

/// Two engines, and a balancer in front of them.
async fn two_engines() -> (Stack, enginelb::Running) {
    let mut launchers = BTreeMap::new();
    for name in ["engine-1", "engine-2"] {
        launchers.insert(
            name.to_owned(),
            engine::KIND
                .launcher(toml::Table::new(), None)
                .expect("the engine kind accepts an empty table"),
        );
    }
    let stack = Stack::start(launchers).await.expect("two engines start");
    let endpoints: Vec<(String, String)> = stack
        .instances()
        .map(|(name, i)| (name.to_owned(), i.http_url()))
        .collect();

    let balancer = enginelb::start(
        ephemeral(),
        endpoints,
        Policy {
            probe_interval: Duration::from_millis(100),
            window: Duration::from_secs(5),
            min_requests: 5,
            error_ratio: 0.2,
            cooldown: Duration::from_secs(2),
            max_cooldown: Duration::from_secs(8),
            slow_multiple: Some(4.0),
            slow_floor: Duration::from_millis(50),
        },
    )
    .await
    .expect("the balancer binds");

    (stack, balancer)
}

/// Call `Evaluate` `count` times through `url`; return how many succeeded.
async fn evaluate(url: &str, count: usize) -> usize {
    let mut ok = 0;
    for _ in 0..count {
        let Ok(mut client) = EngineServiceClient::connect(url.to_owned()).await else {
            continue;
        };
        if client
            .evaluate(EvaluateRequest {
                subject_id: "balancer".into(),
                payload: b"hi".to_vec(),
            })
            .await
            .is_ok()
        {
            ok += 1;
        }
    }
    ok
}

#[tokio::test]
async fn it_forwards_real_grpc_to_the_engines_behind_it() {
    let (stack, lb) = two_engines().await;
    let url = format!("http://{}", lb.addr);

    assert_eq!(
        evaluate(&url, 10).await,
        10,
        "every call through the balancer reaches an engine"
    );

    // Both replicas served: the whole point of balancing above the connection.
    let served: Vec<u64> = stack
        .describe()
        .iter()
        .map(|i| i.requests.map_or(0, |r| r.total))
        .collect();
    assert!(
        served.iter().all(|&n| n > 0),
        "traffic reached both engines, not just one: {served:?}"
    );

    lb.stop().await;
    stack.shutdown().await;
}

#[tokio::test]
async fn a_stopped_engine_is_failed_over() {
    let (mut stack, lb) = two_engines().await;
    let url = format!("http://{}", lb.addr);
    assert_eq!(evaluate(&url, 6).await, 6, "healthy to begin with");

    stack
        .stop_instance("engine-1")
        .await
        .expect("engine-1 stops");
    // Let the health probe notice; it runs every 100ms here.
    tokio::time::sleep(Duration::from_millis(400)).await;

    assert_eq!(
        evaluate(&url, 10).await,
        10,
        "with one engine stopped the other carries every call"
    );

    lb.stop().await;
    stack.shutdown().await;
}

#[tokio::test]
async fn an_engine_that_crawls_is_ejected_for_being_slow() {
    let (stack, lb) = two_engines().await;
    let url = format!("http://{}", lb.addr);

    // Slow, not failing: every request succeeds, and every health check passes.
    // Only a comparison against its peer catches this.
    stack
        .set_behavior(
            "engine-1",
            Behavior::Slow {
                latency: Duration::from_millis(400),
                jitter: Duration::from_millis(0),
            },
        )
        .expect("engine-1 takes a behaviour");

    let ok = evaluate(&url, 14).await;
    assert_eq!(ok, 14, "slow is not failed: every call still succeeded");

    let engine_1 = lb
        .balancer
        .status()
        .await
        .into_iter()
        .find(|e| e.name == "engine-1")
        .expect("engine-1 is an endpoint");
    assert!(
        engine_1.ejected,
        "an endpoint far slower than its peer is taken out of rotation"
    );
    assert_eq!(
        engine_1.failures, 0,
        "it was ejected for being slow, not for failing"
    );

    lb.stop().await;
    stack.shutdown().await;
}

#[tokio::test]
async fn an_engine_that_lies_is_ejected_and_the_calls_keep_working() {
    let (stack, lb) = two_engines().await;
    let url = format!("http://{}", lb.addr);

    // Every request to engine-1 fails from here. It stays up, and it keeps
    // passing its health check — which is exactly why health checks alone
    // cannot save this and outlier ejection has to.
    stack
        .set_behavior(
            "engine-1",
            Behavior::Error {
                rate: 1.0,
                kind: tbd_common::fault::ErrorKind::Unavailable,
                message: "lying".to_owned(),
            },
        )
        .expect("engine-1 takes a behaviour");

    // Enough calls for the ejection window to fill and decide.
    let _ = evaluate(&url, 12).await;
    let ejected = lb
        .balancer
        .status()
        .await
        .into_iter()
        .find(|e| e.name == "engine-1")
        .expect("engine-1 is an endpoint")
        .ejected;
    assert!(ejected, "a failing endpoint is taken out of rotation");

    let ok = evaluate(&url, 10).await;
    assert_eq!(
        ok, 10,
        "with the liar ejected every call lands on the healthy engine"
    );

    lb.stop().await;
    stack.shutdown().await;
}
