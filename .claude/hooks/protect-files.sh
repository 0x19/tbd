#!/usr/bin/env bash
# PreToolUse hook for Edit|Write. Exit 2 blocks the tool call; stderr is shown to Claude.
set -u
INPUT=$(cat)
FILE=$(printf '%s' "$INPUT" | jq -r '.tool_input.file_path // empty')
[ -z "$FILE" ] && exit 0

block() { printf 'Blocked edit to %s: %s\n' "$FILE" "$1" >&2; exit 2; }

case "$FILE" in
  */.git/*|.git/*)                 block "never edit git internals directly" ;;
  *.pem|*.key)                     block "private key material" ;;
  */.env|.env|*/.env.*|.env.*)
    case "$FILE" in *.env.example) ;; *) block "secrets file; edit .env.example instead" ;; esac ;;
  */Cargo.lock|Cargo.lock)         block "Cargo.lock changes come from cargo, not hand edits" ;;
  /mnt/development/0x19/apex/*|/opt/proximity/*)
                                   block "archived repo; read-only, mine for lessons only" ;;
esac
exit 0
