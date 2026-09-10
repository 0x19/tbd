# chaos commands

Global flags, valid before or after the subcommand:

| Flag | Env | Default | Meaning |
|---|---|---|---|
| `--log-format text\|json` | `LOG_FORMAT` | `text` | log encoding |
| `--log-filter <directive>` | `RUST_LOG` | see below | `tracing` filter |
| `--env <name>` | `TBD_ENV` | `local` | which `configs/chaos/<env>.toml` to merge over `base.toml` |
| `--config-dir <dir>` | `CHAOS_CONFIG_DIR` | `configs/chaos` | where those files are |
| `--public-domain <domain>` | `CHAOS_PUBLIC_DOMAIN` | `[links] domain` | base domain of the public edge; UI links become `https://grafana.<domain>` and friends |

Every command loads the configuration first ([config.md](config.md)); flags below
override the fields they name.

When neither the flag nor `RUST_LOG` is given, the default filter is
`warn,tbd_chaos=info,tbd_protocol::error=off,tower_http=off`: the tool's own progress at
info, the in-process services only above warn. The services log every injected failure at
error level, which is right in production and noise here.

Exit codes for every command: `0` success, `1` at least one check, scenario or file
failed, `2` bad arguments.

## `chaos validate`

Hit every surface of a running stack. Checks run concurrently, each with its own timeout.

```
chaos validate [--protocol URL] [--engine URL] [--timeout DURATION] [--ca-cert PEM] [--json]
```

| Flag | Env | Default |
|---|---|---|
| `--protocol` | `CHAOS_PROTOCOL_URL` | `[targets] protocol`, `http://127.0.0.1:8080` in `base.toml` |
| `--engine` | `CHAOS_ENGINE_URL` | `[targets] engine`, `http://127.0.0.1:50051` |
| `--timeout` | | `[validate] timeout`, `5s` |
| `--ca-cert` | `CHAOS_CA_CERT` | `[validate] ca_cert`, none: `https://`/`wss://` targets verify against the public roots |
| `--json` | | off |

Targets may be `http://` (h2c, plain WebSocket) or `https://` (TLS; WebSocket becomes
`wss://`). Both URLs can be the same public edge, `https://api.<base>`: Envoy routes
gRPC by service name, and the protocol's health service answers for the engine too.
`--ca-cert` adds one PEM root (several concatenated are fine) for a staging edge or
Caddy's internal CA; it never disables verification.

Checks, in output order:

| Name | Surface | Passes when |
|---|---|---|
| `http_healthz` | HTTP | `GET /healthz` is 2xx |
| `http_readyz` | HTTP | `GET /readyz` is 2xx, meaning the protocol reaches the engine |
| `rest_evaluate` | REST | `POST /v1/evaluate` returns the subject and a boolean `stub` |
| `sse_events` | SSE | `GET /v1/subjects/{id}/events` delivers two events |
| `graphql_evaluate` | GraphQL | the query has no errors and `engineReady` is true |
| `ws_echo` | WebSocket | a text frame on `/ws` comes back as a `data` frame |
| `grpc_engine_health` | gRPC | `grpc.health.v1.Health/Check` on the engine reports `SERVING` |
| `grpc_engine_evaluate` | gRPC | `EngineService/Evaluate` answers |
| `grpc_engine_subscribe` | gRPC | `EngineService/Subscribe` delivers two events |
| `grpc_protocol_health` | gRPC | health on the protocol port answers |
| `grpc_protocol_ping` | gRPC | `ProtocolService/Ping` echoes the message |

The two streaming checks wait for two events, so they take about one heartbeat interval.

Text output, one line per check:

```
PASS  http_healthz           http          2.3 ms  200 OK ok
FAIL  grpc_engine_evaluate   grpc          0.5 ms  transport error

10 passed, 1 failed
```

JSON output:

```json
{
  "checks": [
    { "name": "http_healthz", "surface": "http", "passed": true,
      "latency_ms": 2.3, "detail": "200 OK ok" }
  ],
  "passed": 10,
  "failed": 1
}
```

`detail` is what was observed on success and the error on failure. A timeout reads
`timed out after 5s`.

## `chaos up`

Start the `[stack]` of a topology or scenario file in this process and keep it running
until Ctrl-C. Prints each instance's address and a ready-made `chaos validate` line.

```
chaos up [FILE]          # default: [paths] topology, topologies/dev.toml
```

Any file with a `[stack]` table works, so a scenario file doubles as a topology. Other
tables in the file are ignored. See [scenarios.md](scenarios.md#stack) for the table.

## `chaos run`

Run scenario files. Each scenario gets a fresh stack on free ports, runs its load and
timeline, evaluates its assertions, and tears down before the next one starts.

```
chaos run [FILES...] [--dir DIR] [--json]
```

| Argument | Meaning |
|---|---|
| `FILES` | paths or glob patterns (`'scenarios/ws_*.toml'`, quote to keep the shell out of it) |
| `--dir`, `-d` | every `*.toml` under the directory, recursively |
| `--json` | one JSON array of results at the end instead of text per scenario |

Files are de-duplicated and run in sorted path order. A file that fails to parse or
cross-check is reported as a failed scenario with the error, and the run continues. A
scenario with `skip = true` is reported as `SKIP` and counts as passed.

Text output is one block per scenario, then a summary line
`4 passed, 0 failed, 0 skipped`. The block format is described in
[README.md](README.md#five-minute-tour).

JSON output, one element per scenario:

```json
{
  "name": "error_injection",
  "file": "scenarios/error_injection.toml",
  "passed": true,
  "skipped": false,
  "duration_s": 3.3,
  "load": {
    "elapsed_s": 3.0,
    "requests_total": 599, "requests_success": 499, "requests_failed": 100,
    "error_rate": 0.1669, "throughput_rps": 199.6,
    "latency": { "p50_ms": 1.2, "p90_ms": 1.8, "p99_ms": 2.2, "max_ms": 2.3, "mean_ms": 1.3 },
    "per_target": { "protocol-1": { "total": 599, "failed": 100 } },
    "per_op": { "rest_evaluate": { "total": 599, "failed": 100, "latency": { "p50_ms": 1.2, "...": 0 } } },
    "errors": { "http 503": 100 }
  },
  "services": { "engine-1": { "total": 658, "failed": 100 } },
  "events": [
    { "at_s": 1.0, "action": "set_behavior engine-1 Error { .. }", "error": null }
  ],
  "assertions": [
    { "name": "max_error_rate", "passed": true, "expected": "<= 30.00%", "actual": "16.69%" }
  ],
  "error": null
}
```

Field notes:

- `load` is absent when the scenario has no `[load]`.
- `latency` covers successful requests only; failures are counted, not timed.
- `services` holds engine-side counters at the end of the run. After a restart they
  belong to the new instance, so they cover the time since the restart.
- `events` are timeline actions as applied; `error` is set when applying failed, which
  fails the scenario.
- `error` at the top level is a failure outside assertions: setup, a panicked task, or a
  file that did not parse.

## `chaos check`

Parse scenario files and run every check that does not need a running stack: unknown
keys, references to services that do not exist, `set_behavior` on something without
fault injection, load assertions without a `[load]` section.

```
chaos check FILES...
```

Prints `ok <file> (<name>)` or `error <message>` per file and exits 1 if any failed.

## `chaos serve`

Run the tool as an HTTP API for the admin UI and for scripts, with the topology stack
in-process, until Ctrl-C. The API is documented in [api.md](api.md).

```
chaos serve [--listen ADDR] [--base-path PATH] [--ui-dir DIR] [--topology FILE]
            [--scenarios DIR] [--results DIR] [--no-stack] [--protocol URL] [--engine URL]
```

| Flag | Env | Default |
|---|---|---|
| `--listen` | `CHAOS_LISTEN_ADDR` | `[serve] listen`, `127.0.0.1:7700` |
| `--base-path` | `CHAOS_BASE_PATH` | `[serve] base_path`, `/api/chaos/v1` |
| `--ui-dir` | `CHAOS_UI_DIR` | `[serve] ui_dir`, none |
| `--topology` | `CHAOS_TOPOLOGY` | `[paths] topology` |
| `--scenarios` | `CHAOS_SCENARIOS_DIR` | `[paths] scenarios` |
| `--scenarios-seed` | `CHAOS_SCENARIOS_SEED` | `[paths] scenarios_seed`, none; seeds `--scenarios` when empty |
| `--results` | `CHAOS_RESULTS_DIR` | `[paths] results`, `.chaos/results` |
| `--no-stack` | | off; sets `[serve] start_stack = false` |
| `--protocol`, `--engine` | `CHAOS_PROTOCOL_URL`, `CHAOS_ENGINE_URL` | `[targets]`, the defaults for API validate |

On start it prints the API URL, the UI URL when configured, and each stack instance. It
exits 1 when the stack cannot start (typically the fixed ports of `topologies/dev.toml`
are taken) or the results directory cannot be created. `mise run chaos:serve` runs it
with `--ui-dir ui/chaos/out`.

## `chaos config`

Print the effective configuration for `--env` as TOML, preceded by comments naming the
files that were merged. Exits 1 when a file is missing, does not parse, or has an
unknown key.

```
chaos config
chaos --env dev config
```
