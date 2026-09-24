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
is also bundled into the admin UI (the knowledge base and the editor's "Reference"
sheet); relative links to other pages under `docs/` resolve there too.

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

[stack.finances.finance-1]
listen = "127.0.0.1:50054"      # optional; `behavior` as for engines; no dependencies
seed = "access"                 # optional; an in-memory world for the access scenarios
database_url = ""               # optional; a Postgres URL for the real store (the books need one)

[stack.humans.humans-1]         # and [stack.playgrounds.<name>]: see kinds.md for every table
```

Instances start in dependency order (engines before the protocols that name them; a
reference to a missing instance or one of the wrong kind fails `chaos check`). Each
instance keeps its port
across a `stop` and `start`, so a restarted engine comes back where the protocol expects
it. Scenarios run on free ports by default so they never collide with a dev stack.

Load targets every instance of a kind that takes load (protocols, ledgers, finances; the
`load target` column of kinds.md), round-robin. Two protocols on one engine is a
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
| `finance_ping` | `FinanceService/Ping` over gRPC on the finance port | the message echoes **and** `stub` is still true; the service is a scaffold, and a stub flag that flips under load is a `contract` failure |
| `finance_money` | `ListTransactions` as the owner, checking the amounts | every amount is the exact minor units it went in as, with the sign its direction implies, `EUR`, scale 2. A magnitude or a sign that changes in flight is a `contract` failure -- the quietest bug in accounting software, because the totals still look plausible |
| `finance_access` | three `ListTransactions` calls as two different callers (`x-jwt-payload`, as Envoy delivers it): the owner, the reader, and the reader naming a party it was not granted | the owner sees both parties, the reader sees the company and **only** the company, and naming a forbidden party directly returns nothing. Any leak is a `contract` failure, which no `max_error_rate` forgives. Needs `seed = "access"` on the instance |
| `finance_trial_balance` | `TrialBalance` of the owner's company for the current year (the first company the owner holds, made through `CreateIssuer` on an empty database and remembered per target) | `balanced`, every row's totals its opening plus its movement, and the rows adding up to the totals the service claims; anything else is a `contract` failure. Needs `database_url` on the instance; without one every call is `UNAVAILABLE`, a transport failure |
| `llm_generate` | `LlmService/Generate` of one of four short prompts, streamed to its done chunk, as caller `chaos-load` | every chunk in order, the stub flag constant within the stream, a done chunk with the engine's usage. The operation meters `prompt_tokens`, `completion_tokens`, `generations`, `answered` (generations whose answer was not empty: a reasoning model can spend a short `max_tokens` thinking and say nothing) and `ttft` (time to the first chunk, reasoning or answer) beside its latency; the latency is the whole stream, so `[load] timeout` must allow a generation (seconds on a real engine, `120s` in the shipped scenario) |
| `llm_generate_deep` | the same on the deep tier (`tier = TIER_DEEP`), so the two tiers are measured with one instrument and told apart by name | as `llm_generate` |
| `finance_import_opening` | `ImportOpeningBalances` of a random balanced set on three accounts of the shipped chart into a year of its own (2001), then `TrialBalance` of that year; every fourth set is off by one cent | the balanced set comes back as the trial balance to the cent over three accounts, and the unbalanced one draws `INVALID_ARGUMENT`; accepted, or refused with anything else, is a `contract` failure. Imports of one year serialise on a lock inside the operation, so the pair runs one at a time per run. Needs `database_url` |

Every operation targets one kind: the four above the ledger rows run against
protocols, the `ledger_*` ones against ledgers, the `finance_*` ones against finances,
`llm_generate` against llms. A mix spreads each operation over its
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
action = "set_behavior"         # kinds with fault injection: the `fault` column of kinds.md
service = "engine-1"
[timeline.behavior]
type = "error"
kind = "unavailable"
rate = 0.5

[[timeline]]
at = "1500ms"
action = "set_store_behavior"   # kinds with store faults (kinds.md): the store fails, not the adapter
service = "ledger-1"
behavior = { type = "error", kind = "unavailable", rate = 0.3 }

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

`set_behavior` is the request adapter: a faulted request is refused before anything
runs. `set_store_behavior` is the service's store failing the way a database does: a
read fails before it runs, a write runs, commits, and then fails, so the client loses
the acknowledgement of something that stands. Only kinds with store faults
([kinds.md](kinds.md)) take it; `chaos check` refuses it on others. The same behaviours
apply to both.

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
| `finance_baseline.toml` | nothing | finance answers under steady gRPC load, p99 under 50 ms, and still says it is a stub |
| `finance_fault.toml` | `unavailable` at 50 % for one second | finance faults arrive as clean gRPC statuses, bounded, then heal |
| `finance_money.toml` | nothing | amounts come back as the exact minor units they went in as, with the right sign |
| `finance_access.toml` | nothing | the reader never receives the owner's personal transaction |
| `finance_access_fault.toml` | `unavailable` at 50 % for one second | the same while the service is failing |
| `finance_books.toml` | nothing; `skip = true`, needs the compose Postgres | opening imports come back as the trial balance to the cent, and an unbalanced one is refused |

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
