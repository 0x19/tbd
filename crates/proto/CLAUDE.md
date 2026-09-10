# crates/proto

Generated code only. Never add hand-written logic here; wrap generated types in the
crate that owns the behaviour.

- `build.rs` discovers every `proto/tbd/<name>/v1/*.proto` at build time and compiles
  them with `protox` (pure Rust, no `protoc`) and `tonic-prost-build`. It also writes
  one descriptor set per package (`engine_descriptor.bin`, `protocol_descriptor.bin`,
  ...) for gRPC reflection, because `compile_fds` does not write them itself. The
  file list and the proto root are resolved when the script runs, never with `env!`:
  `mise run tbd:selfcheck` builds a copy of the tree into the shared target dir and
  Cargo reuses one build-script binary for a path package wherever it lives.
- `lib.rs` exposes `engine::v1` and `protocol::v1` with `DESCRIPTOR_SET` constants.
  Lints are allowed wholesale in this crate; the generated code is not ours to style.

Gotchas:
- Adding a proto file means putting it under `proto/tbd/<name>/v1/` and adding
  `pub mod <name>` with its `DESCRIPTOR_SET` to `lib.rs` (`tbd new service` does both).
  The descriptor set is per `<name>` directory.
- Protos follow buf STANDARD, enforced by `mise run lint`: directory matches package
  (`proto/tbd/<pkg>/v1/`), services end in `Service`, RPC messages are `<Rpc>Request`
  and `<Rpc>Response`, distinct even for bidirectional streams. Generated module names
  follow: `engine_service_server`, `engine_service_client`.
- If `cargo check` ever reports a missing `DESCRIPTOR_SET` after a change here, cargo
  is holding a stale check artifact: `cargo clean -p tbd-proto`.
