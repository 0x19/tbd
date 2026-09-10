# devops/k8s/auth

The identity stack: Postgres, Ory Hydra, Ory Kratos, the reference self-service UI, and a
Job that seeds the OAuth2 clients. Read `docs/auth/README.md` first.

- Applied on its own (`mise run auth:deploy`), not from an overlay, like `observability/`.
  It needs `auth.env` (git-ignored, written by `mise run local:edge-env`) and the
  `auth-secrets` Secret (`mise run auth:secrets`). Never commit either; never put a real
  secret in a manifest.
- Config files are templates: `__AUTH_PUBLIC_URL__` and `__BASE_DOMAIN__` are replaced by
  a `sed` init container. Ory config keys can also be overridden by environment variables
  (`URLS_SELF_ISSUER`, `SERVE_PUBLIC_BASE_URL`), but arrays and nested URLs are awkward
  that way; keep the template approach.
- The issuer is the public URL. Every token carries it and Envoy checks it, so a client
  hitting the stack through a different host name gets tokens it cannot use.
- Kratos is Hydra's login provider through `oauth2_provider.url` (Hydra's admin API). The
  UI's `/consent` route accepts first-party clients (`TRUSTED_CLIENT_IDS`) without a
  screen; Hydra's `skip_consent` on the client makes it skip the screen too.
- `seed-clients.yaml` is a Job: `auth:deploy` deletes the finished one first because a
  completed Job's spec is immutable. Client ids are fixed (`tbd-ui`, `tbd-chaos`,
  `tbd-app`); secrets come from `auth-secrets`. Add a client there, not by hand.
- Postgres is a single StatefulSet with a 20Gi PVC; both databases are created by the
  init SQL on first start only. Deleting the PVC deletes every identity.
- The Envoy short names `hydra`, `kratos`, `auth-ui` are ExternalName Services in
  `base/envoy/service.yaml`, so the app namespace's Envoy can reach the `auth` namespace
  the same way it reaches `otel-collector`.
- `ui.yaml` is our own sign-in UI (`ui/auth`, image `tbd-auth-ui`), not Ory's reference
  container. It needs `AUTH_PUBLIC_URL` (server-rendered consent) and the in-cluster
  Kratos/Hydra URLs; `TRUSTED_CLIENT_IDS` decides which clients skip the consent screen.
- Roles are `metadata_admin.role`; `mise run auth:role` patches them through the Kratos
  admin API from a one-off curl pod. Tokens carry the role from the consent step
  (`ui/auth/lib/server.ts`), so a change shows at the next sign-in.
- E-mail flows are rendered by `config/flows.sh` into `flows.yml` from the optional
  `smtp-uri` / `smtp-from` keys of `auth-secrets` (`mise run auth:smtp`); without a
  relay recovery and verification are off and Kratos never sends mail.
- Social sign-in is rendered by `config/oidc.sh` into a second config file (`-c oidc.yml`)
  so the provider list is data, never a hand-edited YAML with secrets in it. Credentials
  are optional keys on `auth-secrets` (`oidc-<provider>-id/secret`); `mise run auth:oidc`
  sets them. A new provider needs a mapper jsonnet, a block in `oidc.sh`, and the
  callback `https://auth.<domain>/self-service/methods/oidc/callback/<id>` registered
  with the provider.
- Cookies: `AUTH_INSECURE_COOKIES=true` (local, plain http) relaxes the UI's CSRF cookie;
  Hydra and Kratos trust `X-Forwarded-Proto` from the pod network
  (`serve.tls.allow_termination_from`) so they emit secure cookies behind the edge.
