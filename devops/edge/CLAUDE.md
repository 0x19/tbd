# devops/edge

Public TLS edge for a cluster behind a home or office router. Read `README.md` first.

- One Caddyfile, env-driven (`BASE_DOMAIN`, `ACME_EMAIL`, `EDGE_HTTP_PORT`,
  `EDGE_HTTPS_PORT`). Subdomain names are fixed
  (`api`, `grafana`, `logs`, `profiles`, `metrics`, `chaosadmin`); only the base is
  configurable, so
  DNS, docs and dashboards can rely on them. Caddy owns certificate issuance and
  renewal; nothing here needs cert-manager or a TLS listener in Envoy.
- `chaosadmin.` is matched by Envoy's own `chaos.*` / `chaosadmin.*` virtual host, which
  serves the UI at the root; Caddy passes the Host header through unchanged and adds
  nothing but TLS and the credential.
- `mise run edge:up` runs `caddy reload` inside the container after `up -d`, because
  compose does not restart on a bind-mounted config change. The mount is the whole
  `devops/edge` directory at `/etc/caddy`: a single-file bind mount keeps the old inode
  after `sed -i` or an editor save, so Caddy would reload the previous file.
- The `(gated)` snippet is every non-API host: one `reverse_proxy` to Envoy. There is no
  auth in Caddy at all; Envoy's OAuth2 + JWT filters do it (docs/auth/README.md). Do not
  add basic auth back "as a second layer": it breaks the OAuth2 callback and hides the
  real gate.
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
