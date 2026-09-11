<!-- tbd new service ledger --kind grpc --port 50052 --metrics-port 9466 --bacon-key l (tbd-cli 0.1.0) -->
# crates/ledger

The facts ledger: append-only facts about an opaque subject, with provenance,
tombstones and erasure (`docs/design/humans/`), behind a `Store` trait with two
backends, and a thin gRPC service on top. Scaffolded by `tbd new service`
(docs/tbd/README.md); the crate has evolved past the template, which `tbd service check`
reports as `diverged`. The contract is `docs/ledger/README.md`.

- `store/mod.rs`: the crate's API. `Store` (append, current, history, retract,
  request_erasure, restore, execute_due_erasures, claim_events, ack_events,
  purge_idempotency), the types (`Fact`, `NewFact`, `Envelope`, `Query`, `Page`,
  `Cursor`, `PathPattern`, `OutboxEvent`), `StoreError` and its gRPC mapping. No
  driver type appears here: a gRPC-backed implementation could satisfy it unchanged.
- `store/validate.rs` and `store/registry.rs`: write and query validation every backend
  runs first; the path registry is injected (`Registry`), phase 1 ships `ShapeOnly`.
- `store/memory.rs`: `MemoryStore`, one mutex, a monotonic microsecond clock
  (`store/clock.rs`, `ManualClock` for tests). Not a stub: the same contract as Postgres,
  nothing survives the process. Chaos stacks and host runs without a database use it.
- `lib.rs`: `serve`, `serve_on`, `serve_with` (the store comes from `[store]`),
  `serve_store` (an explicit store). `config.rs`: `[store]`, `[analytics]`, `[erasure]`,
  `[idempotency]`, `[health]`; `Config::in_memory(addr)` for embedders;
  `Config::validate()` after flags. The URLs (`LEDGER_DATABASE_URL`,
  `LEDGER_CLICKHOUSE_URL`) are environment only and never serialised.
- `service.rs`: the `LedgerService` impl on `Ledger`. Every RPC starts with `admit()`:
  count the request, start the `RequestTimer`, apply the fault handle, map a `Fault`
  to a gRPC status. `main.rs`: the only file that prints (the `config` subcommand).

Invariants:
- Retraction physically deletes the valued rows and appends a tombstone in the same
  transaction; a retracted value is absent from every cut of history. Erasure denies
  every read and write from the request, cascades at window end, and the erasure
  record survives to publish `subject.erased`. The conformance suite asserts both:
  a privacy claim is a test.
- `recorded_at` is minted by the store after the subject lock and is strictly
  increasing per subject; `(recorded_at, id)` is the order and the cursor.
- The outbox payload carries clear columns only, never a value or an origin.
- `PingResponse.stub` stays `true` until the facts RPCs land; `TCP_NODELAY` is set on
  `TcpIncoming`, not the builder; callers reach this service through Envoy's internal
  listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it/conformance.rs` is the store contract, generic over the backend, run
through the `conformance_suite!` macro (`memory` in `tests/it/main.rs`); `main.rs` boots
the gRPC server on port 0 through `support.rs` with the shipped `configs/ledger` and env
`local`. Never enumerate a backend's behaviour outside the suite: a case added there
runs on every store.
