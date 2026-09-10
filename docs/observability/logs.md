# Logs

Every process writes one JSON object per line to stdout. Nothing writes files, nothing
ships logs itself. An OpenTelemetry Collector agent on each node tails the pod logs,
parses the JSON, adds Kubernetes metadata and sends them to VictoriaLogs.

## Line shape

Services (`tracing-subscriber` JSON with `flatten_event` and the current span):

```json
{"timestamp":"2026-09-10T12:10:18.705233Z","level":"DEBUG","message":"evaluate (stub)",
 "subject_id":"s1","payload_len":0,"target":"tbd_engine::service",
 "span":{"rpc.method":"tbd.engine.v1.EngineService/Evaluate","rpc.system":"grpc",
         "trace_id":"0af7651916cd43dd8448eb211c80319c","name":"grpc.request"}}
```

Envoy access log, one per request, defined in `devops/envoy/envoy.yaml`:

```json
{"timestamp":"...","level":"INFO","message":"access","listener":"0.0.0.0:8080",
 "method":"POST","path":"/v1/evaluate","protocol":"HTTP/1.1","status":200,"grpc_status":null,
 "flags":"-","duration_ms":3,"upstream_ms":2,"bytes_in":32,"bytes_out":71,
 "upstream":"10.42.1.9:8080","cluster":"protocol","client":"10.42.0.1",
 "request_id":"...","traceparent":"00-...-01","trace_id":"...","user_agent":"..."}
```

Levels: services log per-request detail at `debug` and state changes at `info`; server
errors are `error`. Envoy access lines are `info`. Under chaos the injected failures
would flood the error level, so the chaos tool filters them; production keeps them.

## Pipeline

`devops/k8s/observability/otel-collector.yaml`, the `otel-agent` DaemonSet:

1. `filelog` receiver reads `/var/log/pods/*/*/*.log` from the node, starting at the end.
2. `container` operator strips the runtime's wrapper.
3. `json_parser` runs on lines that start with `{`: fields become attributes, `level`
   becomes severity, `timestamp` becomes the record time, `message` becomes the body.
4. `move` lifts `span.trace_id` to a top-level `trace_id` attribute; `trace_parser` also
   sets the record's real trace id so Tempo links work in both directions.
5. `k8sattributes` adds `k8s.namespace.name`, `k8s.pod.name`, `k8s.container.name`,
   deployment and node.
6. `resource/service` sets `service.name` from the container name, so `engine`,
   `protocol` and `envoy` are queryable by name.
7. `otlphttp` exporter posts to VictoriaLogs' OTLP endpoint with
   `VL-Stream-Fields: service.name,k8s.namespace.name,k8s.container.name`, which keeps the
   stream set small and stable.

Stream fields identify "where a line came from" and are the cheap filter. Everything else,
including `trace_id`, `level`, `route`, `status`, is an indexed field you can filter on
without a stream selector.

## Querying

Two UIs. Grafana Explore with the VictoriaLogs datasource has the field browser, the
volume histogram and the `trace_id` link to Tempo. VictoriaLogs' own UI at
http://localhost:9428/select/vmui (port 9428 on the local cluster) is the stronger
explorer: field statistics, hit histograms, completion, every LogsQL pipe. Grafana's
"Logs Drilldown" app is Loki-only and shows a "no Loki datasource" notice here; ignore
it.

LogsQL:

```
trace_id:="0af7651916cd43dd8448eb211c80319c"            # one request, every hop
service.name:engine level:ERROR _time:1h                 # engine errors, last hour
service.name:envoy status:503 _time:15m | fields _time, path, cluster, flags, upstream
service.name:protocol "engine health check failed"       # full-text
_time:5m | stats by (service.name, level) count() lines   # volume by level
service.name:envoy _time:1h | stats by (cluster) quantile(0.99, duration_ms) p99_ms
```

`flags` in Envoy lines is the fastest tell: `UH` no healthy upstream, `UF` upstream
connection failure, `URX` retry limit exceeded, `UT` upstream timeout, `DC` client
disconnected.

## Retention and volume

30 days, set with `-retentionPeriod` on the VictoriaLogs StatefulSet. Envoy's access log
is the largest source: one line per request. At a million requests a day that is well
under a gigabyte a day after compression; the PVC is 200 GiB locally.

## Adding a field

Log it as a `tracing` field on the event or the span: `tracing::info!(subject_id = %id,
"…")`. It appears in JSON automatically and becomes a queryable field in VictoriaLogs with
no pipeline change. Nested span fields are flattened by the datasource as `span.<name>`
except `trace_id`, which the pipeline lifts to the top level.
