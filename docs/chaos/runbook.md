# Runbook

What a failure means, where to look, what to do. One row per symptom, grouped by the
part of the tool that reports it. Add a row whenever a failure taught something; the
admin UI renders this page in its knowledge base, next to the observability links of
the environment it runs in ([observability](../observability/README.md) has the
signal-by-signal map).

## Validate

One check per surface. A failing check names the surface that is broken.

| Symptom | What it means | Look at | Then |
|---|---|---|---|
| `http_readyz` fails, `http_healthz` passes | The protocol is up but cannot reach its engine. Through Envoy: no healthy engine endpoint. | Stack page (is the engine running?), Envoy admin `/clusters` for the engine cluster, engine pod logs. | Start the engine, or fix the engine URL (it must be Envoy's engine LB). |
| Every gRPC check fails with a transport error | Nothing listens on the engine URL, or it is HTTP/1.1 only (gRPC needs h2c or TLS). | The engine URL in Targets; `grpcurl -plaintext <host:port> list`. | Point at the engine LB (`:15051` locally), not the edge; check the port map in [the local cluster](../local-cluster.md#ports). |
| `sse_events` or `grpc_engine_subscribe` time out | Streams are cut before two events arrive: a proxy with a stream timeout, or a heartbeat longer than the check timeout. | Envoy route timeouts (SSE and gRPC routes must be `0s`), the engine's heartbeat setting. | Raise the timeout, or fix the route. The two streaming checks take about one heartbeat interval. |
| `ws_echo` fails, everything else passes | The WebSocket upgrade is not forwarded, or the first frame stalls. | Envoy `upgrade_configs` on the edge; a 40 ms stall points at Nagle (`TCP_NODELAY`). | Route `/ws` with the websocket upgrade; clients and servers set `TCP_NODELAY`. |

## Scenarios

A run is setup, load with a timeline, assertions, teardown. Each part fails differently.

| Symptom | What it means | Look at | Then |
|---|---|---|---|
| Run status `error`, message starts with `setup:` | The stack did not start: a fixed port is taken, or a protocol references an engine that never became ready. | The error text; `ss -ltnp` for the port; the scenario's `[stack]` names. | Scenarios should not use fixed ports. Free the port or drop `listen`. |
| `max_error_rate` fails, errors are all `http 503` | The engine was down or returning `UNAVAILABLE` for longer than the scenario allows. | The timeline: a stop without a start, or a start that errored. Per-service counters. | Check the timeline offsets against the load duration; widen the bound only if the scenario proves something else. |
| `max_p99_ms` fails on CI, passes locally | CI runners are slower and noisier; the bound came from the fastest machine. | p99 in the run record versus the bound; the baseline scenario's p99 on the same runner. | Set bounds from what the scenario proves ([choosing bounds](scenarios.md#choosing-bounds)). Loopback p99 here is about 3 ms; baseline allows 50. |
| A timeline event has an error | The action could not be applied: unknown instance, already running, or no fault injection. | The action text; only kinds with fault injection take `set_behavior` (the Stack page's Fault buttons show which). | Fix the instance name or the action; `chaos check` catches references before running. |
| Throughput is far below `rate` | The pacer stalled at `max_in_flight`: latency times rate exceeds the concurrency cap. | p99 during the run; `requests_total` versus rate × duration. | Raise `max_in_flight`, or accept that a slow engine limits throughput, which is what the run shows. |

## Stress campaigns

A campaign judges every answer against the ledger's contract. A finding is a bug in the
ledger or in the model, never a threshold to loosen ([stress](stress.md#debugging-a-finding)).

| Symptom | What it means | Look at | Then |
|---|---|---|---|
| A finding with `shrink_note: not reproduced on a fresh replay` | The rule broke because of something outside the subject's trace: a peer erased meanwhile, a fault, a race. | The last step's request and the ledger's answer; whether the message names a peer or a counterparty. | Decide from the contract whether the ledger's answer was allowed. If it was, the model is wrong: fix the worker and the interpreter together. |
| Every campaign fails with `clean_errors` transport findings | The workers lost the connection: the ledger restarted, or the pod hit its memory limit. | The ledger pod's restarts and `OOMKilled` events; the finding's `at_ms` against the timeline. | Tolerate the class in `[faults]` only when the campaign injects that fault on purpose; otherwise size the pod. |
| A campaign passes in CI and fails in the cluster | Timing differs: erase cycles complete, pages split, keys collide in a slower environment. | The finding's trace; run the same campaign with `--seed` and the finding's replay against the memory store. | Reproduce on the memory store first; a finding that only recurs on Postgres is a store bug. |

## Serve and the UI

The API behind the pages and the pod it runs in.

| Symptom | What it means | Look at | Then |
|---|---|---|---|
| "chaos API unreachable" in the header | The browser cannot reach `/api/chaos`: serve is down, or `next dev` is talking to the wrong port. | `curl <api>/healthz`; `NEXT_PUBLIC_CHAOS_API` in dev; Envoy's `chaos` cluster in the cluster. | Start `mise run chaos:serve`, or set the API URL. |
| "Live feed off" in the footer | The SSE connection dropped: a proxy timed the stream out, or serve restarted. | The Envoy route for `/api/chaos/` must have `timeout: 0s`; serve logs. | The page reconnects on its own; reload if it does not. |
| `409 a run is already active` | One run at a time, so numbers are not mixed. | The Runs page shows the active run. | Wait, or cancel it. |
| Saving a scenario fails with "read-only file system" | The scenarios directory is inside the image. | `CHAOS_SCENARIOS_DIR` and `CHAOS_SCENARIOS_SEED` on the pod ([config](config.md)). | Point the directory at a volume; the seed fills it on start. |
| A new scenario or campaign is not listed after a rollout | Seeding is additive: files the volume already holds are kept, new ones are copied. | The pod's `/data/scenarios` against the image's `/app/scenarios`. | Delete the stale copy in the UI or on the volume; the next start seeds the file again. |
| Signed out in the middle of scheduling | The ID-token cookie expired and the refresh was rejected; the page reloads through sign-in. | Hydra's refresh-token rotation settings; Envoy's `deny_redirect_matcher` for `/api/`. | Sign in again; the page comes back to the same URL. |

## Observability

Every request carries a trace id: logs, traces and profiles for the same second are one
click apart.

| Symptom | What it means | Look at | Then |
|---|---|---|---|
| A run failed and you want the server side | The engine and protocol logged every failure with the trace id of the request. | Logs, filtered by the time of the run and level `error`; Grafana's `tbd / protocol` and `tbd / engine` dashboards. | Take a `trace_id` from a log line into Explore (Tempo) to see the whole request across Envoy, protocol and engine. |
| Latency is high and nothing errors | CPU or lock contention, not failures. | Profiles (Pyroscope) for the run's minute; Envoy upstream time in the `tbd / envoy` dashboard. | Compare the flame graph against a healthy minute; the hot frame names the code. |
| The ledger dashboard is flat during a load run | The load targeted another ledger, or the metrics scrape is behind. | The run's targets; the `tbd / ledger` dashboard's time range; the pod's `/metrics`. | Target the ledger through Envoy from `[targets]`; widen the range. |
