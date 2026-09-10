# Production notes

## Why this stack

Decided in September 2026 against primary sources. The requirement was one stack that
runs on a laptop and at millions of users, with Grafana on top, OpenTelemetry as the wire
format, and trace ↔ log ↔ metric correlation.

| Component | Pick | Why | Runner-up |
|---|---|---|---|
| Logs | VictoriaLogs | indexes every field, so `trace_id` lookups and full-text search need no label selector; Apache-2.0; a fraction of Loki's RAM and disk in the one published benchmark with method; native OTLP; single binary locally, cluster mode in OSS; supported as a Grafana trace-to-logs target | Loki 3.x: label-first, full-text acceleration still experimental and aimed at 75 TB/month users, AGPL, dev and prod modes behave differently |
| Traces | Tempo 3 | best Grafana integration, TraceQL, metrics-generator; monolithic mode needs no Kafka | Jaeger v2 if you want its own UI |
| Metrics | VictoriaMetrics | Prometheus-compatible, one binary, remote-write target for Tempo, same vendor and operator family as the logs | Prometheus 3 as reference; Mimir if standardising on the Grafana stack |
| Collector | OpenTelemetry Collector (upstream) | backend-neutral, `filelog` and `k8sattributes` presets, one config language | Grafana Alloy if the backends were Loki and Mimir |
| Profiles | Pyroscope + Alloy eBPF | no code changes, links from spans, same vendor UI | in-process `pyroscope` crate for memory profiles |
| UI | Grafana 13 | | |

Rust side: `tracing` + `tracing-opentelemetry` 0.33 + `opentelemetry-otlp` 0.32 for
traces; `metrics` 0.24 + `metrics-exporter-prometheus` 0.18 for metrics, chosen because
that API has been stable since 2024 while OpenTelemetry's Rust metrics API still breaks
every few months. Pin all `opentelemetry*` crates to one version and bump them together.

## Sizing

Local footprint on the k3d cluster is about 5 CPU and 10 GiB of RAM requested across the
stack, with PVCs of 100 GiB (metrics), 200 GiB (logs), 100 GiB (traces), 5 GiB (Grafana)
on the RAID. Envoy adds negligible overhead at this scale.

At production scale: VictoriaMetrics and VictoriaLogs each handle tens of thousands of
samples or lines per second per core on one node; go to cluster mode when a single node's
disk or ingest rate is the limit, not before. Tempo moves to microservices mode with Kafka
and object storage. Collector gateways scale horizontally behind a Service.

## Durability

- VictoriaLogs and VictoriaMetrics cluster mode shards but does not replicate. Run two
  independent instances fed by the collector, and snapshot the PVCs. VictoriaLogs has no
  object-storage backend yet; Loki is the fallback if that becomes a hard requirement, and
  only the collector's exporter and the Grafana datasource change.
- Tempo in production uses object storage and Kafka; the local config uses local blocks.
- Retention is 30 days for logs and metrics locally. Set it per environment on the
  StatefulSets.

## Sampling and volume

- Traces: `OTEL_TRACES_SAMPLER_ARG=1.0` locally. In production lower the root ratio and
  add a tail-sampling processor in the collector gateway that keeps every error trace and
  every slow trace; the collector sees the whole trace, the services do not.
- Logs: Envoy's access log is one line per request and the largest source. Keep it; it is
  what answers "did the request even arrive". Services log per-request detail at `debug`,
  so raising `RUST_LOG` on one pod is the escalation, not a permanent setting.
- Metrics: label cardinality is bounded by design (routes and statuses). Do not add ids
  as labels.

## Security

- Grafana runs with `admin`/`admin` and anonymous **Editor** access locally so Explore
  and the Drilldown apps work without signing in. Change both before anything is
  reachable beyond a developer machine: wire Grafana to the identity provider, drop
  anonymous access or set it to Viewer.
- The observability namespace exposes nothing outside the cluster except through the
  `*-lb` Services in the local overlay. Production reaches Grafana through the ingress
  with auth, and nothing else.
- Envoy's admin port stays cluster-internal.

## Alerting

Not configured yet. The metrics are ready for it: error rate, p99 by route,
`envoy_cluster_membership_healthy` dropping below total, `tbd_requests_in_flight`
climbing, `process_resident_memory_bytes` growth. Grafana alert rules or VictoriaMetrics'
`vmalert` are both options; the dashboards' threshold values are the starting point.
