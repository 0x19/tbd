# ledger

The append-only facts store the humans plane is built on
([docs/design/humans/000](../design/humans/000-facts-ledger.md)). The store is built:
a `Store` trait with a Postgres backend (the source of truth, embedded sqlx migrations)
and an in-memory backend (tests, chaos stacks, host runs without a database), both
proven by one conformance suite. The gRPC surface still answers only `Ping`, which says
`stub` on the wire; the facts RPCs land next, encryption
([005](../design/humans/005-encryption.md)) after that.

| Piece | Where |
|---|---|
| Crate, scaffolded by `tbd new service ledger` | `crates/ledger` ([CLAUDE.md](../../crates/ledger/CLAUDE.md)) |
| Contract | `proto/tbd/ledger/v1/ledger.proto`: `LedgerService.Ping` |
| Config layers | `configs/ledger/{base,local,dev,production}.toml`; `ledger config` prints the merged result |
| Deployment | `devops/k8s/base/ledger`, port 50052, metrics 9464; reached through Envoy's internal listener (`http://envoy:50051`, matched by service name); no edge route |
| Chaos | the `ledger` kind (`crates/chaos/src/kinds/ledger.rs`): `[stack.ledgers.X]` in topologies and scenarios, `chaos validate --target ledger=URL` (`CHAOS_LEDGER_URL`), the `grpc_ledger_ping` check, add and clone in the admin UI |

## The contract today

```proto
rpc Ping(PingRequest) returns (PingResponse);
message PingResponse { string message = 1; string version = 2; bool stub = 3; }
```

`stub` is `true` and stays so until a real RPC lands. A caller may rely on that field
the way it relies on `stub` in the engine's scores: a placeholder never looks like a
result.

```sh
mise run run:ledger                                    # on 127.0.0.1:50052 with configs/ledger local
grpcurl -plaintext -d '{"message":"hi"}' localhost:50052 tbd.ledger.v1.LedgerService/Ping
# through the cluster's internal listener (reflection there is the engine's, so pass the proto)
grpcurl -plaintext -import-path proto -proto tbd/ledger/v1/ledger.proto \
  -d '{"message":"hi"}' localhost:15051 tbd.ledger.v1.LedgerService/Ping
```

## The store

`crates/ledger/src/store/mod.rs` is the crate's API, a trait two backends implement:

| Call | What it does |
|---|---|
| `append(subject, NewFact)` | one fact; the subject is created on its first write; an `idempotency_key` seen before (per subject, for `[idempotency] ttl`) replays the earlier fact, and reused with different content is a conflict |
| `current(subject, Query)` | the latest valued, unexpired fact per `(path, source)`, filtered by path patterns (`traits.warmth` or `traits.*`), sources and consent scopes, paged by an opaque cursor |
| `history(subject, Query)` | everything still held, tombstones included; `at` returns what was held at that instant (a retracted value is absent from every cut, its tombstone visible from its own `recorded_at`) |
| `retract(subject, path, source, origin)` | physically deletes the valued rows of `(subject, path, source)` and appends a tombstone with the latest value's consent, in one transaction |
| `request_erasure(subject)` | sets `erased_at` and records the erasure; every read and write answers `Erased` from that commit; idempotent within the window |
| `restore(subject)` | cancels a pending erasure and reopens the subject; `NotFound` once executed |
| `execute_due_erasures(grace, limit)` | the sweeper: for each erasure past its window, deletes the subject and everything under it in one transaction, tombstones the surviving side of every relation (`counterparty_id` null, origin `{"cause":"counterparty_erased"}`), and keeps the erasure record to publish `subject.erased` |
| `claim_events` / `ack_events` | the outbox lease: unpublished events (clear columns only, never a value) claimed for a lease, acked once shipped; at least once |

A fact is `{subject_id, id, path, source, value, origin, confidence, counterparty_id,
observed_at, recorded_at, expires_at, consent, stub}`; `value` and `origin` are
envelopes (`version` + bytes), version 0 being labelled plaintext JSON. Errors map to
gRPC as: not found → `NotFound`, erased → `FailedPrecondition`, invalid →
`InvalidArgument`, forbidden by the registry → `PermissionDenied`, conflict → `Aborted`,
unavailable → `Unavailable`.

**Backends.** `[store] kind = "postgres"` is the real one: `LEDGER_DATABASE_URL`
(environment only, never a file, never printed), a pool of `max_connections`,
migrations from `crates/ledger/migrations/` applied at start under sqlx's advisory lock
when `migrate_on_start` is true, or by `ledger migrate` (production runs that before the
rollout). `kind = "memory"` is the same contract under one mutex, nothing surviving the
process; chaos stacks and tests use it, and a host run without a database sets
`LEDGER_STORE_KIND=memory`. Neither is a stub: `Ping` reports which one answers.

**Ordering.** `recorded_at` is minted by the store after the subject's row lock
(`clock_timestamp()`, not `now()`), so `(recorded_at, id)` strictly increases per
subject; it is the order of every page and the cursor. Expired facts are excluded from
`current` at read time and kept in history.

**Schema** (`migrations/0001_ledger.sql`): `subjects`, `facts` (PK `(subject_id, id)`,
indexes on `(subject_id, recorded_at, id)`, `(subject_id, path, source)`, `path`,
`counterparty_id`), `facts_current`, `erasures` (no foreign key: the row that survives
the cascade), `outbox` (event rows with a lease), `idempotency`. Where this differs
from [002](../design/humans/002-storage.md): the table is `subjects`, not `humans` (the
ledger's subject is opaque); the outbox holds event rows with `kind`, `payload`,
`claimed_until` rather than `(human_id, fact_id, published_at)`, because a retracted
fact's `fact.recorded` event must outlive the fact and the lease is what makes
at-least-once work; `erasures` gains `event_id`, `cancelled_at` and `claimed_until`;
expiry is excluded at read time rather than by a job; caller-scoped idempotency and the
real path registry are the `humans` service's (the ledger validates shape only and
scopes keys per subject); `Query.scopes = None` (unfiltered) exists for the in-process
caller only.

**Tests.** `crates/ledger/tests/it/conformance.rs` is the contract, generic over the
backend, run on the memory store always and on Postgres in `pg::*` against a real
server: a container each test starts through Docker, or the server named by
`LEDGER_TEST_DATABASE_URL` (an admin URL; every test creates its own database), which is
what CI's `services:` block provides. `pg::cascade_leaves_zero_rows_except_the_erasure`
is the proof the design asks for.

## What comes next

The outbox drainer into ClickHouse, the erasure sweeper and readiness on the store;
then `Append`, `Current`, `History`, `Retract`, `Erase` and `Restore` over gRPC; then
the dedicated Postgres and ClickHouse in the cluster; then the `humans` service on top.
