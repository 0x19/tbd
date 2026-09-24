# crates/sandboxd

The sandbox engine on the host (RFC 0010; `docs/sandbox/README.md` is the contract). Not
a platform service: no gRPC, not in the cluster, not scaffolded; the runner service is
its L2.

- `recipe.rs` is the only place a container option or a step's command is written. A
  request never reaches either: `Language` picks a fixed row, and the source and input
  go on standard input only. Its tests assert every restriction is present and that
  nothing widening one (`--privileged`, a volume, a device, an added capability) ever
  is. A change here is a change to RFC 0010's defence table, and gets the escape suite
  run (`mise run sandbox:escape`).
- `run.rs`: one sandbox per run, each step a `docker exec` with its own deadline and
  output caps. A deadline or a cap removes the container (killing the `docker` client
  would leave the program running). `Sandbox` removes the container on drop, so a
  caller that goes away leaves nothing. `classify` names what an exit status means
  (128 the sandbox ended at a limit, 137 memory, 153 file size).
- `server.rs`: axum; the token compared in constant time and never empty; busy is `429`
  at once (the runner queues).
- `config.rs`: every bound in `configs/sandboxd/base.toml`, the container's included.

Tests: unit tests run everywhere; `tests/it` needs gVisor and the images and is
`#[ignore]`d (`mise run sandbox:test`, `mise run sandbox:escape`).
