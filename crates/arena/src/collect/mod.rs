//! The collectors: one task per configured source, each writing its part of
//! the [`World`]. A source with an empty URL is not started, and the world
//! says "not configured" for it.

pub mod chaos;
pub mod llm;
pub mod metrics;

use crate::{config::Config, world::World};

/// The sources this configuration reads.
#[must_use]
pub fn configured(config: &Config) -> Vec<&'static str> {
    let s = &config.sources;
    [
        ("llm", &s.llm_url),
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
/// The model service's URL does not parse.
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
