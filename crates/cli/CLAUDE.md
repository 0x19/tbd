# crates/cli

The `tbd` binary (package `tbd-cli`): scaffolds a service from embedded templates and
registers it in every shared file, idempotently. Owns no runtime code. The one crate
whose bin name (`tbd`) differs from its directory (`cli`); it is never an image, so the
`BIN=<dir>` Docker convention does not apply.

- `main.rs`: clap and printing, the only file that prints. `mise run tbd -- ...` or
  `cargo tbd ...` (alias in `.cargo/config.toml`).
- `service.rs`: `ServiceName` (`^[a-z][a-z0-9]{1,23}$`, `RESERVED` list), `Service`,
  every identifier derived from the name, and the marker line written first into the
  generated `CLAUDE.md`. `check` and `list` rebuild the `Service` from that marker, so
  they never guess a port.
- `template.rs`: `@@token@@` substitution, nothing else. Unknown token or stray `@@` is
  an error. `templates.rs` is the `include_str!` table of files and snippets under
  `templates/`.
- `edit.rs`: the primitives (`Create`, `InsertAfter`, `InsertBefore`, `Splice`,
  `AppendEof`) on plain text, with `Anchor` (`Exact`/`Prefix`/`Contains`/`LastPrefix`,
  optionally scoped to the first match after another line) and `Status`.
- `registry.rs`: **the registration table, as data, in apply order.** One function,
  one list; `scaffold`, `check` and `list` iterate it. `CHECKLIST` is what it cannot do
  safely (chaos topology, targets, validate, UI schema).
- `scaffold.rs`: two-phase plan/apply over the `repo.rs` cache; nothing reaches disk
  unless every registration resolves; TOML files are re-parsed before commit.
- `fmt.rs`: generated `.rs` content goes through `rustfmt` before it is compared or
  written, so a re-run recognises its own files.

Rules:
- A needle never contains the port. A needle ending in `\n` must match a whole line.
- A splice must leave its own anchor matching afterwards (the anchor is a prefix or a
  substring that the inserted text does not break); the idempotency test proves it.
- `tests/it/main.rs` resolves every registration against the **real** tree. Reformat a
  shared file (`mise.toml`, `envoy.yaml`, an overlay, a docs table) and this test tells
  you which anchor you broke. Fix the anchor in `registry.rs`, in the same commit.
- `docs/tbd/README.md` is the contract: a new registration, flag, token or exit code
  changes that page in the same commit.
- `mise run tbd:selfcheck` (in `ci`) scaffolds `zeta` into a copy of the tree and
  compiles, lints, formats, tests and buf-lints it. Template changes are not done until
  it passes.

Tests: unit tests per module; `tests/it/main.rs` renders every template, resolves every
anchor against the real tree, and applies the plan twice in memory.
