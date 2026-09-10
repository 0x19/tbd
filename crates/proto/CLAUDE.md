# crates/proto

Generated code only. Never add hand-written logic here; wrap generated types in the
crate that owns the behaviour.

- `build.rs` compiles every file listed in `files` under `/proto` with `protox` (pure
  Rust, no `protoc`) and `tonic-prost-build`. It also writes one descriptor set per
  package (`engine_descriptor.bin`, `protocol_descriptor.bin`) for gRPC reflection,
  because `compile_fds` does not write them itself.
- `lib.rs` exposes `engine::v1` and `protocol::v1` with `DESCRIPTOR_SET` constants.
  Lints are allowed wholesale in this crate; the generated code is not ours to style.

Gotchas:
- Adding a proto file means adding it to `files` in `build.rs` **and** deciding which
  descriptor set it belongs to. A file missing from `files` compiles nothing and fails
  silently until something references it.
- Protos follow buf STANDARD, enforced by `mise run lint`: directory matches package
  (`proto/tbd/<pkg>/v1/`), services end in `Service`, RPC messages are `<Rpc>Request`
  and `<Rpc>Response`, distinct even for bidirectional streams. Generated module names
  follow: `engine_service_server`, `engine_service_client`.
- If `cargo check` ever reports a missing `DESCRIPTOR_SET` after a change here, cargo
  is holding a stale check artifact: `cargo clean -p tbd-proto`.
