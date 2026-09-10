# CI

The gate is defined once, in `mise.toml`, as `mise run ci`. GitHub Actions runs the same
checks as separate jobs so they parallelise across runners and each failure has its own
name. If you add a check, add it in both places; this page lists the mapping.

## What runs

| `mise run ci` step | GitHub job | What fails it |
|---|---|---|
| `fmt:check` | `lint` | `cargo fmt --all --check`, `buf format -d --exit-code proto` |
| `typos` | `typos` | a misspelling anywhere `_typos.toml` does not exclude |
| `deny` | `deny` | a RUSTSEC advisory, a licence outside the allow list, a banned crate, an unknown registry |
| `lint` | `lint` | any clippy warning (pedantic is on), any `buf lint` finding |
| `test` | `test` | any failing test, `cargo nextest` plus doctests |
| `doc` | `doc` | any rustdoc warning, broken intra-doc links included |
| `chaos:run` | `scenarios` | any scenario under `scenarios/` failing an assertion or timeline action |
| `ui:check` | `ui` | `ui/chaos`: prettier drift, an eslint finding (React Compiler rules included), a type error; CI also runs `pnpm build` |
| not in the gate | `docker` | any image failing to build; on `main` also failing to push. The chaos image build runs `pnpm build` first so it carries the UI |

Locally the steps run in that order, cheapest first, so a typo or format slip fails in
under two seconds without a compile. They run sequentially on purpose: parallel cargo
invocations only queue on the target directory lock.

## Running one step locally

```sh
mise run fmt:check      # or: cargo fmt --all --check && buf format -d --exit-code proto
mise run typos          # or: typos
mise run deny           # or: cargo deny check
mise run lint           # or: cargo clippy --locked --workspace --all-targets --all-features -- -D warnings && buf lint proto
mise run test           # or: cargo nextest run --workspace --all-features && cargo test --workspace --doc
mise run doc            # or: cargo doc --workspace --all-features --no-deps
mise run chaos:run      # or: cargo run -p tbd-chaos -- run --dir scenarios
mise run ui:check       # or, in ui/chaos: pnpm format:check && pnpm lint && pnpm typecheck
mise run ci             # all of the above, in order
```

`mise run setup` installs the tools the steps need (`cargo-nextest`, `cargo-deny`, `buf`,
`typos`, and the rest listed under `[tools]` in `mise.toml`). Without mise, the `cargo`
commands above still work if those tools are on `PATH`.

## Fixing common failures

| Failure | Fix |
|---|---|
| `fmt:check` | `mise run fmt` (runs `cargo fmt` and `buf format -w`) |
| `typos` | fix the word, or add it to `_typos.toml` under `[default.extend-words]` if it is intentional |
| `deny` advisory | `cargo update -p <crate>`; if no fix exists, add an `ignore` entry with a `reason` in `deny.toml` |
| `deny` licence | add the licence to `[licenses] allow` in `deny.toml` after checking it is acceptable |
| `lint` clippy | fix it; a targeted `#[allow(clippy::...)]` with a comment is acceptable when the lint is wrong for that spot |
| `lint` buf | protos follow buf's STANDARD rules: directory matches package, services end in `Service`, RPC messages are `<Rpc>Request` / `<Rpc>Response` |
| `doc` | usually a `[`Name`]` link to a private or renamed item |
| `scenarios` | run `mise run chaos:run` locally; the report says which assertion failed and by how much. See [chaos/scenarios.md](chaos/scenarios.md) |
| `ui` prettier | `cd ui/chaos && pnpm format` |
| `ui` eslint `set-state-in-effect` | derive the value or move the `setState` into the callback that learns the news; see `src/lib/api/hooks.ts` |

`mise run tbd:selfcheck` sits between `test` and `doc`: it scaffolds a throwaway
service (`zeta`) into a copy of the tree with the `tbd` CLI, runs the CLI again to prove
the second run is a no-op, checks every registration, and compiles, lints, formats,
tests and buf-lints the result. A template or an anchor in `crates/cli` that drifts
from the tree fails here. See [tbd/README.md](tbd/README.md).

## Images

The `docker` job builds every image from `devops/docker/Dockerfile` on every push and
pull request, and pushes them only on `main`, tagged with the short commit SHA and
`main`. `release.yml` runs on `v*` tags and pushes the semver tag plus `latest`.

- `ghcr.io/<ORG>/tbd-engine`
- `ghcr.io/<ORG>/tbd-protocol`
- `ghcr.io/<ORG>/tbd-chaos`

`ORG` defaults to the repository owner. Override it with a repository variable named
`ORG`. Pushing needs no secret beyond the automatic `GITHUB_TOKEN` with `packages: write`.

## Adding a check

1. Add a mise task in `mise.toml` and put it in the `run` list of `[tasks.ci]`, in the
   position its cost deserves.
2. Add a job to `.github/workflows/ci.yml`. Copy the shape of the nearest existing job;
   use `actions-rust-lang/setup-rust-toolchain@v1` for anything that needs Rust (it
   reads `rust-toolchain.toml` and caches) and `taiki-e/install-action@v2` for cargo
   tools.
3. Add a row to the table above.

## Toolchain pinning

`rust-toolchain.toml` pins the exact Rust version; CI reads it, so a bump is one commit.
`Cargo.lock` is committed and CI passes `--locked`, so a dependency change without a
lockfile update fails in `lint` and `test` rather than silently resolving differently.
