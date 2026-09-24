//! The model service's tiers: `ListModels` once a `[collect] llm_every`,
//! called as the arena itself (`svc:arena`), never as a viewer.

use std::time::Duration;

use tbd_proto::{
    arena::v1::TierState,
    llm::v1::{ListModelsRequest, Tier, llm_service_client::LlmServiceClient},
};
use tonic::{
    service::interceptor::InterceptedService,
    transport::{Channel, Endpoint},
};

use super::ServiceCaller;
use crate::world::World;

type Client = LlmServiceClient<InterceptedService<Channel, ServiceCaller>>;

/// Read the tiers forever.
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
    let client = LlmServiceClient::with_interceptor(channel, ServiceCaller);
    Ok(tokio::spawn(run(client, every, timeout, world)))
}

async fn run(mut client: Client, every: Duration, timeout: Duration, world: World) {
    let mut tick = tokio::time::interval(every);
    tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    loop {
        tick.tick().await;
        match tokio::time::timeout(timeout, client.list_models(ListModelsRequest {})).await {
            Ok(Ok(resp)) => {
                let tiers = resp
                    .into_inner()
                    .models
                    .into_iter()
                    .map(|m| TierState {
                        tier: tier_name(m.tier).to_owned(),
                        engine: m.engine,
                        model: m.model,
                        up: m.up,
                        stub: m.stub,
                        in_flight: m.in_flight,
                        max_in_flight: m.max_in_flight,
                        waiting: m.waiting,
                        ..TierState::default()
                    })
                    .collect();
                world.set_tiers(tiers);
                world.source_ok("llm");
            }
            Ok(Err(status)) => {
                world.source_failed("llm", format!("{:?}: {}", status.code(), status.message()));
            }
            Err(_) => world.source_failed("llm", "no answer in time"),
        }
    }
}

/// The wire enum as the page names it.
fn tier_name(wire: i32) -> &'static str {
    match Tier::try_from(wire) {
        Ok(Tier::Fast) => "fast",
        Ok(Tier::Deep) => "deep",
        _ => "unknown",
    }
}
