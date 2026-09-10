# devops

| Directory | What | Entry point |
|---|---|---|
| `docker/` | one multi-stage Dockerfile for every workspace binary | `docker build -f devops/docker/Dockerfile --build-arg BIN=engine -t ghcr.io/0x19/tbd-engine .` |
| `envoy/` | the load balancer config, one file for every environment | `mise run envoy:validate`; see `envoy/README.md` |
| `k8s/` | kustomize base (engine, protocol, envoy) plus `local`, `dev` and `prod` overlays, and the `observability/` stack | `kubectl apply -k devops/k8s/overlays/<env>` |
| `grafana/` | datasources and dashboards, provisioned into the cluster as ConfigMaps | `mise run grafana:reload` |
| `ansible/` | host bootstrap, compose-based deploy, and `local.yml` for the local cluster | see `ansible/README.md` |

Traffic always enters through Envoy: the edge on 8080 (REST, SSE, GraphQL, WebSocket, gRPC)
and the engine load balancer on 50051 that protocol instances use. Services never address
each other directly. In Kubernetes the `engine` and `protocol` Services are headless so Envoy
balances across pods with its own health checks.

Root-level `compose.yaml` runs the same images locally: `docker compose up --build`.

## Deploy paths

- **Local cluster** (the one with Grafana): `mise run local:up && mise run local:build && mise run local:deploy`, or `mise run ansible:local`. Envoy on 18080/15051, Grafana on 3000. See `docs/observability.md`.
- **Compose**: `docker compose up --build`. Envoy on 8080 (edge) and 50051 (engine LB), admin on 9901.
- **Single server**: `ansible/playbooks/bootstrap.yml` once, then `deploy.yml` with an explicit `image_tag`.
- **Cluster**: pin `newTag` in `k8s/overlays/prod/kustomization.yaml`, then `kubectl apply -k`.
  Preview with `kubectl kustomize devops/k8s/overlays/prod`.

CI (`.github/workflows/ci.yml`) builds both images on every push and pushes them to GHCR
on `main` tagged with the short SHA and `main`. `release.yml` pushes semver tags on `v*`.

## Placeholders a human must fill in

- Image namespace is `ghcr.io/0x19`. CI derives it from the repository owner; a repo
  variable `ORG` overrides it.
- Hosts in `ansible/inventory/*.yml`.
- Registry credentials in `ansible/group_vars/vault.yml` (ansible-vault encrypted).
- Immutable `newTag` values in `k8s/overlays/prod`.

Runtime images are distroless with no shell. Health is checked by orchestrator probes
(gRPC probe on engine, HTTP `/healthz` and `/readyz` on protocol), not by in-container commands.
