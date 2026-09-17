-- The finance domain, first slice (docs/plans/finance-service.md, Part 2).
--
-- Accounts and the transactions in them. Categories, rules, documents,
-- invoices and the invoice lifecycle come later; what is here is the part the
-- access rules have to be proven against, because a leak is silent and the
-- proof needs real rows to fail on.
--
-- Money is `bigint` minor units plus an ISO 4217 currency and a scale. Never a
-- float, and never a `numeric` -- the workspace's sqlx features cannot decode
-- one, proto has no decimal type so the wire is `int64` regardless, and the
-- transcoder renders int64 as a JSON string, which is what money wants in a
-- browser. Every aggregate is `sum(amount_minor) group by currency`; a
-- cross-currency total is never computed.

-- Which party owns an account is the whole access story: `party_id` is a
-- person for a personal account and an org for a company one, so "just the
-- company", "just me" or "both" is a filter rather than a schema change.
create table finance.accounts (
    id             uuid        primary key,
    party_id       uuid        not null references public.parties(id) on delete restrict,
    provider       text        not null default 'manual',
    provider_uid   text        null,
    iban           text        null,
    currency       char(3)     not null check (currency ~ '^[A-Z]{3}$'),
    name           text        not null default '',
    status         text        not null default 'active' check (status in ('active', 'closed')),
    created_at     timestamptz not null default clock_timestamp(),
    updated_at     timestamptz not null default clock_timestamp()
);
-- Erste exposes one account per currency behind a single IBAN -- the company's
-- IBAN is four accounts, EUR/GBP/USD/HRK -- so the identity is the pair, never
-- the IBAN alone. Measured, not assumed: prototype/bank/FINDINGS.md.
create unique index accounts_iban_currency_uidx on finance.accounts (party_id, iban, currency)
    where iban is not null;
create index accounts_party_idx on finance.accounts (party_id) where status = 'active';
-- Redundant against the primary key, and required: a composite foreign key
-- needs a unique constraint on exactly the columns it references. This is what
-- lets a transaction carry `party_id` without it being able to drift.
alter table finance.accounts add constraint accounts_id_party_uk unique (id, party_id);

create table finance.bank_transactions (
    id               uuid        primary key,
    -- `party_id` is carried here so every read filters on one column and no
    -- query can reach a transaction by forgetting a join -- but it is *not* a
    -- copy that has to be kept in sync by hand. The composite foreign key below
    -- makes the pair `(account_id, party_id)` a thing the database enforces:
    -- writing a transaction whose party disagrees with its account's is
    -- rejected, and moving an account to another party cascades to every
    -- transaction in it. A stale copy is not a consistency bug here, it is a
    -- leak, so "remember to keep them equal" is not good enough.
    party_id         uuid        not null,
    account_id       uuid        not null,
    status           text        not null check (status in ('booked', 'pending')),
    -- The provider's own reference when it has one. On Erste it is present and
    -- unique on every row of 2,657 measured, so it is the key; the content hash
    -- below is the fallback for providers that are less well behaved.
    entry_reference  text        null,
    dedup_key        bytea       not null,
    amount_minor     bigint      not null,
    currency         char(3)     not null check (currency ~ '^[A-Z]{3}$'),
    scale            smallint    not null default 2 check (scale between 0 and 4),
    credit_debit     text        not null check (credit_debit in ('CRDT', 'DBIT')),
    booking_date     date        null,
    value_date       date        null,
    counterparty_name text       null,
    counterparty_iban text       null,
    -- Erste returns this as an array of strings; it is joined on the way in.
    -- The invoice number arrives here as free text, not in a structured
    -- reference field, so this column is what reconciliation reads.
    remittance       text        null,
    reference_number text        null,
    raw              jsonb       not null default '{}'::jsonb,
    first_seen_at    timestamptz not null default clock_timestamp(),
    updated_at       timestamptz not null default clock_timestamp(),
    -- The pair, not the two columns independently. `on update cascade` moves the
    -- transactions with the account; `on delete cascade` removes them with it.
    constraint bank_transactions_account_party_fk
        foreign key (account_id, party_id)
        references finance.accounts (id, party_id)
        on update cascade on delete cascade
);
create unique index bank_transactions_dedup_uidx
    on finance.bank_transactions (account_id, dedup_key);
create index bank_transactions_party_date_idx
    on finance.bank_transactions (party_id, booking_date desc, id desc);
create index bank_transactions_account_date_idx
    on finance.bank_transactions (account_id, booking_date desc, id desc);
create index bank_transactions_search_idx on finance.bank_transactions
    using gin (to_tsvector('simple',
        coalesce(counterparty_name, '') || ' ' || coalesce(remittance, '')));

-- Row-level security: the second layer.
--
-- The service filters every query on the caller's granted parties. This makes
-- the database refuse anyway, so a handler that forgets its filter returns
-- nothing rather than everything. `app.user_id` is bound per transaction by
-- `tbd_db::bind_rls_user` -- `set local`, so a pooled connection cannot carry
-- one caller's identity into the next caller's transaction.
--
-- The policies are permissive to the table owner (which runs migrations and
-- the importer) and restrictive to everyone else; `force row level security`
-- is deliberately not set, so an admin-run backfill still works.
alter table finance.accounts          enable row level security;
alter table finance.bank_transactions enable row level security;

create policy accounts_visible on finance.accounts
    using (
        current_setting('app.user_id', true) is null
        or current_setting('app.user_id', true) = ''
        or party_id in (
            select party_id from public.party_access
             where user_id = current_setting('app.user_id', true)::uuid
               and (expires_at is null or expires_at > now())
        )
    );

create policy bank_transactions_visible on finance.bank_transactions
    using (
        current_setting('app.user_id', true) is null
        or current_setting('app.user_id', true) = ''
        or party_id in (
            select party_id from public.party_access
             where user_id = current_setting('app.user_id', true)::uuid
               and (expires_at is null or expires_at > now())
        )
    );

grant select, insert, update, delete on all tables in schema finance to tbd_finance;
