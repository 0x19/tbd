# envoy

One static Envoy configuration, `envoy.yaml`, used everywhere: `compose.yaml`, the
Ansible compose template, and Kubernetes (as a ConfigMap via `kustomization.yaml`).
The header comment in `envoy.yaml` documents the listeners, routing and clusters.

| Port | Role |
|---|---|
| 8080 | edge: REST, SSE, GraphQL, WebSocket, gRPC (engine and protocol services by name) |
| 50051 | engine load balancer, gRPC only; protocol instances use it as their engine URL |
| 9901 | admin: `/ready`, `/stats/prometheus`, `/clusters`, `/config_dump` |

Upstreams are found by DNS name. Kubernetes provides `engine` and `protocol` as headless
Services so Envoy sees every pod and balances with least-request, active health checks and
outlier ejection. Compose provides the same names as service names.

Validate a change without a cluster:

```sh
docker run --rm -v "$PWD/devops/envoy/envoy.yaml:/e.yaml:ro" envoyproxy/envoy:v1.39.1 --mode validate -c /e.yaml
```

Then `mise run up` (compose) or `mise run local:deploy` (cluster) and `mise run validate`.
