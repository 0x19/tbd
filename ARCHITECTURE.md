# Architecture

Bird's-eye view, code map, invariants. Kept short and corrected after the code changes,
never before.

## Bird's-eye

```
                      ┌─────────────────────── observability ───────────────────────┐
                      │  metrics ─▶ VictoriaMetrics   traces ─▶ OTel Collector ─▶ Tempo │
                      │  logs ────▶ OTel agent ─▶ VictoriaLogs         Grafana on top   │
                      └──────────────────────────────────────────────────────────────┘
                              ▲ scrape            ▲ OTLP              ▲ stdout JSON
                              │                   │                   │
 clients ──REST/SSE/GraphQL/WS/gRPC──▶  envoy :8080  ──▶  protocol  ──▶  envoy :50051  ──▶  engine
                                        (edge)          (translate)     (engine LB)      (compute)
                                        gRPC by service name goes straight to the engine
                                          │ verifies every token (JWKS)
                      ┌───────────────────┴──────────────────────────────────────────────┐
                      │  identity: hydra (OAuth2/OIDC) ◀── kratos (people, passkeys)     │
                      │            auth.<domain>: sign-in pages, /oauth2/*, JWKS         │
                      └──────────────────────────────────────────────────────────────────┘
```

Three processes plus Envoy. Envoy is the only thing anything talks to: clients hit its
edge, protocol instances reach engines through its engine load balancer. Envoy is also
the only thing that authenticates: bearer JWTs from Hydra on the API, a browser login
(OAuth2 filter) on the UI hosts, nothing behind it checks a token
([docs/auth/README.md](docs/auth/README.md)). The protocol
owns every client-facing surface and no business logic. The engine owns compute and
speaks only gRPC. Services share generated types (`tbd-proto`) and plumbing
(`tbd-common`), nothing else. Every request is traced end to end and measured at every
hop; the same observability stack runs locally and in production.

## Code map

| Crate | Role | May depend on |
|---|---|---|
| `tbd-common` | telemetry (logs, OTLP traces, trace propagation, the shared gRPC span), Prometheus metrics with the shared metric names, shutdown, shared CLI flags, fault injection, the embedder `Runtime`, layered config | tokio, tracing, clap, opentelemetry, metrics, http (types) |
| `tbd-proto` | code generated from `/proto` at build time via `protox` + `tonic-prost-build` | tonic, prost |
| `tbd-engine` | `tbd.engine.v1.EngineService` implementation, health, reflection | common, proto |
| `tbd-humans` | `tbd.humans.v1.HumansService` implementation, health, reflection; scaffolded by `tbd new service`, a stub until its RPCs land | common, proto |
| `tbd-ledger` | the facts ledger: `store::Store` (Postgres via sqlx, or in memory), outbox drained into ClickHouse, erasure sweeper, and the thin `tbd.ledger.v1.LedgerService` over it; readiness follows the store | common, proto |
| `tbd-protocol` | axum router: REST, SSE, WebSocket bridge, GraphQL, protocol gRPC; a registry of traced, measured gRPC backends from `[services]` in `configs/protocol` | common, proto |
| `tbd-cli` | the `tbd` binary: scaffolds services from embedded templates and registers them in every shared file; owns no runtime code | clap, toml |
| `tbd-chaos` | `chaos` binary: runs the services in-process, validates, loads, injects faults; `chaos serve` exposes all of it as an HTTP API and serves the admin UI from `ui/chaos` | everything above |

Each service crate is `lib.rs` + thin `main.rs`. `serve_on(listener, config, shutdown)`
is the seam: `main` binds the configured address, tests bind port 0, and the engine's
`serve_with` adds a `Runtime` carrying the fault handle and counters for embedders.

Protocol surfaces map onto engine RPCs:

| Protocol | Engine RPC |
|---|---|
| `POST /v1/evaluate`, GraphQL `evaluate` | `Evaluate` (unary) |
| `GET /v1/subjects/{id}/events` (SSE) | `Subscribe` (server stream) |
| `/ws` | `Session` (bidirectional stream) |
| `GET /readyz`, GraphQL `engineReady` | `grpc.health.v1.Health/Check` on every registered backend (`/readyz` reports each; the `required` ones gate it) |

Envoy routes, from `devops/envoy/envoy.yaml`:

| Match on the edge (8080) | Upstream | Timeout |
|---|---|---|
| `/healthz`, `/readyz` (no token needed; everything else below needs a bearer JWT) | protocol | 5 s |
| gRPC `/tbd.engine.v1.EngineService/*` | engine | none, retries on connect failure and `UNAVAILABLE` |
| internal LB (50051) gRPC `/tbd.humans.v1.HumansService/*` | humans | none, retries on connect failure and `UNAVAILABLE` |
| internal LB (50051) gRPC `/tbd.ledger.v1.LedgerService/*` | ledger | none, retries on connect failure and `UNAVAILABLE` |
| any other gRPC (`tbd.protocol.v1`, health, reflection) | protocol | none |
| `/ws` | protocol, WebSocket upgrade | none |
| `/v1/subjects/*` (SSE) | protocol | none |
| everything else | protocol | 15 s, retries only when the request was never sent |
| engine LB (50051), all gRPC | engine | none |
| host `auth.*` | hydra / kratos / login pages, open | 15 s |
| hosts `grafana.*`, `logs.*`, `profiles.*`, `metrics.*`, `chaosadmin.*` | the UI, after the browser login | none |

## Invariants

1. **Stubs are labelled on every surface.** `EvaluateResponse.stub` is set by the engine
   and forwarded untouched by REST, GraphQL and SSE. Tests assert it.
2. **The protocol has no business logic.** If a handler does more than translate and
   forward, it belongs in the engine.
3. **`tbd-common` has no service transport.** No tonic, no axum; `http` for header types
   only. The OTLP exporter and the metrics listener are the only network code in it.
4. **`tbd-proto` is generated only.** Wrap generated types where the behaviour lives.
5. **One port per service, plus one for metrics.** The protocol multiplexes HTTP/1.1 and
   h2c on 8080 and re-reports every registered backend's health name on it; the engine
   serves gRPC on 50051; each exposes Prometheus metrics on its own port.
6. **Services never address each other directly.** Every URL in the protocol's
   `[services]` registry is Envoy's internal listener in every deployed environment
   (`PROTOCOL_<NAME>_URL`); Envoy routes by gRPC service name and balances. Only tests
   and the chaos tool connect straight to a service.
7. **Every request is traced and measured.** Envoy starts the trace; each hop adopts the
   caller's `traceparent`, records `trace_id` on its span, and records one metrics sample
   with the shared metric names. A caller-supplied `traceparent` is honoured.
8. **Health is the orchestrator's job.** Images are distroless with no shell; probes are
   gRPC health on the engine, `/healthz` and `/readyz` on the protocol, `/ready` on Envoy.
9. **Envoy is the only authenticator.** Tokens are verified once, in Envoy, against
   Hydra's keys; identity reaches a service only as the `x-jwt-payload` header Envoy
   sets after stripping whatever the client sent. Services read it (`Principal`: the
   subject, its kind, scopes, role, organisation and key) and never verify, decode or
   forward tokens themselves. A new host or route is gated by naming a
   JWT requirement in `envoy.yaml`, not by code in a service.

## Cross-cutting

- **Config**: layered TOML (`configs/<binary>/base.toml` + `<TBD_ENV>.toml`) with clap
  flags over it, every flag with an env var. The one generic family is the protocol's
  `PROTOCOL_<NAME>_URL`, one per registered backend, so a scaffolded service needs no
  new flag. `TelemetryArgs` is flattened into every binary and its flags are global, so
  they work after a subcommand.
- **Logging**: `tracing`; `LOG_FORMAT=json` in containers, `text` locally. JSON lines
  carry the enclosing span's fields, including `trace_id`. Per-request lines are `debug`;
  Envoy's access log is the request-level record at `info`.
- **Tracing**: a tracer provider is always installed so trace ids exist and propagate;
  export to OTLP is on only when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.
- **Metrics**: `metrics` facade, Prometheus exporter, names in
  `crates/common/src/metrics.rs`, a global `service` label per process, process metrics
  included. With no exporter installed the macros are no-ops, which lets chaos run
  several services in one process.
- **Errors**: `thiserror` enums in libraries, `anyhow` only in `main`. The protocol
  answers every surface with one envelope, `Problem` in `crates/protocol/src/error.rs`:
  a stable `code` slug mapped from `tonic::Code` by the standard gRPC-to-HTTP table, a
  sentence, typed `details` from `google.rpc` error details; `internal` redacts the
  downstream message and logs it.
- **JSON in and out at the edge**: every HTTP response body, SSE payload and WebSocket
  text frame the protocol sends is JSON, health included; every request body is JSON
  and anything else is refused with the envelope.
- **Tests**: one integration binary per service at `tests/it/`. The protocol's tests boot a
  real engine in-process; nothing is mocked.
- **Faults**: `tbd_common::fault::FaultHandle`, consulted by the engine on every RPC and
  healthy unless the chaos tool sets it. See `docs/chaos/`.
- **Shutdown**: SIGINT or SIGTERM resolves `tbd_common::shutdown::signal()`, passed to
  both servers' graceful-shutdown hooks; telemetry is flushed after.

Deployment shape, environments and the observability stack: `devops/README.md`,
`docs/local-cluster.md`, `docs/observability/README.md`.
