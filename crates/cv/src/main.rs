//! `cv` binary: load config, install telemetry, serve until signalled.
//! Stdout is the product of `config` and `grant`, so the print lint is
//! allowed here only.

#![allow(clippy::print_stdout)]

use anyhow::Context as _;
use clap::{Parser, Subcommand};
use tbd_common::telemetry::TelemetryArgs;
use tbd_cv::{Config, Overrides, notify::SERVICE_SUBJECT};
use tbd_db::{Capability, PartyId, UserId};
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "cv", version = tbd_common::VERSION, about = "The cv service")]
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
    /// Let this service send mail as the owner: grant its subject (`svc:cv`)
    /// read access to the owner's own parties, where the linked mailbox
    /// lives. One-time; run again after the owner's parties change.
    Grant {
        /// The owner's subject, as `GET /v1/me` shows it.
        #[arg(long)]
        owner_subject: String,
        /// One party only, instead of every party the owner owns.
        #[arg(long)]
        party: Option<Uuid>,
    },
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let (mut config, source) = Config::load(&cli.overrides.config_dir, &cli.overrides.env)?;
    cli.overrides.apply(&mut config);
    match cli.command {
        Some(Command::Config) => {
            println!("# env: {}", source.env);
            for f in &source.files {
                println!("# {}", f.display());
            }
            print!("{}", toml::to_string_pretty(&config)?);
            return Ok(());
        }
        Some(Command::Grant {
            owner_subject,
            party,
        }) => return grant(&config, &owner_subject, party).await,
        None => {}
    }

    let mut telemetry = tbd_common::telemetry::init(&cli.telemetry, "cv")?;
    tracing::info!(env = %source.env, files = ?source.files, "config");
    if let Some(addr) = config.metrics.listen {
        tbd_common::metrics::install(addr, &telemetry.service_name)?;
    }
    let mut profiler = tbd_common::profiling::maybe_start(
        cli.telemetry.pyroscope_server.as_deref(),
        &telemetry.service_name,
    );
    tbd_cv::serve(config, tbd_common::shutdown::signal()).await?;
    if let Some(p) = profiler.as_mut() {
        p.stop();
    }
    telemetry.shutdown();
    Ok(())
}

/// A one-shot: does one thing, prints, exits. No telemetry, no listener.
async fn grant(config: &Config, owner_subject: &str, party: Option<Uuid>) -> anyhow::Result<()> {
    anyhow::ensure!(
        !config.store.url.is_empty(),
        "no database: set CV_DATABASE_URL (the cv-db Secret's URL)"
    );
    let pool = tbd_db::connect_lazy(&tbd_db::PgOptions {
        url: config.store.url.clone(),
        max_connections: 2,
        ..tbd_db::PgOptions::default()
    })?;
    let owner: Option<(Uuid,)> = sqlx::query_as("select id from users where subject = $1")
        .bind(owner_subject)
        .fetch_optional(&pool)
        .await
        .context("looking the owner up")?;
    let Some((owner,)) = owner else {
        anyhow::bail!(
            "no user with subject {owner_subject:?}; sign in to the finance app once first"
        );
    };
    let owner = UserId(owner);
    let service = tbd_db::ensure_user(&pool, SERVICE_SUBJECT, None, "cv service").await?;
    let parties: Vec<(Uuid, String)> = match party {
        Some(id) => vec![(id, String::new())],
        None => tbd_db::visible_parties(&pool, owner)
            .await?
            .into_iter()
            .filter(|p| p.capability == Capability::Own)
            .map(|p| (p.party.id.0, p.party.display_name))
            .collect(),
    };
    anyhow::ensure!(!parties.is_empty(), "the owner owns no party");
    for (id, name) in &parties {
        tbd_db::grant(
            &pool,
            service,
            PartyId(*id),
            Capability::Read,
            Some(owner),
            None,
        )
        .await?;
        println!("granted {SERVICE_SUBJECT} read on {id} {name}");
    }
    Ok(())
}
