# crates/chaos

The `chaos` binary: validate, up, run, check, serve, config, kinds. A framework with two
extension points, `service::Service` and `load::ops::Operation`; this project's
specifics live in `kinds/` (one module per service kind, each with its spec, handle,
validate checks and a `static KIND` registered in `kinds::ALL`), `load/ops.rs` (the
protocol operations) and `load/ledger_ops.rs` (the ledger ones: a per-run `Pool` of
subjects that remembers what was written, so reads hit real facts and a `NotFound` on a
read is a real failure, while a retraction that finds nothing valued is the contract
answering an overlapping retraction; `ledger_lifecycle` and `ledger_erase_cycle` assert
the privacy rules per request, the latter waiting for the cascade only when the grace
the ledger announces fits its budget; `ledger_fuzz` is a seeded hostile generator). Every
`OpKind` has a `target_kind()`; the generator pairs each operation with the targets of
its kind, and `Target` carries `kind` (default `protocol` for API compatibility).
Everything else is generic and meant to move to the next project as-is.

Docs are the contract: `docs/chaos/README.md` (usage), `commands.md` (flags, output
schemas, exit codes), `scenarios.md` (file reference), `api.md` (`chaos serve` routes,
bodies, SSE frames, run record), `config.md` (`configs/chaos/` keys and precedence),
`architecture.md`, `extending.md`. **Any change to a flag, output field, TOML key, API
route, behaviour, error class or check must update the matching page in the same
commit.** The UI in `ui/chaos` mirrors `api.md` in its Zod schemas; a field change here
is a change there too.

Where things are:
- `main.rs` is the only file allowed to print to stdout (`print_stdout` lint allowed
  there). It also sets the default log filter that silences the in-process services.
- Service kinds are data: `kinds/{engine,protocol,ledger}.rs`, registered in
  `kinds::ALL` above the `tbd:kinds-end` marker (`tbd new service` inserts there and
  renders a module like `kinds/ledger.rs`). Topology tables, launchers, cross-checks,
  validate targets and checks, `--target`, `CHAOS_<KIND>_URL`, runtime add and clone,
  `GET /overview kinds` and the UI all derive from it. **Nothing outside `kinds/` may
  match on a kind name**; reach for `kinds::by_name`, the capability flags or
  `Field::InstanceOf`. `docs/chaos/kinds.md` is generated (`mise run chaos:docs`) and
  drift-checked in `ci`.
- `stack/mod.rs` never knows what a service is; keep it that way.
- `scenario/executor.rs` is the one executor. There is no second path for Rust-defined
  scenarios; add hooks there rather than a parallel executor.
- `scenario/assertions.rs` works on an immutable `Snapshot`; assertions are sync.
- `load/generator.rs` is open loop with an absolute-deadline pacer and a
  `max_in_flight` semaphore. Do not turn it closed loop; latency must not lower the
  rate.
- `config.rs` is the schema of `configs/chaos/`; loading and merging is
  `tbd_common::config`. `main.rs` loads it before dispatching and applies flag
  overrides field by field; add a flag there when a key needs an env var.
- `api/` is `chaos serve`. `state.rs` owns the long-lived stack (a `Mutex<Option<Stack>>`)
  and spawns run jobs; `runs.rs` is the record store (one JSON per run under
  `[paths] results`) and the live feed; `routes.rs` only translates HTTP. Runs reuse
  `scenario::run_file_with` / `load::run_with` with `Hooks` (event channel +
  `CancellationToken`); do not add a second executor for the API. `jobs.rs` is the
  queue (in memory, FIFO into the single slot; `AppState::pump` starts the next item
  after every enqueue and every run end, boxed because it recurses through the run
  task) and the schedules file (cron via `croner`, UTC, checked once a second by the
  loop `api::state` spawns; a due schedule whose last job is still queued or running is
  skipped, not stacked). `notify.rs` posts finished runs to one Slack webhook per
  environment (`[notify.slack]`, URL from the environment only); `AppState::notify`
  runs after every record is final, decides with the schedule's mode, and posts in a
  task so a slow Slack never delays a run. `added.rs` is the file of instances added to
  the serve stack at runtime (`[paths] stack`): `AppState::{add,clone,remove}_instance`
  go through it and `AppState::new` replays it after the topology starts, in dependency
  order. The generic `Stack` only knows `add_instance`/`remove_instance`/`next_name`;
  what an instance is (`kind` plus the kind's table) is `AddedSpec`, rebuilt into a
  launcher through the kind's `parse`.

Gotchas:
- Every config struct is `deny_unknown_fields`. `TimelineEvent` is an internally tagged
  enum (`action = ...`) with `at` repeated per variant because `flatten` and
  `deny_unknown_fields` do not combine. `StackConfig` and `[targets]` have hand-written
  serde impls keyed by the registry; `ValidateRequest` and `AddInstance` flatten a map
  of the kind's keys and validate them against the registry instead.
- `chaos check` runs `ScenarioFile::check`; add cross-checks there, not in the executor.
- WebSocket connections are opened via `load::ops::connect_ws`, which sets
  `TCP_NODELAY`; `connect_async` does not and adds 40 ms to the first frame.
- `auth.rs` + `tls.rs`: the bearer token rides on `Trust`. `Trust::snapshot().await`
  fetches it once (client credentials, cached, refreshed a minute before expiry) and
  every client built from that snapshot sends it: reqwest default header, WebSocket
  handshake header, tonic interceptor (`tls::Grpc`). `validate::run` and
  `load::run_with` snapshot at the start of a run; the executor's in-process stacks use
  `Trust::default()` (no Envoy, no token). Never build a reqwest/tonic client elsewhere.
- `tls.rs` is the one place that decides what `https://`/`wss://` targets trust: web PKI
  roots always, plus `--ca-cert` for validate. It builds the tonic endpoint, the reqwest
  client and the WebSocket connector; do not construct those elsewhere or a target
  scheme will silently work on one surface and not another. The chaos crate enables
  tonic's `tls-ring`/`tls-webpki-roots` and tungstenite's rustls features itself; the
  services stay h2c.
- Scenarios run on port 0 by default; `topologies/dev.toml` uses fixed ports. Do not put
  fixed ports in `scenarios/` or CI runs collide.
- Counters reset on restart; assertions on a restarted instance cover the time since
  the restart.
- Result types (`ScenarioResult`, `LoadSnapshot`, `Report`, ...) derive `Deserialize`
  because run records round-trip through JSON files; config types derive `Serialize`
  because the API returns parsed scenarios. Keep both when adding fields.
- One run at a time in serve (`RunStore::begin`); scenario runs use ephemeral ports so
  they never touch the serve stack. A `running` record found on start is marked
  `error: interrupted`.
- `serve` in the cluster binds `0.0.0.0` via `CHAOS_LISTEN_ADDR`; `base.toml` stays on
  loopback on purpose.

Tests: `tests/it/main.rs` boots stacks in-process: a stack of every registered kind
built from the registry (validate passes, capabilities match, a fault on any fault kind
fails only its own and its dependents' checks), the registry is consistent, a fault maps
to 503, an engine restarts on its port, a full scenario with a fault timeline runs, bad
files are rejected. Never enumerate kinds in a test; a scaffolded kind must pass here
unchanged (`mise run tbd:selfcheck` runs these tests with `zeta` registered). `tests/it/api.rs` boots `chaos serve` on port 0 with a temp
scenarios directory and drives it like the UI: stack calls, scenario run over SSE,
ad-hoc load cancel, scenario write/check/delete, validate (every kind with a target,
rejects unknown keys), add/clone/remove of every addable kind, the queue (run all expands,
items drain one at a time, remove and clear) and schedules (persist to the file, fire on
their own within seconds on a per-second cron, run now, disable, delete). The shipped
scenarios are run by `mise run ci`.
