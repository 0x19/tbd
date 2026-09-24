//! The sandbox runner (RFC 0010): `ListLanguages` once a `[collect]
//! runner_every`, called as the arena itself (`svc:arena`). Whether it
//! answered is `up`; what it runs and how full it is come from the answer;
//! how often it runs and how long a run takes come from the metrics store.

use std::time::Duration;

use tbd_proto::{
    arena::v1::RunnerState,
    runner::v1::{ListLanguagesRequest, runner_service_client::RunnerServiceClient},
};
use tonic::{
    service::interceptor::InterceptedService,
    transport::{Channel, Endpoint},
};

use super::ServiceCaller;
use crate::world::World;

type Client = RunnerServiceClient<InterceptedService<Channel, ServiceCaller>>;

/// Read the runner forever.
///
/// # Errors
/// The URL does not parse.
pub fn start(
    url: &str,
    every: Duration,
    timeout: Duration,
    world: World,
) -> Result<tokio::task::JoinHandle<()>, tonic::transport::Error> {
    let channel = Endpoint::from_shared(url.to_owned())?
        .connect_timeout(timeout)
        .connect_lazy();
    let client = RunnerServiceClient::with_interceptor(channel, ServiceCaller);
    Ok(tokio::spawn(run(client, every, timeout, world)))
}

async fn run(mut client: Client, every: Duration, timeout: Duration, world: World) {
    let mut tick = tokio::time::interval(every);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        match tokio::time::timeout(timeout, client.list_languages(ListLanguagesRequest {})).await {
            Ok(Ok(resp)) => {
                let r = resp.into_inner();
                world.set_runner(RunnerState {
                    up: true,
                    stub: r.stub,
                    in_flight: r.in_flight,
                    max_in_flight: r.max_in_flight,
                    languages: r.languages.into_iter().map(|l| l.name).collect(),
                    ..RunnerState::default()
                });
                world.source_ok("runner");
            }
            Ok(Err(status)) => {
                world.runner_down();
                world.source_failed(
                    "runner",
                    format!("{:?}: {}", status.code(), status.message()),
                );
            }
            Err(_) => {
                world.runner_down();
                world.source_failed("runner", "no answer in time");
            }
        }
    }
}
