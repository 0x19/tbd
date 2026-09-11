# devops

Deployment for the two binaries, `engine` and `protocol`. Read `devops/README.md` first;
this file is the non-obvious part.

- `docker/Dockerfile` is one multi-stage build for every binary, selected with
  `--build-arg BIN=engine|protocol|chaos` and `--build-arg PORT=`. cargo-chef caches
  dependencies; the runtime image is distroless with no shell, so there are no
  in-container health commands. Health is the orchestrator's job: gRPC probe on the
  engine, `/healthz` and `/readyz` on the protocol, `/api/chaos/v1/healthz` on chaos.
- The runtime stage has `WORKDIR /app` and copies `configs/`, `scenarios/` and
  `topologies/` so chaos finds them at its default relative paths; `ui/chaos/out` is
  copied to `/app/ui` when it exists (glob on the first path segment, so the COPY is
  valid without it). `.dockerignore` is an allowlist; a new top-level directory the
  build needs must be added there.
- `k8s/chaos/` is its own kustomization (not in `base/`), pulled in by the `local` and
  `dev` overlays only. Envoy routes `/api/chaos/` (any version) and the `chaos.*`/`chaosadmin.*` hosts to the headless `chaos`
  Service; where the pod is absent Envoy answers 503. `TBD_ENV` in `tbd-env` selects
  `configs/chaos/<env>.toml` (`production` in base, overridden per overlay).
- Protos compile inside the image without `protoc` (protox). Do not add protoc to the
  builder.
- `k8s/` is kustomize: `base/` plus `overlays/dev` and `overlays/prod`. Image names are
  `ghcr.io/0x19/tbd-*`; `newTag` in the prod overlay is pinned per release. Probes and security context (non-root, read-only rootfs, no caps) are
  in `base/`.
- `ansible/` runs from its own directory (`ansible.cfg` is there); the mise tasks set
  `dir` accordingly. `bootstrap.yml` once per host, `deploy.yml` per release with
  `-e image_tag=...`; it refuses `REPLACE_ME`. Secrets go in `group_vars/vault.yml`
  (ansible-vault), never in `all.yml`.
- Env vars the services read: `ENGINE_LISTEN_ADDR`, `PROTOCOL_LISTEN_ADDR`,
  `PROTOCOL_<NAME>_URL` for every backend in the protocol's `[services]` registry
  (`ENGINE`, `HUMANS`, `LEDGER`, and one per scaffolded service), `RUST_LOG`,
  `LOG_FORMAT`, `TBD_ENV`; chaos adds `CHAOS_*`
  (see `docs/chaos/config.md`). A new one must be added to `k8s/base/configmap.yaml`
  (or `k8s/chaos/deployment.yaml`), `compose.yaml`, `.env.example` and the ansible
  compose template.

- `envoy/envoy.yaml` is the only routing config and is used verbatim in compose, ansible
  and k8s (ConfigMap via `envoy/kustomization.yaml`). Upstreams are DNS names `engine`,
  `protocol`, `otel-collector`; k8s provides them as headless Services plus an ExternalName.
  Validate with `mise run envoy:validate` before applying. Proto type URLs are picky: the
  OpenTelemetry tracer is `envoy.config.trace.v3.OpenTelemetryConfig`.
- `k8s/overlays/local` is the k3d cluster: host ports come from separate `*-lb` Services
  (`envoy-lb`, `grafana-lb`, ...) whose port numbers must match the `-p` mappings in
  `mise.toml` `local:up`. Never change the in-cluster `envoy` Service ports; the protocol's
  engine URL depends on `envoy:50051`.
- `k8s/observability` is Tempo 3 monolithic (no Kafka; no `traces_storage` key),
  VictoriaMetrics with a k8s-SD scrape config, VictoriaLogs, an OTel Collector agent
  DaemonSet (filelog + json_parser + k8sattributes) and gateway Deployment (OTLP → Tempo),
  Grafana provisioned from `devops/grafana` via `../../grafana` in the kustomization.
- k3s must be pinned (`rancher/k3s:v1.34.9-k3s1` in `mise.toml` and `ansible/playbooks/local.yml`);
  v1.35 crash-looped on this host. PVCs land under `/mnt/raid0/tbd-k3d/storage`.

CI builds both images on every push and pushes on `main`; `release.yml` pushes on `v*`
tags. See `docs/ci.md`.
- `k8s/auth/` is the identity stack (Ory Hydra + Kratos). Envoy is the only thing that
  verifies tokens; services never do. Its public host is `auth.<domain>`; the URL is the
  token issuer, so it is deploy-time config (`auth.env`), not a runtime default.
- `envoy/envoy.yaml` gates everything (docs/auth/README.md): bearer JWT on the API,
  browser login (oauth2 filter + ID-token cookie) on the UI hosts, nothing on `auth.*`
  and `chaos.*`. It contains one placeholder, `__AUTH_PUBLIC_URL__`, rendered by the
  Envoy pod's init container (from the `tbd-edge` ConfigMap, written by `local:edge-env`)
  and by the compose service. The OAuth2 secrets are file-based SDS from the `envoy-oauth`
  Secret (`mise run auth:envoy-secrets`). Every overlay must provide `tbd-edge` with
  `AUTH_PUBLIC_URL` and that Secret, or Envoy will not start.
- Env vars added for auth: `CHAOS_TOKEN`, `CHAOS_AUTH_TOKEN_URL`, `CHAOS_AUTH_CLIENT_ID`,
  `CHAOS_AUTH_CLIENT_SECRET` (chaos deployment, compose, ansible template).
- Scaffolded services (`tbd new service`) register their `<NAME>_LISTEN_ADDR` and
  `<NAME>_METRICS_ADDR` everywhere themselves; `CHAOS_LEDGER_URL` points chaos at the
  ledger (through Envoy's internal listener in the cluster).
- The ledger owns two databases, `k8s/ledger-db/` (Postgres with pgvector, ClickHouse),
  in the `tbd` namespace, applied by `mise run ledger:deploy` after `ledger:secrets`
  created the `ledger-db` Secret (passwords and both URLs; never in a file). The ledger
  Deployment reads `LEDGER_DATABASE_URL` and `LEDGER_CLICKHOUSE_URL` from that Secret,
  `LEDGER_STORE_KIND` from the ConfigMap; compose and the ansible template carry the
  same three (`LEDGER_POSTGRES_PASSWORD`/`LEDGER_CLICKHOUSE_PASSWORD` in `.env`, the
  vault on hosts). Databases are the ledger's, not services: they are dialled directly,
  the Envoy rule does not apply. Prod brings managed instances and passes their URLs to
  `ledger:secrets`. The image pins live in three places, `compose.yaml`,
  `k8s/ledger-db/` and `ansible/group_vars/all.yml`: bump all or none.
- `edge/` is the only thing that faces the internet from a home/office deployment. Caddy
  terminates TLS and forwards to Envoy's edge on the host port (18080 for the local
  cluster). gRPC is matched on `Content-Type: application/grpc*` and gets the h2c
  transport; everything else, WebSocket upgrades included, goes over HTTP/1.1, because
  Caddy cannot carry an upgrade over h2c. Observability UIs are never routed here.
