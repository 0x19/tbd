---
paths:
  - "devops/**"
  - "compose.yaml"
  - ".github/workflows/**"
---
# devops rules

- Before saying a devops change is done, run what applies and show the output:
  `python3 -c 'import yaml,glob; [list(yaml.safe_load_all(open(f))) for f in glob.glob("devops/**/*.yaml", recursive=True)]'`,
  `kustomize build devops/k8s/observability > /dev/null`, `kustomize build devops/k8s/overlays/local > /dev/null`,
  `mise run envoy:validate`, `docker compose config -q`.
- One config per concern, used everywhere: `devops/envoy/envoy.yaml` for routing,
  `devops/k8s/base/configmap.yaml` for service env. Environment differences go in
  overlays, DNS names or env vars, never in copies of a file.
- Every new service env var goes into `.env.example`, `compose.yaml`,
  `devops/k8s/base/configmap.yaml` and `devops/ansible/roles/tbd_app/templates/compose.yaml.j2`.
- Services never address each other directly; they go through Envoy. Do not point a
  service at another service's DNS name.
- Pins are deliberate: k3s `v1.34.9-k3s1`, Envoy `v1.39.1`, the observability images.
  Bump one at a time with the reason in the commit.
- Host port map for the local cluster lives in `mise.toml` `local:up`,
  `devops/ansible/playbooks/local.yml`, `devops/k8s/overlays/local/envoy-lb.yaml` and
  `devops/k8s/observability/local-services.yaml`; change all or none.
- Update `docs/local-cluster.md`, `docs/observability/*.md` or `devops/*/README.md` in the
  same commit as the change they describe.
