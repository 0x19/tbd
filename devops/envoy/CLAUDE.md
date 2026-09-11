# devops/envoy

- `envoy.yaml` is used verbatim by compose, Ansible and Kubernetes. Anything
  environment-specific is expressed through DNS names (`engine`, `protocol`,
  `otel-collector`), never by editing the file per environment.
- Always run `mise run envoy:validate` after an edit. It uses Envoy's own validate mode
  and catches wrong `@type` URLs, which are the most common mistake (the OpenTelemetry
  tracer is `envoy.config.trace.v3.OpenTelemetryConfig`).
- Streaming routes (`/ws`, `/v1/subjects/`, `^/v1/.*/events$`, all gRPC, the engine LB)
  have `timeout: 0s`. Never give them a timeout; the connection manager's idle timeouts
  are also disabled. The regex is the contract with the protocol's transcoder: a
  server-streaming RPC's template must end in `/events`, so no new streaming route is
  ever needed here.
- REST retries are limited to `connect-failure,refused-stream` so a `POST` is never
  replayed after it was sent. gRPC adds `unavailable` because the calls are idempotent.
  Revisit that the day a non-idempotent RPC exists.
- The `protocol` cluster uses `use_downstream_protocol_config`; do not switch it to an
  explicit h2 config or WebSocket breaks.
- The YAML anchors (`access_log_json`, `grpc_retry`) are shared between listeners; edit
  once.
