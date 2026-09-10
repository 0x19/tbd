---
paths:
  - "**/*.rs"
  - "**/Cargo.toml"
---
# Rust rules

- Edition 2024, workspace lints inherited via `[lints] workspace = true` in every crate.
- Errors: `thiserror` enums in library crates, `anyhow` only in binaries and xtask.
  No `unwrap()` or `expect()` outside tests; the workspace lint denies it.
- `unsafe_code = "forbid"` at the workspace level. If a crate genuinely needs it,
  override per crate and put a `// SAFETY:` comment stating the invariant on every block.
- Logging is `tracing`, never `println!`/`eprintln!`/`dbg!` (denied by lint).
- Every provider trait ships with a `Mock` implementation that exercises the full
  flow, including failure modes, from the first commit. No exceptions.
- A stub value is typed or named as a stub and labelled in every place it surfaces.
  Never format a placeholder as if it were a measurement.
- A privacy claim is a test: write the test that fails if the guarantee regresses.
- Unit tests in `#[cfg(test)] mod tests` next to the code. Integration tests in one
  binary per crate at `tests/it/main.rs` with submodules. HTTP vendors are mocked
  with `wiremock`; interaction mocks with `mockall` only inside unit tests.
- Before saying a change is done: `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  and the tests, and show the output.
