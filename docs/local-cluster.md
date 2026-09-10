# The local cluster

A real Kubernetes cluster on this machine, with Envoy, both services and the whole
observability stack, so what you debug locally is what runs in production. It is k3d:
k3s nodes as Docker containers, with a load balancer container that maps host ports.

## Lifecycle

```sh
mise run local:up        # create the cluster if missing; ~40 s
mise run local:build     # build both images and import them (no registry needed)
mise run local:deploy    # apply observability, host services and the app; waits for readiness
mise run local:traffic   # 30 rounds of `chaos validate` through Envoy (TRAFFIC_ROUNDS=n)
mise run local:restart   # after a code change: rebuild, import, roll the app pods
mise run local:status    # every pod
mise run local:urls      # the port map below
mise run k9s             # TUI on the cluster
mise run local:down      # delete the cluster; PVC data on the RAID is kept
```

`mise run ansible:local` runs `devops/ansible/playbooks/local.yml`, which does the same
steps idempotently and is the template for provisioning a real box the same way. Pass
`-e rebuild=false` to skip the image build.

## What runs where

| Namespace | Workload | Replicas | Purpose |
|---|---|---|---|
| `tbd` | `envoy` | 2 | edge and engine load balancer |
| `tbd` | `protocol` | 2 | REST, SSE, GraphQL, WebSocket, gRPC |
| `tbd` | `engine` | 2 | gRPC compute |
| `observability` | `victoria-metrics` | 1 | metrics store and scraper |
| `observability` | `victoria-logs` | 1 | log store |
| `observability` | `tempo` | 1 | trace store, span metrics |
| `observability` | `otel-agent` | one per node | tails pod logs |
| `observability` | `otel-collector` | 1 | OTLP gateway for traces |
| `observability` | `grafana` | 1 | UI |

## Ports

k3d's load balancer maps these host ports to Services of type `LoadBalancer` with the
same port number. They are chosen to avoid 80, 443, 8080 and 50051, which other things
on this machine use.

| Host | Service | What |
|---|---|---|
| 18080 | `tbd/envoy-lb` | Envoy edge: REST, SSE, GraphQL, WebSocket, gRPC |
| 15051 | `tbd/envoy-lb` | Envoy engine load balancer, gRPC |
| 3000 | `observability/grafana-lb` | Grafana, admin/admin |
| 9090 | `observability/victoria-metrics-lb` | VictoriaMetrics UI and API |
| 14317 | `observability/otel-collector-lb` | OTLP/gRPC into the collector, for processes on the host |

The mapping lives in two places that must agree: the `-p` flags in `mise.toml`
`local:up` (and the Ansible playbook), and the `*-lb` Services in
`devops/k8s/overlays/local/envoy-lb.yaml` and
`devops/k8s/observability/local-services.yaml`. Adding a port means a cluster
recreate, because k3d fixes the mappings at creation.

In-cluster names never change: `envoy:8080`, `envoy:50051`, `otel-collector.observability.svc:4317`.

## Storage

k3s' local-path provisioner backs every PVC, and the cluster mounts
`/mnt/raid0/tbd-k3d/storage` as that directory on every node, so data lands on the
RAID. `local:down` deletes the cluster but not that directory; a fresh cluster reattaches
new PVCs to new subdirectories, so old data is kept but not reused. Clear the directory
by hand when you want a clean slate.

## Sending traces from the host

Processes started with `mise run dev` or `chaos up` can export into the cluster's
collector:

```sh
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:14317 mise run dev
```

They then show up in Tempo alongside the cluster's own services. Their metrics are not
scraped, and their logs are not collected; use the cluster deployment for those.

## Pinning

- k3s image `rancher/k3s:v1.34.9-k3s1`. k3s 1.35 crash-looped its cloud-controller-manager
  on this host at startup; do not bump without checking `docker logs k3d-tbd-server-0`.
- Images are tagged `dev` and imported straight from the Docker daemon;
  `imagePullPolicy: IfNotPresent`. A rebuild needs `local:restart` to roll the pods,
  because the tag does not change.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `local:up` says a port is in use | `ss -ltnp \| grep <port>`; stop the other process or change the map in both places and recreate |
| `kubectl` says the server cannot handle the request right after creation | wait 20 s; the API server is still bootstrapping |
| `k3d-tbd-server-0` keeps restarting | `docker logs k3d-tbd-server-0`; check the k3s pin above |
| a pod is Running but never Ready | `kubectl -n <ns> describe pod <name>` for probe failures; for `protocol`, the engine is unreachable through Envoy |
| `ImagePullBackOff` on engine or protocol | the image was not imported: `mise run local:build` |
| everything is up but Grafana is empty | `mise run local:traffic`, then wait 15 s for the scrape |
| Tempo pod crash-loops after a config edit | the config keys changed in Tempo 3; `kubectl -n observability logs tempo-0` names the field |

## Teardown

```sh
mise run local:down                   # cluster gone, ~5 s
rm -rf /mnt/raid0/tbd-k3d/storage     # optional: the data too
```
