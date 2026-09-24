//! The collectors: one task per configured source, each writing its part of
//! the [`World`]. A source with an empty URL is not started, and the world
//! says "not configured" for it.

pub mod chaos;
pub mod llm;
pub mod metrics;
pub mod runner;

use base64::Engine as _;
use tbd_common::telemetry::propagation;
use tonic::{Status, service::Interceptor};

use crate::{config::Config, world::World};

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

/// The sources this configuration reads.
#[must_use]
pub fn configured(config: &Config) -> Vec<&'static str> {
    let s = &config.sources;
    [
        ("llm", &s.llm_url),
        ("runner", &s.runner_url),
        ("metrics", &s.metrics_url),
        ("chaos", &s.chaos_url),
    ]
    .into_iter()
    .filter(|(_, url)| !url.trim().is_empty())
    .map(|(name, _)| name)
    .collect()
}

/// Start every configured collector and the ticker; the handles stop them.
///
/// # Errors
/// The model service's or the runner's URL does not parse.
pub fn start(
    config: &Config,
    world: &World,
) -> Result<Vec<tokio::task::JoinHandle<()>>, tonic::transport::Error> {
    let s = &config.sources;
    let c = &config.collect;
    let mut tasks = vec![tokio::spawn(world.clone().tick(config.watch.tick))];
    if !s.llm_url.trim().is_empty() {
        tasks.push(llm::start(
            &s.llm_url,
            c.llm_every,
            c.timeout,
            world.clone(),
        )?);
    }
    if !s.runner_url.trim().is_empty() {
        tasks.push(runner::start(
            &s.runner_url,
            c.runner_every,
            c.timeout,
            world.clone(),
        )?);
    }
    if !s.metrics_url.trim().is_empty() {
        tasks.push(metrics::start(
            &s.metrics_url,
            c.metrics_every,
            c.timeout,
            world.clone(),
        ));
    }
    if !s.chaos_url.trim().is_empty() {
        tasks.push(chaos::start(
            &s.chaos_url,
            &s.validate_cron,
            c.chaos_every,
            c.timeout,
            world.clone(),
        ));
    }
    Ok(tasks)
}
