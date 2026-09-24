# The sandbox's Rust toolchain (docs/sandbox/README.md, RFC 0010). Built locally by
# `mise run sandbox:images`, run only by sandboxd under gVisor, never pushed.
# Pinned by digest: a new toolchain is a deliberate change. `rustc` is called
# directly, never Cargo: no build scripts, no macros from elsewhere, no crates.
FROM rust:1.98.1-slim@sha256:f47a8de237dcbb0b0ce1099901e60a89728e3d51f24e664b40e947171538ade7

# Nothing runs as root in the sandbox; the recipe also sets the user.
USER 65534:65534
WORKDIR /work
