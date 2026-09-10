//! `engine` binary: parse config, install telemetry, serve until signalled.

use clap::Parser;
use tbd_engine::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    let mut telemetry = tbd_common::telemetry::init(&config.telemetry, "engine")?;
    if let Some(addr) = config.metrics_addr {
        tbd_common::metrics::install(addr, &telemetry.service_name)?;
    }
    let mut profiler = tbd_common::profiling::maybe_start(
        config.telemetry.pyroscope_server.as_deref(),
        &telemetry.service_name,
    );
    tbd_engine::serve(config, tbd_common::shutdown::signal()).await?;
    if let Some(p) = profiler.as_mut() {
        p.stop();
    }
    telemetry.shutdown();
    Ok(())
}
