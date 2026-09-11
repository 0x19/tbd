-- The facts ledger, phase 1 (docs/design/humans/002-storage.md, docs/ledger/README.md).
-- The subject is opaque: this schema knows nothing about people.

create extension if not exists vector;

create table subjects (
    id           uuid        primary key,
    enrolled_at  timestamptz not null default clock_timestamp(),
    erased_at    timestamptz null                       -- set at the erasure request; every read and write denied while set
);

create table facts (
    subject_id       uuid        not null references subjects(id) on delete cascade,
    id               bigint      generated always as identity,
    path             text        collate "C" not null,
    source           text        not null check (source in ('verified','declared','inferred','symbolic','observed')),
    value            bytea       null,                  -- envelope; null is a tombstone
    origin           bytea       not null,              -- envelope, same version as value
    envelope_version smallint    not null default 0 check (envelope_version >= 0),   -- 0 = plaintext JSON
    confidence       real        null check (confidence >= 0 and confidence <= 1),
    counterparty_id  uuid        null references subjects(id) on delete cascade,
    observed_at      timestamptz not null,
    recorded_at      timestamptz not null default clock_timestamp(),   -- minted after the subject lock
    expires_at       timestamptz null,
    consent          text[]      not null check (cardinality(consent) > 0),
    stub             boolean     not null default false,
    primary key (subject_id, id)                        -- partitionable by subject_id later
);
create index facts_subject_recorded_idx    on facts (subject_id, recorded_at, id);
create index facts_subject_path_source_idx on facts (subject_id, path, source);
create index facts_path_idx                on facts (path);
create index facts_counterparty_idx        on facts (counterparty_id) where counterparty_id is not null;

-- The latest fact per (subject, path, source), maintained in the same
-- transaction as every append and retraction.
create table facts_current (
    subject_id  uuid   not null,
    path        text   collate "C" not null,
    source      text   not null,
    fact_id     bigint not null,
    primary key (subject_id, path, source),
    foreign key (subject_id, fact_id) references facts (subject_id, id) on delete cascade
);

-- No foreign key on purpose: this row survives the cascade and drives the
-- subject.erased publication after the subject is gone.
create table erasures (
    subject_id     uuid        not null,
    event_id       uuid        not null unique,
    requested_at   timestamptz not null default clock_timestamp(),
    executed_at    timestamptz null,
    cancelled_at   timestamptz null,
    claimed_until  timestamptz null,
    published_at   timestamptz null,
    primary key (subject_id, requested_at)
);
create unique index erasures_pending_uidx on erasures (subject_id)
    where executed_at is null and cancelled_at is null;
create index erasures_due_idx on erasures (requested_at)
    where executed_at is null and cancelled_at is null;
create index erasures_unpublished_idx on erasures (executed_at)
    where executed_at is not null and published_at is null;

-- Clear columns only; never a value or an origin.
create table outbox (
    seq           bigint      generated always as identity primary key,
    event_id      uuid        not null unique,
    subject_id    uuid        not null references subjects(id) on delete cascade,
    kind          text        not null check (kind in ('fact.recorded','fact.retracted')),
    fact_id       bigint      null,                     -- no FK: a retracted fact's recorded event stays
    payload       jsonb       not null,
    recorded_at   timestamptz not null default clock_timestamp(),
    claimed_until timestamptz null,
    published_at  timestamptz null
);
create index outbox_unpublished_idx on outbox (seq) where published_at is null;

create table idempotency (
    subject_id   uuid        not null references subjects(id) on delete cascade,
    key          text        not null check (octet_length(key) between 1 and 255),
    fact_id      bigint      not null,                  -- no FK: a replay after retraction is a conflict, not a resurrection
    fingerprint  bytea       not null,
    created_at   timestamptz not null default clock_timestamp(),
    primary key (subject_id, key)
);
create index idempotency_created_idx on idempotency (created_at);
