# Extending chaos

Every recipe below is a few dozen lines. The reference implementation to copy is named
in each.

## Add a service kind

Reference: `crates/chaos/src/service/protocol.rs` (about 60 lines), and
`service/engine.rs` for fault and counter wiring.

1. Implement `service::Service`: `kind()`, `depends_on()` if it needs other instances,
   and `start(name, listen, peers)` which binds, spawns the service's library entry point
   on a task, and returns an `Instance`.
2. Implement `service::InstanceHandle` for its handle: `ready()` must answer true only
   when the service can take traffic; `stop()` signals the graceful-shutdown future and
   awaits the task. Return `Some` from `fault()` and `requests()` if the service exposes
   them.
3. Add its spec to `topology.rs` and a `Launcher` in `StackConfig::launchers`. Add any
   cross-checks to `StackConfig::check`.
4. If the service should receive load, decide whether it is a target (see operations).

The service itself needs a `serve_on(listener, config, shutdown)`-style entry point so
it can run on a caller-supplied listener. Both existing services have one.

## Give a service fault injection

Reference: `crates/engine/src/service.rs`, `admit` and `status_from`.

1. Keep a `tbd_common::fault::FaultHandle` in the service's runtime state and let
   embedders pass one in (the engine takes a `Runtime`).
2. Call `handle.apply().await` at the start of each request; map `Fault.kind` onto the
   service's error type.
3. For streams, call `handle.stream_error()` per emitted item.
4. Return the handle from the chaos adapter's `InstanceHandle::fault`.

## Add a behaviour

Add a variant to `Behavior` in `crates/common/src/fault.rs`, handle it in `apply` and,
if it should affect streams, in `stream_error`. Every service gets it; the TOML tag is
the variant name in `snake_case`. Add a line to the behaviour table in
[scenarios.md](scenarios.md#behaviours).

## Add an operation

Reference: `crates/chaos/src/load/ops.rs`, `RestEvaluate` is the smallest.

1. Implement `Operation`: `name()` and `run(&Clients, &Target)`. Use the shared
   `Clients` for connections; it holds a pooled HTTP client, a WebSocket pool per target
   and a lazy gRPC channel per target. Add a new pool there if the operation needs one.
2. Return `Err(OpError::...)` with the right class. `Transport` for connection
   problems, `Http(status)` or `Grpc(code)` for protocol-level failures, `Contract(..)`
   for a well-formed reply that breaks the API.
3. Add a variant to `OpKind` and map it in `OpKind::build`.
4. Add a row to the operations table in [scenarios.md](scenarios.md#load).

The generator adds timing, the timeout, bookkeeping and the error class `timeout`.
Operations should not catch timeouts themselves.

## Add a timeline action

Reference: `crates/chaos/src/scenario/timeline.rs`.

1. Add a variant to `TimelineEvent` with an `at` field and its parameters.
2. Handle it in `at()`, `describe()`, `apply()` and, if it targets a service, in
   `service()` so `chaos check` verifies the name.
3. If it needs a capability from the instance, add a method to `InstanceHandle` with a
   `None` default and implement it in the adapters that support it.

## Add an assertion

Reference: `crates/chaos/src/scenario/assertions.rs`.

Add a field to `Assertions` or `ServiceAssertions`, then one `check(...)` line in
`evaluate`. Assertions see an immutable `Snapshot` of load metrics and engine counters;
if you need something new, add it to the snapshot in the executor first. Keep the
`expected` and `actual` strings human-readable, they are what the report prints.

## Add a validate check

Reference: `crates/chaos/src/validate.rs`.

Write `async fn name(t: Targets) -> Result<String, String>` and add a line to `all()`.
`Ok(detail)` passes, `Err(detail)` fails; the runner adds the timeout and the timing.
Keep checks independent: they run concurrently.

## Add a load pattern

Add a variant to `load::Pattern` and a case in `rate_at(base, elapsed, total)`. It
returns requests per second at a point in the run; the pacer reads it every tick, so
any shape expressible as a function of time works.

## Rust-defined scenarios

`scenario::run_scenario` takes a `ScenarioFile`, which is a plain struct. A registry of
Rust scenarios only needs to build that struct, or the executor can grow hooks the way
the original design intended: `on_setup`, `on_pre_assert`, `on_teardown` around the
five phases in `executor.rs`.

## Reusing chaos in another project

Keep `service/mod.rs`, `stack/`, `load/generator.rs`, `load/metrics.rs`, `scenario/`
and `validate.rs`'s runner as they are. Replace `topology.rs` with your services,
`service/*.rs` with your adapters, `load/ops.rs` with your operations, and the check
list in `validate.rs` with your surfaces. The scenario file format, the reports and the
CLI stay the same.
