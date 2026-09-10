# devops/edge

The public TLS entry point for a cluster running on a machine behind a home or office
router. Caddy on the host terminates TLS with automatic Let's Encrypt certificates and
forwards each subdomain to one of the cluster's host ports. Nothing else is exposed.

```
internet ──443──▶ router (port forward) ──▶ this host: caddy ──┬─▶ envoy :18080        api.<base>
                                                               ├─▶ grafana :3000       grafana.<base>   basic auth
                                                               ├─▶ victoria-logs :9428 logs.<base>      basic auth
                                                               ├─▶ pyroscope :4040     profiles.<base>  basic auth
                                                               └─▶ victoria-metrics :9090  metrics.<base>  basic auth
```

| Host | Upstream | Auth | Notes |
|---|---|---|---|
| `api.<base>` | Envoy edge 18080 | none (the API's own, later) | REST, SSE, GraphQL, WebSocket, gRPC |
| `grafana.<base>` | Grafana 3000 | HTTP basic | Grafana itself still has `admin`/`admin` + anonymous Editor locally |
| `logs.<base>` | VictoriaLogs 9428 | HTTP basic | `/` redirects to `/select/vmui/` |
| `profiles.<base>` | Pyroscope 4040 | HTTP basic | |
| `metrics.<base>` | VictoriaMetrics 9090 | HTTP basic | `/` redirects to `/vmui/` |
| `chaosadmin.<base>` | Envoy 18080, `chaosadmin.*` virtual host | HTTP basic | the chaos admin UI at the root, API at `/api/chaos/v1/` |
| `auth.<base>` | Envoy 18080, `auth.*` virtual host | none (it is the sign-in) | Ory Hydra, Ory Kratos and the login/registration/consent pages |

## Setup

1. DNS: six records (`api`, `grafana`, `logs`, `profiles`, `metrics`, `chaosadmin`)
   under the base domain pointing at the public IP, or `CNAME`s to the router's DynDNS
   name. Create the record before starting Caddy for it: every failed certificate
   attempt counts toward Let's Encrypt's five failed authorizations per hostname per
   hour, and a name added later waits that hour out.
2. On the router, forward TCP 80 and TCP 443 (and UDP 443 for HTTP/3) to this machine.
   80 is needed for the certificate challenge and redirects to 443.
3. `cp devops/edge/.env.example devops/edge/.env`, fill in `BASE_DOMAIN` and
   `ACME_EMAIL`, run `mise run edge:password` and paste the hash line as printed (its
   `$` are doubled because compose expands variables inside `.env`).
4. `mise run edge:up` (also after any Caddyfile change: it reloads the running Caddy
   gracefully). The first start requests one certificate per host;
   `mise run edge:logs` shows them being issued. A host whose DNS does not resolve yet
   keeps retrying in the background without affecting the others.
5. `chaos validate --protocol https://api.<base> --engine https://api.<base>` from
   anywhere; the observability hosts prompt for the basic-auth credential.

## Behind Cloudflare

When the names are proxied by Cloudflare (orange cloud), Cloudflare connects to this edge
on 80/443 through the router forward; the certificate challenge (HTTP-01) still works
through the proxy. Three zone settings matter:

- **Network → gRPC: on.** Off, Cloudflare answers every `application/grpc` request with
  its own HTML `403 Forbidden` and the request never reaches Caddy; tonic reports it as
  `invalid compression flag: 60` (the `<` of the HTML).
- **SSL/TLS → Full (strict)** once the Let's Encrypt certificates are on the origin, so
  Cloudflare verifies them rather than accepting anything.
- WebSocket and SSE need nothing extra. HTTP/3 to the origin is not used by Cloudflare;
  clients get HTTP/3 from Cloudflare's edge.

Access logs then show Cloudflare's addresses as `remote_ip`; the client is in
`Cf-Connecting-Ip`. Trace and metric labels are unaffected.

For the observability hosts, Cloudflare Access (Zero Trust → Applications, free for a
handful of users) in front of `grafana`, `logs`, `profiles` and `metrics` is the better
lock: identity-based login, and the basic-auth credential becomes the second layer.

## What the basic auth does and does not cover

It stops the internet from reaching Grafana, VictoriaLogs, Pyroscope and VictoriaMetrics
without the credential. It does not change what those tools allow once inside: local
Grafana is configured with anonymous Editor and `admin`/`admin`. For a shared or long-lived
deployment, set a real Grafana admin password and turn anonymous access off in
`devops/k8s/observability/grafana.yaml` before handing out the credential.

## Notes

- gRPC works through Caddy: clients speak h2 over TLS, Caddy speaks h2c to Envoy.
  Only the `@grpc` matcher uses the h2c transport; a WebSocket upgrade cannot cross it.
- Envoy still sees the client address in `X-Forwarded-For`, and `X-Forwarded-Proto` is
  set, so access logs and traces stay accurate.
- The host firewall can allow 22, 80 and 443 from anywhere and everything else from the
  LAN only; the router only forwards what is configured, so the cluster's other host
  ports are not reachable from outside regardless.
- Test without DNS: run Caddy with `local_certs` in the global block, then
  `curl -k --resolve grafana.<base>:443:127.0.0.1 https://grafana.<base>/`.
