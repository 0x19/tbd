#!/usr/bin/env bash
# Stop hook. Deterministic gate: if Rust sources changed and a workspace exists,
# clippy and tests must pass before Claude may finish. Claude Code overrides after
# 8 consecutive blocks, so this cannot loop forever.
set -u
INPUT=$(cat)
[ "$(printf '%s' "$INPUT" | jq -r '.stop_hook_active // false')" = "true" ] && exit 0
CWD=$(printf '%s' "$INPUT" | jq -r '.cwd // empty')
[ -n "$CWD" ] && cd "$CWD" 2>/dev/null
ROOT=$(git rev-parse --show-toplevel 2>/dev/null) || exit 0
cd "$ROOT" || exit 0
[ -f Cargo.toml ] || exit 0

# Only gate when Rust-relevant files changed (tracked or untracked).
CHANGED=$( { git diff --name-only HEAD -- '*.rs' 'Cargo.toml' '*/Cargo.toml' 2>/dev/null; \
             git ls-files --others --exclude-standard -- '*.rs' '*/Cargo.toml' 2>/dev/null; } | head -1)
[ -z "$CHANGED" ] && exit 0

if ! OUT=$(cargo clippy --workspace --all-targets --all-features -- -D warnings 2>&1 | tail -40); then
  jq -n --arg r "clippy failed; fix before finishing:
$OUT" '{decision:"block", reason:$r}'
  exit 0
fi

if command -v cargo-nextest >/dev/null 2>&1; then
  TEST_CMD="cargo nextest run --workspace --all-features"
else
  TEST_CMD="cargo test --workspace --all-features"
fi
if ! OUT=$($TEST_CMD 2>&1 | tail -40); then
  jq -n --arg r "tests failed; fix before finishing:
$OUT" '{decision:"block", reason:$r}'
  exit 0
fi
exit 0
