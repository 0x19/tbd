#!/usr/bin/env bash
# Smoke test every surface of a running local stack. Exit non-zero on the first failure.
#   PROTOCOL=localhost:8080 ENGINE=localhost:50051 scripts/smoke.sh
set -euo pipefail
PROTOCOL="${PROTOCOL:-localhost:8080}"
ENGINE="${ENGINE:-localhost:50051}"

step() { printf '\n\033[1m== %s\033[0m\n' "$*"; }

step "HTTP liveness  GET /healthz"
curl -sf "http://$PROTOCOL/healthz"; echo

step "HTTP readiness GET /readyz (gateway → engine health)"
curl -sf "http://$PROTOCOL/readyz"; echo

step "REST           POST /v1/evaluate"
curl -sf -X POST "http://$PROTOCOL/v1/evaluate" \
  -H 'content-type: application/json' \
  -d '{"subject_id":"s1","payload":"hello"}'; echo

step "SSE            GET /v1/subjects/s1/events (3 seconds)"
curl -sN --max-time 3 "http://$PROTOCOL/v1/subjects/s1/events" || true

step "GraphQL        POST /graphql"
curl -sf -X POST "http://$PROTOCOL/graphql" \
  -H 'content-type: application/json' \
  -d '{"query":"{ version engineReady evaluate(subjectId:\"s1\"){ subjectId score stub modelVersion } }"}'; echo

step "WebSocket      /ws (send one frame, print the echo)"
# -1: one message each way, then exit. RUST_LOG is unset so websocat's own logging stays quiet.
printf 'hello over ws\n' | env -u RUST_LOG websocat -1 "ws://$PROTOCOL/ws"

step "gRPC engine    reflection + health + Evaluate on $ENGINE"
grpcurl -plaintext "$ENGINE" list
grpcurl -plaintext -d '{"service":"tbd.engine.v1.Engine"}' "$ENGINE" grpc.health.v1.Health/Check
grpcurl -plaintext -d '{"subject_id":"s1","payload":"aGVsbG8="}' "$ENGINE" tbd.engine.v1.Engine/Evaluate

step "gRPC engine    Subscribe (server stream, 2 events)"
grpcurl -plaintext -max-msg-sz 1048576 -d '{"subject_id":"s1"}' "$ENGINE" tbd.engine.v1.Engine/Subscribe \
  | head -n 12 || true

step "gRPC gateway   Ping on $PROTOCOL (same port as HTTP, h2c)"
grpcurl -plaintext "$PROTOCOL" list
grpcurl -plaintext -d '{"message":"hi"}' "$PROTOCOL" tbd.protocol.v1.Gateway/Ping

printf '\n\033[32mall surfaces answered\033[0m\n'
