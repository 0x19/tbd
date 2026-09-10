# .claude/

Project-level Claude Code configuration. Committed, except `settings.local.json`.

| Path | Purpose |
|---|---|
| `settings.json` | Permissions, hooks, plugins shared by everyone on the repo |
| `settings.local.json` | Personal overrides, gitignored, created by Claude Code on "don't ask again" |
| `hooks/protect-files.sh` | `PreToolUse`: blocks edits to secrets, `Cargo.lock`, `.git/` and the archived repos |
| `hooks/rustfmt-file.sh` | `PostToolUse`: formats an edited `.rs` file |
| `hooks/stop-check.sh` | `Stop`: runs clippy and tests when Rust files changed |
| `rules/rust.md` | Loaded only when Claude touches `.rs` or `Cargo.toml` files |
| `rules/design-docs.md` | Loaded only when Claude touches `docs/design/` |
| `rules/devops.md` | Loaded only when Claude touches `devops/`, `compose.yaml` or the workflows: validation commands, the one-config rule, the port map |
| `skills/design-doc/` | `/design-doc <plane> <title>` creates a numbered decision doc |
| `agents/design-reviewer.md` | Reviews a design doc against the others for contradictions |
| `agents/apex-archaeologist.md` | Read-only digger for lessons in apex and Proximity |

Rules of thumb, from the official docs: `CLAUDE.md` is context, hooks are
enforcement, rules are path-scoped context, skills are on-demand procedures.
Keep `CLAUDE.md` under 200 lines. Run `/doctor` to have it trimmed.
