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
    /// Read issued invoices (the PDFs of the years before this service) back
    /// into the books. Every file is parsed and shown; nothing is written
    /// without --apply. Needs poppler's pdftotext on this machine.
    ImportInvoices {
        /// A directory searched recursively for *.pdf.
        #[arg(long)]
        dir: std::path::PathBuf,
        /// The issuing party.
        #[arg(long)]
        party: uuid::Uuid,
        /// Write what parsed cleanly.
        #[arg(long)]
        apply: bool,
        /// Show every parsed line under its invoice.
        #[arg(long)]
        lines: bool,
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
                println!("{id}: {:?}", syncer.refresh(*id, now, None).await?);
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
        Command::ImportInvoices {
            dir,
            party,
            apply,
            lines,
        } => {
            import_invoices(dir, *party, *apply, *lines, config).await?;
        }
        Command::Import {
            dir,
            profile,
            party,
        } => import_prototype(dir, profile, *party, config).await?,
    }
    Ok(())
}

/// Every `*.pdf` under a directory, recursively.
fn pdfs_under(dir: &std::path::Path, out: &mut Vec<std::path::PathBuf>) -> anyhow::Result<()> {
    for entry in std::fs::read_dir(dir)? {
        let path = entry?.path();
        if path.is_dir() {
            pdfs_under(&path, out)?;
        } else if path
            .extension()
            .is_some_and(|e| e.eq_ignore_ascii_case("pdf"))
        {
            out.push(path);
        }
    }
    Ok(())
}

/// `import-invoices`: parse every PDF under the directory, show the table,
/// and with `--apply` write what parsed cleanly.
async fn import_invoices(
    dir: &std::path::Path,
    party: uuid::Uuid,
    apply: bool,
    show_lines: bool,
    config: &Config,
) -> anyhow::Result<()> {
    let mut files = Vec::new();
    pdfs_under(dir, &mut files)?;
    files.sort();
    let mut good = Vec::new();
    println!("number         issued            client                            total  file");
    for f in &files {
        match tbd_finance::invoice::import::read_file(f) {
            Ok(p) => {
                let total = format!("{} {}", minor(p.total_minor), p.currency);
                println!(
                    "{:<14} {:<17} {:<24} {:>14}  {}  ({} lines{})",
                    p.number(),
                    p.issued_at.format("%Y-%m-%d %H:%M"),
                    p.client_name.chars().take(24).collect::<String>(),
                    total,
                    f.display(),
                    p.lines.len(),
                    if p.kind == "advance" { ", advance" } else { "" }
                );
                if show_lines {
                    for l in &p.lines {
                        println!(
                            "      {:>6}.{:03} × {:>12} = {:>12}  {}",
                            l.quantity_milli / 1000,
                            l.quantity_milli.rem_euclid(1000),
                            minor(l.unit_price_minor),
                            minor(l.amount_minor),
                            l.description
                        );
                    }
                }
                good.push(p);
            }
            Err(e) => println!(
                "{:<14} {:<17} {:<24} {:>14}  {}  SKIPPED: {e}",
                "-",
                "",
                "",
                "",
                f.display()
            ),
        }
    }
    if apply {
        let pool = pool_for(config, "importing invoices")?;
        let access = tbd_db::Access::for_parties(tbd_db::UserId(uuid::Uuid::nil()), vec![party]);
        let report = tbd_finance::invoice::import::apply(&pool, &access, party, &good).await?;
        println!(
            "imported {}: {}",
            report.imported.len(),
            report.imported.join(", ")
        );
        println!(
            "already present {}: {}",
            report.present.len(),
            report.present.join(", ")
        );
        for (n, why) in &report.refused {
            println!("refused {n}: {why}");
        }
    } else {
        println!(
            "{} readable of {} files; add --apply to write them",
            good.len(),
            files.len()
        );
    }
    Ok(())
}

/// Minor units as `-5622.82`.
fn minor(v: i64) -> String {
    let sign = if v < 0 { "-" } else { "" };
    format!("{sign}{}.{:02}", v.abs() / 100, v.abs() % 100)
}

/// `import`: the prototype's saved provider responses as rows.
async fn import_prototype(
    dir: &std::path::Path,
    profile: &str,
    party: uuid::Uuid,
    config: &Config,
) -> anyhow::Result<()> {
    let pool = pool_for(config, "importing")?;
    let report = tbd_finance::import::from_prototype(&pool, dir, profile, party).await?;
    println!(
        "{profile}: {} accounts, {} balance snapshots, {} inserted, {} already present, {} skipped",
        report.accounts, report.balances, report.inserted, report.duplicates, report.skipped
    );
    Ok(())
}
