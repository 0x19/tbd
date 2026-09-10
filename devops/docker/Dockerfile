# syntax=docker/dockerfile:1.7
#
# One Dockerfile for every workspace binary. Select with --build-arg BIN=engine|protocol|chaos.
#
#   docker build -f devops/docker/Dockerfile --build-arg BIN=engine   -t ghcr.io/0x19/tbd-engine   .
#   docker build -f devops/docker/Dockerfile --build-arg BIN=protocol -t ghcr.io/0x19/tbd-protocol .
#   docker build -f devops/docker/Dockerfile --build-arg BIN=chaos --build-arg PORT=7700 -t ghcr.io/0x19/tbd-chaos .
#
# The runtime working directory is /app and holds configs/, scenarios/ and topologies/,
# so chaos finds its layered config and scenario files at their default relative paths.
# If ui/chaos/out exists in the build context it is copied to /app/ui (the admin UI).
#
# Stages: chef (toolchain + cargo-chef) -> planner (recipe.json) -> cacher (deps only)
#         -> builder (workspace) -> runtime (distroless).
# Dependencies are rebuilt only when Cargo.toml/Cargo.lock change, not on every source edit.
# Proto files are compiled by crates/proto/build.rs with protox, so no protoc is installed here.

ARG RUST_VERSION=1.98.1

# ---------------------------------------------------------------------------
FROM rust:${RUST_VERSION}-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /app

# ---------------------------------------------------------------------------
FROM chef AS planner
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo
COPY crates ./crates
COPY proto ./proto
RUN cargo chef prepare --recipe-path recipe.json

# ---------------------------------------------------------------------------
FROM chef AS cacher
ARG BIN=engine
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --locked --release -p tbd-${BIN} --recipe-path recipe.json

# ---------------------------------------------------------------------------
FROM chef AS builder
ARG BIN=engine
COPY --from=cacher /app/target ./target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
COPY Cargo.toml Cargo.lock ./
COPY .cargo ./.cargo
COPY crates ./crates
COPY proto ./proto
RUN cargo build --locked --release -p tbd-${BIN} \
 && cp target/release/${BIN} /usr/local/bin/app

# ---------------------------------------------------------------------------
# Runtime: distroless cc (glibc + libgcc, no shell, no package manager, non-root uid 65532).
# There is no curl/wget inside, so container-level HEALTHCHECK is not possible; liveness and
# readiness are the orchestrator's job (k8s gRPC/HTTP probes, compose healthchecks documented
# in compose.yaml). The trade-off is accepted: a smaller attack surface beats an in-image probe.
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
ARG BIN=engine
ARG PORT=50051
ENV LOG_FORMAT=json \
    RUST_LOG=info \
    BIN=${BIN}
WORKDIR /app
COPY --from=builder /usr/local/bin/app /usr/local/bin/app
COPY configs ./configs
COPY scenarios ./scenarios
COPY topologies ./topologies
# Optional: the built admin UI. The glob on the first segment keeps the COPY valid when absent.
COPY u[i]/chaos/out ./ui
USER nonroot:nonroot
EXPOSE ${PORT}
ENTRYPOINT ["/usr/local/bin/app"]
