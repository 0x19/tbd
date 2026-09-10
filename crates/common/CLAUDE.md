# crates/common

Plumbing every service binary shares. **Transport-free**: no tonic, no axum, no hyper.
If a change needs one of those, it belongs in the service, not here.

- `telemetry.rs`: `LogArgs` (clap, `#[command(flatten)]`-ed into every binary; its flags
  are `global = true` so they work after a subcommand) and `init()`. Call `init` once,
  first thing in `main`. Calling it twice returns `TelemetryError::AlreadySet`.
- `shutdown.rs`: `signal()` resolves on SIGINT or SIGTERM. Pass it to every server's
  graceful-shutdown hook.
- `fault.rs`: runtime fault injection. `Behavior` is the TOML-facing enum (tag `type`,
  `deny_unknown_fields`); `FaultHandle` is the shared handle; `apply()` at request start,
  `stream_error()` per stream item. Returns a transport-neutral `Fault`; each service
  maps `ErrorKind` to its own error. Healthy by default, so production cost is one lock
  read per request.

Gotchas:
- Adding a `Behavior` variant is a contract change for scenario files. Update
  `docs/chaos/scenarios.md` (behaviours table) in the same commit.
- `DelayedFailure` is resolved against the instant the behaviour was *set*, not process
  start. `FaultHandle::set` restarts that clock.
- `VERSION` is `CARGO_PKG_VERSION` of this crate, which equals the workspace version.
