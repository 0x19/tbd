//! Plumbing shared by every service binary. Deliberately transport-free: no
//! tonic, no axum; `http` is used for header types only.

pub mod config;
pub mod fault;
pub mod metrics;
pub mod profiling;
pub mod runtime;
pub mod shutdown;
pub mod telemetry;

/// Version string baked in at build time, surfaced by every service.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
