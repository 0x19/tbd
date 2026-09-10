# Identity and access

Everything a person or a program does against this system is authenticated by one
OAuth2 / OpenID Connect provider, and Envoy is the only place that checks it. Nothing
behind Envoy has its own login, and no service trusts a caller it cannot see a token for.

| Piece | What it is | Where |
|---|---|---|
| **Ory Hydra** | the OAuth2 / OIDC provider: issues JWT access tokens, ID tokens, refresh tokens; OAuth 2.1 discovery | `devops/k8s/auth/hydra.yaml`, config in `config/hydra.yml` |
| **Ory Kratos** | identities and credentials: registration, login, passkeys, TOTP, sessions; acts as Hydra's login provider | `devops/k8s/auth/kratos.yaml`, `config/kratos.yml`, `config/identity.schema.json` |
| **self-service UI** | Ory's reference pages for login, registration, settings and the consent step | `devops/k8s/auth/ui.yaml` |
| **Postgres** | one instance, databases `hydra` and `kratos` | `devops/k8s/auth/postgres.yaml` |
| **Envoy** | routes `auth.<domain>` to the three above; verifies tokens on every other host (phase 2) | `devops/envoy/envoy.yaml` |

All of it is open-source Ory, pinned to the `v26.2.0` line (Ory moved Hydra and Kratos
to shared calendar versions in 2025).

## One host for signing in

`auth.<domain>` is the only public entry point of the identity stack. Envoy splits it by
path:

| Path | Service | Purpose |
|---|---|---|
| `/oauth2/auth`, `/oauth2/token`, `/oauth2/sessions/logout`, `/userinfo` | Hydra | the OAuth2 endpoints |
| `/.well-known/openid-configuration`, `/.well-known/jwks.json`, `/.well-known/oauth-authorization-server` | Hydra | discovery and the signing keys Envoy verifies against |
| `/self-service/...`, `/sessions/whoami`, `/schemas/...`, `/.well-known/ory/...` | Kratos | the flows behind the pages, and the session API |
| everything else: `/login`, `/registration`, `/settings`, `/consent`, `/welcome`, `/error` | UI | the pages |

Locally the same host is `http://auth.localhost:18080/` (browsers resolve `*.localhost`
to loopback); with the public edge it is `https://auth.<domain>/`. The URL is baked into
Hydra's issuer and Kratos' return URLs at deploy time from `devops/k8s/auth/auth.env`,
which `mise run local:edge-env` writes from `devops/edge/.env`. **Tokens carry that
issuer**, so the URL a client uses must be the one the stack was deployed with.

## Clients

`seed-clients.yaml` registers three OAuth2 clients on every deploy (create or update):

| Client id | Who | Grant | Scope / audience | Notes |
|---|---|---|---|---|
| `tbd-ui` | Envoy's OAuth2 filter on the browser hosts (grafana, logs, profiles, metrics, chaosadmin) | authorization code + refresh | `openid offline_access email profile`, audience `tbd-ui` | confidential, `client_secret_post`; no consent screen (first party) |
| `tbd-chaos` | the chaos tool, CI, scripts | client credentials | `tbd.api`, audience `tbd-api` | confidential, `client_secret_basic` |
| `tbd-app` | the future first-party app | authorization code with PKCE + refresh | `openid offline_access email profile tbd.api`, audience `tbd-api` | public client, `tbd://callback` and `localhost:3001`; no consent screen |

Secrets live only in the `auth-secrets` Kubernetes Secret, created once by
`mise run auth:secrets` with random values, never written to disk or git. Read one back
with `kubectl -n auth get secret auth-secrets -o jsonpath='{.data.client-chaos-secret}' | base64 -d`.

## Flows

**A person signs up or in.** The browser opens `https://auth.<domain>/registration` (or
`/login`). The page drives a Kratos self-service flow: password (10+ characters), or a
passkey, optionally TOTP as a second factor. Kratos sets its session cookie on
`auth.<domain>`.

**An app gets tokens for that person** (authorization code): the client sends the browser
to `/oauth2/auth?client_id=...&response_type=code&scope=openid...&redirect_uri=...&state=...`.
Hydra redirects to `/login?login_challenge=...`; Kratos recognises the challenge, logs the
person in (or reuses the session) and hands the challenge back to Hydra; the consent step
is skipped for first-party clients; Hydra redirects to the client's `redirect_uri` with a
`code`; the client exchanges it at `/oauth2/token` for an access token (JWT), an ID token
and a refresh token.

**A machine gets a token** (client credentials):

```sh
mise run auth:token          # the chaos client, against the deployed stack
# or by hand:
curl -u tbd-chaos:$SECRET -d grant_type=client_credentials -d scope=tbd.api -d audience=tbd-api \
  https://auth.<domain>/oauth2/token
```

The access token is a JWT signed by Hydra. Its `iss` is the issuer above, `aud` is what
the client asked for (`tbd-api`), `scp` the granted scopes, `sub` the client id (or the
person's identity id for user tokens), `exp` one hour out.

## What the services see (phase 2 and 3)

Envoy verifies every bearer token on `api.<domain>` against Hydra's JWKS (cached), rejects
missing or invalid ones except on `/healthz` and `/readyz`, and forwards the verified
claims to the protocol in a header. The protocol reads the subject from that header and
nothing else; it never validates tokens itself and never trusts the header from any
address but Envoy's. Browser hosts get Envoy's OAuth2 filter (login redirect, cookies,
refresh) with the same verification, replacing the edge's basic auth. `chaos` obtains a
client-credentials token and attaches it to HTTP, WebSocket and gRPC.

## Operating it

```sh
mise run auth:secrets     # once per cluster; refuses to overwrite
mise run auth:deploy      # apply + wait; part of local:deploy
mise run auth:token       # a token for the chaos client
kubectl -n auth logs deploy/hydra
kubectl -n auth logs deploy/kratos
```

- Hydra and Kratos render their config at pod start (`__AUTH_PUBLIC_URL__`,
  `__BASE_DOMAIN__` substituted from the `auth-env` ConfigMap) and run their SQL
  migrations in an init container, so a config change is `mise run auth:deploy`.
- Recovery and verification e-mails are off: there is no SMTP relay yet. Turn them on in
  `config/kratos.yml` once `courier.smtp.connection_uri` points at a real relay.
- Both emit traces to the collector and Prometheus metrics on their admin ports, so they
  appear in Grafana and Tempo like every other service.
- Changing the domain means redeploying the stack (issuer in every token) and re-seeding
  the clients (redirect URIs); `auth:deploy` does both.
