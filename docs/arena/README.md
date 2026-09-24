# The arena

One live view of what the platform is doing, for the lab's pages (RFC 0004). The arena
reads three sources and republishes them as one snapshot, a second apart, to every
viewer over the gateway: `GET /v1/arena/snapshot` (one answer), `GET /v1/arena/events`
(server-sent events), and `tbd.arena.v1.ArenaService/Watch` on the multiplexed socket.

| Source | Read | Gives | Config |
|---|---|---|---|
| the model service | `ListModels` every `[collect] llm_every`, as `svc:arena` | per tier: engine, model, up, stub, running, slots, waiting | `[sources] llm_url` (`ARENA_LLM_URL`), Envoy's internal listener when deployed |
| the metrics store | four PromQL instant queries every `[collect] metrics_every` | per tier: completion tokens a second (1 min), time to the first token p50 and p99 (5 min), refusals in the last minute | `[sources] metrics_url` (`ARENA_METRICS_URL`), VictoriaMetrics' `/api/v1/query` |
| the chaos tool | `GET /overview` every `[collect] chaos_every`; a running run's `GET /runs/{id}/events`; a new `last_validate`'s `GET /runs/{id}` | the current run (kind, name, phase, requests a second, failed share, p99, tokens a second), and each way in's last end-to-end check | `[sources] chaos_url` (`ARENA_CHAOS_URL`), the API root ending in `/api/chaos/v1` |

- **Absent, never zero.** A source with an empty URL is "not configured"; one that did
  not answer keeps its last figures and says why in `sources` (with the age of its last
  good read). A rate with no data behind it (no generation in the window, a histogram
  with nothing in it) is absent from the snapshot, and the page says "no traffic".
- **The ways in are checked, not inferred.** The surfaces (`rest`, `sse`, `websocket`,
  `mcp`, `grpc`) are the chaos tool's own end-to-end checks against the deployed
  stack (`rest_evaluate`, `sse_events`, `ws_mux`, `http_mcp_tools`, `grpc_protocol_ping`),
  with the time they ran; the MCP tool count is what `http_mcp_tools` saw. On start the
  arena creates one chaos schedule, `arena: every way in`, if it is missing: a validate
  job on `[sources] validate_cron` (every two minutes) with Slack notifications off.
- **Rates from a chaos run** are diffs of consecutive `load` frames, which are
  cumulative since the run began; counts that go back (a phase that reset them) keep
  the last rates.
- **Who may watch.** `[watch] require_role` (`admin` while the lab is private): a
  caller without that role is `PERMISSION_DENIED`, none at all `UNAUTHENTICATED`. Empty
  opens the arena to anyone at publication (RFC 0006). Envoy gates `/v1/arena/` on the
  site's host the same way as the lab; on the site's socket the service's check is the
  gate (docs/auth/README.md).
- **How many may watch.** `[watch] max_viewers` streams at once; one more is
  `RESOURCE_EXHAUSTED` ("busy: ..."). Each viewer gets the current snapshot at once,
  then the broadcast; a slow one skips to the next whole frame.
- **Addressing.** The model service is reached through Envoy like every service. The
  metrics store is observability and the chaos tool is the operator's tool; both are
  dialled directly, like a database, and only by overlays that deploy them
  (`ARENA_CHAOS_URL` is set by the local and dev overlays; empty in production).
- **Embedded** (`serve_with`, the chaos kind): no collector runs, so the chaos tool
  never watches itself; the snapshot still ticks, every source "not configured".

Metrics: `tbd_arena_viewers`, `tbd_arena_source_ok{source}`,
`tbd_arena_source_age_seconds{source}` (docs/observability/metrics.md).
