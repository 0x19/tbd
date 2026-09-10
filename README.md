# tbd

> Working name. Pre-alpha. Not for production.

A Rust base for services that speak gRPC, REST, WebSocket and GraphQL, with the tooling
to build, test, fault-test, ship and deploy them. Two services exist today; the product
that will sit on them comes later. Everything here is meant to be reused by the next
project as-is.

| Piece | What it is |
|---|---|
| **engine** | gRPC streaming compute service: unary `Evaluate`, server-streaming `Subscribe`, bidirectional `Session`. Health and reflection built in. Port 50051. |
| **protocol** | One port, four surfaces: REST and SSE under `/v1`, WebSocket at `/ws`, GraphQL at `/graphql`, gRPC over h2c. Forwards to the engine, owns no logic. Port 8080. |
| **envoy** | The load balancer in front of everything: edge on 8080 for REST, SSE, GraphQL, WebSocket and gRPC; engine load balancer on 50051. One config for compose, Ansible and Kubernetes. |
| **chaos** | Runs both services in one process, validates every surface, generates load, injects faults on a timeline and asserts. Used for development and in CI. |
| **observability** | Prometheus metrics, OpenTelemetry traces and JSON logs with trace ids from every service and Envoy, into VictoriaMetrics, Tempo and VictoriaLogs, with Grafana dashboards. |
| **devops** | One Dockerfile, kustomize overlays including a local k3d cluster, Ansible playbooks, compose. |

Every score the engine returns today is a **stub** and carries `stub: true` on every
surface. There is no model yet. That flag is part of the contract, not metadata.

## Quick start

```sh
# once
curl https://mise.run | sh          # task runner + tool installer
mise trust && mise run setup        # cargo-nextest, cargo-deny, buf, typos, kubectl, grpcurl, ...

# run it
mise run chaos:up                   # engine :50051 + protocol :8080 in one process
mise run validate                   # 11 checks against them, one line each

# or the services separately
mise run dev                        # both, output prefixed per service
mise run run:engine                 # one
mise run run:protocol               # the other

# talk to it
curl -s localhost:8080/readyz
curl -s -XPOST localhost:8080/v1/evaluate -H 'content-type: application/json' \
     -d '{"subject_id":"s1","payload":"hi"}'
open http://localhost:8080/graphql  # GraphiQL
grpcurl -plaintext localhost:50051 list
```

`mise tasks` lists every task; `make <task>` is a shim for `mise run <task>`. Plain
`cargo build`, `cargo test` and `cargo clippy` work without mise: Rust is pinned by
`rust-toolchain.toml` and protos compile without `protoc`.

## Everyday tasks

| Task | Does |
|---|---|
| `mise run ci` | the whole gate, in order: fmt, typos, deny, clippy, tests, docs, scenarios |
| `mise run test` | `cargo nextest` plus doctests |
| `mise run lint` | clippy with warnings denied, `buf lint` |
| `mise run fmt` | `cargo fmt`, `buf format -w` |
| `mise run chaos:run` | every scenario under `scenarios/`, fresh stack each, exit 1 on failure |
| `mise run chaos:check` | parse and cross-check scenarios without running them |
| `mise run validate` | `chaos validate` against a running stack |
| `mise run watch:engine` | rebuild and restart on change (bacon); same for `watch:protocol` |

## Testing and CI

Three layers, all run by `mise run ci` and by GitHub Actions:

1. **Unit and integration tests** per crate. Integration tests boot the real servers on
   port 0; nothing is mocked.
2. **Scenarios** in `scenarios/`: chaos starts a stack, applies load, injects faults,
   asserts. The baseline scenario found a 40 ms Nagle stall in both servers before any
   user did; it now guards against it.
3. **Static gates**: rustfmt, clippy pedantic with warnings as errors, cargo-deny,
   typos, buf lint, rustdoc with warnings as errors.

Details, the local-to-CI mapping and how to fix each kind of failure: [docs/ci.md](docs/ci.md).

## The chaos tool

```sh
chaos validate [--protocol URL] [--engine URL] [--json]   # every surface, per-check pass/fail
chaos up [topology.toml]                                    # stack in one process until Ctrl-C
chaos run scenarios/*.toml | --dir scenarios [--json]       # load + faults + assertions
chaos check scenarios/*.toml                                # validate files only
```

A scenario is one TOML file: which engines and protocols to run, how much load of which
kind, what to break and when, what must be true afterwards. Faults are injected into the
real services through a handle they consult on every request, so there are no mocks.

Start with [docs/chaos/README.md](docs/chaos/README.md). Writing scenarios:
[docs/chaos/scenarios.md](docs/chaos/scenarios.md). Adding services or operations:
[docs/chaos/extending.md](docs/chaos/extending.md).

## Local cluster with Grafana

```sh
mise run local:up          # k3d on this machine, k3s 1.34, storage on the RAID
mise run local:build       # both images into the cluster
mise run local:deploy      # Envoy + services + VictoriaMetrics + Tempo + VictoriaLogs + collector + Grafana
mise run local:traffic     # some requests through Envoy
open http://localhost:3000 # admin / admin, dashboards tagged "tbd"
mise run k9s
```

`mise run ansible:local` does the same through Ansible. Dashboards live in
`devops/grafana/dashboards/`; `mise run grafana:reload` pushes edits in seconds. How the
signals flow and how to follow one request across Envoy, protocol and engine:
[docs/observability.md](docs/observability.md).

## Ship

```sh
mise run docker:build               # both images; IMAGE_REGISTRY / IMAGE_TAG from env
mise run up                         # compose stack on 50051 / 8080
mise run k8s:render                 # KUBE_OVERLAY=dev|prod
mise run ansible:bootstrap          # once per host
mise run ansible:deploy             # IMAGE_TAG to ANSIBLE_INVENTORY hosts
mise run ship                       # ci → docker:push → ansible:deploy
```

Before the first real deploy a human fills in the hosts in `devops/ansible/inventory/`
and registry credentials in an Ansible vault. See
[devops/README.md](devops/README.md).

## Layout

```
crates/
  common/     telemetry, shutdown, shared CLI flags, fault injection. Transport-free.
  proto/      generated gRPC code from proto/. No hand-written logic.
  engine/     the engine service (lib + bin + tests/it)
  protocol/   the protocol service (lib + bin + tests/it)
  chaos/      the chaos tool (lib + bin + tests/it)
proto/        .proto sources, buf STANDARD naming
scenarios/    chaos scenarios: load + timeline + assertions
topologies/   stacks for `chaos up`
devops/       docker/, envoy/, k8s/ (base, overlays, observability), grafana/, ansible/
docs/         ci.md, observability.md, chaos/, design/ (earlier idea material, not a spec)
compose.yaml  Envoy + services from the same images
```

[ARCHITECTURE.md](ARCHITECTURE.md) has the shape and the invariants. Each crate and the
`devops/` and `scenarios/` directories carry a `CLAUDE.md` with the non-obvious facts
about that directory: boundaries, invariants, gotchas. Written for AI assistants, and the
shortest accurate orientation for people. [docs/README.md](docs/README.md) indexes
everything.

## Conventions

- Rust 1.98.1 pinned, MSRV 1.96, edition 2024. Clippy pedantic with warnings denied,
  `unwrap` denied outside tests, `unsafe` forbidden.
- Protos follow buf's STANDARD rules: directory matches package, services end in
  `Service`, RPC messages are `<Rpc>Request` / `<Rpc>Response`.
- Errors: `thiserror` in libraries, `anyhow` at the binary edge. Logs: `tracing`.
- Every new env var goes on a clap flag with `env = ...` and into `.env.example`,
  `compose.yaml` and `devops/k8s/base/configmap.yaml`.
- Conventional Commits, one logical change per commit.
