//! `finance` binary: load config, install telemetry, serve until signalled.
//! Stdout is the product of `config`, so the print lint is allowed here only.

#![allow(clippy::print_stdout)]

use clap::{Parser, Subcommand};
use tbd_common::telemetry::TelemetryArgs;
use tbd_finance::{Config, Overrides};

#[derive(Parser)]
#[command(name = "finance", version = tbd_common::VERSION, about = "The finance service")]
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
    /// Apply the party's rules to its transactions.
    ///
    /// A pass is a pure function of the rules: what a rule decided is cleared
    /// and redecided, what a person declared is left alone.
    Categorise {
        /// The party whose rules and transactions to run over.
        #[arg(long)]
        party: uuid::Uuid,
    },
    /// Ask the provider about a session: one signed GET that proves the key,
    /// the application and TLS, and spends none of an account's daily
    /// allowance.
    Session {
        /// The provider's session id.
        #[arg(long)]
        id: String,
    },
    /// Import transactions the prototype already pulled from the provider.
    ///
    /// Reads saved responses rather than calling the API: the ASPSP allows only
    /// a handful of fetches a day, and a re-import has to be free. Idempotent,
    /// so running it twice is the test.
    Import {
        /// The prototype's data directory, holding `sessions/` and `raw/`.
        #[arg(long, default_value = "prototype/bank/data")]
        dir: std::path::PathBuf,
        /// Which session: `business` or `personal`.
        #[arg(long, default_value = "business")]
        profile: String,
        /// The party the accounts belong to.
        #[arg(long)]
        party: uuid::Uuid,
    },
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

    if let Some(Command::Categorise { party }) = &cli.command {
        if config.store.url.is_empty() {
            anyhow::bail!("set FINANCE_DATABASE_URL: categorising needs a store");
        }
        let pool = tbd_db::connect_lazy(&tbd_db::PgOptions {
            url: config.store.url.clone(),
            max_connections: 4,
            ..tbd_db::PgOptions::default()
        })?;
        let report = tbd_finance::categorise::apply_rules(&pool, *party).await?;
        println!(
            "{} categorised by rule, {} kept as declared, {} still unmatched",
            report.categorised, report.declared_kept, report.unmatched
        );
        return Ok(());
    }

    if let Some(Command::Session { id }) = &cli.command {
        let provider = tbd_finance::banking::from_config(&config.provider)?;
        let status = tbd_finance::banking::Provider::session(&provider, id).await?;
        println!(
            "status: {}\nvalid until: {}\naccounts: {}",
            status.status,
            status
                .valid_until
                .map_or_else(|| "unknown".to_owned(), |t| t.to_rfc3339()),
            status.account_uids.len()
        );
        for uid in &status.account_uids {
            println!("  {uid}");
        }
        return Ok(());
    }

    if let Some(Command::Import {
        dir,
        profile,
        party,
    }) = &cli.command
    {
        if config.store.url.is_empty() {
            anyhow::bail!("set FINANCE_DATABASE_URL: importing needs a store");
        }
        let pool = tbd_db::connect_lazy(&tbd_db::PgOptions {
            url: config.store.url.clone(),
            max_connections: 4,
            ..tbd_db::PgOptions::default()
        })?;
        let report = tbd_finance::import::from_prototype(&pool, dir, profile, *party).await?;
        println!(
            "{profile}: {} accounts, {} balance snapshots, {} inserted, {} already present, {} skipped",
            report.accounts, report.balances, report.inserted, report.duplicates, report.skipped
        );
        return Ok(());
    }

    let mut telemetry = tbd_common::telemetry::init(&cli.telemetry, "finance")?;
    tracing::info!(env = %source.env, files = ?source.files, "config");
    if let Some(addr) = config.metrics.listen {
        tbd_common::metrics::install(addr, &telemetry.service_name)?;
    }
    let mut profiler = tbd_common::profiling::maybe_start(
        cli.telemetry.pyroscope_server.as_deref(),
        &telemetry.service_name,
    );
    tbd_finance::serve(config, tbd_common::shutdown::signal()).await?;
    if let Some(p) = profiler.as_mut() {
        p.stop();
    }
    telemetry.shutdown();
    Ok(())
}
