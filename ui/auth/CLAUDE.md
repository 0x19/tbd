# ui/auth

The sign-in UI: Ory Elements flows rendered with the shadcn kit from `ui/chaos`. Read
`README.md`, then `docs/auth/README.md`.

- Same toolchain and conventions as `ui/chaos`: `src/` layout, `@/*` alias, Prettier
  (tailwind plugin, 110 cols), ESLint with simple-import-sort, `src/components/ui/*`
  copied verbatim from the kit (regenerate there, do not hand-edit; excluded from lint
  and format), `src/app/globals.css` is the kit's token file. A look change is a token
  change in both apps.
- Ory Elements owns the flows. Never hand-build a login form or call Kratos submit
  endpoints; add or change an override in `src/components/ory/components.tsx` instead.
  Keep Kratos' input/button names and values intact (the browser check depends on them).
- The SDK URL is the page origin (`src/lib/ory-config.ts`), never a build-time constant:
  the same image serves `auth.localhost:18080` and `auth.<domain>`.
- The consent page is the one place claims enter tokens (`src/lib/server.ts` `claimsFor`).
  Envoy and Grafana read `role` / `grafana_role` from there; a new claim goes there, into
  `docs/auth/README.md`, and into Envoy's `claim_to_headers` if a header needs it.
- Roles: `metadata_admin.role` on the Kratos identity. Never derive a role from traits.
- `src/app/api/*` are the only server routes; they need the in-cluster Hydra admin and
  Kratos URLs and are reachable only through Envoy's `auth.*` host.
- Build output is standalone Node on distroless; `readOnlyRootFilesystem` with `/tmp` as
  emptyDir. `NEXT_PUBLIC_*` flags are baked at image build (`auth:ui-build-args`).
- `mise run ui:auth:check` is in `ci`. The browser check is `mise run auth:e2e`.
