# tbd

> Working name. Pre-alpha. Not for production.

Two Rust services and the tooling to build, ship and deploy them.

| Service | Binary | Port | Does |
|---|---|---|---|
| **engine** | `engine` | 50051 | gRPC streaming compute service: unary `Evaluate`, server-streaming `Subscribe`, bidirectional `Session`. Health and reflection built in. |
| **protocol** | `protocol` | 8080 | API gateway on one port: REST + SSE under `/v1`, WebSocket at `/ws`, GraphQL at `/graphql`, gRPC over h2c. Forwards everything to the engine. |

Every score the engine returns today is a **stub** and carries `stub: true` on every
surface. There is no model yet. That flag is part of the contract, not metadata.

## Quick start

```sh
# once
curl https://mise.run | sh          # task runner + tool installer
mise trust && mise run setup        # installs cargo-nextest, cargo-deny, buf, kubectl, ...

# every day
mise run ci                         # fmt, clippy -D warnings, tests, docs, deny, typos
mise run run:engine                 # terminal 1
mise run run:protocol               # terminal 2
curl -s localhost:8080/readyz
curl -s -XPOST localhost:8080/v1/evaluate -H 'content-type: application/json' \
     -d '{"subject_id":"s1","payload":"hi"}'
open http://localhost:8080/graphql  # GraphiQL
```

`mise run smoke` hits every surface of a running stack (HTTP, SSE, GraphQL, WebSocket,
gRPC) with `curl`, `websocat` and `grpcurl`; the commands are in `scripts/smoke.sh`.

`make <task>` is a shim for `mise run <task>`; `mise tasks` lists everything.
Plain `cargo build`, `cargo test` and `cargo clippy` work without mise: Rust is
pinned by `rust-toolchain.toml`, protos compile without `protoc`.

## Layout

```
crates/
  common/     telemetry, shutdown, shared CLI flags. Transport-free.
  proto/      generated gRPC code from proto/. No hand-written logic.
  engine/     the engine service (lib + bin + tests/it)
  protocol/   the gateway service (lib + bin + tests/it)
proto/        .proto sources, buf-linted
devops/
  docker/     one Dockerfile for both binaries (cargo-chef, distroless)
  k8s/        kustomize base + dev/prod overlays
  ansible/    host bootstrap + compose deploy
compose.yaml  local stack
docs/design/  earlier design notes; idea material, not a spec
```

See [ARCHITECTURE.md](ARCHITECTURE.md) for the shape and the invariants.

## Ship

```sh
mise run docker:build               # both images, IMAGE_REGISTRY/IMAGE_TAG from env
mise run up                         # compose stack on 50051 / 8080
mise run k8s:render                 # KUBE_OVERLAY=dev|prod
mise run ansible:bootstrap          # once per host
mise run ansible:deploy             # IMAGE_TAG to ANSIBLE_INVENTORY hosts
mise run ship                       # ci → docker:push → ansible:deploy
```

Placeholders a human fills in before the first real deploy: `ORG` in image names,
hosts in `devops/ansible/inventory/`, registry credentials in an ansible vault. See
[devops/README.md](devops/README.md).

## Toolchain

Rust 1.98.1 pinned, MSRV 1.96, edition 2024. Clippy pedantic with warnings denied,
`unwrap` denied outside tests, `unsafe` forbidden. CI runs exactly `mise run ci`.
