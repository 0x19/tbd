# The local cluster

A real Kubernetes cluster on this machine, with Envoy, both services and the whole
observability stack, so what you debug locally is what runs in production. It is k3d:
k3s nodes as Docker containers, with a load balancer container that maps host ports.

## Lifecycle

```sh
mise run local:up        # create the cluster if missing; ~40 s
mise run local:build     # build the engine, protocol and chaos images and import them (no registry needed)
mise run local:deploy    # apply observability, host services and the app; waits for readiness
mise run local:traffic   # 30 rounds of `chaos validate` through Envoy (TRAFFIC_ROUNDS=n)
mise run local:load      # sustained load through Envoy, 8 workers (LOAD_SECONDS=n); fills dashboards and profiles
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
| `tbd` | `chaos` | 1 | `chaos serve`: API and admin UI, behind Envoy at `/api/chaos` and `/chaos` |
| `observability` | `victoria-metrics` | 1 | metrics store and scraper |
| `observability` | `victoria-logs` | 1 | log store |
| `observability` | `tempo` | 1 | trace store, span metrics |
| `observability` | `otel-agent` | one per node | tails pod logs |
| `observability` | `otel-collector` | 1 | OTLP gateway for traces |
| `observability` | `pyroscope` | 1 | profile store |
| `observability` | `grafana` | 1 | UI |

## Ports

k3d's load balancer maps these host ports to Services of type `LoadBalancer` with the
same port number. They are chosen to avoid 80, 443, 8080 and 50051, which other things
on this machine use.

| Host | Service | What |
|---|---|---|
| 18080 | `tbd/envoy-lb` | Envoy edge: REST, SSE, GraphQL, WebSocket, gRPC |
| 15051 | `tbd/envoy-lb` | Envoy engine load balancer, gRPC |
| 18080 | `tbd/envoy-lb` | `/api/chaos/` and `/chaos/` on the same edge port: the chaos API and admin UI ([chaos/ui.md](chaos/ui.md)) |
| 3000 | `observability/grafana-lb` | Grafana, admin/admin |
| 9090 | `observability/victoria-metrics-lb` | VictoriaMetrics UI and API |
| 14317 | `observability/otel-collector-lb` | OTLP/gRPC into the collector, for processes on the host |
| 9428 | `observability/victoria-logs-lb` | VictoriaLogs UI at `/select/vmui` and its query API |
| 4040 | `observability/pyroscope-lb` | Pyroscope UI and API |

The mapping lives in two places that must agree: the `-p` flags in `mise.toml`
`local:up` (and the Ansible playbook), and the `*-lb` Services in
`devops/k8s/overlays/local/envoy-lb.yaml` and
`devops/k8s/observability/local-services.yaml`. Adding a port: `k3d cluster edit tbd --port-add "PORT:PORT@loadbalancer"` re-creates
only the load balancer container, then add the `*-lb` Service.

In-cluster names never change: `envoy:8080`, `envoy:50051`, `otel-collector.observability.svc:4317`.

## Node mounts

Besides storage, every node container gets the host's `/sys/kernel/tracing` and
`/sys/kernel/debug`, so the eBPF profiler manifest can be applied for testing. It still
cannot profile here (the node's PID namespace is a container's), so the local cluster
relies on the services' in-process profiler. Both mounts are fixed at cluster creation.

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

## Reaching it from the internet

The cluster's host ports are LAN-only. To publish it, put `devops/edge` in front: Caddy
on the host terminates TLS with Let's Encrypt certificates and forwards one subdomain to
each host port. The API is open; the four observability UIs sit behind one basic-auth
credential.

```
internet ─443─▶ FRITZ!Box (port forward) ─▶ host: Caddy ─┬─▶ api.<base>      Envoy :18080
                                                        ├─▶ grafana.<base>  :3000   (auth)
                                                        ├─▶ logs.<base>     :9428   (auth)
                                                        ├─▶ profiles.<base> :4040   (auth)
                                                        └─▶ metrics.<base>  :9090   (auth)
```

1. **Public names.** `api`, `grafana`, `logs`, `profiles` and `metrics` under the base
   domain. On the FRITZ!Box, *Internet → Permit Access → DynDNS* (or MyFRITZ!) keeps a
   hostname pointing at the box's public IPv4; `CNAME`s from your own domain to that
   hostname, or `A` records you update yourself, both work. This machine's box
   reports its external address over UPnP and it matches what the internet sees, so there is
   no carrier-grade NAT in the way; the box has no public IPv6, so the edge is IPv4 only.
2. **Port forwards.** *Internet → Permit Access → Port Sharing*, device = this machine:
   TCP 80 and TCP 443 (UDP 443 too for HTTP/3). 80 is needed only for the certificate
   challenge and redirects to 443. If 80/443 on the host are taken (they are on this
   machine: another stack's Envoy holds them), set `EDGE_HTTP_PORT`/`EDGE_HTTPS_PORT`
   and forward external 80 → that port and 443 → that port.
3. **Start the edge.** Copy `devops/edge/.env.example` to `devops/edge/.env`, fill in
   `BASE_DOMAIN` and `ACME_EMAIL`, paste the hash from `mise run edge:password`, then
   `mise run edge:up`. `mise run edge:logs` shows the certificates being issued.
4. **Verify from outside** (a phone off Wi-Fi, or any other network):

   ```sh
   chaos validate --protocol https://api.$BASE_DOMAIN --engine https://api.$BASE_DOMAIN
   ```

   All 11 checks pass through the edge, including gRPC streaming and WebSocket. If the
   name is proxied by Cloudflare, gRPC must be switched on in the zone's Network settings
   first; see `devops/edge/README.md`. Any
   `https://`/`wss://` target is verified against the public roots; a private CA (a
   staging edge, Caddy's `tls internal`) needs `--ca-cert root.crt`.

What stays private: the OTLP port and the engine load balancer. Grafana, VictoriaLogs,
Pyroscope and VictoriaMetrics are reachable on their subdomains only with the basic-auth
credential from `devops/edge/.env`; behind Cloudflare, put Cloudflare Access in front of
those four as well. Grafana still runs with `admin`/`admin` and anonymous Editor, so the
credential is what protects it; tighten Grafana before sharing it. The FRITZ!Box's
WireGuard VPN (*Internet → Permit Access → VPN (WireGuard)*) remains the way to reach
the raw host ports.

Before leaving the edge up for long: enable the host firewall (`ufw` is installed but
inactive) allowing 22, 80 and 443 from anywhere and the cluster's host ports only from
`192.168.178.0/24`. The router forwards only what is configured, so those ports are not
reachable from outside anyway; the firewall is the second layer.

## Pinning

- k3s image `rancher/k3s:v1.34.9-k3s1`. k3s 1.35 crash-looped its cloud-controller-manager
  on this host at startup; do not bump without checking `docker logs k3d-tbd-server-0`.
- Images are tagged `dev` and imported straight from the Docker daemon;
  `imagePullPolicy: IfNotPresent`. A rebuild needs `local:restart` to roll the pods,
  because the tag does not change. The import uses `-m direct`: k3d's default
  tools-node mode silently keeps an image that already exists under the same tag, so
  the pods would restart on the old binary.

## Troubleshooting

| Symptom | Fix |
|---|---|
| `local:up` says a port is in use | `ss -ltnp \| grep <port>`; stop the other process or change the map in both places and recreate |
| `kubectl` says the server cannot handle the request right after creation | wait 20 s; the API server is still bootstrapping |
| `k3d-tbd-server-0` keeps restarting | `docker logs k3d-tbd-server-0`; check the k3s pin above |
| a pod is Running but never Ready | `kubectl -n <ns> describe pod <name>` for probe failures; for `protocol`, the engine is unreachable through Envoy |
| `ImagePullBackOff` on engine or protocol | the image was not imported: `mise run local:build` |
| pods restarted but still run the old code | compare `docker exec k3d-tbd-agent-0 crictl images \| grep tbd-` with `docker image ls`; if they differ, the import was not `-m direct` |
| everything is up but Grafana is empty | `mise run local:traffic`, then wait 15 s for the scrape |
| Tempo pod crash-loops after a config edit | the config keys changed in Tempo 3; `kubectl -n observability logs tempo-0` names the field |
| `edge:up` never gets a certificate | `mise run edge:logs`: a challenge failure means port 80/443 is not forwarded to this machine or DNS still points elsewhere; `curl -sI http://$PUBLIC_DOMAIN` from a phone off Wi-Fi tells which |
| WebSocket returns 502 through the edge | the upgrade went over h2c; keep the `@grpc` matcher in `devops/edge/Caddyfile` the only route with the h2c transport |

## Teardown

```sh
mise run local:down                   # cluster gone, ~5 s
rm -rf /mnt/raid0/tbd-k3d/storage     # optional: the data too
```
