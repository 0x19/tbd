//! Plumbing shared by every service binary. Deliberately transport-free.

pub mod shutdown;
pub mod telemetry;

/// Version string baked in at build time, surfaced by every service.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
