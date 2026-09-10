# Tracing

One trace per request, across Envoy, the protocol and the engine, exported over OTLP to
Tempo and correlated with logs by `trace_id`.

## Spans

| Service | Span | Fields |
|---|---|---|
| Envoy | `envoy ingress` | Envoy's standard attributes: upstream cluster, response code, request id |
| protocol | `http.request` | `http.request.method`, `http.route`, `trace_id` |
| protocol | `grpc.request` (gRPC on the same port) | `rpc.system`, `rpc.method`, `trace_id` |
| protocol → engine | client span from `tracing-opentelemetry` under the request span | RPC path |
| engine | `grpc.request` | `rpc.system`, `rpc.method`, `trace_id` |

Spans are created by `tower_http::trace::TraceLayer` with a custom span factory on the
protocol (`crates/protocol/src/observe.rs`) and by tonic's `trace_fn` on the engine
(`request_span` in `crates/engine/src/lib.rs`).

## Propagation

W3C `traceparent`. Envoy generates it when absent and forwards it; the protocol adopts
it with `propagation::adopt_parent` and injects it into engine calls with a tonic
interceptor (`TraceInject` in `crates/protocol/src/state.rs`); the engine adopts it from
gRPC metadata. A client that sends its own `traceparent` becomes the root; the
integration test `request_span_carries_a_trace_id_and_propagates_to_the_engine` proves
it.

Every request span records `trace_id` as a field, so JSON log lines written inside the
span carry it. That is the join key for logs.

## Export

`tbd_common::telemetry::init` always installs a tracer provider, so trace ids exist even
with no backend. Export happens only when `OTEL_EXPORTER_OTLP_ENDPOINT` is set (OTLP over
gRPC, batched). In the cluster that is `http://otel-collector.observability.svc:4317`; from
a process on the host, `http://localhost:14317`.

The collector gateway adds Kubernetes attributes (namespace, pod, deployment, node) by the
connection's source IP and forwards to Tempo. Envoy sends its spans to the same collector
through the `otel-collector` cluster in its config.

## Sampling

`OTEL_TRACES_SAMPLER_ARG` (default `1.0`) sets the root sampling ratio, parent-based, so
a sampled trace stays sampled across hops. Keep `1.0` until Tempo's write rate is the
problem; then lower the root ratio and add tail sampling in the collector gateway so
error traces are always kept. Envoy's tracer follows the incoming `traceparent` flag.

## Viewing traces

Tempo has no UI; Grafana is the front end, three ways in:

- **Traces Drilldown** (Grafana menu → Drilldown → Traces, or
  `/a/grafana-exploretraces-app/explore`): query-free. Starts from services and their
  rate, error and duration, lets you click a spike, compares slow against fast spans by
  attribute, and opens the waterfall. Backed by TraceQL metrics, which Tempo 3 serves
  natively in monolithic mode. Best first stop.
- **Explore** (`/explore`, datasource Tempo): the Search tab has dropdowns for service,
  span name, status and duration; the TraceQL tab takes a query; the Trace ID tab takes
  an id. Results open the waterfall.
- **The Traces dashboard**: a search panel over the last 30 minutes and the service
  graph.

Locally the anonymous role is Editor so all three work without signing in; in
production it is Viewer, which hides Explore and the Drilldown apps, so sign in.

## Querying

Grafana Explore, datasource Tempo. TraceQL:

```
{ resource.service.name = "protocol" }                       # everything through the protocol
{ resource.service.name = "engine" && duration > 50ms }       # slow engine calls
{ span.http.route = "/v1/evaluate" && status = error }        # failed evaluates
{ .rpc.method = "tbd.engine.v1.EngineService/Session" }       # sessions
```

From a span, "Logs for this span" opens VictoriaLogs with `trace_id:="<id>"` over a window
around the span. The Traces dashboard has a search panel, the service graph, and rates from
span metrics.

## Adding a traced hop

A new service needs three things: a span per request that calls `adopt_parent` on the
incoming carrier and records `trace_id`; `propagation::inject` on every outgoing call; and
`telemetry::init` in `main` with a service name. The two existing services are the
templates. Envoy needs nothing new for a service behind it.
