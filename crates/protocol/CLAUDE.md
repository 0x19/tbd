# crates/protocol

The protocol service: one port, four surfaces, no business logic. Every handler
translates and forwards to a registered backend; if a handler does more than that,
the logic belongs in the service. The contract is `docs/protocol/README.md`.

- `config.rs`: layered TOML (`configs/protocol/base.toml` + `<TBD_ENV>.toml`) with
  `deny_unknown_fields`, the `[services.<name>]` registry, `Overrides` (flags with
  `PROTOCOL_*` env vars) and `Config::validate()`. The backend URLs are the one generic
  env family in the workspace: `Overrides::apply` reads `PROTOCOL_<NAME>_URL` for every
  registered name, so `tbd new service` adds a table and no flag; `--service-url
  name=URL` is the flag form. `Config::embedded(listen, [(name, url)])` is what chaos
  and the tests build. `main.rs` has the `config` subcommand and is the only file that
  prints.
- `lib.rs`: `router()` merges `http` (REST + SSE), `ws`, `graphql` and `grpc` routes;
  `serve_on` runs it with h2c so gRPC and HTTP/1.1 share the port.
- `state.rs`: `AppState` is the registry. One lazy, reconnecting tonic channel per
  distinct URL (behind Envoy every backend is `http://envoy:50051`), a `Backend` per
  name with its `grpc.health.v1` name and `required` flag. `engine()` is the typed
  engine client; any other backend is `client(name, Client::new)` over the same
  `Transport` (`Measured` client metrics with a `backend` label, `TraceInject`).
  `readiness()` probes every backend concurrently within `[health] probe_timeout`.
- The router has an explicit `fallback`: tonic's merged router would otherwise answer
  every unknown REST path with HTTP 200 + `grpc-status: 12`. Unknown paths are a JSON
  404 (`Problem::not_found`) with route label `unmatched`; unknown gRPC methods keep the
  gRPC answer.
- `principal.rs`: the caller identity. Envoy verifies the JWT and forwards the claims
  in `x-jwt-payload` (base64url JSON) and strips that header from clients; `attach`
  (middleware with state) puts a `Principal { sub, kind, scopes, role, org, key }` in
  the request extensions and the span (`enduser.id`, `enduser.kind`, `enduser.org`,
  `enduser.key`). The kind is `Service` for a `sub` in `[principals] services`,
  `Client` when `client_id`/`azp` equals `sub`, else `Person` (with the client id it
  came through). Claims are read top-level or under `ext`. Handlers take `Principal`
  (401 `unauthenticated` when absent) or `Option<Principal>`; `/v1/me` returns it. The
  protocol never verifies tokens: services are reachable only through Envoy, and a
  second check would be a second implementation to keep in sync. `org`, `key`,
  `parent` are the claim contract for the id plane; nothing mints them yet.
- `error.rs`: `Problem { code: Code, message, details }` is the one error on every
  surface. `Code::from_grpc` is the standard gRPC-to-HTTP table (`internal` 500,
  `unavailable` 503, `timeout` 504, `rate_limited` 429, `failed_precondition` 400, plus
  the HTTP-only `unsupported_media_type` 415 and `payload_too_large` 413); the slugs
  are the frozen `code` vocabulary in `docs/protocol/README.md`. `From<tonic::Status>`
  translates `google.rpc` details (`BadRequest`, `ErrorInfo`, `RetryInfo`, which also
  sets `Retry-After`) and redacts `internal` messages into the log. REST answers
  `{"code","error","details"}`, SSE sends the same JSON as the `error` event, `ws.rs`
  as the `error` frame (`message` stays: chaos reads it), GraphQL puts `code` and
  `details` in `extensions`. The chaos error classes (`http 503`, `http 500`, `http 429`,
  `http 504`) reflect this table; change it and update `docs/chaos/scenarios.md` and
  the chaos test that asserts it.
- OpenAPI: `http::openapi_router()` builds the REST routes from `#[utoipa::path]`
  annotations (utoipa-axum), so the router and the document share one source;
  `lib::openapi()` is the document, served at `/openapi.json` and printed by `protocol
  openapi`. `docs/protocol/openapi.json` is committed and a test keeps it equal to the
  served document: after touching a handler or a body type, `mise run protocol:openapi`.
  Bodies derive `ToSchema`; `ErrorBody` is the envelope's schema, named `Problem`.
- `json.rs`: the request-side `Json<T>` extractor. Every body is JSON in and JSON out,
  health included; a non-JSON body is `unsupported_media_type`, a body that does not
  parse is `bad_request` with a `field` detail named `body`.
- `ws.rs`: bridges a WebSocket to an engine `Session` stream, one session per socket,
  JSON envelope `{type: data|heartbeat|close|error, ...}` outbound.
- `grpc.rs`: the protocol's own gRPC (`ProtocolService/Ping`), health and reflection,
  mounted into the axum router via `Routes::into_axum_router`. Health also reports
  every registered backend under its `service` name (`ENGINE_SERVICE` is the engine's),
  refreshed every `[health] probe_interval` by one task per backend: behind the Envoy
  edge every `grpc.health.v1.Health` call lands on the protocol, so this is how an
  edge-only client learns a backend is up. `routes()` spawns those tasks and therefore
  needs a Tokio runtime.

- Observability: `observe.rs` has the span factory (parents to `traceparent`, records
  `trace_id`, classifies gRPC by content type) and the metrics middleware; `state.rs`
  wraps every backend channel in `Measured` (client metrics per backend and route) and
  `TraceInject` (propagates `traceparent`). Metrics listen on `[metrics] listen`
  (`PROTOCOL_METRICS_ADDR`, default `:9465`). Every backend URL is Envoy's internal
  listener in every deployed environment.

Invariants:
- Forward the engine's `stub` flag untouched on REST, GraphQL and SSE.
- `/readyz` fails only for a `required` backend (the engine today). Envoy and
  Kubernetes eject a replica on a failed `/readyz`, so an optional backend's outage
  must show in the body, never in the status.
- `TCP_NODELAY` is applied with `ListenerExt::tap_io` in `serve_on`. axum 0.8 has no
  `tcp_nodelay` on `Serve`; without the tap, a WebSocket's first frame after the 101
  waits 40 ms.
- Server errors are logged at `error` level per request. Under chaos that is expected
  noise and filtered by `RUST_LOG`; do not downgrade the level to quiet the tool.

Tests: `config.rs` loads every shipped environment file and exercises the overrides;
`tests/it/main.rs` boots a real engine and this service on port 0 through
`support.rs` (`Config::embedded`) and drives every surface, including a raw WebSocket
client and a gRPC client on the HTTP port.
