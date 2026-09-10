# tbd, the scaffolding CLI

One command turns a name into a complete service: a crate that compiles, lints and
tests; its proto, config layers and Kubernetes manifests; and the entry in every shared
file the repository keeps per service. Running it again does nothing. This page is the
contract: what it writes, where, and how it decides something is already there.

```sh
mise run tbd -- new service ledger              # or: cargo tbd new service ledger
mise run tbd -- new service ledger --dry-run    # show the plan, write nothing
mise run tbd -- service check ledger            # every registration present? (CI-usable)
mise run tbd -- service list                    # every binary crate and its state
```

## Commands

| Command | Flags | Exit code |
|---|---|---|
| `new service <name>` | `--port N` (default: one above the highest service port in use), `--metrics-port N` (developer-machine Prometheus port; containers use 9464), `--kind grpc`, `--bacon-key c`, `--dry-run`, `--force` (overwrite generated files that differ) | 0 done or already scaffolded; 1 blocked (an anchor missing, a file differs); 2 bad name |
| `service check <name> [--fix]` | `--fix` applies missing registrations, never creates files | 0 all present; 1 some missing, one line each; 2 no marker |
| `service list` | | 0 |
| any | `--repo PATH` (`TBD_REPO`): the workspace root; default is the one above the current directory | |

A name is 2 to 24 lowercase letters and digits starting with a letter. Reserved:
`engine protocol chaos common proto cli envoy auth base tbd`.

## What a generated gRPC service contains

| Path | What |
|---|---|
| `crates/<name>/` | `lib.rs` with `serve`/`serve_on`/`serve_with`, health, reflection, `TCP_NODELAY` on the incoming stream, the shared `grpc_request_span`; `service.rs` with `admit()` (counter, `RequestTimer`, fault handle) and a `Ping` RPC that answers `stub: true`; `config.rs` on the layered TOML loader with `deny_unknown_fields` and `<NAME>_*` flag overrides; `main.rs` with a `config` subcommand; `tests/it` booting on port 0 with the shipped `local` config; `CLAUDE.md` whose first line is the marker |
| `proto/tbd/<name>/v1/<name>.proto` | `<Name>Service { rpc Ping }`, buf STANDARD clean |
| `configs/<name>/{base,local,dev,production}.toml` | every key in `base.toml`; the others exist so a mistyped `TBD_ENV` fails at start |
| `devops/k8s/base/<name>/` | Deployment (gRPC probes, `/tmp` emptyDir, non-root, read-only), headless Service, kustomization |
| `crates/chaos/src/kinds/<name>.rs` | the chaos kind ([chaos/kinds.md](../chaos/kinds.md)): the spec struct that starts the service in-process with a `Runtime`, readiness by named health check, a `grpc_<name>_ping` validate check, and `KIND` (fault injection, counters, addable, target on the service's port) |

The marker: `<!-- tbd new service ledger --kind grpc --port 50052 --metrics-port 9466 --bacon-key l (tbd-cli 0.1.0) -->`.

## Tokens

Templates use `@@token@@`; `{{ }}` is left alone (Rust format strings, Jinja, GitHub
expressions). For `ledger` on port 50052:

| Token | Value |
|---|---|
| `name`, `Name`, `NAME`, `plural` | `ledger`, `Ledger`, `LEDGER`, `ledgers` (a name already ending in `s` stays as is: `humans`) |
| `package`, `crate` | `tbd-ledger`, `tbd_ledger` |
| `proto_package`, `grpc_service`, `proto_path` | `tbd.ledger.v1`, `tbd.ledger.v1.LedgerService`, `tbd/ledger/v1/ledger.proto` |
| `port`, `metrics_port` | `50052`, `9466` |
| `image`, `bacon_key`, `cli_version`, `marker` | `ghcr.io/0x19/tbd-ledger`, `l`, the CLI version, the marker line |

## Registrations, in the order applied

Every edit is anchored to a line that exists today and has a *needle*, text whose
presence means the edit was made. Needles never contain the port. Creates are refused
when the file exists with different content, unless `--force`.

| Id | File | Where | Needle |
|---|---|---|---|
| `create:*` | the files above | | identical content |
| `cargo:default-members` | `Cargo.toml` | `default-members = [` … before `]` | `"crates/<name>"` |
| `cargo:dependency` | `Cargo.toml` | after the last `tbd-` line | `tbd-<name> ` |
| `proto:module` | `crates/proto/src/lib.rs` | end of file | `pub mod <name> ` |
| `chaos:module`, `chaos:kind`, `chaos:dependency` | `crates/chaos/src/kinds/mod.rs`, `crates/chaos/Cargo.toml` | after the last `pub mod` (rustfmt sorts them at apply time), before the `// tbd:kinds-end` marker in `ALL`, after `tbd-protocol.workspace` | `pub mod <name>;`, `&<name>::KIND,`, the dependency line |
| `chaos:topology` | `topologies/dev.toml` | end of file | `[stack.<plural>.` (`<name>s`, or `<name>` when it already ends in `s`) |
| `chaos:targets:{base,dev,production,cluster}` | `configs/chaos/*.toml` | after `engine = ` under `[targets]` | `<name> = "http` |
| `chaos:k8s-env`, `chaos:compose-env`, `chaos:ansible-env` | `devops/k8s/chaos/deployment.yaml`, `compose.yaml`, the ansible compose template | after `CHAOS_ENGINE_URL` | `CHAOS_<NAME>_URL` |
| `env:example` | `.env.example` | end of file | `<NAME>_LISTEN_ADDR=` (the block also carries a commented `CHAOS_<NAME>_URL`) |
| `k8s:configmap`, `k8s:base` | `devops/k8s/base/{configmap,kustomization}.yaml` | after `PROTOCOL_METRICS_ADDR:`, after `  - protocol` | `<NAME>_LISTEN_ADDR:`, `  - <name>` |
| `k8s:overlay:{local,dev,prod}:{image,patch}` | the overlay kustomizations | before `patches:`, before `configMapGenerator:` | the image name, the patch target line |
| `envoy:header`, `envoy:route`, `envoy:cluster` | `devops/envoy/envoy.yaml` | the cluster list comment; before the `engine-lb` catch-all route; before `- name: chaos` under `clusters:` | `` `<name>`, ``, `/tbd.<name>.v1.<Name>Service/`, `    - name: <name>` |
| `compose:service`, `compose:envoy-depends` | `compose.yaml` | before `  envoy:`; after `      - chaos` under `envoy:` | `tbd-<name>:`, `      - <name>` |
| `ansible:image`, `ansible:port`, `ansible:pull`, `ansible:compose`, `ansible:local:*` | `devops/ansible/…` | after the protocol lines, before `{% if chaos_enabled`, the build/import/restart/wait lists | `<name>_image`, `<name>_port`, `bin: <name>,`, `deployment/<name>` |
| `ci:matrix`, `ci:release` | `.github/workflows/{ci,release}.yml` | after the last matrix entry | `- bin: <name>` |
| `mise:*` | `mise.toml` | `run:`/`watch:` tasks, `dev` depends, `docker:build`, `docker:push`, `local:build` (build line and `k3d image import`), `local:deploy` and `local:restart` rollout lists | the task headers, the image name, `deployment/<name>` |
| `bacon:job`, `bacon:key` | `bacon.toml` | before `[keybindings]`, after `p = "job:protocol"` | `[jobs.<name>]`, `= "job:<name>"` |
| `docs:*` | `docs/local-cluster.md`, `ARCHITECTURE.md`, `README.md`, `docs/README.md`, `devops/k8s/README.md`, `docs/ci.md` | one row or tree line each | the row |

Internal only: the Envoy route lives on the internal listener (`:50051`, the engine is
the catch-all, other services are matched by service name), never on the edge. A
service that must be public gets its edge route by hand, with the JWT requirement.

## What it does not do

Printed as a checklist after scaffolding, because the code has no safe anchor or the
step is a build:

- `cargo check -p tbd-<name>`, which updates `Cargo.lock`.
- `mise run chaos:docs`, which regenerates `docs/chaos/kinds.md` with the new kind.
- The devops gate: `kustomize build devops/k8s/overlays/local`, `docker compose config -q`,
  `mise run envoy:validate`.
- A Grafana dashboard, if the service wants one.

## Idempotency and rollback

A plan resolves every registration first; if any anchor is missing or any generated
file differs, nothing is written and the blockers are listed. Re-running with the same
name reads the marker and keeps the ports the crate was scaffolded with. TOML files are
re-parsed before commit. There is no undo beyond git: `git checkout -- . && git clean
-fd crates/<name> proto/tbd/<name> configs/<name> devops/k8s/base/<name>` returns the
tree to the state before the run.

Generated `.rs` files are passed through `rustfmt` before they are compared or written,
so `cargo fmt --check` passes and a re-run recognises its files.

## Adding a registration site

1. Add the entry to `registrations()` in `crates/cli/src/registry.rs`, with an anchor
   that exists in the tree and a needle without the port.
2. Run `cargo nextest run -p tbd-cli`: the anchors test resolves it against the real
   tree, the idempotency test applies it twice.
3. Add a row to the table above, in the same commit.

`mise run tbd:selfcheck` (part of `mise run ci`) scaffolds `zeta` into a copy of the
tree, runs it again, checks it, and compiles, lints, formats, tests and buf-lints the
result.
