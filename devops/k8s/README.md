# devops/k8s

kustomize only, no Helm. Three trees:

```
base/            engine, protocol, humans, ledger, envoy (deployments, headless services, configmap, envoy config)
ledger-db/       the ledger's Postgres (pgvector) and ClickHouse StatefulSets; applied by `mise run ledger:deploy`
overlays/
  local/         the k3d cluster on this machine: 2 replicas each, host-facing envoy-lb
  dev/           1 replica, debug logging, images pulled
  prod/          replicas and pinned tags per release
observability/   VictoriaMetrics, VictoriaLogs, Tempo, Pyroscope, OTel Collector agent + gateway, Grafana
  ebpf/          Alloy eBPF profiler DaemonSet; real nodes only, applied separately
                 (dashboards and datasources come from ../../grafana via kustomize)
```

## Apply order

Observability first, then the app; the app's ConfigMap points traces at the collector.

```sh
kubectl apply -k devops/k8s/observability
kubectl apply -f devops/k8s/observability/local-services.yaml   # local cluster only
kubectl apply -k devops/k8s/overlays/local                       # or dev | prod
```

`mise run local:deploy` does exactly this and waits for every rollout.

## Base

- `configmap.yaml` (`tbd-env`) holds every service env var: listen and metrics addresses,
  `PROTOCOL_<NAME>_URL=http://envoy:50051` per protocol backend, `OTEL_EXPORTER_OTLP_ENDPOINT`, sampling, log
  format and filter. Overlays merge into it with `configMapGenerator` + `behavior: merge`.
- `engine/` and `protocol/`: Deployments with gRPC or HTTP probes, non-root, read-only
  root filesystem, all capabilities dropped, `prometheus.io/*` annotations on the pod
  template, and **headless** Services so Envoy resolves pod IPs.
- `envoy/`: Deployment (2 replicas, admin `/ready` probes, uid 101), the ClusterIP
  Service `envoy` on 8080 and 50051, an ExternalName Service `otel-collector`, and the
  config ConfigMap generated from `devops/envoy/envoy.yaml`.

## Overlays

| Overlay | Replicas | Images | Notes |
|---|---|---|---|
| `local` | engine 2, protocol 2, envoy 2, humans 1, ledger 1 | `:dev`, `IfNotPresent`, imported by k3d | adds `envoy-lb.yaml` with host ports 18080 and 15051; `RUST_LOG=info,tbd=debug` |
| `dev` | engine 1, protocol 1, envoy 2, humans 1, ledger 1 | `:dev`, `Always` | `RUST_LOG=debug` |
| `prod` | see file | pinned `newTag` per release | resources raised |

An overlay changes replicas, image tags, pull policy and env; it does not redefine
workloads.

## Observability

Each component is one file with its ConfigMap, workload and Service. Storage is a PVC per
StatefulSet through the default StorageClass. Host-facing `LoadBalancer` Services for
Grafana, VictoriaMetrics and the collector are in `local-services.yaml`, applied
separately so other environments do not get them.

Grafana's provisioning files and dashboards are generated into ConfigMaps by
`devops/grafana/kustomization.yaml`, pulled in as `../../grafana`. The generated names
carry a content hash, so any dashboard change rolls Grafana on apply.

## Adding things

- **An env var**: `base/configmap.yaml`, plus `.env.example`, `compose.yaml` and the
  Ansible compose template. Overlays override with `configMapGenerator`.
- **A service**: a directory under `base/` modelled on `protocol/`, a headless Service,
  the `prometheus.io` annotations, a cluster and route in `devops/envoy/envoy.yaml`, and
  an entry in `base/kustomization.yaml`.
- **A host port on the local cluster**: a `-p` flag in `mise.toml` `local:up` and the
  Ansible playbook, a port on the matching `*-lb` Service, then recreate the cluster.
- **An image version**: the observability files pin `victoriametrics/victoria-metrics`,
  `victoriametrics/victoria-logs`, `grafana/tempo`, `otel/opentelemetry-collector-*`,
  `grafana/grafana`; `base/envoy/deployment.yaml` pins Envoy. Bump one at a time and
  check `kubectl -n observability logs` for config-key changes.

## Validating without a cluster

```sh
kustomize build devops/k8s/observability > /dev/null
kustomize build devops/k8s/overlays/local > /dev/null
mise run envoy:validate
```
