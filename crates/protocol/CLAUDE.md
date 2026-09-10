# crates/protocol

The protocol service: one port, four surfaces, no business logic. Every handler
translates and forwards to the engine; if a handler does more than that, the logic
belongs in the engine.

- `lib.rs`: `router()` merges `http` (REST + SSE), `ws`, `graphql` and `grpc` routes;
  `serve_on` runs it with h2c so gRPC and HTTP/1.1 share the port.
- `state.rs`: `AppState` holds one lazy, reconnecting tonic channel to the engine.
  `engine_ready()` is the readiness check and asks the engine's health service for
  `tbd.engine.v1.EngineService`.
- `error.rs`: `ApiError` maps `tonic::Code` to HTTP status. This mapping is what the
  chaos error classes (`http 503` etc.) reflect; change it and update
  `docs/chaos/scenarios.md`.
- `ws.rs`: bridges a WebSocket to an engine `Session` stream, one session per socket,
  JSON envelope `{type: data|heartbeat|close|error, ...}` outbound.
- `grpc.rs`: the protocol's own gRPC (`ProtocolService/Ping`), health and reflection,
  mounted into the axum router via `Routes::into_axum_router`.

- Observability: `observe.rs` has the span factory (parents to `traceparent`, records
  `trace_id`, classifies gRPC by content type) and the metrics middleware; `state.rs`
  wraps the engine channel in `Measured` (client metrics per route) and `TraceInject`
  (propagates `traceparent`). Metrics listen on `PROTOCOL_METRICS_ADDR` (default `:9465`).
  The engine URL is Envoy's engine LB in every deployed environment.

Invariants:
- Forward the engine's `stub` flag untouched on REST, GraphQL and SSE.
- `TCP_NODELAY` is applied with `ListenerExt::tap_io` in `serve_on`. axum 0.8 has no
  `tcp_nodelay` on `Serve`; without the tap, a WebSocket's first frame after the 101
  waits 40 ms.
- Server errors are logged at `error` level per request. Under chaos that is expected
  noise and filtered by `RUST_LOG`; do not downgrade the level to quiet the tool.

Tests: `tests/it/main.rs` boots a real engine and this service on port 0 and drives
every surface, including a raw WebSocket client and a gRPC client on the HTTP port.
