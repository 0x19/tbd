<!-- tbd new service ledger --kind grpc --port 50052 --metrics-port 9466 --bacon-key l (tbd-cli 0.1.0) -->
# crates/ledger

The ledger service. gRPC only. Scaffolded by `tbd new service` (docs/tbd/README.md);
`Ping` is a labelled stub until the service's real RPCs land beside it.

- `lib.rs`: `serve` (binds `[server] listen`), `serve_on` (caller-supplied listener,
  default `Runtime`), `serve_with` (listener plus a `Runtime`). Tests and the chaos
  tool use the last two on port 0.
- `service.rs`: the `LedgerService` trait impl on `Ledger`. Every RPC starts with
  `admit()`: count the request, start the `RequestTimer`, apply the fault handle, map
  a `Fault` to a gRPC status.
- `config.rs`: layered TOML, `configs/ledger/base.toml` < `<env>.toml` < flags and
  `LEDGER_*` environment variables (`Overrides`). Every key lives in `base.toml`;
  `deny_unknown_fields` makes a mistyped key fail at start. `ledger config` prints
  the effective result.
- `main.rs`: the only file that prints (the `config` subcommand).

- Observability: `tbd_common::telemetry::grpc_request_span` is the `trace_fn`, so every
  call gets a `grpc.request` span with the caller's `traceparent` adopted and
  `trace_id` recorded; `admit()` starts the `RequestTimer`. Metrics listen on
  `[metrics] listen` (`LEDGER_METRICS_ADDR`), `None` for embedders.

Invariants:
- A stub says so on the wire: `PingResponse.stub` is `true` until a real implementation
  replaces it, and the tests assert it. Do not let a placeholder look like a
  measurement.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder. With a caller-supplied
  listener the builder setting does nothing, and small responses stall 40 ms.
- Services never address each other directly; a caller reaches this one through
  Envoy's internal listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it/main.rs` boots the server on port 0 through `support.rs` with the
shipped `configs/ledger` and env `local`, and exposes the `Runtime` so tests can
inject faults and read counters.
