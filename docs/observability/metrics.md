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
| `tbd_requests_total` | counter | `transport` (`http`, `grpc`), `route`, `status` | a request finishes; `status` is `ok`, an HTTP code, or a gRPC code name |
| `tbd_request_duration_seconds` | histogram | `transport`, `route` | same moment; for streams this is time to first response |
| `tbd_requests_in_flight` | gauge | `transport` | incremented on admission, decremented on completion, including early returns |
| `tbd_streams_active` | gauge | `kind` (`subscribe`, `session`, `ws`, `sse`) | a stream opens or closes |
| `tbd_stream_items_total` | counter | `kind`, `direction` (`in`, `out`) | an item crosses a stream |
| `tbd_engine_client_requests_total` | counter | `route`, `status` | the protocol finishes a call to the engine, measured on the client side |
| `tbd_engine_client_duration_seconds` | histogram | `route` | same moment |
| `tbd_faults_injected_total` | counter | `kind` | the fault handle rejects a request (chaos) |
| `tbd_build_info` | gauge | `version` | once at start, always 1; join on it to label by version |
| `process_cpu_seconds_total`, `process_resident_memory_bytes`, `process_virtual_memory_bytes`, `process_open_fds`, `process_max_fds`, `process_threads`, `process_start_time_seconds` | mixed | | every 5 s by `metrics-process` |

`route` values: axum's matched pattern for HTTP (`/v1/evaluate`, `/v1/subjects/{subject_id}/events`),
the RPC path for gRPC (`EngineService/Evaluate` on the engine,
`tbd.protocol.v1.ProtocolService/Ping` on the protocol).

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
