# devops/edge

Public TLS edge for a cluster behind a home or office router. Read `README.md` first.

- One Caddyfile, env-driven (`BASE_DOMAIN`, `ACME_EMAIL`, `SITE_DOMAIN`, `EDGE_HTTP_PORT`,
  `EDGE_HTTPS_PORT`). Subdomain names are fixed
  (`api`, `grafana`, `logs`, `profiles`, `metrics`, `chaosadmin`, `finance`, `auth`),
  and the base domain itself serves the company site;
  only the base is configurable, so
  DNS, docs and dashboards can rely on them. Caddy owns certificate issuance and
  renewal; nothing here needs cert-manager or a TLS listener in Envoy.
- `chaosadmin.` is matched by Envoy's own `chaos.*` / `chaosadmin.*` virtual host, which
  serves the UI at the root; Caddy passes the Host header through unchanged and adds
  nothing but TLS and the credential.
- `mise run edge:up` runs `caddy reload` inside the container after `up -d`, because
  compose does not restart on a bind-mounted config change. The mount is the whole
  `devops/edge` directory at `/etc/caddy`: a single-file bind mount keeps the old inode
  after `sed -i` or an editor save, so Caddy would reload the previous file.
- `finance.` is Envoy's `finance.*` virtual host. Two routes there are deliberately open
  (`typed_per_filter_config` disabling oauth2, jwt_authn and rbac): the bank's OAuth
  callback at `/connect/callback`, which a browser reaches with no session of ours yet,
  and the published policy pages on `chaosadmin.` (`/privacy.html`, `/terms.html`), which
  exist to be read by people who are not signed in. Opening a route means all three
  filters, not one: oauth2 alone would still redirect the browser to the login page.
- The apex (`{$BASE_DOMAIN}`) is the company site (`ui/www`). It is the one host whose
  `Host` header the edge rewrites: to `www.{$BASE_DOMAIN}`, because Envoy matches the site
  on `www.*` and `envoy.yaml` must not learn the domain. `www.` redirects to the apex, so
  there is a single canonical URL and no duplicate content.
- `SITE_DOMAIN` (compose defaults it to `BASE_DOMAIN`) is the domain the site is canonical
  on. When it differs from the base, the base's apex and `www.` redirect there with 301
  (`@elsewhere not host {$SITE_DOMAIN}`), and `mise run local:build` bakes the same domain
  into the site image (`www:build-args`), so the canonical URL, the sitemap and the
  robots rule all name the domain people actually reach. A site build that named the base
  domain as canonical while the base served a copy told search engines the original lived
  there, and the build's own `indexable` rule asked them to stay out of both.
- `sites.d/*.caddy` (git-ignored, one file per domain) serves the company site under
  a domain of its own: each file is `www.<domain>` redirecting to `<domain>`, which
  `import site`s, and may add `api.<domain>` importing `api`, the API host's snippet
  (Envoy's API virtual host matches any name). A glob that matches nothing is not an
  error, so the file is simply absent where the site has one domain. Behind Cloudflare's proxy the zone must be in
  *Full (strict)* SSL mode: in *Flexible* mode Cloudflare fetches the origin over
  plain HTTP, Caddy answers with its redirect to HTTPS, and the browser sees a loop.
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
