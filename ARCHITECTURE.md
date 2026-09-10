# Architecture

Bird's-eye view, code map, invariants. Kept short and corrected after the code changes,
never before.

## Bird's-eye

```
 clients ──HTTP/WS/GraphQL/gRPC──▶  protocol  ──gRPC──▶  engine
                                    (protocol)             (compute)
```

Two processes. The protocol owns every client-facing protocol and no business logic.
The engine owns compute and speaks only gRPC. They share generated types
(`tbd-proto`) and plumbing (`tbd-common`), nothing else.

## Code map

| Crate | Role | May depend on |
|---|---|---|
| `tbd-common` | telemetry init, shutdown signal, shared CLI flags | tokio, tracing, clap |
| `tbd-proto` | code generated from `/proto` at build time via `protox` + `tonic-prost-build` | tonic, prost |
| `tbd-engine` | `tbd.engine.v1.EngineService` implementation, health, reflection | common, proto |
| `tbd-protocol` | axum router: REST, SSE, WebSocket bridge, GraphQL, protocol gRPC | common, proto |
| `tbd-chaos` | `chaos` binary: runs the services in-process, validates, loads, injects faults | everything above |

Each service crate is `lib.rs` + thin `main.rs`. `serve_on(listener, config, shutdown)`
is the seam: `main` binds the configured address, tests bind port 0.

Protocol surfaces map onto engine RPCs:

| Protocol | Engine RPC |
|---|---|
| `POST /v1/evaluate`, GraphQL `evaluate` | `Evaluate` (unary) |
| `GET /v1/subjects/{id}/events` (SSE) | `Subscribe` (server stream) |
| `/ws` | `Session` (bidirectional stream) |
| `GET /readyz`, GraphQL `engineReady` | `grpc.health.v1.Health/Check` |

## Invariants

1. **Stubs are labelled on every surface.** `EvaluateResponse.stub` is set by the engine
   and forwarded untouched by REST, GraphQL and SSE. Tests assert it.
2. **The protocol has no business logic.** If a handler does more than translate and
   forward, it belongs in the engine.
3. **`tbd-common` is transport-free.** No tonic, no axum.
4. **`tbd-proto` is generated only.** Wrap generated types where the behaviour lives.
5. **One port per service.** The protocol multiplexes HTTP/1.1 and h2c on 8080;
   the engine serves gRPC on 50051.
6. **Health is the orchestrator's job.** Images are distroless with no shell; probes are
   gRPC health on the engine, `/healthz` and `/readyz` on the protocol.

## Cross-cutting

- **Config**: clap derive, every flag has an env var. `LogArgs` is flattened into both.
- **Logging**: `tracing`; `LOG_FORMAT=json` in containers, `text` locally.
- **Errors**: `thiserror` enums in libraries, `anyhow` only in `main`. The protocol maps
  `tonic::Code` to HTTP status in `crates/protocol/src/error.rs`.
- **Tests**: one integration binary per service at `tests/it/`. The protocol's tests boot a
  real engine in-process; nothing is mocked.
- **Faults**: `tbd_common::fault::FaultHandle`, consulted by the engine on every RPC and
  healthy unless the chaos tool sets it. See `docs/chaos/`.
- **Shutdown**: SIGINT or SIGTERM resolves `tbd_common::shutdown::signal()`, passed to
  both servers' graceful-shutdown hooks.
