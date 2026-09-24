//! The model service's tiers: `ListModels` once a `[collect] llm_every`,
//! called as the arena itself (`svc:arena`), never as a viewer.

use std::time::Duration;

use base64::Engine as _;
use tbd_common::telemetry::propagation;
use tbd_proto::{
    arena::v1::TierState,
    llm::v1::{ListModelsRequest, Tier, llm_service_client::LlmServiceClient},
};
use tonic::{
    Status,
    service::{Interceptor, interceptor::InterceptedService},
    transport::{Channel, Endpoint},
};

use crate::world::World;

/// The subject the arena calls other services as.
pub const SERVICE_SUBJECT: &str = "svc:arena";

/// Marks every call as this service and carries the trace along.
#[derive(Debug, Clone, Copy)]
pub struct ServiceCaller;

struct MetadataInjector<'a>(&'a mut tonic::metadata::MetadataMap);

impl propagation::Injector for MetadataInjector<'_> {
    fn set(&mut self, key: &str, value: String) {
        if let (Ok(key), Ok(value)) = (
            tonic::metadata::MetadataKey::from_bytes(key.as_bytes()),
            value.parse::<tonic::metadata::MetadataValue<_>>(),
        ) {
            self.0.insert(key, value);
        }
    }
}

impl Interceptor for ServiceCaller {
    fn call(&mut self, mut request: tonic::Request<()>) -> Result<tonic::Request<()>, Status> {
        propagation::inject(&mut MetadataInjector(request.metadata_mut()));
        let claims = serde_json::json!({ "sub": SERVICE_SUBJECT });
        let payload = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(claims.to_string());
        if let Ok(value) = payload.parse() {
            request
                .metadata_mut()
                .insert(tbd_common::principal::PAYLOAD_HEADER, value);
        }
        Ok(request)
    }
}

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
