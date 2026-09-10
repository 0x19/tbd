//! Runtime configuration, from flags or environment.

use std::net::SocketAddr;

use clap::Parser;
use tbd_common::telemetry::TelemetryArgs;

/// Protocol configuration.
#[derive(Debug, Clone, Parser)]
#[command(name = "protocol", version = tbd_common::VERSION, about)]
pub struct Config {
    /// Address the protocol binds. Serves HTTP/1.1 and h2c (gRPC) on the same port.
    #[arg(long, env = "PROTOCOL_LISTEN_ADDR", default_value = "0.0.0.0:8080")]
    pub listen_addr: SocketAddr,

    /// Engine gRPC endpoint.
    #[arg(
        long,
        env = "PROTOCOL_ENGINE_URL",
        default_value = "http://127.0.0.1:50051"
    )]
    pub engine_url: String,

    /// Prometheus `/metrics` listener. Unset in embedded use.
    #[arg(long, env = "PROTOCOL_METRICS_ADDR", default_value = "0.0.0.0:9465")]
    pub metrics_addr: Option<SocketAddr>,

    /// Logs and traces.
    #[command(flatten)]
    pub telemetry: TelemetryArgs,
}
