# protocol

The edge service: one port serving REST and server-sent events, WebSocket, GraphQL and
gRPC, forwarding every request to a registered backend over gRPC and owning no
business logic. Envoy in front of it verifies tokens and balances; the protocol routes
by service and adds policy. It is becoming a gateway that exposes any gRPC service
(engine, humans, ledger, whatever `tbd new service` adds) with JSON in and JSON out;
this page is the contract of what exists today.

| Piece | Where |
|---|---|
| Crate | `crates/protocol` ([CLAUDE.md](../../crates/protocol/CLAUDE.md)) |
| Own contract | `proto/tbd/protocol/v1/protocol.proto`: `ProtocolService/Ping`; the REST, SSE, WebSocket (`/ws` and the multiplexed `/v1/ws`) and GraphQL surfaces below; `docs/protocol/openapi.json` (served at `/openapi.json`) for REST |
| Config layers | `configs/protocol/{base,local,dev,production}.toml`; `protocol config` prints the merged result |
| Deployment | `devops/k8s/base/protocol`, port 8080, metrics 9465 (9464 in the cluster); Envoy's edge (`:8080`) routes every host to it and health-checks `/readyz` |
| Chaos | the `protocol` kind (`crates/chaos/src/kinds/protocol.rs`): `[stack.protocols.X]` with an `engine`, the `http_*`, `rest_evaluate`, `sse_events`, `graphql_evaluate`, `ws_echo`, `ws_mux`, `grpc_protocol_*` checks, the protocol load operations |

## The services registry

`[services.<name>]` in `configs/protocol/base.toml` names every backend the gateway can
forward to:

```toml
[services.ledger]
url = "http://127.0.0.1:50052"            # gRPC endpoint
service = "tbd.ledger.v1.LedgerService"   # grpc.health.v1 name probed and re-reported
required = false                           # whether /readyz fails while it is down
```

- **One lazy channel per distinct URL.** Behind Envoy every backend is
  `http://envoy:50051`, the internal listener, which routes by gRPC service name and
  balances across replicas; the protocol then holds one connection pool for all of
  them. On the host every backend is on its own loopback port (`local.toml` keeps the
  base values); `dev.toml` and `production.toml` point everything at Envoy.
- **`PROTOCOL_<NAME>_URL` overrides `url`** for any registered name, read generically
  so a scaffolded service needs no new flag; `--service-url name=URL` is the flag form
  and wins over the variable. `PROTOCOL_ENGINE_URL` is one of them. `.env.example`,
  `compose.yaml`, the ConfigMap and the ansible template carry one per backend.
- **`engine` is mandatory**: the typed engine client (`AppState::engine()`) needs it.
  Any other backend is reached through `AppState::client(name, Client::new)` or
  `backend(name).transport()`, the same traced and measured transport.
- **`tbd new service <name>`** inserts a `[services.<name>]` table (`required = false`)
  before the `# tbd:services-end` marker and the `PROTOCOL_<NAME>_URL` line in every
  deployed environment ([tbd/README.md](../tbd/README.md)).
- **Embedders** (tests, the chaos tool) build the registry with
  `Config::embedded(listen, [(name, url)])`: every backend required, health names by the
  scaffolder's convention (`tbd.<name>.v1.<Name>Service`), no metrics listener.

## Readiness and health

`GET /readyz` probes every backend's `grpc.health.v1` service concurrently, within
`[health] probe_timeout`, and answers:

```json
{"ready": true, "services": {"engine": "serving", "humans": "serving", "ledger": "not_serving"}}
```

`ready` is true, and the status 200, when every `required` backend is `serving`; else
503 with the same body. States are `serving`, `not_serving` (the backend answered so) and
`unknown` (unreachable or slower than the probe timeout). Envoy and Kubernetes take a
replica out of rotation on a failed `/readyz`, which is why only the engine is required
today: a required ledger would turn a ledger outage into an edge outage. `GET /healthz`
is liveness only.

The protocol's own gRPC health service reports its own name and every backend under
its `service` name, refreshed every `[health] probe_interval` by one task per backend
(transitions logged at info). A probe carries `x-tbd-backend: <name>`: the health path is
the same for every service and Envoy's internal listener routes by path, so the header
is what sends a ledger probe to the ledger instead of the engine catch-all (an RPC path
routes itself and needs no header). Behind the edge every `grpc.health.v1.Health/Check` lands
on the protocol, so an edge-only client learns each backend's state from there.

## Configuration

Every key lives in `configs/protocol/base.toml`; an environment file carries
differences; flags and `PROTOCOL_*` variables override both; `protocol config` prints
the effective result.

| Table | Keys | Flags / env |
|---|---|---|
| `[server]` | `listen` | `--listen-addr`, `PROTOCOL_LISTEN_ADDR` |
| `[metrics]` | `listen` (optional) | `--metrics-addr`, `PROTOCOL_METRICS_ADDR` |
| `[services.<name>]` | `url`, `service`, `required` | `PROTOCOL_<NAME>_URL`, `--service-url name=URL` |
| `[health]` | `probe_interval`, `probe_timeout` | |
| `[principals]` | `services` (token subjects that are our own services) | |
| | environment and directory | `--env`/`TBD_ENV`, `--config-dir`/`PROTOCOL_CONFIG_DIR` |

`Config::validate()` refuses a registry without `engine`, a name that cannot be an
environment variable (2 to 24 lowercase letters or digits, starting with a letter), a
URL that does not parse, an empty health name, or a zero duration.

## JSON in and out

Every body the protocol sends is JSON: REST responses (`/healthz` is
`{"status":"ok"}`, `/readyz` the readiness report above, errors the envelope below), the
`data:` of every SSE event, and every WebSocket text frame. Every request body is JSON
too: another content type is refused with `415 unsupported_media_type`, a body that
does not parse with `400 bad_request` and a `field` detail named `body`, one over the
size limit with `413 payload_too_large`. GraphQL is JSON by definition; gRPC is gRPC.

## Errors

One envelope on every surface, downstream-agnostic:

```json
{"code": "bad_request", "error": "subject_id is required", "details": [{"type": "field", "field": "subject_id", "description": "is required"}]}
```

`code` is a stable slug from the table below, `error` a sentence for a person,
`details` typed entries translated from the `google.rpc` details a backend attaches to
its status. The field names and the slugs are frozen: an SDK is generated from them.

| `code` | from gRPC | HTTP |
|---|---|---|
| `bad_request` | `INVALID_ARGUMENT`, `OUT_OF_RANGE`, or a body that does not parse | 400 |
| `failed_precondition` | `FAILED_PRECONDITION` (an erased subject) | 400 |
| `unauthenticated` | `UNAUTHENTICATED`, or a route that needs a caller and got none | 401 |
| `forbidden` | `PERMISSION_DENIED` | 403 |
| `not_found` | `NOT_FOUND`, or no route | 404 |
| `already_exists` | `ALREADY_EXISTS` | 409 |
| `conflict` | `ABORTED` | 409 |
| `payload_too_large` | a request body over the limit | 413 |
| `unsupported_media_type` | a request body that is not JSON | 415 |
| `rate_limited` | `RESOURCE_EXHAUSTED` | 429 |
| `cancelled` | `CANCELLED` | 499 |
| `internal` | `INTERNAL`, `UNKNOWN`, `DATA_LOSS` | 500 |
| `unimplemented` | `UNIMPLEMENTED` | 501 |
| `unavailable` | `UNAVAILABLE` | 503 |
| `timeout` | `DEADLINE_EXCEEDED` | 504 |

Details: `{"type":"field","field","description"}` from `google.rpc.BadRequest`,
`{"type":"info","reason","domain","metadata"}` from `google.rpc.ErrorInfo`,
`{"type":"retry","after_seconds"}` from `google.rpc.RetryInfo`, which also sets the
`Retry-After` header. An `internal` error never carries the backend's message: the
sentence is `internal error` and the real text is on the error-level log line with
the request's `trace_id`. Server-side codes (`internal`, `unavailable`, `timeout`,
`unimplemented`) are logged at error level; the rest are the caller's problem and are
not.

Per surface: REST sends the envelope as the body with the HTTP status; SSE sends it as
the JSON of an `event: error` and leaves the stream to the client; the WebSocket bridge
sends `{"type":"error","code","message","details"}`; GraphQL sends the sentence as the
error message with `code` and `details` under `extensions`. The chaos error classes
(`http 503`, `http 500`, `http 429`, `http 504`) are this table seen from the load
generator ([chaos/scenarios.md](../chaos/scenarios.md)).

## Principals

The caller on every request, read from the claims Envoy verified and forwarded in
`x-jwt-payload` ([auth/README.md](../auth/README.md#principals)):

```json
{"subject": "tbd-chaos", "kind": "client", "client_id": "tbd-chaos", "org": null, "key": null, "scopes": ["tbd.api"], "role": null}
```

`kind` is `person` (with the `client_id` the person came through, when any), `client`
(a client-credentials token: `client_id` equals `sub`, an organisation's machine key) or
`service` (a `sub` listed in `[principals] services`). `org` and `key` (`{id, parent}`)
are read now and minted by the id plane later. `GET /v1/me` returns the principal, or
`401 unauthenticated` when Envoy forwarded none; handlers take it as an extractor, the
streams as an optional one. Span fields `enduser.id`, `enduser.kind`, `enduser.org` and
`enduser.key` carry it into traces and logs.

## OpenAPI

`GET /openapi.json` is the OpenAPI 3.1 document of the REST surface, generated from the
handlers with utoipa (`#[utoipa::path]` on each handler, `ToSchema` on each body; the
router is built from the same annotations, so a route cannot exist without its path).
`docs/protocol/openapi.json` is the same document, committed; `mise run protocol:openapi`
regenerates it and a protocol test fails when the two differ, which is the CI diff.
The `Problem` schema is the error envelope above. The transcoded routes below are in
the same document, generated from the proto descriptors: one component per message,
named by its proto full name (`tbd.ledger.v1.Fact`), `operationId` `Service.Method`,
one tag per backend, and `default` → `Problem` on every operation. GraphQL, WebSocket
and gRPC are outside the document. `protocol openapi` builds it from the loaded
configuration, so the file carries exactly the backends `base.toml` registers.

## Transcoding

An RPC annotated with `option (google.api.http)` in its `.proto`
(`proto/google/api/` is vendored for it) becomes a route at startup, read from the
descriptor set in `tbd_proto::DESCRIPTOR_SET_ALL`. Nothing is written per RPC: a
service in package `tbd.<name>.v1` forwards to the registry backend `<name>` over the
same traced and measured transport as the engine, and an annotated service whose
backend is not in `[services]` is skipped with one warning. `crates/protocol/src/
transcode/` is the implementation.

| Annotation | Route |
|---|---|
| `get: "/v1/ledger/subjects/{subject_id}/facts"` on a unary RPC | `GET`, the path variable sets `subject_id`, every other request field is a query parameter (`?scopes=self&scopes=other&limit=10`, dotted for nested messages) |
| `post: "..." body: "*"` | the JSON body is the whole request; path variables override their fields; query parameters are refused |
| `post: "..." body: "field"` | the JSON body is that message field; the rest binds as above |
| `delete: "..."` | no body; a body is refused |
| any verb on a server-streaming RPC | server-sent events; the template must end in `/events` (Envoy's one streaming rule) |
| `additional_bindings` | one route each |

Rules: the JSON uses proto field names, 64-bit integers as strings, enums by name
(numbers accepted on input), `bytes` as base64, `Timestamp` as RFC 3339, and every
field is emitted (a default too), so `stub` is present on every response. Input is
strict: an unknown body field, an unknown query key, a singular field given twice or a
value that does not convert is `400 bad_request` with a `field` detail. A body must be
`application/json` (`415`) and at most 2 MiB (`413`); a gRPC response is capped at
tonic's 4 MiB. A gRPC status becomes the envelope through the table above; a known
path with a verb it does not serve is `405 method_not_allowed`. Refused at startup,
each with its reason in the error: `*` and `**` segments, `{a=b/*}` sub-paths,
`:verb` suffixes, custom verbs, `body: "*"` on `GET` or `DELETE`, client or
bidirectional streaming, a streaming template without `/events` or a unary one with
it, and a duplicate `(verb, path)` including the hand-written routes.

The ledger is the first service exposed this way ([ledger/README.md](../ledger/README.md)
lists its routes); the engine's `Subscribe` is `GET /v1/engine/subjects/{subject_id}/events`.
`Evaluate` and `Session` stay hand-written.

## The multiplexed socket

`GET /v1/ws` is one WebSocket that carries every public RPC, addressed by name instead
of by path. It is the same callable set as the REST routes above, read from the same
annotations, so the two surfaces cannot drift apart: an RPC is here exactly when it
carries `google.api.http` and its backend is registered. The whole request message is
the frame's `body`; path and query rules belong to HTTP and do not apply here.

Frames are JSON text, one object each. From the client:

| Frame | Meaning |
|---|---|
| `{"type":"call","id":"1","method":"tbd.ledger.v1.LedgerService/Current","body":{…}}` | start a call; `body` may be omitted for an empty request message |
| `{"type":"cancel","id":"1"}` | stop a call in flight |

To the client:

| Frame | Meaning |
|---|---|
| `{"type":"data","id":"1","body":{…}}` | one response message, the same JSON REST answers; a unary call sends exactly one |
| `{"type":"end","id":"1"}` | the call is over and the id is free again |
| `{"type":"error","id":"1","code":…,"error":…,"details":[…]}` | the call failed; the envelope's keys, with the id it belongs to |
| `{"type":"error","code":…,"error":…,"details":[…]}` | the frame itself made no sense, so no call is named; the socket stays open |

The `id` is the client's, unique among the calls it has in flight, and free again after
`end`. Calls are independent: a unary call answers while streams stay open, and every
call ends with exactly one `end` or one `error`, cancellation included. A refusal never
closes the socket. `[socket] max_calls` is how many calls one connection may have in
flight (a further one is `rate_limited`), and `max_frame_bytes` is the largest frame a
client may send.

A client that reads slowly stalls only its own calls: frames wait in one bounded queue
per connection rather than piling up in memory. Envoy gives the path no timeout, like
every other stream.

```
{"type":"call","id":"1","method":"tbd.ledger.v1.LedgerService/Ping","body":{"message":"hi"}}
{"type":"data","id":"1","body":{"message":"hi","version":"0.1.0","stub":false,"store":"postgres"}}
{"type":"end","id":"1"}
```

`/ws` is the older, single-purpose socket: it bridges one engine `Session` and keeps its
own frame shape, which chaos reads. New clients use `/v1/ws`. Neither socket is in the
`OpenAPI` document.

## Over MCP

`/mcp` offers public RPCs as tools over the Model Context Protocol (streamable HTTP,
stateless: JSON-RPC over `POST`, either replica answers any request, no session). It is
a fourth rendering of the same registry as REST, SSE and the socket, narrowed by an
allowlist.

- **Default deny.** Only the RPCs `[mcp] tools` names are tools; the rest are neither
  listed nor callable (`no tool named ...`), whoever asks. The shipped list is the
  models (`llm_generate`, `llm_list_models`, `llm_get_budget`, `llm_embed`) and the
  health pings. Nothing that writes, deletes, sends, moves money, reads personal data,
  injects a fault or streams without end is ever added; a test holds every listed
  name to the registry and the code's default to the file, and the chaos check
  `http_mcp_tools` fails on a stack that offers such a tool.
- **Names.** `<backend>_<method>` in snake case: `llm_generate`, `llm_list_models`,
  `ledger_ping`.
- **Descriptions and arguments.** The RPC's comment in the `.proto` is the tool's
  description; the arguments are the request message, described as an inline JSON
  Schema by the same spellings the `OpenAPI` document uses (64-bit integers as strings,
  enums by name, timestamps as RFC 3339), fields described by their comments. Unknown
  fields are refused as `bad_request`, as on the socket.
- **The caller.** Every call is the RPC made as whoever the gateway verified: the same
  forwarded identity, the same rights, the same budget, the same record. A call with no
  verified caller is a protocol error. `tools/list` needs none.
- **Answers.** One text block holding the response message as JSON. A failure is a
  tool result marked as an error whose JSON is the envelope (`code`, `error`,
  `details`), so an agent reads why. A streaming RPC is collected into one answer: at
  most `[mcp] max_stream_items` messages within `[mcp] stream_timeout`, and `cut` says
  so when it stopped early; when the messages carry `text` (a generation's chunks), the
  text is joined into `text`, reasoning into `reasoning`, and the last message is kept
  whole under `last` for its usage.
- **The audit trail.** Every call is one log line, target `audit`: the tool, the RPC,
  the caller's subject and kind, the outcome code and the time taken. Never the
  arguments or the answer.
- **The gate.** On the API host, like every route there: a verified bearer token for
  the API audience ([auth/README.md](../auth/README.md)). Envoy gives `/mcp` no timeout;
  the protocol's own cap bounds a call. `Host` checking is off on purpose: it guards a
  local server against a page borrowing a browser's credentials, and this one takes
  only a bearer token.

Connecting Claude Code:

```sh
claude mcp add --transport http tbd https://api.<domain>/mcp \
  --header "Authorization: Bearer $(mise run auth:token | jq -r .access_token)"
```

A machine token expires; when calls start failing with 401, mint a new one and add the
server again.

## Metrics

Client-side, per backend: `tbd_engine_client_requests_total{backend,route,status}` and
`tbd_engine_client_duration_seconds{backend,route}` (the names predate the registry;
`backend` is the `[services]` name). Per request: the shared `tbd_requests_*` with
`transport` and `route`; a call on the multiplexed socket is one of those, with
transport `ws` and the RPC name as its route, and a tool call over MCP likewise with
transport `mcp`. Streams: `tbd_streams_active{kind}` and
`tbd_stream_items_total`, where a multiplexed connection is kind `mux`. See [observability/metrics.md](../observability/metrics.md).

## What comes next

Policy declared in the proto contract: scopes, request validation, idempotency keys and
weighted rate limits metered per organisation and per key. Both call surfaces already
resolve to the same RPC table, so it is enforced once, for REST, events and the socket
alike.
