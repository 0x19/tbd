# tbd

Rust workspace: `engine` (gRPC streaming service) and `protocol` (HTTP/WS/GraphQL/gRPC
protocol). Read `ARCHITECTURE.md` before changing crate boundaries. `docs/design/` is
earlier idea material for a product direction, not a spec; do not "fix" it.

## Commands

- `mise run ci` is exactly what CI runs: fmt, clippy `-D warnings`, nextest + doctests,
  docs, cargo-deny, typos. Run it before saying a change is done and show the output.
- Single test: `cargo nextest run -p tbd-protocol -E 'test(name)'`.
- `mise run run:engine` / `mise run run:protocol`; `mise run up` for the compose stack.
- Local cluster with Envoy and the observability stack: `mise run local:up`, `local:build`,
  `local:deploy`, then `local:traffic`; after code changes `local:restart`. Grafana on
  :3000 (admin/admin), Envoy edge on :18080, engine LB on :15051. Dashboards reload with
  `mise run grafana:reload`; Envoy config check with `mise run envoy:validate`.
  Docs: `docs/local-cluster.md`, `docs/observability/README.md`.
- `mise run chaos:up` runs both in one process; `mise run validate` checks every surface;
  `mise run chaos:run` runs the scenarios; `mise run chaos:serve` is the HTTP API the
  admin UI (`ui/chaos`) uses. Docs under `docs/chaos/` are the tool's contract: a change
  to a flag, output field, TOML key, API route, behaviour or check updates the matching
  page in the same commit (`api.md` for routes and SSE frames, `config.md` for
  `configs/chaos/`). CI is described in `docs/ci.md`.
- Identity stack (Ory Hydra + Kratos, `devops/k8s/auth`): `mise run auth:secrets` once,
  `auth:deploy` (part of `local:deploy`), `auth:envoy-secrets`, `auth:token` for a
  machine JWT, `auth:e2e` for the browser check, `auth:oidc <provider> <id> <secret>` for
  social sign-in, `auth:role EMAIL ROLE`, `auth:smtp URI FROM`, `auth:rotate WHAT`. The
  sign-in pages are `ui/auth` (Next.js on Ory Elements, `ui:auth:check` in `ci`). Envoy gates every host (`devops/envoy/envoy.yaml`); against a deployed
  stack `chaos validate` and load need `CHAOS_AUTH_*` or `--token`. Docs:
  `docs/auth/README.md`.
- New services come from the CLI: `mise run tbd -- new service <name>` scaffolds a gRPC
  service (crate, proto, configs, k8s, chaos adapter) and registers it in every shared
  file; `tbd service check <name>` verifies that; `mise run tbd:selfcheck` is the CI
  proof. `docs/tbd/README.md` is its contract; the anchors it relies on are tested
  against the real tree, so reformatting a shared file is a CLI change too.
- Binaries read layered config from `configs/<binary>/base.toml` + `<TBD_ENV>.toml`
  (`tbd_common::config`); every key lives in `base.toml`, env files carry differences,
  flags and env vars override. `chaos config` prints the effective result.
- Protos compile without `protoc` (`protox` in `crates/proto/build.rs`); edit `/proto`
  and rebuild. `buf lint proto` runs in `mise run lint` with buf's STANDARD rules:
  directory matches package (`proto/tbd/engine/v1/`), services end in `Service`,
  streaming RPCs use distinct `XRequest`/`XResponse` messages. `proto/google/api/` is
  vendored (not linted) so an RPC can carry `option (google.api.http)`; the protocol
  serves every annotated RPC over REST or SSE (`docs/protocol/README.md`).

## Conventions that differ from defaults

- Clippy pedantic is on and warnings are errors. `unwrap`/`expect` denied outside tests;
  `println`/`dbg` denied; `unsafe` forbidden.
- Release builds keep symbols and frame pointers on purpose (profiling); do not add
  `strip` back.
- Stub values are labelled stubs on every surface (`stub: true`, `stub-` model versions).
  Never let a placeholder look like a measurement.
- Protocol handlers translate and forward only. Business logic goes in the engine.
- `tbd-common` stays transport-free; `tbd-proto` stays generated-only.
- Integration tests boot real servers on port 0 via `serve_on`; no mocks of our own
  services.
- Every new env var goes on a clap flag with `env = ...` and into `.env.example`,
  `compose.yaml`, `devops/k8s/base/configmap.yaml` and the ansible compose template.
  The one generic family is the protocol's `PROTOCOL_<NAME>_URL`, one per backend in
  `configs/protocol/base.toml` `[services]`, read by name (`--service-url name=URL` is
  the flag form); `tbd new service` registers it everywhere.
- Every new metric name goes into `crates/common/src/metrics.rs` `names` and the table in
  `docs/observability/metrics.md`; every request path gets a span with `trace_id`
  recorded and a `RequestTimer`; streams get a `StreamGuard`.
- Services never address each other directly: the engine URL is Envoy's engine LB.
- Services never verify tokens. Envoy does, and forwards the verified claims in
  `x-jwt-payload`; handlers take `Principal` (`crates/protocol/src/principal.rs`). A new
  route or host is gated in `envoy.yaml` by naming a JWT requirement; health paths stay
  open. Secrets live only in Kubernetes Secrets created by mise tasks, never in files.
- `envoy.yaml` has exactly one placeholder, `__AUTH_PUBLIC_URL__`; anything else that
  differs per environment goes through DNS names or ConfigMaps, not more placeholders.

## Git

- Conventional Commits, one logical change per commit, commit only when asked.
- Branch `main`; never force-push.

## Working style

- Senior principal Rust engineer: one recommendation, trade-off stated, primary source
  cited. Research online before answering questions about current tooling.
- Plan mode for anything touching more than one crate or the proto contract.
