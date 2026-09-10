//! `tracing` initialisation, configured from CLI flags or environment.

use clap::{Args, ValueEnum};
use tracing_subscriber::{EnvFilter, fmt, layer::SubscriberExt, util::SubscriberInitExt};

/// Output format for logs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, ValueEnum)]
pub enum LogFormat {
    /// Human-readable, for terminals.
    Text,
    /// One JSON object per line, for containers and log shippers.
    Json,
}

/// Logging flags, `#[command(flatten)]`-ed into each service's config.
#[derive(Debug, Clone, Args)]
pub struct LogArgs {
    /// Log output format.
    #[arg(
        long = "log-format",
        env = "LOG_FORMAT",
        default_value = "text",
        value_enum
    )]
    pub format: LogFormat,

    /// `tracing` filter directive, e.g. `info,tbd=debug`.
    #[arg(long = "log-filter", env = "RUST_LOG", default_value = "info")]
    pub filter: String,
}

/// Errors from telemetry setup.
#[derive(Debug, thiserror::Error)]
pub enum TelemetryError {
    /// The filter directive did not parse.
    #[error("invalid log filter {0:?}: {1}")]
    Filter(String, tracing_subscriber::filter::ParseError),
    /// A global subscriber was already installed.
    #[error("tracing subscriber already set: {0}")]
    AlreadySet(#[from] tracing_subscriber::util::TryInitError),
}

/// Install the global `tracing` subscriber. Call once, first thing in `main`.
pub fn init(args: &LogArgs) -> Result<(), TelemetryError> {
    let filter = EnvFilter::try_new(&args.filter)
        .map_err(|e| TelemetryError::Filter(args.filter.clone(), e))?;
    let registry = tracing_subscriber::registry().with(filter);
    match args.format {
        LogFormat::Text => registry.with(fmt::layer().with_target(true)).try_init()?,
        LogFormat::Json => registry
            .with(
                fmt::layer()
                    .json()
                    .flatten_event(true)
                    .with_current_span(true),
            )
            .try_init()?,
    }
    Ok(())
}
