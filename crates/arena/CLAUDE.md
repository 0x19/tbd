<!-- tbd new service arena --kind grpc --port 50058 --metrics-port 9471 --bacon-key a (tbd-cli 0.1.0) -->
# crates/arena

The arena service: one live snapshot of what the platform is doing, for the lab's pages
(docs/arena/README.md is the contract). gRPC only; the gateway renders `GetSnapshot` and
`Watch` as REST, SSE and calls on the socket. `Ping` stays the scaffold's labelled stub.

- `world.rs`: the `World`, the one place the collectors write and the ticker reads. A
  source's failure keeps its last figures and records why; `snapshot()` fills each
  tier's rates from the metrics store's map and leaves a rate absent (never zero) when
  nothing measured it. `tick()` broadcasts a whole snapshot every `[watch] tick`.
- `collect/`: one task per configured source. `llm.rs` calls `ListModels` and
  `runner.rs` `ListLanguages`, both as `svc:arena` (`ServiceCaller` in `mod.rs`, the cv
  service's pattern); `metrics.rs` runs the PromQL queries, four per tier and four
  summed for the runner (constants, so tests match them exactly); `chaos.rs` reads `/overview`,
  follows a running run's SSE feed and diffs its `load` frames into rates, turns a new
  `last_validate` report into the surfaces through `SURFACES` (way in to check name),
  and keeps the `arena: every way in` schedule there (recreated within a minute if it goes missing), notifications off.

- `lib.rs`: `serve` and `serve_on` start every configured collector; `serve_with`
  (the chaos kind) starts none, so the chaos tool never watches itself, and still ticks.
- `service.rs`: the `ArenaService` trait impl on `Arena`. Every RPC starts with
  `admit()`: count the request, start the `RequestTimer`, apply the fault handle, map
  a `Fault` to a gRPC status. `GetSnapshot` and `Watch` then check `[watch]
  require_role` (the gate on the site's anonymous socket), and `Watch` takes a viewer
  place (`[watch] max_viewers`, `RESOURCE_EXHAUSTED` "busy: ..." past it), held by the
  stream and given back on drop.
- `config.rs`: layered TOML, `configs/arena/base.toml` < `<env>.toml` < flags and
  `ARENA_*` environment variables (`Overrides`). Every key lives in `base.toml`;
  `deny_unknown_fields` makes a mistyped key fail at start. `arena config` prints
  the effective result.
- `main.rs`: the only file that prints (the `config` subcommand).

- Observability: `tbd_common::telemetry::grpc_request_span` is the `trace_fn`, so every
  call gets a `grpc.request` span with the caller's `traceparent` adopted and
  `trace_id` recorded; `admit()` starts the `RequestTimer`. Metrics listen on
  `[metrics] listen` (`ARENA_METRICS_ADDR`), `None` for embedders.

Invariants:
- A stub says so on the wire: `PingResponse.stub` is `true` until a real implementation
  replaces it, and the tests assert it. Do not let a placeholder look like a
  measurement.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder. With a caller-supplied
  listener the builder setting does nothing, and small responses stall 40 ms.
- Services never address each other directly; a caller reaches this one through
  Envoy's internal listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it/main.rs` boots the server on port 0 through `support.rs` with the
shipped `configs/arena` and env `local`, and exposes the `Runtime` so tests can
inject faults and read counters. `tests/it/snapshot.rs` runs the collectors against a
real llm and a real runner on their stub engines (`support::start_live`) and wiremock servers playing the
metrics store and the chaos tool; `support::as_role` is a caller Envoy verified.
