# devops/grafana

Grafana provisioning and the dashboards for the tbd services. Everything here is
mounted into the Grafana pod by the kustomize manifests under `devops/k8s/observability`;
nothing is created by hand in the UI.

| Path | What | Mounted at |
|---|---|---|
| `provisioning/datasources/datasources.yaml` | the three datasources, with trace ↔ log ↔ metric links | `/etc/grafana/provisioning/datasources/` |
| `provisioning/dashboards/dashboards.yaml` | one file provider that loads every JSON below into a "tbd" folder | `/etc/grafana/provisioning/dashboards/` |
| `dashboards/tbd-overview.json` | traffic, errors, latency, streams, faults, process, logs, versions across all services | `/var/lib/grafana/dashboards/` |
| `dashboards/tbd-engine.json` | the engine: per-route traffic and latency, heatmap, streams, faults, logs | same |
| `dashboards/tbd-protocol.json` | the protocol: every surface, status codes, WS/SSE, calls to the engine, logs | same |
| `dashboards/tbd-traces.json` | TraceQL search, service map, span metrics | same |
| `dashboards/tbd-envoy.json` | Envoy: listeners, upstream clusters, retries, health checks, process, access logs | same |
| `dashboards/tbd-ledger.json` | the ledger: requests, time inside the store and pool saturation, facts by source and size, erasures and windows, outbox lag into ClickHouse, table growth, process, logs | same |

## How dashboards reach the cluster

kustomize builds a ConfigMap from `dashboards/*.json` and mounts it at
`/var/lib/grafana/dashboards`. The ConfigMap name carries a content hash, so a changed
file produces a new ConfigMap and Grafana's Deployment rolls onto it; the provider in
`dashboards.yaml` then loads the directory. End to end this takes a few seconds.

```sh
mise run grafana:reload     # apply, wait for the rollout, print the dashboards URL
```

Dashboards are provisioned read-only (`allowUiUpdates: false`). To change one:

1. Edit the JSON directly, or
2. Edit a copy in the UI (Save as… to another folder), then Share → Export with
   "Export for sharing externally" **off**, and paste the JSON back over the file. Keep
   the `uid` and `schemaVersion: 39`, and keep datasource references as
   `{"type": ..., "uid": "victoriametrics|victorialogs|tempo"}`.

Validate before applying:

```sh
for f in devops/grafana/dashboards/*.json; do python3 -c "import json,sys; json.load(open('$f'))" && echo "ok $f"; done
```

## Datasources

| UID | Type | URL | Notes |
|---|---|---|---|
| `victoriametrics` | prometheus | `http://victoria-metrics.observability.svc:8428` | default; PromQL |
| `victorialogs` | victoriametrics-logs-datasource | `http://victoria-logs.observability.svc:9428` | LogsQL; the `trace_id` field links to Tempo |
| `tempo` | tempo | `http://tempo.observability.svc:3200` | TraceQL; "Trace to logs" runs `trace_id:="<id>"` in VictoriaLogs; "Trace to profiles" opens Pyroscope by `service_name` |
| `pyroscope` | grafana-pyroscope-datasource | `http://pyroscope.observability.svc:4040` | CPU profiles from the eBPF agent; Profiles Drilldown |

Correlation works in both directions: click `trace_id` on a log line to open the trace;
click a span to see the logs written inside it.

## Metric names

Emitted by every service through `crates/common/src/metrics.rs`. The `service` label is
added globally by the exporter; the others are set per sample.

| Metric | Type | Labels | Meaning |
|---|---|---|---|
| `tbd_requests_total` | counter | `transport`, `route`, `status` | requests handled; `status` is `ok`, an HTTP code, or a gRPC code |
| `tbd_request_duration_seconds` | histogram | `transport`, `route` | time to answer, or to first response on streams |
| `tbd_requests_in_flight` | gauge | `transport` | requests currently being handled |
| `tbd_streams_active` | gauge | `kind` | open streams: `subscribe`, `session`, `ws`, `sse` |
| `tbd_stream_items_total` | counter | `kind`, `direction` | items on streams, `in` or `out` |
| `tbd_engine_client_requests_total` | counter | `route`, `status` | calls from the protocol to the engine |
| `tbd_engine_client_duration_seconds` | histogram | `route` | engine call latency, caller side |
| `tbd_faults_injected_total` | counter | `kind` | faults fired by the fault handle (chaos) |
| `tbd_build_info` | gauge | `version` | always 1 |
| `process_*` | mixed | | `process_cpu_seconds_total`, `process_resident_memory_bytes`, `process_open_fds`, `process_threads`, … from `metrics-process` |

Histogram buckets run from 0.5 ms to 30 s. `histogram_quantile` over `_bucket` with
`rate(...[1m])` is the pattern every latency panel uses.

## Queries that need extra plumbing

- `tbd-traces` span-metric panels (`traces_spanmetrics_*`) and the service map stay empty
  until Tempo's metrics-generator is enabled and remote-writes to VictoriaMetrics.
- The "Log lines/s by level" panel on the engine dashboard uses a LogsQL `stats by`
  query with `queryType: stats_range`. If the VictoriaLogs plugin renders nothing, drop
  the `queryType` field and let the plugin infer it.
