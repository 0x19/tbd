# How chaos works

```
crates/chaos/src
├── main.rs            CLI: up, validate, run, check, serve, config
├── config.rs          configs/chaos/ schema, loaded via tbd_common::config
├── topology.rs        this project's [stack] TOML → launchers      (replace per project)
├── api/               chaos serve
│   ├── mod.rs         router: API under base_path, built UI under ui_path
│   ├── state.rs       AppState: config, the long-lived stack, run jobs, global feed
│   ├── runs.rs        RunRecord/RunStore (JSON files), ActiveRun (live feed)
│   ├── routes.rs      handlers and SSE
│   └── error.rs       ApiError → status + {"error"}
├── service/           the Service extension point (engine, protocol, ledger adapters)
│   ├── mod.rs         Service, InstanceHandle, Instance, Peers
│   ├── engine.rs      engine adapter: serve_with + Runtime (fault, counters)
│   └── protocol.rs    protocol adapter: serve_on, /readyz
├── stack/mod.rs       generic runner: order, readiness, stop/start, shutdown
├── load/
│   ├── mod.rs         LoadConfig, Pattern
│   ├── ops.rs         the Operation extension point + four operations + Clients
│   ├── generator.rs   open-loop pacer, JoinSet, drain
│   └── metrics.rs     HDR histograms and counters, snapshot
├── scenario/
│   ├── config.rs      ScenarioFile and cross-checks
│   ├── timeline.rs    TimelineEvent and apply()
│   ├── assertions.rs  Assertions over an immutable Snapshot
│   ├── executor.rs    the one executor
│   └── report.rs      text rendering
└── validate.rs        the checks and their concurrent runner
```

## Services and the stack

A `Service` knows how to start one instance on an address and returns an `Instance`.
The instance's handle answers `ready()`, stops on request, and optionally exposes a
`FaultHandle` and request counters. The stack never learns what a service is.

```
StackConfig (topology.rs)          Stack (stack/mod.rs)
  engines: {name → spec}   ──►      launchers: {name → (Arc<dyn Service>, listen addr)}
  protocols: {name → spec}          instances: {name → Instance}
```

`Stack::start` repeatedly starts every launcher whose `depends_on()` are all running,
so engines come up before the protocols that name them. After each start it polls
`ready()` up to ten seconds. On the first start of an instance bound to port 0 the chosen
port is written back into the launcher, so `stop_instance` followed by `start_instance`
lands on the same address. `shutdown` stops leaves first: anything no running instance
depends on. `add_instance`, `clone_instance` and `remove_instance` change the launcher
set at runtime (what `chaos serve` exposes as replicas): a new launcher is built from
the same spec on a fresh port, and only instances added this way can be removed.
`api/added.rs` writes them to `[paths] stack` so a restart re-adds them.

Both adapters run the real service's library entry point on a tokio task and stop it
through the same graceful-shutdown future `main` uses. The engine adapter builds a
`tbd_engine::Runtime` and keeps it, which is where the fault handle and counters come
from.

## Faults

`tbd_common::fault::FaultHandle` is a shared, runtime-mutable `Behavior`. The engine
calls `apply()` at the start of every RPC and `stream_error()` per emitted stream item.
`apply` sleeps for `slow`, never returns for `hang`, and returns a `Fault` for `error`
that the engine maps to a gRPC status. `delayed_failure` is resolved against the time the
behaviour was set. The type is transport-free so any service in any project can adopt it.

## Load

`load::run` builds one `Clients` (a pooled HTTP client, a WebSocket pool per target, a
lazy gRPC channel per target) and runs an optional warmup phase followed by the measured
phase against the same `Metrics`, resetting in between.

The pacer is open loop with an absolute deadline: `next += 1 / rate_at(elapsed)`, sleep
until `next`, then acquire a permit from a semaphore of `max_in_flight`, pick a target
round-robin and an operation by weight, and spawn the request into a `JoinSet`. Finished
tasks are reaped every iteration so the set stays small. When the duration ends, the
generator waits for in-flight requests up to one timeout plus a margin, then aborts the
rest.

Each request is wrapped in the per-request timeout. Its outcome is `Ok`, an `OpError`
class, or `"timeout"`, and `Metrics::record` files it globally, per target, per operation
and per error class. Latency goes into HDR histograms, one global and one per operation,
successful requests only.

## Scenarios

`run_scenario` is the only executor:

1. Start the stack from `[stack]`. Failure here is reported as `error`.
2. Collect the protocol instances as load targets.
3. Spawn the timeline task with the load-start instant. It sleeps until each event's
   offset, locks the stack, applies the event, and records the outcome.
4. Run the load, or if there is none, sleep until the last timeline event.
5. Join the timeline, take the stack back, read engine counters.
6. Evaluate assertions over a `Snapshot` of the load metrics and counters.
7. Shut the stack down.

`passed` requires no top-level error, every timeline event applied cleanly, and every
assertion true. Assertions are synchronous functions over an immutable snapshot; there
is nothing to await or lock inside them.

## Validate

Each check is `async fn(Targets) -> Result<String, String>`. The runner spawns all of
them into a `JoinSet` with the timeout wrapped around each, collects results, restores
the declared order, and counts. Text and JSON render the same `Report`.

## Serve

`chaos serve` wraps the same pieces in an axum router ([api.md](api.md)). The stack from
the topology lives in `AppState` behind a mutex for the life of the process; stack
routes lock it, act, and broadcast `stack_changed`. Runs go through hooks the executor
and the load generator already expose: `scenario::Hooks` and `load::Hooks` carry an
event channel and a `CancellationToken`, so the same `run_scenario_with` that the CLI
calls (with default hooks) streams phases, per-second snapshots and timeline events
to the API, and stops early on cancel. A run's events are kept in memory while it is
active (late clients get a replay) and its final record is one JSON file under
`[paths] results`. One run is active at a time; scenario runs use their own ephemeral
stack so the serve stack is never disturbed.

## Conventions the code relies on

- Instance names are unique across kinds; a name cannot be both an engine and a
  protocol.
- `set_behavior` requires the target's handle to expose a `FaultHandle`; today only
  engines do.
- Engine counters are per instance and reset on restart.
- The protocol's engine channel is lazy and reconnecting, which is why an engine restart
  needs no action on the protocol.
- WebSocket clients set `TCP_NODELAY` themselves; `tokio_tungstenite::connect_async`
  does not, and the 40 ms it costs would be attributed to the server.
