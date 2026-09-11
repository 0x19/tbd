# Extending chaos

Every recipe below is a few dozen lines. The reference implementation to copy is named
in each.

## Add a service kind

One module under `crates/chaos/src/kinds/` and one line in `kinds::ALL`; `tbd new
service` writes both. Reference: `kinds/ledger.rs`, the smallest (no dependency), which
is what the CLI renders; `kinds/protocol.rs` for a kind with a dependency field;
`kinds/engine.rs` for a field with a default.

1. The spec struct is the service: `#[derive(Deserialize, Serialize)]` with
   `deny_unknown_fields`, its fields the keys of `[stack.<plural>.<name>]` minus
   `listen`. Implement `service::Service` on it: `kind()` returns `KIND.name`,
   `depends_on()` if it needs other instances, and `start(name, listen, peers)` which
   binds, spawns the service's library entry point on a task, and returns an `Instance`.
2. Implement `service::InstanceHandle` for its handle: `ready()` must answer true only
   when the service can take traffic; `stop()` signals the graceful-shutdown future and
   awaits the task. Return `Some` from `fault()` and `requests()` if the service exposes
   them.
3. Declare `pub static KIND: Kind { .. }`: `name`, `label`, `plural`, `surface`, the
   `target` (help and default URL) and `checks` (see below), `fields` (one `Field` per
   key: `Text`, `Duration`, or `InstanceOf("engine")` for a dependency), and the
   capability flags `fault`, `counters`, `load_target`, `addable`, matching what the
   handle really exposes; `parse: kinds::parse::<Spec>`.
4. Add `pub mod <name>;` and `&<name>::KIND,` above the `tbd:kinds-end` marker in
   `kinds/mod.rs`, a `[stack.<plural>.<name>-1]` table to `topologies/dev.toml`,
   `<name> = "…"` under `[targets]` in every `configs/chaos/*.toml`, and
   `CHAOS_<NAME>_URL` where the chaos pod, compose and ansible set the others.
5. `mise run chaos:docs` regenerates `docs/chaos/kinds.md`.

From the registry derive: the topology table and its cross-checks, `chaos up`,
validate's target and checks, `--target <name>=`, `CHAOS_<NAME>_URL`, `[targets]
<name>`, `POST /stack {"kind": "<name>"}`, clone and re-add, `set_behavior` and
`[assertions.services]` rules in `chaos check`, `GET /overview kinds`, and the admin
UI's add-instance form, validate targets and copy. The integration tests build a stack
of every registered kind, so a kind whose flags lie fails `cargo nextest -p tbd-chaos`.

The service itself needs a `serve_on(listener, config, shutdown)`-style entry point so
it can run on a caller-supplied listener. Every scaffolded service has one.

## Give a service fault injection

Reference: `crates/engine/src/service.rs`, `admit` and `status_from`.

1. Keep a `tbd_common::fault::FaultHandle` in the service's runtime state and let
   embedders pass one in (the engine takes a `Runtime`).
2. Call `handle.apply().await` at the start of each request; map `Fault.kind` onto the
   service's error type.
3. For streams, call `handle.stream_error()` per emitted item.
4. Return the handle from the chaos adapter's `InstanceHandle::fault`.

## Give a service store faults

Reference: `crates/ledger/src/store/faulty.rs`.

A service with a store can fail like its database does, which the adapter-level fault
cannot imitate: a read fails before it runs, a write runs, commits, and then fails, so
the client loses an acknowledgement of something that stands. Keep a second
`FaultHandle` (`Runtime.store_fault`), wrap the store in a decorator that applies it
before reads and after successful writes, return the handle from
`InstanceHandle::store_fault`, and set `store_fault: true` on the kind. `set_store_behavior`,
`PUT /stack/{name}/store_behavior`, `InstanceInfo.store_behavior` and the campaign checks
follow from the flag.

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

Reference: the `checks` of `crates/chaos/src/kinds/ledger.rs`.

Write `async fn name(e: validate::Endpoint) -> Result<String, String>` in the kind's
module (`e.url` is the kind's target, `e.http()`, `e.grpc()`, `e.connect_ws(path)` carry
the trust and the token) and add a `Check { name, surface, doc, run: |e|
Box::pin(name(e)) }` to the kind's `checks`. `Ok(detail)` passes, `Err(detail)` fails;
the runner adds the timeout and the timing. Keep checks independent: they run
concurrently. Then `mise run chaos:docs`.

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

Keep `service/mod.rs`, `stack/`, `topology.rs`, `load/generator.rs`, `load/metrics.rs`,
`scenario/` and `validate.rs` as they are. Replace the modules under `kinds/` with your
kinds (each with its checks) and `load/ops.rs` with your operations. The scenario file
format, the reports, the API and the CLI stay the same.
