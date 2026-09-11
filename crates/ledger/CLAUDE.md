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
- `store/pg.rs`: `PgStore` on sqlx (0.9, rustls/ring, `query_as` + `FromRow`, no
  macros, so no database at build time). One transaction per write; the subject row is
  locked `FOR NO KEY UPDATE` first and `recorded_at` is `clock_timestamp()` after the
  lock; lock order is the `erasures` row, survivors' `subjects` rows in id order, then
  the subject. `MIGRATOR` embeds `migrations/` (`build.rs` reruns on change; `*.sql` is
  LF by `.gitattributes` because the checksum is over the bytes). `store/sql.rs`
  translates path patterns into `= any` / `like any` with `\ % _` escaped, checked
  against `PathPattern::matches` by a property test.
- `outbox.rs`: `Drainer` (claim → publish → ack, at least once) over a `Publisher`;
  `RecordingPublisher` when analytics is off and in tests. `clickhouse.rs`: the
  ClickHouse sink (`ledger.facts_events`, DDL on start, lightweight DELETE on
  `subject.erased`). `sweeper.rs`: due erasures and idempotency purge on a timer.
  `health.rs`: the readiness probe loop that drives the gRPC health status.
- `lib.rs`: `serve`, `serve_on`, `serve_with` (the store comes from `[store]`),
  `serve_store` (an explicit store); it spawns the health, sweeper, drainer and
  pool-gauge tasks on a `CancellationToken` tied to the shutdown future. `config.rs`: `[store]`, `[analytics]`, `[erasure]`,
  `[idempotency]`, `[health]`; `Config::in_memory(addr)` for embedders;
  `Config::validate()` after flags. The URLs (`LEDGER_DATABASE_URL`,
  `LEDGER_CLICKHOUSE_URL`) are environment only and never serialised.
- `service.rs`: the `LedgerService` impl on `Ledger`, a thin adapter: every RPC runs
  through `call()`, which does `admit()` (count the request, start the `RequestTimer`,
  apply the fault handle) and maps every `StoreError` to a status in one place
  (`status_of`). Wire types are translated, nothing is decided here; `scopes` must be
  non-empty on the wire. `main.rs`: the only file that prints (`config`, `migrate`).

Invariants:
- Retraction physically deletes the valued rows and appends a tombstone in the same
  transaction; a retracted value is absent from every cut of history. Erasure denies
  every read and write from the request, cascades at window end, and the erasure
  record survives to publish `subject.erased`. The conformance suite asserts both:
  a privacy claim is a test.
- `recorded_at` is minted by the store after the subject lock and is strictly
  increasing per subject; `(recorded_at, id)` is the order and the cursor.
- The outbox payload carries clear columns only, never a value or an origin.
- Envelopes are capped (`validate::MAX_ENVELOPE_LEN`, 64 KiB each) and the server
  decodes at most `MAX_MESSAGE_LEN` (256 KiB, `OutOfRange` above it): the ledger's
  memory is bounded by its own limits, not by what a client sends.
- Nothing is a stub: `PingResponse.stub` is `false` and `store` names the backend.
  `TCP_NODELAY` is set on `TcpIncoming`, not the builder; callers reach this service
  through Envoy's internal listener (`http://envoy:50051`, matched by service name).

Tests: `tests/it/conformance.rs` is the store contract, generic over the backend, run
through the `conformance_suite!` macro: `memory::*` in `tests/it/main.rs`, `pg::*` in
`tests/it/pg.rs` against a real Postgres (a `pgvector` container each test starts
through `testcontainers`, or the admin URL in `LEDGER_TEST_DATABASE_URL`, which is what
CI's services block sets; a database per test). Never enumerate a backend's behaviour
outside the suite: a case added there runs on every store. `clickhouse::*` drives the
sink against a real ClickHouse the same way (`LEDGER_TEST_CLICKHOUSE_URL` or a
container; ClickHouse logs to files, so readiness is the first DDL call succeeding).
`main.rs` boots the gRPC server on port 0 through `support.rs` with the shipped
`configs/ledger` and env `local`. Docker-less machines: `mise run test` skips
`pg::`/`clickhouse::` with a warning.
