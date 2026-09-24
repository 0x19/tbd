//! `sandboxd` binary: load config, read the token, serve until signalled.
//! Stdout is the product of `config`, so the print lint is allowed here only.

#![allow(clippy::print_stdout)]

use anyhow::Context as _;
use clap::{Parser, Subcommand};
use tbd_common::telemetry::TelemetryArgs;
use tbd_sandboxd::{
    Config,
    config::Overrides,
    server::{AppState, router},
};

#[derive(Parser)]
#[command(name = "sandboxd", version = tbd_common::VERSION, about = "Runs one Go or Rust program in one throwaway gVisor sandbox")]
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
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (mut config, source) = Config::load(&cli.overrides.config_dir, &cli.overrides.env)?;
    cli.overrides.apply(&mut config);
    if let Some(Command::Config) = cli.command {
        println!("# env: {}", source.env);
        for f in &source.files {
            println!("# {}", f.display());
        }
        print!("{}", toml::to_string_pretty(&config)?);
        return Ok(());
    }

    let mut telemetry = tbd_common::telemetry::init(&cli.telemetry, "sandboxd")?;
    let path = cli
        .overrides
        .token_file
        .clone()
        .context("a token file is required (--token-file or SANDBOXD_TOKEN_FILE)")?;
    let token = std::fs::read(&path).with_context(|| format!("reading {}", path.display()))?;
    let token: Vec<u8> = token.trim_ascii().to_vec();
    anyhow::ensure!(token.len() >= 32, "the token must be at least 32 bytes");
    if let Some(addr) = config.server.metrics {
        tbd_common::metrics::install(addr, &telemetry.service_name)?;
    }
    let listen = config.server.listen;
    tracing::info!(env = %source.env, %listen, runtime = %config.images.runtime, "sandboxd listening");
    let listener = tokio::net::TcpListener::bind(listen).await?;
    axum::serve(listener, router(AppState::new(config, token)))
        .with_graceful_shutdown(tbd_common::shutdown::signal())
        .await?;
    telemetry.shutdown();
    Ok(())
}
