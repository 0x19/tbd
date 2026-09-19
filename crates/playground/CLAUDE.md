<!-- tbd new service playground --kind grpc --port 50055 --metrics-port 9469 --bacon-key g (tbd-cli 0.1.0) -->
# crates/playground

The public sandbox: one shared stack under constant load that anyone on the internet may
try to break, served over REST, SSE and the socket through the gateway. The game's rules
and its limits are [docs/playground/README.md](../../docs/playground/README.md); read that
first. Scaffolded by `tbd new service`; `Ping` is still a labelled stub beside the four
real RPCs.

Invariants that are the whole point of the service:
- **The wire carries a move, a name and nothing else.** No duration, no rate, no address.
  `faults.rs` holds every parameter, so there is nothing to clamp at runtime. If a future
  RPC accepts a number from a caller, it is wrong until it is bounded and tested.
- **`hang` and `delayed_failure` stay unreachable.** A test fails if either becomes
  injectable; the first never returns, the second nests without bound.
- **Load targets come from the stack.** `kind::load_targets(&stack, CORE)` and nowhere
  else, so this can never be pointed at another host. A balanced kind is addressed at
  its balancer instead of per replica (`World::targets`), or the load would route
  around nothing.
- **One fault is survivable; the tests say so.** `tests/it/game.rs` boots the real
  sandbox on the shipped configuration and fails if a single move breaches or if
  breaking both engines does not. The balancer policy in `sandbox.rs` was set by
  measurement; the numbers are in its doc comment. Retune it there, and re-measure.
- **The ledger stays single.** Two in-memory ledgers behind round-robin break
  read-after-write (measured: 1.7% failures with nothing injected). `stall store` is
  rated under the budget because nothing routes around it; `faults.rs` tests that.
- **Every traffic number is one second's.** `traffic.rs` drains the generator's
  metrics each tick; the world judges a window of those. Never feed it a cumulative
  snapshot again — that was the "it does not recover" bug.
- **`serve_with` starts no sandbox.** Only the binary runs the game. An embedder — the
  chaos kind, the integration tests — gets the plain server, because chaos launching a
  playground that launches four services inside itself is a loop.
- **The server binds before the world exists.** The sandbox fills a `OnceLock` when it is
  warm, and the game RPCs answer `UNAVAILABLE` until then; a server that waited would fail
  its liveness probe and be killed mid-start.

- `lib.rs`: `serve` (binds `[server] listen`), `serve_on` (caller-supplied listener,
  default `Runtime`), `serve_with` (listener plus a `Runtime`). Tests and the chaos
  tool use the last two on port 0.
- `service.rs`: the `PlaygroundService` trait impl on `Playground`. Every RPC starts with
  `admit()`: count the request, start the `RequestTimer`, apply the fault handle, map
  a `Fault` to a gRPC status.
- `config.rs`: layered TOML, `configs/playground/base.toml` < `<env>.toml` < flags and
  `PLAYGROUND_*` environment variables (`Overrides`). Every key lives in `base.toml`;
  `deny_unknown_fields` makes a mistyped key fail at start. `playground config` prints
  the effective result.
- `main.rs`: the only file that prints (the `config` subcommand).

- Observability: `tbd_common::telemetry::grpc_request_span` is the `trace_fn`, so every
  call gets a `grpc.request` span with the caller's `traceparent` adopted and
  `trace_id` recorded; `admit()` starts the `RequestTimer`. Metrics listen on
  `[metrics] listen` (`PLAYGROUND_METRICS_ADDR`), `None` for embedders.

Invariants:
- A stub says so on the wire: `PingResponse.stub` is `true` until a real implementation
  replaces it, and the tests assert it. Do not let a placeholder look like a
  measurement.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder. With a caller-supplied
  listener the builder setting does nothing, and small responses stall 40 ms.
- Services never address each other directly; a caller reaches this one through
  Envoy's internal listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it/main.rs` boots the server on port 0 through `support.rs` with the
shipped `configs/playground` and env `local`, and exposes the `Runtime` so tests can
inject faults and read counters.
