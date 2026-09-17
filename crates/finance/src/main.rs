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
    /// Adopt a consent the prototype established, so the syncer can use it.
    ///
    /// Reads `sessions/<profile>.json`, records the session as an authorized
    /// connection, and links the accounts already imported by their provider
    /// uid. One-time: from then on consents come through the connect flow.
    Adopt {
        /// The prototype's data directory, holding `sessions/`.
        #[arg(long, default_value = "prototype/bank/data")]
        dir: std::path::PathBuf,
        /// Which session: `business` or `personal`.
        #[arg(long)]
        profile: String,
        /// The party the accounts belong to.
        #[arg(long)]
        party: uuid::Uuid,
    },
    /// Run one sync tick against the bank, now, and report what it did.
    ///
    /// Spends from the scheduler's share of the daily allowance, exactly as
    /// the worker would. With `--account`, a manual refresh of that one
    /// account from the reserve instead.
    Sync {
        /// Refresh one account from the reserve budget.
        #[arg(long)]
        account: Option<uuid::Uuid>,
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

/// A small pool for a one-shot command, or a clear refusal.
fn pool_for(config: &Config, what: &str) -> anyhow::Result<sqlx::PgPool> {
    if config.store.url.is_empty() {
        anyhow::bail!("set FINANCE_DATABASE_URL: {what} needs a store");
    }
    Ok(tbd_db::connect_lazy(&tbd_db::PgOptions {
        url: config.store.url.clone(),
        max_connections: 4,
        ..tbd_db::PgOptions::default()
    })?)
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (mut config, source) = Config::load(&cli.overrides.config_dir, &cli.overrides.env)?;
    cli.overrides.apply(&mut config);
    if let Some(command) = &cli.command {
        return one_shot(command, &config, &source).await;
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

/// A subcommand: does one thing, prints, exits. No telemetry, no listener.
async fn one_shot(
    command: &Command,
    config: &Config,
    source: &tbd_finance::config::Source,
) -> anyhow::Result<()> {
    match command {
        Command::Config => {
            println!("# env: {}", source.env);
            for f in &source.files {
                println!("# {}", f.display());
            }
            print!("{}", toml::to_string_pretty(config)?);
        }
        Command::Categorise { party } => {
            let pool = pool_for(config, "categorising")?;
            let report = tbd_finance::categorise::apply_rules(&pool, *party).await?;
            println!(
                "{} categorised by rule, {} kept as declared, {} still unmatched",
                report.categorised, report.declared_kept, report.unmatched
            );
        }
        Command::Session { id } => {
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
        }
        Command::Adopt {
            dir,
            profile,
            party,
        } => {
            let pool = pool_for(config, "adopting")?;
            let report =
                tbd_finance::banking::adopt::from_prototype(&pool, dir, profile, *party).await?;
            println!(
                "{profile}: connection {} ({}), valid until {}, {} accounts linked",
                report.connection_id,
                report.status,
                report
                    .valid_until
                    .map_or_else(|| "unknown".to_owned(), |t| t.to_rfc3339()),
                report.linked
            );
        }
        Command::Sync { account } => {
            let pool = pool_for(config, "syncing")?;
            let provider = tbd_finance::banking::from_config(&config.provider)?;
            let syncer = tbd_finance::sync::Syncer::new(
                pool,
                std::sync::Arc::new(provider),
                config.sync.clone(),
            );
            let now = chrono::Utc::now();
            if let Some(id) = account {
                println!("{id}: {:?}", syncer.refresh(*id, now).await?);
            } else {
                let tick = syncer.tick(now).await?;
                for (id, outcome) in &tick.synced {
                    println!("{id}: {outcome:?}");
                }
                for (id, why) in &tick.skipped {
                    println!("{id}: skipped, {why:?}");
                }
                if tick.synced.is_empty() && tick.skipped.is_empty() {
                    println!("nothing due");
                }
            }
        }
        Command::Import {
            dir,
            profile,
            party,
        } => {
            let pool = pool_for(config, "importing")?;
            let report = tbd_finance::import::from_prototype(&pool, dir, profile, *party).await?;
            println!(
                "{profile}: {} accounts, {} balance snapshots, {} inserted, {} already present, {} skipped",
                report.accounts,
                report.balances,
                report.inserted,
                report.duplicates,
                report.skipped
            );
        }
    }
    Ok(())
}
