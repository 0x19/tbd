//! `protocol` binary: load config, install telemetry, serve until signalled.
//! Stdout is the product of `config`, so the print lint is allowed here only.

#![allow(clippy::print_stdout)]

use clap::{Parser, Subcommand};
use tbd_common::telemetry::TelemetryArgs;
use tbd_protocol::{Config, Overrides};

#[derive(Parser)]
#[command(name = "protocol", version = tbd_common::VERSION, about = "The protocol service")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
    #[command(flatten)]
    overrides: Overrides,
    #[command(flatten)]
    telemetry: TelemetryArgs,
}

#[derive(Subcommand)]
enum Command {
    /// Print the effective configuration for the environment as TOML.
    Config,
    /// Print the `OpenAPI` document for the REST surface as JSON.
    Openapi,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (mut config, source) = Config::load(&cli.overrides.config_dir, &cli.overrides.env)?;
    cli.overrides.apply(&mut config);
    if let Some(Command::Openapi) = cli.command {
        println!(
            "{}",
            serde_json::to_string_pretty(&tbd_protocol::openapi(&config)?)?
        );
        return Ok(());
    }
    if let Some(Command::Config) = cli.command {
        println!("# env: {}", source.env);
        for f in &source.files {
            println!("# {}", f.display());
        }
        print!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }
    config.validate()?;

    let mut telemetry = tbd_common::telemetry::init(&cli.telemetry, "protocol")?;
    tracing::info!(env = %source.env, files = ?source.files, "config");
    if let Some(addr) = config.metrics.listen {
        tbd_common::metrics::install(addr, &telemetry.service_name)?;
    }
    let mut profiler = tbd_common::profiling::maybe_start(
        cli.telemetry.pyroscope_server.as_deref(),
        &telemetry.service_name,
    );
    tbd_protocol::serve(config, tbd_common::shutdown::signal()).await?;
    if let Some(p) = profiler.as_mut() {
        p.stop();
    }
    telemetry.shutdown();
    Ok(())
}
