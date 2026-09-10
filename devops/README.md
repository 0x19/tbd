# devops

| Directory | What | Entry point |
|---|---|---|
| `docker/` | one multi-stage Dockerfile for every workspace binary | `docker build -f devops/docker/Dockerfile --build-arg BIN=engine -t ghcr.io/ORG/tbd-engine .` |
| `k8s/` | kustomize base plus `dev` and `prod` overlays | `kubectl apply -k devops/k8s/overlays/dev` |
| `ansible/` | host bootstrap and compose-based deploy | see `ansible/README.md` |

Root-level `compose.yaml` runs the same images locally: `docker compose up --build`.

## Deploy paths

- **Local**: `docker compose up --build`. Ports 50051 (engine gRPC) and 8080 (protocol HTTP).
- **Single server**: `ansible/playbooks/bootstrap.yml` once, then `deploy.yml` with an explicit `image_tag`.
- **Cluster**: pin `newTag` in `k8s/overlays/prod/kustomization.yaml`, then `kubectl apply -k`.
  Preview with `kubectl kustomize devops/k8s/overlays/prod`.

CI (`.github/workflows/ci.yml`) builds both images on every push and pushes them to GHCR
on `main` tagged with the short SHA and `main`. `release.yml` pushes semver tags on `v*`.

## Placeholders a human must fill in

- `ORG`: image namespace. In CI it defaults to the repository owner (override with a repo
  variable `ORG`). Set it in `k8s/base/*/deployment.yaml`, `k8s/overlays/*/kustomization.yaml`
  `images:` and `ansible/group_vars/all.yml` (`image_org`).
- Hosts in `ansible/inventory/*.yml`.
- Registry credentials in `ansible/group_vars/vault.yml` (ansible-vault encrypted).
- Immutable `newTag` values in `k8s/overlays/prod`.

Runtime images are distroless with no shell. Health is checked by orchestrator probes
(gRPC probe on engine, HTTP `/healthz` and `/readyz` on protocol), not by in-container commands.
