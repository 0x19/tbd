# Observability

What you can see about a running system, where it comes from, and how to reach it.
The same stack runs on the local cluster and in production; only sizing and storage
change.

| Signal | Produced by | Shipped by | Stored in | Seen in |
|---|---|---|---|---|
| Metrics | every service on `/metrics` (Prometheus text), Envoy on its admin port | VictoriaMetrics scrapes pods annotated `prometheus.io/scrape` | VictoriaMetrics, 30 d | Grafana dashboards, PromQL |
| Traces | every service (OTLP/gRPC), Envoy as the root span | OpenTelemetry Collector gateway → Tempo | Tempo, local blocks | Grafana Explore, TraceQL, service graph |
| Logs | every service as JSON on stdout with `trace_id`, Envoy access logs as JSON | OpenTelemetry Collector agent tails pod logs, parses the JSON, adds k8s metadata | VictoriaLogs, 30 d | Grafana Explore, VictoriaLogs UI, LogsQL |
| Profiles | every service in-process (`pprof-rs`); every container via eBPF on real nodes | pushed to Pyroscope | Pyroscope, filesystem | Profiles Drilldown, Pyroscope UI, flame graphs linked from spans |
| Span metrics, service graph | Tempo's metrics-generator from traces | remote-write | VictoriaMetrics | Traces dashboard, node graph |

Why these: VictoriaLogs indexes every field, so `trace_id:="…"` is a direct lookup and
full-text search works without a label selector; it and VictoriaMetrics are Apache-2.0,
run as one binary each and cluster later. Tempo is the best-integrated trace store for
Grafana. The upstream OpenTelemetry Collector keeps the pipeline backend-neutral.
The reasoning behind the choices is in [production.md](production.md#why-this-stack).

| Page | Covers |
|---|---|
| [metrics.md](metrics.md) | every metric name and label, how they are recorded, PromQL patterns |
| [tracing.md](tracing.md) | spans, propagation, sampling, TraceQL |
| [logs.md](logs.md) | log shape, the collector pipeline, LogsQL |
| [dashboards.md](dashboards.md) | the five dashboards, editing and reloading them |
| [profiling.md](profiling.md) | continuous CPU profiles, flame graphs, trace-to-profile |
| [production.md](production.md) | why this stack, sizing, durability, sampling, retention |
| [../local-cluster.md](../local-cluster.md) | the k3d cluster this all runs on locally |

## Reaching it locally

```sh
mise run local:up        # k3d cluster, once
mise run local:build     # images into the cluster
mise run local:deploy    # observability + envoy + services, waits for readiness
mise run local:traffic   # 30 rounds of validate through Envoy so there is data
mise run local:load      # 60 s of sustained load; needed for meaningful profiles
mise run k9s             # the cluster in a TUI
```

Details of the cluster itself are in [local-cluster.md](../local-cluster.md).

| URL | What |
|---|---|
| http://localhost:3000 | Grafana, `admin` / `admin` on the LAN; publicly `grafana.<domain>` signs you in through Envoy and creates your user ([auth](../auth/README.md)). Dashboards tagged `tbd`. |
| http://localhost:9090 | VictoriaMetrics UI: PromQL, targets, series |
| http://localhost:18080 | Envoy edge: REST, SSE, GraphQL, WebSocket, gRPC |
| http://localhost:15051 | Envoy engine load balancer, gRPC |
| http://localhost:14317 | OTLP/gRPC into the collector, for processes running on the host |

Or through Ansible, same result: `mise run ansible:local`.

## Following one request

1. Every request gets a `trace_id`. Envoy starts the trace and propagates W3C
   `traceparent`; the protocol adopts it and forwards it to the engine; a client can also
   supply its own `traceparent` and it is honoured.
2. In Grafana Explore with the Tempo datasource, search `{ resource.service.name = "protocol" }`
   or paste a trace id. The trace shows Envoy → protocol → engine spans with the route and
   RPC names.
3. From any span, "Logs for this span" opens VictoriaLogs filtered by that `trace_id`.
   From a log line, the `trace_id` field links back to the trace.
4. In VictoriaLogs directly: `trace_id:="<id>"` returns every line from every service and
   Envoy for that request. `service.name:engine level:ERROR _time:1h` is a typical hunt.

## Following one request

1. Every request gets a `trace_id`. Envoy starts the trace and propagates W3C
   `traceparent`; the protocol adopts it and forwards it to the engine; a client can also
   supply its own `traceparent` and it is honoured.
2. In Grafana Explore with the Tempo datasource, search `{ resource.service.name = "protocol" }`
   or paste a trace id. The trace shows Envoy → protocol → engine spans with the route and
   RPC names.
3. From any span, "Logs for this span" opens VictoriaLogs filtered by that `trace_id`.
   From a log line, the `trace_id` field links back to the trace.
4. In VictoriaLogs directly: `trace_id:="<id>"` returns every line from every service and
   Envoy for that request. `service.name:engine level:ERROR _time:1h` is a typical hunt.

## Troubleshooting

| Symptom | Look at |
|---|---|
| A service pod is Running but not Ready | `kubectl -n tbd logs deploy/protocol`: readiness fails when the engine is unreachable through Envoy |
| No traces | `OTEL_EXPORTER_OTLP_ENDPOINT` in the ConfigMap, then `kubectl -n observability logs deploy/otel-collector` |
| No logs from a pod | `kubectl -n observability logs ds/otel-agent`; the agent excludes its own logs and needs `/var/log/pods` on the node |
| No metrics for a pod | it needs the `prometheus.io/scrape`, `port` and `path` annotations; VictoriaMetrics' targets page at :9090 lists what it sees |
| Envoy returns 503 | Envoy's admin `clusters` endpoint (port-forward 9901): hosts marked unhealthy failed the active health check; `envoy_cluster_membership_healthy` on the Envoy dashboard shows it over time |
| Dashboards changed but Grafana shows old ones | `mise run grafana:reload`; the ConfigMap name carries a content hash, so a re-apply rolls the pod |
| A trace has Envoy but no service spans | the service's `OTEL_EXPORTER_OTLP_ENDPOINT` is unset or wrong; the service still logs the `trace_id`, so logs work |
