# crates/engine

The streaming compute service. gRPC only. Talks to nothing else yet.

- `lib.rs`: `serve` (binds the configured address), `serve_on` (caller-supplied
  listener, default `Runtime`), `serve_with` (listener plus a `Runtime`). Tests and the
  chaos tool use the last two on port 0.
- `service.rs`: the `EngineService` trait impl on `Engine`. Every RPC starts with
  `admit()`: count the request, apply the fault handle, map a `Fault` to a gRPC status.
- `stats.rs`: two per-instance counters, `requests_total` and `requests_failed`, read
  by the chaos tool through `Runtime.stats`. Not production metrics.
- `config.rs`: clap `Config`; every field has an env var (`ENGINE_*`).

- Observability: `request_span` in `lib.rs` makes one `grpc.request` span per call,
  adopts a caller's `traceparent` from metadata and records `trace_id`; `admit()` in
  `service.rs` starts the `RequestTimer` and counts injected faults; streams hold a
  `StreamGuard`. Metrics listen on `ENGINE_METRICS_ADDR` (default `:9464`), set to
  `None` by embedders.

Invariants:
- Every score is a stub and says so: `stub = true`, `model_version` starts with
  `stub-`. Do not let a placeholder look like a measurement.
- `Subscribe`'s interval lives in the unfold state so it survives across awaits; a
  fresh `tokio::time::interval` per step ticks immediately and floods the stream (this
  bug shipped once).
- `Session`'s first heartbeat waits one full interval (`interval_at`) so it cannot
  race the echo of the client's opening frame.
- `TCP_NODELAY` is set on `TcpIncoming`, not the server builder. With a caller-supplied
  listener the builder setting does nothing, and small responses stall 40 ms.

Tests: `tests/it/main.rs` boots the server on port 0 through `support.rs`, which also
exposes the `Runtime` so tests can inject faults and read counters.
