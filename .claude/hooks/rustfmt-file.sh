#!/usr/bin/env bash
# PostToolUse hook for Edit|Write. Formats the edited file if it is Rust.
set -u
INPUT=$(cat)
FILE=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty')
case "$FILE" in
  *.rs)
    command -v rustfmt >/dev/null 2>&1 || exit 0
    [ -f "$FILE" ] || exit 0
    rustfmt --edition 2024 "$FILE" 2>&1 | head -20
    ;;
esac
exit 0
