# Dashboards

Five dashboards, provisioned from `devops/grafana/dashboards/*.json` into a "tbd" folder,
tagged `tbd`, read-only in the UI. http://localhost:3000/dashboards?tag=tbd locally.

| Dashboard | Use it when |
|---|---|
| **tbd / Overview** | first look: RPS, error rate, p50/p99 and in-flight by service, active streams, protocol → engine client calls, injected faults, CPU and memory, recent logs, running versions |
| **tbd / Engine** | the engine is suspect: per-RPC rate and errors, latency heatmap, streams by kind, items per second, faults, its logs |
| **tbd / Protocol** | the edge is suspect: per-route rate by transport, status codes, latency, WebSocket and SSE gauges, engine-client latency and errors, its logs |
| **tbd / Envoy** | routing or balancing is suspect: request rate and codes per listener, upstream health and ejections, retries, health-check outcomes, access logs |
| **tbd / Traces** | you have a trace id or want the service graph: TraceQL search, node graph, span-metric rates |

Every dashboard has a `service` or equivalent variable where it applies, refreshes every
10 seconds and opens on the last 30 minutes.

## Editing

The JSON files are the source of truth. Two workflows:

- Edit the JSON and run `mise run grafana:reload`. kustomize regenerates the ConfigMap
  with a new content hash, Grafana rolls, and the change is live in a few seconds.
- Design in the UI: Save as… into another folder, iterate, then Share → Export with
  "export for sharing externally" **off**, paste the JSON over the file, keep the `uid`,
  keep `schemaVersion: 39`, keep datasource references as `{ "type": …, "uid": … }` with
  one of the three UIDs below. Then reload.

Validate before reloading:

```sh
for f in devops/grafana/dashboards/*.json; do python3 -c "import json; json.load(open('$f'))" && echo "ok $f"; done
```

## Datasources

Provisioned from `devops/grafana/provisioning/datasources/datasources.yaml`, not editable
in the UI.

| UID | Type | Correlation |
|---|---|---|
| `victoriametrics` | Prometheus-compatible | default datasource |
| `victorialogs` | VictoriaLogs plugin | the `trace_id` field links to Tempo |
| `tempo` | Tempo | trace-to-logs runs `trace_id:="${__span.traceId}"` in VictoriaLogs; trace-to-metrics and service map use VictoriaMetrics |

## Adding a dashboard

1. Create `devops/grafana/dashboards/tbd-<name>.json` with a unique `uid`, tag `tbd`,
   and the links block copied from an existing one so cross-navigation stays complete.
2. Add the file to the `grafana-dashboards` generator in `devops/grafana/kustomization.yaml`.
3. Add a row to the table in this page and in `devops/grafana/README.md`.
4. `mise run grafana:reload`.

## Conventions

- Rates use `rate(...[1m])`; latency panels use `histogram_quantile` over
  `sum by (le, …)`; units are set (`reqps`, `s`, `percentunit`, `bytes`).
- Error-rate stats have thresholds: green under 1 %, amber under 5 %, red above.
- A logs panel at the bottom of each service dashboard, filtered to that service, so the
  numbers and the words are on one screen.
