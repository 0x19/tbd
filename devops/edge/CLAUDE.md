# devops/edge

Public TLS edge for a cluster behind a home or office router. Read `README.md` first.

- One Caddyfile, env-driven (`BASE_DOMAIN`, `ACME_EMAIL`, `OBS_USER`,
  `OBS_PASSWORD_HASH`, `EDGE_HTTP_PORT`, `EDGE_HTTPS_PORT`). Subdomain names are fixed
  (`api`, `grafana`, `logs`, `profiles`, `metrics`); only the base is configurable, so
  DNS, docs and dashboards can rely on them. Caddy owns certificate issuance and
  renewal; nothing here needs cert-manager or a TLS listener in Envoy.
- `OBS_PASSWORD_HASH` in `.env` must have every `$` doubled: compose interpolates
  `$name` in env files, so a raw bcrypt hash arrives truncated and every login is 401.
  `mise run edge:password` prints it escaped; `docker inspect` on the container shows
  what Caddy actually got (60 characters, starting `$2a$14$`).
- The `(observability)` snippet carries the basic auth for every non-API host. A new
  UI host imports it; never add a host that bypasses it. `api.` has no auth on purpose:
  that is the product's job.
- Grafana runs behind the edge with its own security unchanged (anonymous Editor,
  `admin`/`admin` locally). The basic auth is the only thing between the internet and
  that; tighten Grafana before sharing the credential widely.
- Two `reverse_proxy` blocks on purpose. `@grpc` (matched on
  `Content-Type: application/grpc*`) uses the h2c transport so the h2 stream survives
  end to end. The other block is HTTP/1.1: WebSocket upgrades fail with 502 over Caddy's
  h2c transport, and SSE streams fine over 1.1 with `flush_interval -1`. Do not merge them.
- `cluster` resolves to the host (`extra_hosts: host-gateway`), so the same file works
  for the k3d cluster (Envoy on 18080, UIs on 3000/9428/4040/9090) and compose (change
  the ports).
- Host ports 80/443 are parameters because other stacks on a dev box may hold them; the
  router forward then targets the alternative port. The certificate challenge still
  needs external 80 and 443 to land on Caddy.
- Nothing from `observability/` is routed here. Adding a route for Grafana or the
  VictoriaLogs UI requires real authentication first; local Grafana is `admin`/`admin`
  with anonymous Editor.
- Test without DNS: run with `local_certs` in the global block, then `curl -k --resolve
  <host>:<port>:127.0.0.1`; for `chaos validate`, copy
  `/data/caddy/pki/authorities/local/root.crt` out of the container and pass `--ca-cert`.
