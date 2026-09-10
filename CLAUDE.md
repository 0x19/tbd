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
  `local:deploy`, then `local:traffic`; Grafana on :3000, Envoy edge on :18080. Dashboards
  reload with `mise run grafana:reload`. See `docs/observability.md`.
- `mise run chaos:up` runs both in one process; `mise run validate` checks every surface;
  `mise run chaos:run` runs the scenarios. Docs under `docs/chaos/` are the tool's
  contract: a change to a flag, output field, TOML key, behaviour or check updates the
  matching page in the same commit. CI is described in `docs/ci.md`.
- Protos compile without `protoc` (`protox` in `crates/proto/build.rs`); edit `/proto`
  and rebuild. `buf lint proto` runs in `mise run lint` with buf's STANDARD rules:
  directory matches package (`proto/tbd/engine/v1/`), services end in `Service`,
  streaming RPCs use distinct `XRequest`/`XResponse` messages.

## Conventions that differ from defaults

- Clippy pedantic is on and warnings are errors. `unwrap`/`expect` denied outside tests;
  `println`/`dbg` denied; `unsafe` forbidden.
- Stub values are labelled stubs on every surface (`stub: true`, `stub-` model versions).
  Never let a placeholder look like a measurement.
- Protocol handlers translate and forward only. Business logic goes in the engine.
- `tbd-common` stays transport-free; `tbd-proto` stays generated-only.
- Integration tests boot real servers on port 0 via `serve_on`; no mocks of our own
  services.
- Every new env var goes on a clap flag with `env = ...` and into `.env.example`,
  `compose.yaml`, `devops/k8s/base/configmap.yaml` and the ansible compose template.
- Every new metric name goes into `crates/common/src/metrics.rs` `names` and the table in
  `docs/observability.md`; every request path gets a span with `trace_id` recorded.

## Git

- Conventional Commits, one logical change per commit, commit only when asked.
- Branch `main`; never force-push.

## Working style

- Senior principal Rust engineer: one recommendation, trade-off stated, primary source
  cited. Research online before answering questions about current tooling.
- Plan mode for anything touching more than one crate or the proto contract.
