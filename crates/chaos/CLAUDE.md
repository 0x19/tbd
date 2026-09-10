# crates/chaos

The `chaos` binary: validate, up, run, check. A framework with two extension points,
`service::Service` and `load::ops::Operation`; this project's specifics live in
`topology.rs`, `service/{engine,protocol}.rs`, `load/ops.rs` and the check list in
`validate.rs`. Everything else is generic and meant to move to the next project as-is.

Docs are the contract: `docs/chaos/README.md` (usage), `commands.md` (flags, output
schemas, exit codes), `scenarios.md` (file reference), `architecture.md`, `extending.md`.
**Any change to a flag, output field, TOML key, behaviour, error class or check must
update the matching page in the same commit.**

Where things are:
- `main.rs` is the only file allowed to print to stdout (`print_stdout` lint allowed
  there). It also sets the default log filter that silences the in-process services.
- `stack/mod.rs` never knows what a service is; keep it that way.
- `scenario/executor.rs` is the one executor. There is no second path for Rust-defined
  scenarios; add hooks there rather than a parallel executor.
- `scenario/assertions.rs` works on an immutable `Snapshot`; assertions are sync.
- `load/generator.rs` is open loop with an absolute-deadline pacer and a
  `max_in_flight` semaphore. Do not turn it closed loop; latency must not lower the
  rate.

Gotchas:
- Every config struct is `deny_unknown_fields`. `TimelineEvent` is an internally tagged
  enum (`action = ...`) with `at` repeated per variant because `flatten` and
  `deny_unknown_fields` do not combine.
- `chaos check` runs `ScenarioFile::check`; add cross-checks there, not in the executor.
- WebSocket connections are opened via `load::ops::connect_ws`, which sets
  `TCP_NODELAY`; `connect_async` does not and adds 40 ms to the first frame.
- `tls.rs` is the one place that decides what `https://`/`wss://` targets trust: web PKI
  roots always, plus `--ca-cert` for validate. It builds the tonic endpoint, the reqwest
  client and the WebSocket connector; do not construct those elsewhere or a target
  scheme will silently work on one surface and not another. The chaos crate enables
  tonic's `tls-ring`/`tls-webpki-roots` and tungstenite's rustls features itself; the
  services stay h2c.
- Scenarios run on port 0 by default; `topologies/dev.toml` uses fixed ports. Do not put
  fixed ports in `scenarios/` or CI runs collide.
- Engine counters reset on restart; assertions on a restarted engine cover the time
  since the restart.

Tests: `tests/it/main.rs` boots stacks in-process: validate passes, a fault maps to
503, an engine restarts on its port, a full scenario with a fault timeline runs, bad
files are rejected. The shipped scenarios are run by `mise run ci`.
