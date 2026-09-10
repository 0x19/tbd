//! In-process continuous CPU profiling, pushed to Pyroscope.
//!
//! Off unless `PYROSCOPE_SERVER_ADDRESS` is set. Samples the process with
//! `pprof-rs` (SIGPROF, 100 Hz) and uploads every ten seconds tagged with the
//! service name, so profiles line up with `service.name` on traces and logs.
//! On a real cluster the eBPF agent covers every container without this; in a
//! container-based cluster such as k3d only this works, and it gives exact
//! Rust frames everywhere.

use pyroscope::{PyroscopeAgent, pyroscope::PyroscopeAgentRunning};
use pyroscope_pprofrs::{PprofConfig, pprof_backend};

const SAMPLE_RATE_HZ: u32 = 100;

/// Errors from starting the profiler.
#[derive(Debug, thiserror::Error)]
#[error("pyroscope: {0}")]
pub struct ProfilingError(String);

/// A running profiler. Keep it alive; call [`Profiler::stop`] on exit.
pub struct Profiler {
    agent: Option<PyroscopeAgent<PyroscopeAgentRunning>>,
}

impl std::fmt::Debug for Profiler {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Profiler")
            .field("running", &self.agent.is_some())
            .finish()
    }
}

impl Profiler {
    /// Start profiling `service_name` and push to `server`.
    pub fn start(server: &str, service_name: &str) -> Result<Self, ProfilingError> {
        let agent = PyroscopeAgent::builder(server, service_name)
            .backend(pprof_backend(
                PprofConfig::new().sample_rate(SAMPLE_RATE_HZ),
            ))
            .tags(vec![
                ("service_name", service_name),
                ("version", crate::VERSION),
            ])
            .build()
            .map_err(|e| ProfilingError(e.to_string()))?
            .start()
            .map_err(|e| ProfilingError(e.to_string()))?;
        tracing::info!(%server, service = service_name, "in-process profiling on");
        Ok(Self { agent: Some(agent) })
    }

    /// Stop and flush. Safe to call once; later calls are no-ops.
    pub fn stop(&mut self) {
        if let Some(agent) = self.agent.take() {
            match agent.stop() {
                Ok(stopped) => stopped.shutdown(),
                Err(error) => tracing::warn!(%error, "profiler stop"),
            }
        }
    }
}

impl Drop for Profiler {
    fn drop(&mut self) {
        self.stop();
    }
}

/// Start the profiler if `server` is set; log and continue if it fails.
/// Profiling must never keep a service from starting.
pub fn maybe_start(server: Option<&str>, service_name: &str) -> Option<Profiler> {
    let server = server?;
    match Profiler::start(server, service_name) {
        Ok(p) => Some(p),
        Err(error) => {
            tracing::warn!(%error, "in-process profiling disabled");
            None
        }
    }
}
