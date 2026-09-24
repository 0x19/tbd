# Metrics

Every service exposes Prometheus text on its own port, tagged with one `service` label
per process. VictoriaMetrics scrapes any pod annotated for it every 15 seconds. Envoy is
scraped on its admin port. Tempo adds span metrics derived from traces.

## Service metrics

Names are constants in `crates/common/src/metrics.rs`; dashboards and alerts use the
same strings. All histograms share buckets from 0.5 ms to 30 s, chosen so a hung
backend is visible instead of clipped.

| Metric | Type | Labels | Recorded when |
|---|---|---|---|
| `tbd_requests_total` | counter | `transport` (`http`, `grpc`, `ws`), `route`, `status` | a request finishes; `status` is `ok`, an HTTP code, or a gRPC code name |
| `tbd_request_duration_seconds` | histogram | `transport`, `route` | same moment; for streams this is time to first response |
| `tbd_requests_in_flight` | gauge | `transport` | incremented on admission, decremented on completion, including early returns |
| `tbd_streams_active` | gauge | `kind` (`subscribe`, `session`, `ws`, `sse`, `mux`) | a stream opens or closes |
| `tbd_cv_notifications_total` | counter | `kind` (`owner`, `requester`), `outcome` (`sent`, `refused`, `failed`, `no_mailbox`, `unreachable`) | the cv service tried to send a mail through the finance service's linked mailbox |
| `tbd_llm_tokens_total` | counter | `tier` (`fast`, `deep`), `engine` (`ollama`, `llamacpp`, `stub`), `model`, `kind` (`prompt`, `completion`) | a generation's done chunk arrived with the engine's token counts |
| `tbd_llm_time_to_first_token_seconds` | histogram | `tier`, `engine` | a generation's first text chunk arrived; measured from admission |
| `tbd_llm_in_flight` | gauge | `tier` | requests running on the tier now (admission's slots taken) |
| `tbd_llm_queued` | gauge | `tier` | requests waiting for a slot on the tier now |
| `tbd_llm_queue_wait_seconds` | histogram | `tier` | an admitted request got its slot; how long it waited in line |
| `tbd_llm_refused_total` | counter | `tier`, `reason` (`queue_full`, `queue_timeout`) | admission refused a request with `RESOURCE_EXHAUSTED` |
| `tbd_sandbox_runs_total` | counter | `language` (`go`, `rust`), `outcome` (`ok`, `exit`, `compile_error`, `killed`) | the sandbox daemon on the host finished a run (docs/sandbox/README.md) |
| `tbd_sandbox_duration_seconds` | histogram | `language`, `step` (`compile`, `run`) | a sandbox step ended, by itself or stopped from outside |
| `tbd_sandbox_in_flight` | gauge | | sandbox runs in progress on the host |
| `tbd_arena_viewers` | gauge | | streams of the arena's snapshot (`ArenaService/Watch`) open now |
| `tbd_arena_source_ok` | gauge | `source` (`llm`, `metrics`, `chaos`) | the arena's last read of that source: 1 when it succeeded, else 0 |
| `tbd_arena_source_age_seconds` | gauge | `source` | seconds since the arena last read that source successfully; absent until the first success |
| `tbd_radar_fetches_total` | counter | `source` (the configured name), `outcome` (`ok`, `failed`) | the radar read one of its sources, on the timer or through `Refresh` |
| `tbd_radar_items_new_total` | counter | `source` | items the radar saw for the first time in that read |
| `tbd_radar_digests_total` | counter | `language` (`go`, `rust`), `lang` (`en`, `hr`), `outcome` (`written`, `no_items`, `failed`) | the radar tried to write a week's digest, on the schedule or through `RunDigest` |
| `tbd_llm_engine_up` | gauge | `tier`, `engine` | the llm service's probe of a tier's engine (every `[engines] probe_interval`): 1 while it answers, else 0; never drives the service's own health |
| `tbd_stream_items_total` | counter | `kind`, `direction` (`in`, `out`) | an item crosses a stream |
| `tbd_engine_client_requests_total` | counter | `backend` (`engine`, `ledger`, …: the protocol's `[services]` name), `route`, `status` | the protocol finishes a call to a backend, measured on the client side |
| `tbd_engine_client_duration_seconds` | histogram | `backend`, `route` | same moment |
| `tbd_faults_injected_total` | counter | `kind` | the fault handle rejects a request (chaos) |
| `tbd_build_info` | gauge | `version` | once at start, always 1; join on it to label by version |
| `tbd_ledger_store_up` | gauge | | the ledger's readiness probe of its store: 1 while it answers within `[health] probe_timeout`; the gRPC health status follows it |
| `tbd_ledger_outbox_batches_total` | counter | `status` (`ok`, `error`) | the outbox drainer hands a batch to the analytics sink; `error` batches stay leased and are retried |
| `tbd_ledger_outbox_events_total` | counter | `kind` (`fact.recorded`, `fact.retracted`, `subject.erased`) | events acked after the sink took them |
| `tbd_ledger_erasures_executed_total` | counter | | the sweeper executed an erasure cascade |
| `tbd_db_pool_connections` | gauge | `state` (`idle`, `in_use`) | sampled every five seconds from the ledger's Postgres pool |
| `tbd_db_pool_max_connections` | gauge | | the pool's configured maximum, so `in_use / max` is saturation |
| `tbd_ledger_store_op_duration_seconds` | histogram | `op` (`append`, `current`, `history`, `retract`, `request_erasure`, `restore`, `execute_due_erasures`, `claim_events`, `ack_events`, `purge_idempotency`, `subject`, `ping`) | every store call, whoever made it (the gRPC adapter, the sweeper, the drainer); recorded by `store::Instrumented`, which `build_store` wraps every backend in |
| `tbd_ledger_store_ops_total` | counter | `op`, `result` (`ok`, `not_found`, `erased`, `invalid`, `forbidden`, `conflict`, `unavailable`, `internal`) | same moment; `unavailable` and `internal` are the ledger failing, the rest is the contract answering |
| `tbd_ledger_store_faults_injected_total` | counter | `kind` | the `Faulty` store decorator failed a call on chaos's `set_store_behavior` (a read before it ran, a write after it committed); never in production. Outside `Instrumented`, so not in `tbd_ledger_store_ops_total` |
| `tbd_ledger_facts_appended_total` | counter | `source` | a fact was written; an idempotent replay is not a fact |
| `tbd_ledger_appends_replayed_total` | counter | | an append matched its idempotency key and returned the earlier fact |
| `tbd_ledger_facts_retracted_total` | counter | | a retraction wrote its tombstone and deleted the values |
| `tbd_ledger_envelope_bytes` | histogram (64 B to 64 KiB) | `field` (`value`, `origin`) | the sizes of accepted envelopes |
| `tbd_ledger_page_facts` | histogram (1 to 1000) | `op` (`current`, `history`) | facts per page returned |
| `tbd_ledger_erasures_requested_total` | counter | | an erasure opened its grace window |
| `tbd_ledger_erasures_restored_total` | counter | | a restore cancelled one inside the window |
| `tbd_ledger_erasure_tombstones_total` | counter | | tombstones a cascade wrote on surviving subjects |
| `tbd_ledger_erasures_pending` | gauge | `state` (`pending`, `due`) | sampled every five seconds from Postgres: inside the window, and past it waiting for the sweeper |
| `tbd_ledger_idempotency_purged_total` | counter | | idempotency rows purged after their TTL |
| `tbd_ledger_outbox_pending` | gauge | | unpublished outbox events, sampled every five seconds from Postgres |
| `tbd_ledger_outbox_oldest_seconds` | gauge | | age of the oldest unpublished event, same sample; zero when empty |
| `tbd_ledger_outbox_publish_duration_seconds` | histogram | | one batch handed to the analytics sink |
| `tbd_ledger_outbox_lag_seconds` | histogram (10 ms to 1 h) | | at ack: the recorded-to-acked age of the oldest event in the batch |
| `tbd_ledger_analytics_deletes_total` | counter | | an erased subject's rows were deleted from ClickHouse |
| `tbd_ledger_table_rows` | gauge | `table` | the planner's row estimate per ledger table, sampled every five seconds |
| `tbd_ledger_table_bytes` | gauge | `table` | on-disk size per ledger table with indexes, same sample |
| `process_cpu_seconds_total`, `process_resident_memory_bytes`, `process_virtual_memory_bytes`, `process_open_fds`, `process_max_fds`, `process_threads`, `process_start_time_seconds` | mixed | | every 5 s by `metrics-process` |

`route` values: axum's matched pattern for HTTP (`/v1/evaluate`, `/v1/subjects/{subject_id}/events`),
the RPC path for gRPC (`EngineService/Evaluate` on the engine, `LedgerService/Ping` on
the ledger,
`tbd.protocol.v1.ProtocolService/Ping` on the protocol). An HTTP request that matches
no route is `unmatched` (answered 404), never its raw path, so internet scanners cannot
grow the label set.

How they are recorded: `RequestTimer::start(transport, route)` at admission, its `Drop`
records the counter and histogram and decrements in-flight, so an early `return Err`
cannot skip it. `StreamGuard::open(kind)` does the same for the gauge; `item(direction)`
counts. The engine calls `admit()` first in every RPC; the protocol has one middleware for
every route including the gRPC ones on the same port, and wraps the engine channel in a
tower service for client-side samples.

## Envoy metrics

Envoy exposes its stats at `:9901/stats/prometheus`. The ones the dashboards use:

| Metric | Meaning |
|---|---|
| `envoy_http_downstream_rq_total{envoy_http_conn_manager_prefix}` | requests per listener (`edge`, `engine_lb`) |
| `envoy_http_downstream_rq_xx{envoy_response_code_class}` | response code classes per listener |
| `envoy_http_downstream_rq_time_bucket` | downstream latency histogram, milliseconds |
| `envoy_cluster_upstream_rq_total{envoy_cluster_name}` | requests per upstream cluster (`engine`, `protocol`, `otel-collector`) |
| `envoy_cluster_upstream_rq_xx` | upstream response classes |
| `envoy_cluster_upstream_rq_retry`, `_retry_success` | retries and how many rescued the request |
| `envoy_cluster_membership_healthy`, `_total` | hosts passing active health checks vs known |
| `envoy_cluster_outlier_detection_ejections_active` | hosts currently ejected |
| `envoy_cluster_health_check_failure`, `_success` | active health check outcomes |

## Span metrics

Tempo's metrics-generator derives `traces_spanmetrics_calls_total`,
`traces_spanmetrics_latency_bucket` and `traces_service_graph_request_total` from traces and
remote-writes them to VictoriaMetrics. They power the service graph and are a second
opinion on the request counts, computed from a different source.

## PromQL patterns

```promql
# requests per second by service
sum by (service) (rate(tbd_requests_total[1m]))

# error rate, HTTP and gRPC together
sum(rate(tbd_requests_total{status!~"ok|2..|3.."}[1m])) / sum(rate(tbd_requests_total[1m]))

# p99 by route on the protocol
histogram_quantile(0.99, sum by (le, route) (rate(tbd_request_duration_seconds_bucket{service="protocol"}[1m])))

# is the protocol's view of the engine slower than the engine's own view? (Envoy hop cost)
histogram_quantile(0.99, sum by (le) (rate(tbd_engine_client_duration_seconds_bucket[5m])))
  - on() histogram_quantile(0.99, sum by (le) (rate(tbd_request_duration_seconds_bucket{service="engine"}[5m])))

# open WebSocket sessions
sum(tbd_streams_active{kind="ws"})

# faults being injected right now
sum by (kind) (rate(tbd_faults_injected_total[1m]))
```

## Adding a metric

1. Add the name to `names` in `crates/common/src/metrics.rs` with a doc comment, and
   describe it in `describe()` with a unit where one applies.
2. Record it with the `metrics` macros. Prefer the existing guards over ad-hoc calls.
3. Add a row to the table above and a panel to the relevant dashboard.
4. Keep label cardinality bounded: routes and statuses, never ids.

## Local access

VictoriaMetrics UI on http://localhost:9090 (targets, series explorer, PromQL). Raw
endpoints: `curl localhost:9464/metrics` for a service running on the host,
`kubectl -n tbd port-forward deploy/envoy 9901` then `localhost:9901/stats/prometheus`.
