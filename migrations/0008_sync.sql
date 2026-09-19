-- What the sync worker needs to remember, kept in rows and not in a process.
--
-- The whole design of the syncer follows from one number: most banks allow
-- four unattended fetches per account per day, and the fifth is a 429 that
-- costs the rest of the day. So the budget, the backoff and the watermark all
-- live on the account row, where a restart, a second replica and a manual
-- refresh see one truth. A token bucket in memory would be reset by the very
-- restart that makes it spend the day's allowance twice.

-- A consent: one authorization at one bank, covering some accounts, for a
-- while. The session id is what every later call is made with; the consent
-- has an end date and nothing about it renews on its own.
create table finance.connections (
    id              uuid        primary key,
    party_id        uuid        not null references public.parties(id) on delete cascade,
    provider        text        not null,
    -- `business` or `personal` at Erste: different login flows.
    psu_type        text        not null check (psu_type in ('business', 'personal')),
    aspsp_name      text        not null,
    aspsp_country   char(2)     not null,
    -- The provider's session id. Behaves like a credential: it is what reads
    -- the accounts. Never logged.
    session_id      text        null,
    status          text        not null check (status in
                       ('pending', 'authorized', 'expired', 'revoked', 'failed')),
    -- Ours, single-use, from `start_authorization`. Compared on the callback
    -- so a code cannot be exchanged into someone else's connection.
    state           text        not null unique,
    valid_until     timestamptz null,
    authorized_at   timestamptz null,
    created_at      timestamptz not null default clock_timestamp(),
    updated_at      timestamptz not null default clock_timestamp(),
    -- Why it is `failed`, when it is. Never the code, never the session id.
    failure         text        null
);
create index connections_party_idx on finance.connections (party_id, status);

alter table finance.accounts
    add column connection_id      uuid        null references finance.connections(id) on delete set null,
    -- Sync state. Every column is written inside the transaction that claims
    -- the account, so two workers cannot both decide to spend the same call.
    add column sync_enabled       boolean     not null default true,
    add column sync_budget_day    date        null,
    add column sync_budget_used   integer     not null default 0
                                              check (sync_budget_used >= 0),
    add column sync_backoff_until timestamptz null,
    add column last_synced_at     timestamptz null,
    -- The latest booking date seen on a successful sync. The next routine
    -- fetch starts `overlap_days` before it, never from "now".
    add column last_booked_through date       null,
    -- What the last sync said, for the UI. `ok`, `rate_limited`,
    -- `consent_invalid`, `transport`, `error`.
    add column last_sync_status   text        null,
    add column last_sync_error    text        null;

create index accounts_sync_due_idx
    on finance.accounts (last_synced_at nulls first)
    where sync_enabled;

-- One row per attempt, whatever the outcome. This is the audit trail for the
-- question "why is my data stale": the answer is a row, not a log line
-- somebody has to go and find.
create table finance.sync_runs (
    id              uuid        primary key,
    account_id      uuid        not null references finance.accounts(id) on delete cascade,
    -- `scheduled` spends from the scheduler's share of the budget, `manual`
    -- from the reserve. Both count against the bank's total.
    trigger         text        not null check (trigger in ('scheduled', 'manual', 'initial')),
    started_at      timestamptz not null default clock_timestamp(),
    finished_at     timestamptz null,
    date_from       date        not null,
    date_to         date        not null,
    -- `ok`, `rate_limited`, `consent_invalid`, `transport`, `error`.
    outcome         text        null,
    pages           integer     not null default 0,
    inserted        integer     not null default 0,
    duplicates      integer     not null default 0,
    skipped         integer     not null default 0,
    balances        integer     not null default 0,
    error           text        null
);
create index sync_runs_account_idx on finance.sync_runs (account_id, started_at desc);

grant select, insert, update, delete on finance.connections, finance.sync_runs to tbd_finance;

alter table finance.connections enable row level security;
create policy connections_visible on finance.connections
    using (
        current_setting('app.user_id', true) is null
        or current_setting('app.user_id', true) = ''
        or party_id in (
            select party_id from public.party_access
             where user_id = current_setting('app.user_id', true)::uuid
               and (expires_at is null or expires_at > now())
        )
    );
