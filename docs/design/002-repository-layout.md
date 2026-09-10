# 002 — Repository layout

Status: decided — implemented 2026-09-09. The live layout is in the root README and `ARCHITECTURE.md`; where they differ, the code wins.

How the repository is laid out, built, linted, tested and released once code
starts. Nothing here is created until Q2 and Q3 in
[001-open-questions.md](001-open-questions.md) are `decided`.

---

## Baseline facts, September 2026

- Stable Rust is 1.98.1 (2026-09-03); 1.99 lands 2026-10-01. Edition 2024 has been
  stable since 1.85. There is no edition 2027 yet.
  ([blog](https://blog.rust-lang.org/2026/09/03/Rust-1.98.1/))
- This machine runs rustc 1.95.0 and needs `rustup update` before the toolchain pin
  below will resolve.
- OAuth 2.1 is still an IETF draft (draft-ietf-oauth-v2-1-15). Docs say
  "OAuth 2.1 (draft)", never "RFC".
  ([datatracker](https://datatracker.ietf.org/doc/html/draft-ietf-oauth-v2-1-15))
- `build.warnings = "deny"` in Cargo config replaced `RUSTFLAGS=-Dwarnings` in
  Cargo 1.97. ([changelog](https://doc.rust-lang.org/cargo/CHANGELOG.html))

## Layout

Flat `crates/` under a virtual root manifest. This is what cargo, rust-analyzer,
zed and meilisearch do; tokio and axum keep crates flat at the root but have five
or fewer. We will have more.
([Cargo book](https://doc.rust-lang.org/cargo/reference/workspaces.html),
[zed](https://github.com/zed-industries/zed/blob/main/Cargo.toml))

```
tbd/
├── Cargo.toml                 virtual manifest, resolver = "3", members = ["crates/*"]
├── Cargo.lock                 committed; CI runs --locked
├── rust-toolchain.toml        exact stable pin + rustfmt, clippy, rust-src
├── rustfmt.toml  clippy.toml  deny.toml  _typos.toml  .editorconfig
├── justfile                   the one place every check is defined
├── .cargo/config.toml         rustdocflags, xtask alias when needed
├── .config/nextest.toml       default + ci profiles
├── .github/workflows/         ci.yml, latest-deps.yml (weekly)
├── docs/design/               these docs
├── deploy/                    Dockerfile, compose; never inside crates/
└── crates/
    ├── <p>-core               domain types, error enums, traits; no I/O deps
    ├── <p>-providers          one trait per capability, Mock impls behind `mock` feature
    ├── <p>-providers-didit    one crate per real vendor, added when integrated
    ├── <p>-oidc               OIDC / OAuth 2.1 protocol types and validation
    ├── <p>-id                 ID service binary: lib.rs + thin main.rs, tests/it/main.rs
    ├── <p>-test-support       fixtures, wiremock and testcontainers helpers
    └── xtask                  only the day a task needs Rust code
```

`<p>` is the crate prefix and equals the product name, which is still "tbd". Pick the
name before the first crate; renaming crates later is a sed, not a redesign. Hyphens
in package names, matching crates.io convention.
([API guidelines](https://rust-lang.github.io/api-guidelines/naming.html))

Engine and API planes get their own crates when their design exists, not before.

## Manifest decisions

| Decision | Choice | Why |
|---|---|---|
| Resolver | `resolver = "3"`, written explicitly | Edition 2024 default, MSRV-aware. A virtual manifest has no edition, so it must be stated. ([edition guide](https://doc.rust-lang.org/edition-guide/rust-2024/cargo-resolver.html)) |
| Edition | 2024 via `[workspace.package]` | Only current edition |
| MSRV | `rust-version = "1.96"`, policy N-2, bumped in minor releases | We control the deploy target; stricter only if we publish libraries. ([Cargo book](https://doc.rust-lang.org/cargo/reference/rust-version.html)) |
| Toolchain | exact pin in `rust-toolchain.toml`, bumped by PR | Reproducible clippy output; no "new lint broke main" |
| `publish` | `false` workspace-wide | Flip per crate if ever published |
| Dependencies | all versions in `[workspace.dependencies]`; `optional` set in the member, never at workspace level | Single source of truth; workspace deps cannot be optional |
| Lints | `[workspace.lints]`, every crate `[lints] workspace = true` | Stable since 1.74 ([RFC 3389](https://rust-lang.github.io/rfcs/3389-manifest-lint.html)) |

Lint baseline, ratcheted to `deny` lint by lint as the code grows:

```toml
[workspace.lints.rust]
unsafe_code = "forbid"
missing_docs = "warn"
rust_2018_idioms = { level = "warn", priority = -1 }

[workspace.lints.clippy]
all = { level = "warn", priority = -2 }
pedantic = { level = "warn", priority = -1 }
unwrap_used = "deny"
expect_used = "warn"
dbg_macro = "deny"
print_stdout = "deny"
print_stderr = "deny"
module_name_repetitions = "allow"

[workspace.lints.rustdoc]
broken_intra_doc_links = "deny"
```

`clippy.toml` allows unwrap, expect, dbg and print in tests. `rustfmt.toml` uses
stable options only: `style_edition = "2024"`, `max_width = 100`. Import grouping
options are still nightly-only and are not used.
([rustfmt](https://github.com/rust-lang/rustfmt/blob/main/Configurations.md))

## Tooling

| Concern | Tool | Note |
|---|---|---|
| Tasks | `just` | Zero-Rust, readable, prefix-safe for Claude permissions. `mise exec` is not prefix-safe. `xtask` when a task needs Rust. |
| Tests | `cargo-nextest` | Parallel per test; does not run doctests, so `cargo test --doc` runs alongside |
| Supply chain | `cargo-deny` | Advisories, licence allow-list, bans (`openssl`), sources. Replaces cargo-audit. |
| Feature matrix, MSRV | `cargo-hack` | `--each-feature --no-dev-deps`; `--rust-version` |
| Unused deps | `cargo-shear` | Understands workspace deps; weekly job, not per PR |
| Spelling | `typos` | Used by rust-lang, zed, axum |
| Hooks | `prek` | Rust drop-in for pre-commit |
| Release | `release-plz` | Conventional commits, tags and changelog; `publish = false` |
| Errors | `thiserror` 2 in libs, `anyhow` at the binary edge | OIDC errors map to RFC 6749 error codes in `-oidc` |
| Observability | `tracing`; OpenTelemetry bridge isolated in one module | OTel Rust is pre-1.0 and breaks every minor |

`just check` runs: fmt check, clippy `-D warnings` on all targets and features,
nextest plus doctests, `cargo doc` with `-D rustdoc::all`, `cargo deny check`.
CI runs exactly `just check` plus the feature/MSRV matrix and a weekly
`cargo update` job with `continue-on-error`.
([axum CI](https://github.com/tokio-rs/axum/blob/main/.github/workflows/CI.yml),
[Cargo CI guide](https://doc.rust-lang.org/cargo/guide/continuous-integration.html))

## Testing layout

- Unit tests in-module. One integration binary per crate at `tests/it/main.rs`
  with submodules, so container and mock-server setup lives in one place.
  ([matklad](https://matklad.github.io/2021/02/27/delete-cargo-integration-tests.html))
- Vendor HTTP and webhooks: `wiremock`. Postgres: `testcontainers-modules`.
  Snapshots: `insta`. Property tests: `proptest`, with `proptest-regressions/` committed.
- The `Mock` provider is product code, not test scaffolding: a real `Mock*` type in
  `-providers` behind a `mock` feature, tested via a self dev-dependency, with
  `cargo hack --each-feature` proving both builds. `mockall` only inside unit tests.
- The scenario-testing pattern from chaos (TOML scenarios, ground truth, pass/fail
  assertions) is rebuilt in `-test-support` when the Engine needs it.

## READMEs

Root README: status line (pre-alpha, not for production), one-paragraph pitch,
crate map table, MSRV statement, pointer to `docs/design/`. Per crate: five lines
stating purpose and what the crate must not depend on. Never a file-by-file tree;
it goes stale in a week, as apex's did.

## Decisions that must be right on day one

1. `crates/` layout and the blind-broker indirection in the API path
   ([id/008](id/008-federation.md)); both change every integration if retrofitted.
2. Anchor-agnostic `person_id` ([id/007](id/007-identity-anchors.md)).
3. The pepper and per-org salts in an HSM-backed KMS with versioned, never-destroyed
   keys ([id/001](id/001-uniqueness.md)).

Everything else in this document is reversible and should be treated as such.

## Contested choices, stated

- `resolver = "3"` versus `"2"`: cargo and zed still pin 2 on edition 2024. We are a
  fresh repo with a stated MSRV, so 3.
- Pinned toolchain versus `channel = "stable"`: pin, and bump by PR.
- Pedantic clippy on versus off: on at `warn` while the codebase is small.
- Mock in the providers crate versus a `-mock` crate: same crate, so trait and mock
  change in one PR.
