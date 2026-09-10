//! Runtime configuration, from flags or environment.

use std::{net::SocketAddr, time::Duration};

use clap::Parser;
use tbd_common::telemetry::TelemetryArgs;

/// Engine configuration.
#[derive(Debug, Clone, Parser)]
#[command(name = "engine", version = tbd_common::VERSION, about)]
pub struct Config {
    /// Address the gRPC server binds.
    #[arg(long, env = "ENGINE_LISTEN_ADDR", default_value = "0.0.0.0:50051")]
    pub listen_addr: SocketAddr,

    /// Interval between heartbeat events on streams, in milliseconds.
    #[arg(long, env = "ENGINE_HEARTBEAT_MS", default_value_t = 1_000)]
    pub heartbeat_ms: u64,

    /// Prometheus `/metrics` listener. Unset in embedded use.
    #[arg(long, env = "ENGINE_METRICS_ADDR", default_value = "0.0.0.0:9464")]
    pub metrics_addr: Option<SocketAddr>,

    /// Logs and traces.
    #[command(flatten)]
    pub telemetry: TelemetryArgs,
}

impl Config {
    /// Heartbeat interval as a [`Duration`].
    pub fn heartbeat_interval(&self) -> Duration {
        Duration::from_millis(self.heartbeat_ms)
    }
}
