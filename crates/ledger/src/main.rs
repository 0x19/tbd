//! `ledger` binary: load config, install telemetry, serve until signalled.
//! Stdout is the product of `config`, so the print lint is allowed here only.

#![allow(clippy::print_stdout)]

use clap::{Parser, Subcommand};
use tbd_common::telemetry::TelemetryArgs;
use tbd_ledger::{Config, Overrides};

#[derive(Parser)]
#[command(name = "ledger", version = tbd_common::VERSION, about = "The ledger service")]
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
    /// Apply the embedded migrations to `LEDGER_DATABASE_URL` and exit.
    Migrate,
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
        println!(
            "# store.url: {} (LEDGER_DATABASE_URL); analytics.clickhouse_url: {} (LEDGER_CLICKHOUSE_URL)",
            if config.store.url.is_empty() {
                "unset"
            } else {
                "set"
            },
            if config.analytics.clickhouse_url.is_empty() {
                "unset"
            } else {
                "set"
            },
        );
        return Ok(());
    }
    config.validate()?;
    if let Some(Command::Migrate) = cli.command {
        let _telemetry = tbd_common::telemetry::init(&cli.telemetry, "ledger")?;
        anyhow::ensure!(
            config.store.kind == tbd_ledger::StoreKind::Postgres,
            "migrate needs store.kind = postgres and LEDGER_DATABASE_URL"
        );
        let store = tbd_ledger::PgStore::connect_lazy(&tbd_ledger::PgOptions {
            url: config.store.url.clone(),
            max_connections: 1,
            acquire_timeout: config.store.acquire_timeout,
        })?;
        store.migrate().await?;
        tracing::info!("migrations applied");
        return Ok(());
    }

    let mut telemetry = tbd_common::telemetry::init(&cli.telemetry, "ledger")?;
    tracing::info!(env = %source.env, files = ?source.files, "config");
    if let Some(addr) = config.metrics.listen {
        tbd_common::metrics::install(addr, &telemetry.service_name)?;
    }
    let mut profiler = tbd_common::profiling::maybe_start(
        cli.telemetry.pyroscope_server.as_deref(),
        &telemetry.service_name,
    );
    tbd_ledger::serve(config, tbd_common::shutdown::signal()).await?;
    if let Some(p) = profiler.as_mut() {
        p.stop();
    }
    telemetry.shutdown();
    Ok(())
}
