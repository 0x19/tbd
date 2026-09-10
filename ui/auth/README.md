# ui/auth

The sign-in UI for `auth.<domain>`: login, registration, account settings, recovery,
verification, the OAuth2 consent step and global sign-out. Next.js on
[Ory Elements](https://www.ory.com/docs/elements) (`@ory/elements-react`) for the flow
logic, rendered with the same shadcn kit primitives, theme tokens and fonts as `ui/chaos`
so it looks like the rest of the product. The design, flows and gates:
`docs/auth/README.md`.

```sh
mise run ui:auth:check    # prettier, eslint, tsc (in `mise run ci`)
mise run ui:auth:build    # the image (devops/docker/Dockerfile.auth-ui)
mise run ui:auth:dev      # next dev on :3002 (see below)
mise run auth:e2e         # the browser check against a running cluster
```

## How it is put together

```
src/
├── app/                      one folder per page, all client components except consent
│   ├── layout.tsx            fonts (Inter, Geist Mono), theme provider
│   ├── login/ registration/ recovery/ verification/ settings/   useFlow + an Elements component
│   ├── consent/page.tsx      server component: auto-accept first-party clients, else the Consent card
│   ├── logout/page.tsx       global sign-out
│   ├── error/page.tsx        Kratos (?id=) and Hydra (?error=) errors
│   ├── page.tsx              signed-in home
│   └── api/{consent,logout,health}/route.ts
├── components/
│   ├── ui/                   the kit's primitives, copied verbatim from ui/chaos; regenerate, never hand-edit
│   ├── ory/components.tsx    Ory Elements overrides rendered with those primitives
│   ├── auth-shell.tsx        brand row, theme switch, centred card
│   ├── flow.tsx              useFlow(kind): start or fetch a Kratos flow for the page
│   ├── logo.tsx theme-provider.tsx theme-switch.tsx   from ui/chaos
├── lib/
│   ├── ory-config.ts         Ory Elements configuration (SDK URL = the page's origin)
│   ├── client.ts             browser Kratos client (same origin)
│   └── server.ts             Hydra admin, Kratos admin/public clients; the claims each token gets
└── data/site.ts              name and logo
```

- Ory Elements decides which nodes a flow shows, submits forms and follows redirects; the
  overrides only decide how a card, form group, input, button, label or message renders.
  Input and button names/values are Kratos' own, which `e2e/auth.mjs` and scripts rely on.
- Everything Kratos and Hydra is same-origin: Envoy serves `/self-service`, `/sessions`,
  `/oauth2`, `/.well-known` on `auth.<domain>` and sends the rest here.
- `consent/page.tsx` accepts first-party clients (`TRUSTED_CLIENT_IDS`) and remembered
  consents at once, stamping the person's claims (email, name, role, grafana_role; see
  `lib/server.ts`). Other clients get the consent card, posting to `api/consent`.
- `logout`: the server revokes every Hydra session and consent for the person and answers
  Hydra's logout challenge; the page ends the Kratos session and returns to `/login`.
- Roles come from the identity's `metadata_admin.role` (`mise run auth:role`), never from
  traits the person can edit.

## Runtime

A standalone Next.js server (`output: "standalone"`) on distroless Node, port 3000.
Environment: `KRATOS_PUBLIC_URL`, `KRATOS_ADMIN_URL`, `HYDRA_ADMIN_URL` (in-cluster),
`AUTH_PUBLIC_URL` (the origin, for the server-rendered consent page),
`TRUSTED_CLIENT_IDS`. Build-time: `NEXT_PUBLIC_RECOVERY_ENABLED`,
`NEXT_PUBLIC_VERIFICATION_ENABLED` (set by the image build once an SMTP relay exists).

## Developing

The pages need Kratos on the same origin, so `next dev` alone is not enough. Use the
cluster (`https://auth.<domain>/` or `http://auth.localhost:18080/`) for real flows, or
iterate with the built image (`mise run ui:auth:build`, then `mise run auth:deploy`).
