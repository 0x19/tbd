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

## Social sign-in

Kratos can sign people in through Google, GitHub, Apple or any OpenID Connect provider
(`selfservice.methods.oidc`). The buttons appear on the login and registration pages as
soon as a provider has credentials; without any, the method is off and the pages show
password and passkey only. The wiring is in `config/oidc.sh` (renders the provider list
at pod start from `auth-secrets`) and one claims mapper per provider
(`config/oidc.<provider>.jsonnet`, which decides what becomes an identity trait: for
Google only a verified e-mail is accepted).

To turn a provider on:

1. Create an OAuth client in the provider's console and register the callback
   `https://auth.<domain>/self-service/methods/oidc/callback/<provider>`
   (`google` or `github`). Google: APIs & Services → Credentials → OAuth client ID, type
   Web application; the OAuth consent screen must be configured first. GitHub: Settings →
   Developer settings → OAuth Apps.
2. `mise run auth:oidc google <client-id> <client-secret>`. The credentials go into the
   `auth-secrets` Secret and Kratos restarts; its init log says which providers are on.

A person who registered with a password and later signs in with Google using the same
verified e-mail gets the accounts linked through the settings page, not merged
silently. Apple requires a team id, key id and private key rather than a secret; add it
to `oidc.sh` the same way when the iOS app needs it (Apple requires Sign in with Apple
when any other social login is offered).

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

## The gates: what Envoy checks

Every check lives in `devops/envoy/envoy.yaml`; nothing behind Envoy checks anything.

| Host | Filter | Accepts | On failure |
|---|---|---|---|
| `api.<domain>`, `localhost:18080`, `chaos.api.*` | `jwt_authn`, requirement `api` | `Authorization: Bearer <JWT>` signed by Hydra with audience `tbd-api` | 401 with `Jwt is missing` / `Jwt verification fails` |
| same hosts, `/healthz`, `/readyz`, `/api/chaos/v1/healthz` | none | anything | probes and uptime checks stay unauthenticated |
| `grafana.`, `logs.`, `profiles.`, `metrics.`, `chaosadmin.` | `oauth2` then `jwt_authn`, requirement `ui` | the ID-token cookie the OAuth2 filter set (audience `tbd-ui`), or a bearer token | redirect to `auth.<domain>` to sign in |
| `auth.<domain>`, `chaos.localhost` | none | anything | the sign-in itself, and the open local UI |

**API calls.** The JWT filter fetches Hydra's JWKS through the internal `hydra` cluster
(cached ten minutes, refreshed in the background), checks signature, expiry and audience,
and forwards the verified claims to the protocol as base64url JSON in `x-jwt-payload`.
It strips that header, `x-user-sub`, `x-user-email` and `x-user-name` from whatever a
client sent, so those headers mean "verified by Envoy" and nothing else. The issuer is not
pinned in Envoy, because the config is one file for every environment; the signing key
is, which binds tokens to this Hydra.

**Browser sessions.** On the five UI hosts Envoy's OAuth2 filter runs the authorization
code flow as client `tbd-ui`: no cookie means a redirect to
`https://auth.<domain>/oauth2/auth`, the person signs in (Kratos), Hydra redirects back
to `https://<host>/oauth2/callback`, Envoy exchanges the code with Hydra directly and
sets the cookies `tbd_id` (ID token), `tbd_access`, `tbd_refresh`, `tbd_hmac` and
`tbd_expires`. The JWT filter then verifies `tbd_id` on every request and maps its
claims to `x-user-sub`, `x-user-email`, `x-user-name`. Access tokens refresh on their own
(`use_refresh_token`); `/oauth2/signout` clears the cookies. Requests that already carry
a bearer token skip the browser flow, so scripts can hit `chaosadmin.<domain>/api/chaos/v1`
with a machine token.

**Grafana** runs with its auth proxy on: Envoy adds `X-WEBAUTH-USER` (email),
`X-WEBAUTH-NAME` and `X-WEBAUTH-ROLE: Editor` from the verified claims, Grafana creates
the user on first sight and signs it in. Anonymous access is off. The admin form is still
reachable on the LAN port 3000 for break-glass; the proxy headers are only accepted from
the pod network (`GF_AUTH_PROXY_WHITELIST`).

**What the protocol does with it.** `crates/protocol/src/subject.rs` reads `sub` from
`x-jwt-payload` into a request extension and the span (`enduser.id`). `GET /v1/me`
returns it, or 401 when Envoy forwarded no identity. Handlers that need the caller take
`Subject` as an extractor. The protocol never verifies a token itself: with services
reachable only through Envoy, that would be a second implementation of the same check.

## How `chaos` gets in

`validate`, ad-hoc load and API-driven load runs against a deployed stack carry a token
on HTTP, WebSocket (handshake header) and gRPC (metadata). The source is `[auth]` in
`configs/chaos/*.toml` or the flags: a fixed `--token` / `CHAOS_TOKEN`, or the
client-credentials grant with `CHAOS_AUTH_TOKEN_URL`, `CHAOS_AUTH_CLIENT_ID` and
`CHAOS_AUTH_CLIENT_SECRET` (the `tbd-chaos` client). Tokens are fetched once per run and
refreshed a minute before they expire. In the cluster the chaos pod has these from the
`chaos-auth` Secret (`mise run auth:envoy-secrets`); in-process stacks (`chaos up`,
scenarios) have no Envoy and send nothing.

```sh
# from anywhere, against the public API
export CHAOS_AUTH_TOKEN_URL=https://auth.<domain>/oauth2/token
export CHAOS_AUTH_CLIENT_SECRET=$(kubectl -n auth get secret auth-secrets -o jsonpath='{.data.client-chaos-secret}' | base64 -d)
chaos validate --protocol https://api.<domain> --engine https://api.<domain>
# or with a token you already have
chaos validate --protocol https://api.<domain> --engine https://api.<domain> --token "$(mise run auth:token | jq -r .access_token)"
```

A missing or rejected token shows as one failed `auth_token` check before any other
check runs.

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
- Envoy's OAuth2 filter reads the `tbd-ui` client secret and its cookie HMAC key from
  the `envoy-oauth` Secret in the app namespace as file-based SDS resources;
  `mise run auth:envoy-secrets` derives it from `auth-secrets` (and `chaos-auth` for the
  chaos pod). The browser-facing authorization URL is the one environment-specific value
  in `envoy.yaml` (`__AUTH_PUBLIC_URL__`), rendered by the Envoy pod's init container
  from the `tbd-edge` ConfigMap and by the compose service from `AUTH_PUBLIC_URL`.
- `docker compose up` runs the same identity stack (Postgres, Hydra, Kratos, UI, seeded
  clients) with local-only default secrets, so the compose Envoy gates the same way.
- Changing the domain means redeploying the stack (issuer in every token) and re-seeding
  the clients (redirect URIs); `auth:deploy` does both.
