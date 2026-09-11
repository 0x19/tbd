# Writing scenarios

A scenario is one TOML file with up to five tables. Every table rejects unknown keys, so
a typo fails at `chaos check` instead of silently doing nothing.

```
[scenario]      name, description, skip
[stack]         which instances of which kinds to run
[load]          how much traffic, of what kind, for how long
[[timeline]]    what happens while load runs, and when
[assertions]    what must be true afterwards
```

The shipped files under `scenarios/` are the reference. Copy the closest one. This page
is also bundled into the admin UI (the Scenario reference page and the editor's
"Reference" sheet), so keep it self-contained: no relative links to other pages.

## `[scenario]`

```toml
[scenario]
name = "latency"                # required; shown in reports
description = "..."             # optional
skip = false                    # optional; reported as SKIP when running a directory
```

## `[stack]`

Same table `chaos up` uses. Names are yours. Each table under `[stack]` is a kind's
plural, `[stack.<plural>.<name>]`; the kinds, their tables and their keys are in
[kinds.md](kinds.md). `listen` is common to all; the other keys are the kind's own.

```toml
[stack.engines.engine-1]
listen = "127.0.0.1:50051"      # optional; omit for any free loopback port
heartbeat = "1s"                # optional; stream heartbeat interval
[stack.engines.engine-1.behavior]
type = "healthy"                # optional initial behaviour, see Behaviours

[stack.protocols.protocol-1]
listen = "127.0.0.1:8080"       # optional
engine = "engine-1"             # required; the engine this protocol forwards to

[stack.ledgers.ledger-1]
listen = "127.0.0.1:50052"      # optional; `behavior` as for engines; no dependencies
grace = "7d"                    # optional; the erasure window (0s to watch erasures execute)
database_url = ""               # optional; a Postgres URL for the real store, else in memory
```

Instances start in dependency order (engines before the protocols that name them; a
reference to a missing instance or one of the wrong kind fails `chaos check`). Each
instance keeps its port
across a `stop` and `start`, so a restarted engine comes back where the protocol expects
it. Scenarios run on free ports by default so they never collide with a dev stack.

Load targets every instance of a kind that takes load (protocols), round-robin. Two protocols on one engine is a
valid way to test the protocol under a split load.

## `[load]`

Optional. A scenario without it only plays its timeline.

```toml
[load]
rate = 100                      # requests/s across all targets; default 50
duration = "3s"                 # required; the measured window
warmup = "300ms"                # optional; same load first, metrics discarded
timeout = "5s"                  # optional; per request, counts as failure class "timeout"
max_in_flight = 256             # optional; the pacer stalls when reached

[load.pattern]                  # optional; default constant
type = "constant"

[load.pattern]
type = "ramp"                   # linear from start_rate to end_rate over duration
start_rate = 10
end_rate = 200

subjects = 100                  # optional; the subject pool of the ledger operations
seed = 0                        # optional; seeds ledger_fuzz and the pool

[[load.operations]]             # weighted mix; default is rest_evaluate only
op = "rest_evaluate"
weight = 3

[[load.operations]]
op = "ws_echo"
weight = 1
```

Operations and what they exercise:

| `op` | Path | Success means |
|---|---|---|
| `rest_evaluate` | `POST /v1/evaluate` | 2xx and a boolean `stub` in the body |
| `graphql_evaluate` | `POST /graphql` | 2xx and no `errors` |
| `ws_echo` | `/ws` | the sent frame comes back as `data`; connections are pooled per target |
| `grpc_ping` | `ProtocolService/Ping` over h2c on the protocol port | the message echoes |
| `ledger_append` | `LedgerService/Append` on a pooled subject | a fact with an id comes back |
| `ledger_current` | `LedgerService/Current` on a pooled subject | an answer, or `NotFound` before the subject's first write |
| `ledger_history` | `LedgerService/History` with a random cut and page size | same |
| `ledger_retract` | `LedgerService/Retract` of a pooled path | an answer, or `NotFound` when nothing was valued |
| `ledger_lifecycle` | append, current, retract, history cut on a fresh subject | current showed the fact, the cut does not show the value: the retraction rule per request |
| `ledger_erase_cycle` | append, erase, restore, erase, on a fresh subject; then, when the grace the ledger announces is shorter than 1.5 s (`[stack.ledgers.X] grace = "0s"`), wait past it | reads denied while erased, back after restore, and the subject gone after the window when it was awaited; against a ledger with the real grace the request passes after the denial |
| `ledger_fuzz` | seeded hostile requests (`[load] seed`) | a clean refusal (`InvalidArgument`, `NotFound`, `FailedPrecondition`, `ResourceExhausted`, `OutOfRange`, `Aborted`) or a clean answer; `Internal`, `Unknown` or a dropped connection is a `contract` failure |

Every operation targets one kind: the four above the ledger rows run against
protocols, the `ledger_*` ones against ledgers. A mix spreads each operation over its
own kind's instances, and `chaos check` refuses a scenario whose stack lacks a kind an
operation needs. `grpc_ping` never reaches the engine. Mixing it in shows whether a
problem is in the protocol or behind it. The ledger operations share a pool of
`[load] subjects` (default 100) and a `[load] seed` (default 0) so a run is
reproducible.

Load is open loop: requests are scheduled on an absolute clock at the configured rate
regardless of how long earlier ones take. A slow backend shows up as latency, not as a
quietly lower rate. If `max_in_flight` is reached the pacer waits, throughput drops, and
`min_throughput` can catch it.

## `[[timeline]]`

Offsets are from the moment load starts, after warmup. Events are sorted by `at`.

```toml
[[timeline]]
at = "1s"
action = "set_behavior"         # kinds with fault injection: engines, ledgers
service = "engine-1"
[timeline.behavior]
type = "error"
kind = "unavailable"
rate = 0.5

[[timeline]]
at = "2s"
action = "stop"                 # any instance; its port stays reserved
service = "engine-1"

[[timeline]]
at = "3s"
action = "start"                # restart on the same port
service = "engine-1"

[[timeline]]
at = "4s"
action = "log"                  # a marker in the log and the report
message = "engine has been back for one second"
```

A timeline action that fails, for example `start` on something already running, fails
the scenario and is shown on the `event` line.

### Behaviours

The same table wherever a behaviour appears: initial `[stack.engines.X.behavior]` or a
`set_behavior` event.

```toml
type = "healthy"

type = "slow"
latency = "50ms"
jitter = "20ms"                 # optional; uniform, added on top

type = "hang"                   # never answers; clients hit their timeout

type = "error"
kind = "unavailable"            # unavailable | internal | overloaded | timeout
rate = 0.3                      # optional; default 1.0
message = "chaos"               # optional

type = "delayed_failure"
healthy_for = "3s"              # measured from when the behaviour was set
[then]
type = "error"
kind = "internal"
```

What each `kind` becomes on the wire, and therefore which error class the report shows:

| `kind` | gRPC (direct to engine) | HTTP through the protocol | report class |
|---|---|---|---|
| `unavailable` | `UNAVAILABLE` | 503 | `http 503` |
| `internal` | `INTERNAL` | 500 | `http 500` |
| `overloaded` | `RESOURCE_EXHAUSTED` | 429 | `http 429` |
| `timeout` | `DEADLINE_EXCEEDED` | 504 | `http 504` |

Streams: an `error` behaviour also fires per emitted item, so a live `Subscribe` stream
ends with the injected status.

## `[assertions]`

All optional. Absent means not checked. Bounds are inclusive.

```toml
[assertions]
max_error_rate = 0.01           # failed / total over the measured window
max_p50_ms = 50                 # over successful requests
max_p99_ms = 200
min_requests = 250              # sent in the measured window
min_throughput = 90             # requests/s

[assertions.services.engine-1]  # an engine: its own counters
min_requests = 200
max_requests = 100000
max_failed = 0

[assertions.services.protocol-1] # a protocol: what the generator sent it
min_requests = 100
```

## Choosing bounds

Derive each bound from what the scenario injects, then leave headroom for slower
machines. Worked example, `error_injection.toml`: 3 s of load, the engine fails 50 % of
requests for 1 s of it. Expected error rate is about 17 %. The bound is 30 %: it catches
the failure mode that matters (errors far above the injected rate, or the protocol
falling over) without flaking on timing.

For latency, loopback p99 on a developer machine is about 3 ms. `baseline.toml` asserts
50 ms: generous enough for CI, tight enough to catch the 40 ms Nagle stall this scenario
originally found.

## The shipped scenarios

| File | Injects | Proves |
|---|---|---|
| `baseline.toml` | nothing | a mixed workload over all four operations runs clean; sets the latency floor |
| `error_injection.toml` | `unavailable` at 50 % for one second | faults surface as real 503s bounded by the injected rate, then heal |
| `latency.toml` | 50 ms ± 20 ms after one second | the median and p99 move with the injection, with no errors |
| `engine_restart.toml` | `stop`, then `start` a second later | the protocol fails fast instead of hanging and recovers on its own |
| `ledger_baseline.toml` | nothing | a mixed append/read/retract load on the ledger runs clean under 50 ms p99 |
| `ledger_lifecycle.toml` | nothing | the retraction rule holds for every request under concurrency |
| `ledger_fault.toml` | `unavailable` at 50 % for one second | ledger faults surface as clean gRPC errors, bounded, then heal |
| `ledger_erasure.toml` | a zero grace window | erase, restore, erase again: the subject is gone after the window |
| `ledger_fuzz.toml` | seeded hostile requests | the ledger refuses cleanly and never answers `Internal` |

## Debugging a failing scenario

1. Read the assertion line: it shows the bound and the observed value.
2. Read the `error` lines: the class tells you the layer. `http 503` is the engine
   unreachable or injecting `unavailable`; `timeout` is a hang or an overloaded box;
   `transport` is a connection that could not be made or broke; `contract: ...` is a
   well-formed reply that violates the API.
3. Read the `event` lines: an action with `(error)` after it failed to apply.
4. Compare `services` with `per_target`: the engine counts what reached it, the
   generator counts what it sent. A gap is the protocol.
5. Run the scenario alone with full logs:
   `chaos run scenarios/x.toml --log-filter info`.
