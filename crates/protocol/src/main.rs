//! `protocol` binary: parse config, install telemetry, serve until signalled.

use clap::Parser;
use tbd_protocol::Config;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let config = Config::parse();
    tbd_common::telemetry::init(&config.log)?;
    tbd_protocol::serve(config, tbd_common::shutdown::signal()).await?;
    Ok(())
}
