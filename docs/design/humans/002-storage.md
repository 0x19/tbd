# 002 — Storage

Status: open — proposed for approval. Approval closes the storage line of Q6 in
[../001-open-questions.md](../001-open-questions.md).

Postgres is the ledger. Redis is the hot projection and the only home of presence.
ClickHouse arrives later, for analytics only.

---

## The decision

| Role | Store | Why this one |
|---|---|---|
| **Source of truth** | Postgres 17, its own instance with a pgvector image (`pgvector/pgvector:pg17`), separate from the id plane's per [id/README](../id/README.md) | `facts`, append-only except retraction ([000](000-facts-ledger.md)), plus materialised `facts_current`; transactions; foreign keys make erasure a cascade the commit proves; JSONB values; pgvector for embeddings until scale says otherwise. Postgres is already operated in the cluster (`devops/k8s/auth/postgres.yaml`). |
| **Hot path and presence** | Redis 7, new infrastructure | One hash per human with the current facts a reader may see, rebuilt from Postgres. Presence (current location) lives **only** here, with an expiry and an explicit sweep of the geo member. Candidate sets for fan-out. |
| **Analytics, later** | ClickHouse | Observed facts and outcomes streamed through an outbox. Cohorts, funnels, model evaluation over millions of rows. Never on a request path. |
| **Inputs and keys** | Postgres `inputs` (ciphertext as `bytea`) in v1, an object store when size says so; per-human keys as a separate, destroyable key class in the KMS that holds the id plane's pepper, which is never destroyed ([id/001](../id/001-uniqueness.md)) | One erasure mechanism for blobs: destroy the key. |

## Presence is not a fact

apex promised "never store location history" and kept it forever: `update_location`
added members to a Redis geo set with no expiry and `cleanup_stale` returned
`Ok(0)` with a comment that TTL handles it, which it does not for geo members
(`apex/backend/src/providers/location.rs:62,165-168`,
[../000-premise.md](../000-premise.md)). An append-only ledger would repeat that by
construction. So presence is a command the protocol forwards to the `humans`
service ([004](004-surface.md)), which writes Redis only: a key with `EXPIRE` for
the metadata and a sweep that removes geo members older than the retention. The
claim, as the [003](003-consent-and-erasure.md) scenario asserts it: after the
retention window, no location of a silent person exists in Redis; snapshots hold
it for their own retention, as the erasure table below states. Redis GEO over
PostGIS because the data must expire and the query must not touch the ledger;
PostGIS returns if the radius queries outgrow Redis.

## Why not ClickHouse as the ledger

The hot path is "every current fact about one human" and "one fact about one
human", point reads by key. ClickHouse's documentation answers directly: the short
answer is no, key-value workloads are among the top cases when not to use it,
because the sparse primary index points at every N-th row and a point query scans
from the nearest granule
([Can I use ClickHouse as a key-value storage?](https://clickhouse.com/docs/resources/support-center/knowledge-base/general-faqs/key-value)).
Row deletes are heavyweight mutations
([ALTER TABLE DELETE](https://clickhouse.com/docs/reference/statements/alter/delete))
or lightweight marks merged later
([DELETE](https://clickhouse.com/docs/reference/statements/delete)); transactions
are experimental, non-replicated MergeTree only, Keeper required
([Transactional support](https://clickhouse.com/docs/concepts/features/operations/insert/transactions)).
An erasure cannot be proven at commit there. It is the right tool for scans over
every outcome ever, which is how Q3 gets answered once there are outcomes.

## Why not SurrealDB or Redis-only

SurrealDB is provisioned on this host, reports unhealthy, and nothing depends on it
([../000-premise.md](../000-premise.md), "Infrastructure ran ahead of code").
Nothing in this plane needs a graph query a relations table cannot answer.
Redis-only was apex's choice and its only store; it failed on its first real
requirement, above. Redis stays a projection here, so losing it loses nothing but
presence, which is meant to be lost.

## The schema, in outline

```sql
humans (id uuid primary key, enrolled_at timestamptz,
        erased_at timestamptz null);                -- set at the request; unreadable while set

facts (
  human_id         uuid not null references humans(id) on delete cascade,
  id               bigint generated always as identity,
  path             text not null,
  source           text not null check (source in ('verified','declared','inferred','symbolic','observed')),
  value            jsonb,                          -- null is a tombstone
  origin           jsonb not null,
  confidence       real,
  counterparty_id  uuid null references humans(id) on delete cascade,
  observed_at      timestamptz not null,
  recorded_at      timestamptz not null default now(),
  consent          text[] not null,                -- registry scope ids
  stub             boolean not null default false,
  primary key (human_id, id)
);
fact_inputs   (human_id, fact_id, input_fact_id null, input_hash null)
               references facts(human_id, id) on delete cascade, inputs
facts_current (human_id, path, source, fact_id)   references facts(human_id, id) on delete cascade
inputs        (human_id references humans on delete cascade, content_hash, kind, ciphertext bytea)
outbox        (human_id, fact_id, published_at)    references facts(human_id, id) on delete cascade
erasures      (human_id, requested_at, executed_at null, published_at null)   -- no cascade; survives
```

The `humans` service tombstones an inferred fact whose last `fact_inputs` row goes,
in the same transaction as the retraction. The primary key is `(human_id, id)` so
that partitioning by `human_id` hash is possible later
([PostgreSQL 17, partitioning limitations](https://www.postgresql.org/docs/17/ddl-partitioning.html):
a primary key on a partitioned table must include the partition key); the table is
**not** partitioned in v1, because the targets below are index point reads and
partitioning does not help them. Converting later is a data move into a new
parent. Index `(human_id, path, source)` for the point path; `path` alone for the
batch jobs that run under `engine.base` ([003](003-consent-and-erasure.md)).

## Erasure across stores

The `erasures` row is written at the request and drives everything below; it is
the one row that survives the cascade, so Redis, ClickHouse and webhooks can be
verified after the `humans` row is gone.

| Store | What erasure does | Residue |
|---|---|---|
| Postgres | at the request: `erased_at` set, every read denied. At window end ([003](003-consent-and-erasure.md)): one transaction cascades from `humans` through `facts`, `counterparty_id`, `fact_inputs`, `facts_current`, `inputs`, `outbox`, and tombstones the surviving side of each relation | base backups and WAL for the backup retention, stated in the enrolment text |
| Redis | at the request: `DEL` of the hash, `ZREM` of the geo member | RDB or AOF snapshots for their retention |
| ClickHouse | lightweight `DELETE WHERE human_id = ?`, verified by the 003 scenario; proposed SLA 24 h | until the merge runs |
| KMS | the per-human key is destroyed at window end | none: ciphertext without a key |
| logs and traces | fact values never appear in them; ids do, for the log retention | ids only |

"Proven at commit" is true for Postgres and only there. Elsewhere it is asynchronous
and checked by the [003](003-consent-and-erasure.md) chaos scenario.

## Performance target

| Read | Target |
|---|---|
| all current facts for one human, from Redis | p99 under 2 ms |
| the same, from Postgres (cache miss) | p99 under 10 ms |
| candidates within radius times pair features, from Redis | p99 under 50 ms for 500 candidates |

Measured by chaos scenarios against the deployed stack, the way latency is measured
today (`docs/chaos/scenarios.md`, `max_p99_ms`).

## Unrecoverable if wrong

The primary key shape; the decision that presence never enters the ledger; and the
`erasures` record surviving the cascade, without which nothing records what still
needs deleting elsewhere. All cheap now and a rewrite of every row later.
