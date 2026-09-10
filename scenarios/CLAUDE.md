# scenarios

Chaos scenarios run by `mise run chaos:run` and by CI. Each file is a stack, load, a
timeline and assertions; the full reference is `docs/chaos/scenarios.md`.

Rules for files here:
- No fixed `listen` addresses. Scenarios run on free ports so CI jobs and a dev stack
  never collide. Fixed ports belong in `topologies/`.
- Keep runs short: warmup under a second, load a few seconds. The whole directory runs
  on every `mise run ci`.
- Derive every bound from what the scenario injects, then leave headroom for CI runners.
  Write the arithmetic in a comment next to the bound (see `error_injection.toml`).
- One fault per scenario, named after it. A scenario that proves two things is two
  files.
- The header comment states what the scenario proves, in one or two sentences. That is
  what a reader sees first when it fails.
- Run `mise run chaos:check` after editing; it rejects unknown keys and dangling
  service names before anything starts.

`baseline.toml` is the regression guard for latency: it asserts p99 under 50 ms because
it once caught a 40 ms Nagle stall. Do not loosen it to make a slow box pass; find out
why the box is slow.
