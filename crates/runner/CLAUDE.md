<!-- tbd new service runner --kind grpc --port 50060 --metrics-port 9473 --bacon-key u (tbd-cli 0.1.0) -->
# crates/runner

The runner service: the sandbox's L2 (RFC 0010; `docs/sandbox/README.md` is the
contract, including the order of the checks a run passes). gRPC only; the gateway
renders `Run` and `ListLanguages` as REST and on the socket. `Ping` stays the scaffold's
labelled stub.

- `service.rs`: `Run` is fault handle, caller and role, bounds, allowance, slot, engine,
  in that order, so a refused request spends nothing and never reaches the sandbox. The
  `audit` line carries sizes and outcomes, never the source or the output
  (`tests/it/runs.rs` fails if it does).
- `engine.rs`: `Engine` with `Sandboxd` (HTTP, bearer token, its `Debug` never prints
  the token) and `Stub` (markers `<<compile_error>>`, `<<timeout>>`, `<<busy>>`,
  `<<down>>`, `<<hang>>`; always `stub: true` upstream). `build` refuses `sandboxd`
  without a token; `main` refuses the stub in production.
- `gate.rs`: `Gate` (slots and a bounded line) and `Allowance` (runs per caller per UTC
  day, in memory).
- `config.rs`: `[engine]` (never from a flag), `[access]`, `[admission]`, `[budget]`,
  `[limits]`; `sandbox_token` is `#[serde(skip)]`, so `runner config` cannot print it.
  `Config::stub` is what tests and the chaos kind run.

Tests: `tests/it/runs.rs` on the stub (caller, role, bounds, allowance, busy, the audit
line) and against a wiremock playing `sandboxd` (the token, the answer, 429/401/503).
