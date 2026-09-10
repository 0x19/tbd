# chaos

One tool for validating, load-testing and fault-testing the stack, on a laptop and in CI.
It starts the real services in this process, drives them through their real surfaces,
injects faults into them at runtime, and asserts on what happened. No mocks, no
containers, no ports to free up afterwards.

It is also a framework. This project plugs in two services (engine, protocol) and four
operations (REST, GraphQL, WebSocket, gRPC). A future project plugs in its own and reuses
the runner, the load generator, the timeline, the assertions and the reports unchanged.

| Command | Use it to | Page |
|---|---|---|
| `chaos validate` | prove a running stack answers on every surface | [commands.md](commands.md#chaos-validate) |
| `chaos up` | run engines and protocols in one process while developing | [commands.md](commands.md#chaos-up) |
| `chaos run` | execute scenarios: load, faults on a timeline, assertions | [commands.md](commands.md#chaos-run) |
| `chaos check` | validate scenario files without running them | [commands.md](commands.md#chaos-check) |
| `chaos serve` | run the tool as an HTTP API for the admin UI: stack, scenarios, runs, live progress | [api.md](api.md) |
| the admin UI | the same, in a browser: `ui/chaos`, served at the root of its host | [ui.md](ui.md) |
| `chaos config` | print the effective `configs/chaos/` configuration for an environment | [config.md](config.md) |

Further reading: [scenarios.md](scenarios.md) for writing scenarios,
[config.md](config.md) for the layered configuration, [api.md](api.md) for the HTTP
API the UI uses, [architecture.md](architecture.md) for how it works,
[extending.md](extending.md) for adding services, operations, actions, assertions and
checks.

## Five-minute tour

```sh
mise run setup                     # once: installs the tools

mise run chaos:up                  # terminal 1: engine on :50051, protocol on :8080
mise run validate                  # terminal 2: 11 checks, one line each
chaos validate --json              # same, for machines

mise run chaos:run                 # all scenarios, fresh stack per scenario, exit 1 on failure
chaos run scenarios/latency.toml   # one scenario
chaos check scenarios/*.toml       # parse and cross-check without running

mise run chaos:serve               # API on :7700 + dev stack; the admin UI when built
curl -s localhost:7700/api/chaos/v1/overview | jq .
```

What a passing scenario looks like:

```
PASS  error_injection  (3.3s)
      file      scenarios/error_injection.toml
      load      599 req, 16.69% errors, 199.6 rps, p50 1.2 ms, p99 2.2 ms, max 2.3 ms
      op        rest_evaluate          599 sent    100 failed   p50    1.2 ms  p99    2.2 ms  max    2.3 ms
      error     http 503               100
      service   engine-1               658 served    100 failed
      event       1.00s  set_behavior engine-1 Error { kind: Unavailable, rate: 0.5, .. }
      event       2.00s  set_behavior engine-1 Healthy
      ok  max_error_rate               expected <= 30.00%      actual 16.69%
      ok  engine-1.max_failed          expected <= 200         actual 100
```

Read it top down: what load was sent and how it did, per operation, which errors
occurred and how often, what each service instance counted on its own side, what the
timeline did and when, and each assertion with its bound and the observed value.

## What it can do today

- Start any number of engines and protocols on free or fixed ports, in dependency order,
  and wait for each to be ready.
- Generate open-loop load at a constant or ramping rate over a weighted mix of REST,
  GraphQL, WebSocket and gRPC operations, spread round-robin across protocol instances.
- Make an engine slow, failing at a rate, hung, or healthy-then-failing, at any second of
  the run.
- Stop and restart any instance mid-run on the same port.
- Assert on error rate, latency percentiles, throughput, request counts, and per-service
  counters.
- Report as text or JSON with a non-zero exit on failure, so it runs unattended in CI.
- Serve all of it over HTTP (`chaos serve`): stop, start and fault instances by hand,
  edit and push scenario files, run scenarios and ad-hoc load with per-second progress
  over Server-Sent Events, keep every run as a JSON record, and validate the deployed
  stack through Envoy. This is what the admin UI in `ui/chaos` drives.
- Read its configuration from `configs/chaos/base.toml` merged with the environment's
  file (`local`, `dev`, `production`), overridable by flags and env vars.

## What it does not do yet

- Run a scenario against a stack it did not start. `validate` works against any URL and
  an ad-hoc load run through the API takes explicit targets, but `run` always starts its
  own stack because the timeline needs fault handles on the instances.
- Scenarios defined in Rust. Only TOML, though the executor is written so a Rust source
  can be added.
- Burst and step load patterns, per-operation assertions, multi-engine failover through
  the protocol.

Each of these has a documented seam in [extending.md](extending.md).

## Frequently hit

**A scenario passes locally and fails in CI on latency.** CI runners are slower and
noisier. Loopback p99 here is about 3 ms; `baseline.toml` allows 50. Set bounds from what
the scenario proves, not from the fastest machine.

**`chaos up` says the address is in use.** Something else, often a stack from a previous
terminal, holds the port. `ss -ltnp | grep 50051` finds it.

**Every request fails with `http 503` in a scenario.** The engine is down or the
protocol cannot reach it. Check the `event` lines: a `stop` without a `start`, or a
`start` that errored, shows there.

**The service logs flood the output.** Under `mise run chaos:run` they are silenced by
the task's `RUST_LOG`. Running the binary directly, pass `--log-filter
'warn,tbd_chaos=info,tbd_protocol::error=off,tower_http=off'` or set `RUST_LOG`.

**Which engine served a request?** Engine-side counters are in the `service` lines and
in `services` in JSON. Client-side counts per protocol instance are under `per_target`.
