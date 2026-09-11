# ledger

The append-only facts store the humans plane is built on
([docs/design/humans/000](../design/humans/000-facts-ledger.md)): a `Store` trait with a
Postgres backend (the source of truth, embedded sqlx migrations) and an in-memory
backend (tests, chaos stacks, host runs without a database), both proven by one
conformance suite; an outbox drained into ClickHouse; a sweeper for erasures; and a
thin gRPC surface over it all. Nothing here is a stub any more. Encryption
([005](../design/humans/005-encryption.md)) is the next phase; the `humans` service
embeds this crate and owns the JSON surface and the path registry.

| Piece | Where |
|---|---|
| Crate, scaffolded by `tbd new service ledger` | `crates/ledger` ([CLAUDE.md](../../crates/ledger/CLAUDE.md)) |
| Contract | `proto/tbd/ledger/v1/ledger.proto`: `LedgerService` with `Ping`, `Append`, `Current`, `History`, `Retract`, `Erase`, `Restore` |
| Config layers | `configs/ledger/{base,local,dev,production}.toml`; `ledger config` prints the merged result |
| Deployment | `devops/k8s/base/ledger`, port 50052, metrics 9464; reached through Envoy's internal listener (`http://envoy:50051`, matched by service name); no edge route |
| Databases | `devops/k8s/ledger-db`: Postgres 17 + pgvector and ClickHouse, applied by `mise run ledger:deploy` (part of `local:deploy`) after `ledger:secrets` made the `ledger-db` Secret; `ledger:psql` and `ledger:clickhouse` for a shell; compose runs the same two on host ports 15432 and 18123 |
| Chaos | the `ledger` kind (`crates/chaos/src/kinds/ledger.rs`): `[stack.ledgers.X]` in topologies and scenarios (`grace`, `database_url`), `chaos validate --target ledger=URL` (`CHAOS_LEDGER_URL`), the `grpc_ledger_ping` and `grpc_ledger_facts` checks, the `ledger_*` load operations and the five `scenarios/ledger_*.toml` (baseline, lifecycle, fault, erasure, fuzz; [docs/chaos/scenarios.md](../chaos/scenarios.md)), add and clone in the admin UI |

## The gRPC contract

`proto/tbd/ledger/v1/ledger.proto`. The subject id on the wire is the ledger's opaque
uuid (the `humans` service maps `sub` to it; nothing here knows a person). Every RPC
runs through fault injection first, then the store; store errors map to codes as in
the table below. `scopes` is required and non-empty on every read: the gRPC surface
never performs an unfiltered read.

| RPC | Request | Answer |
|---|---|---|
| `Ping` | `message` | echo, `version`, `store` (`postgres` / `memory`), `stub: false` |
| `Append` | `subject_id`, `path`, `source`, `value` and `origin` (envelopes: `version` 0 = plaintext JSON bytes), `confidence?`, `counterparty_id?`, `observed_at`, `expires_at?`, `consent[]`, `stub`, `idempotency_key?` | the `Fact` as held (`id`, `recorded_at` minted by the store) and `replayed` when the key matched an earlier append |
| `Current` | `subject_id`, `paths[]` (`traits.warmth` or `traits.*`), `sources[]`, `scopes[]`, `cursor`, `limit` (0 = 100, max 1000) | the latest valued, unexpired fact per `(path, source)`, ordered by `(recorded_at, id)`; `next` cursor when there is more |
| `History` | the same plus `at` | everything still held, tombstones (no `value`) included; `at` cuts at that instant |
| `Retract` | `subject_id`, `path`, `source`, `origin` | the tombstone |
| `Erase` | `subject_id` | `requested_at`, `executes_after` (plus the grace window); idempotent within the window |
| `Restore` | `subject_id` | empty; `NOT_FOUND` once executed |

| Store outcome | gRPC code |
|---|---|
| not found (subject, fact, counterparty) | `NOT_FOUND` |
| erased (pending or executed) | `FAILED_PRECONDITION` |
| invalid (path shape, confidence, consent, envelope, cursor, limit, ids) | `INVALID_ARGUMENT` |
| the registry refuses the source on the path | `PERMISSION_DENIED` |
| idempotency key reused with other content, or its fact retracted since | `ABORTED` |
| the store unreachable | `UNAVAILABLE` |
| injected faults | `UNAVAILABLE`, `INTERNAL`, `RESOURCE_EXHAUSTED`, `DEADLINE_EXCEEDED` |

```sh
mise run run:ledger                                    # on 127.0.0.1:50052 with configs/ledger local (LEDGER_STORE_KIND=memory without a database)
grpcurl -plaintext -d '{"message":"hi"}' localhost:50052 tbd.ledger.v1.LedgerService/Ping
S=$(uuidgen | tr A-F a-f)
grpcurl -plaintext -d "{\"subject_id\":\"$S\",\"path\":\"profile.name\",\"source\":\"SOURCE_DECLARED\",
  \"value\":{\"version\":0,\"bytes\":\"$(printf '"Ada"' | base64)\"},\"origin\":{\"version\":0,\"bytes\":\"$(printf '{}' | base64)\"},
  \"observed_at\":\"$(date -u +%Y-%m-%dT%H:%M:%SZ)\",\"consent\":[\"self\"]}" localhost:50052 tbd.ledger.v1.LedgerService/Append
grpcurl -plaintext -d "{\"subject_id\":\"$S\",\"scopes\":[\"self\"]}" localhost:50052 tbd.ledger.v1.LedgerService/Current
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
envelopes (`version` + bytes), version 0 being labelled plaintext JSON, each at most
64 KiB (a fact is a claim, not a document; the cap is what bounds a page, an outbox
batch and the server's memory). Errors map to gRPC as: not found → `NotFound`, erased →
`FailedPrecondition`, invalid → `InvalidArgument`, forbidden by the registry →
`PermissionDenied`, conflict → `Aborted`, unavailable → `Unavailable`. A request over
256 KiB is `OutOfRange` from the codec before any of it is buffered.

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

## Outbox, analytics, sweeper, readiness

Every write leaves an event in the outbox in the same transaction: `fact.recorded`,
`fact.retracted` (carrying the tombstone) and, from the erasure record, `subject.erased`.
The payload is clear columns only (`path`, `source`, `tombstone`, `confidence`,
`counterparty_id`, `observed_at`, `recorded_at`, `expires_at`, `consent`, `stub`); a
value or an origin never leaves the ledger this way, and a test asserts it.

The **drainer** (`[analytics]`) leases a batch (`batch`, for `lease`), hands it to the
sink, acks it, and sleeps `period` when the outbox is empty. At least once: a batch the
sink refuses stays leased and is shipped again after the lease; consumers dedupe on
`event_id`. Two replicas never share rows. With `LEDGER_CLICKHOUSE_URL` set the sink is
ClickHouse: on start it creates `ledger.facts_events` (`MergeTree`, ordered by
`(subject_id, recorded_at, event_id)`, partitioned by month), fact events are one
batched insert, and a `subject.erased` event first runs a lightweight
`DELETE ... WHERE subject_id = ?` (waited on) and then inserts the erased event row, so
the deletion can be verified afterwards (`SELECT count() ... SETTINGS
apply_deleted_mask = 0` after `OPTIMIZE ... FINAL` is what the test does). With the URL
empty, events are counted and dropped: analytics off, nothing else changes.

The **sweeper** (`[erasure]`) runs every `sweep_interval`: it executes the erasures whose
`grace` has passed (`batch` per pass) and purges idempotency keys older than
`[idempotency] ttl`. Safe on any number of replicas.

**Readiness** (`[health]`) follows the store: one probe before the listener accepts,
then one every `probe_interval`; a probe slower than `probe_timeout` or failing marks
the gRPC health status NOT_SERVING (the readiness gate in Kubernetes) and
`tbd_ledger_store_up` 0. The memory store is always up.

Metrics: the dashboard **tbd / Ledger** ([docs/observability/dashboards.md](../observability/dashboards.md))
reads them all; every store call is timed and counted by `store::Instrumented`
(`tbd_ledger_store_op_duration_seconds{op}`, `tbd_ledger_store_ops_total{op,result}`), facts,
replays, retractions, envelope and page sizes, erasures and their backlog, the outbox's
pending count, age and lag, and rows and bytes per table are recorded next to
`tbd_ledger_store_up`, `tbd_ledger_outbox_batches_total{status}`,
`tbd_ledger_outbox_events_total{kind}`, `tbd_ledger_erasures_executed_total`,
`tbd_db_pool_connections{state}` ([docs/observability/metrics.md](../observability/metrics.md)).

## Deployment

In the cluster the ledger pod gets `LEDGER_STORE_KIND=postgres` from the ConfigMap and
both URLs from the `ledger-db` Secret, which `mise run ledger:secrets` creates once with
random passwords (pass a Postgres URL and a ClickHouse URL to use managed instances, as
production does; the prod overlay carries no databases). `ledger:deploy` applies the
StatefulSets and waits for them; `local:deploy` and `local:restart` include that, so a
fresh cluster comes up with its databases before the ledger. Readiness follows the
store: scale `ledger-postgres` to zero and the ledger pod turns NOT READY within a probe
interval, and back. Migrations run when the ledger starts (`migrate_on_start`, on by
default) under an advisory lock, so two replicas cannot race; production sets it off
and runs `ledger migrate` before the rollout. Compose runs the same two databases; a
host run without any (`mise run run:ledger`) sets `LEDGER_STORE_KIND=memory` in `.env`.

Lines for `.env.example` (the file is edited by hand):

```
LEDGER_STORE_KIND=memory                     # postgres in compose and the cluster
#LEDGER_DATABASE_URL=postgres://ledger:local-ledger@127.0.0.1:15432/ledger?sslmode=disable
#LEDGER_CLICKHOUSE_URL=http://ledger:local-ledger@127.0.0.1:18123
LEDGER_POSTGRES_PASSWORD=local-ledger        # compose only
LEDGER_CLICKHOUSE_PASSWORD=local-ledger      # compose only
#LEDGER_TEST_DATABASE_URL=                   # tests: an admin URL instead of Docker
#LEDGER_TEST_CLICKHOUSE_URL=
```

## What comes next

The `humans` service on top (the path registry, `sub` resolution, the JSON surface,
the Redis projection), then encryption ([005](../design/humans/005-encryption.md)).
