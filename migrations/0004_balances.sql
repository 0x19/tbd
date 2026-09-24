-- Balances, and what they are for.
--
-- A sync worker writes unattended. The only way it can tell "I have every
-- transaction" from "I have most of them" is to check its own arithmetic
-- against a number the bank computed independently: between two balance
-- snapshots, the transactions booked in between must account for the
-- difference exactly.
--
-- One snapshot cannot prove completeness -- the gap to the running total is the
-- opening balance of whatever window the provider gave us, and an opening
-- balance is indistinguishable from silently dropped history. Two snapshots
-- can. So every pull records one, and the reconciliation runs across the pair.
--
-- Measured on the first real pull: the company's EUR account sums to -15,057.59
-- against a reported balance of 10,948.51, and the personal one to 733.59
-- against 1,849.09. Those differences are almost certainly opening balances,
-- but nothing here can assert that yet, and the system should not pretend
-- otherwise.

create table finance.balances (
    id           uuid        primary key,
    account_id   uuid        not null references finance.accounts(id) on delete cascade,
    -- The provider's own code: ITBD interim booked, ITAV interim available,
    -- CLBD closing booked. Kept verbatim rather than mapped, because which one
    -- reconciles against booked transactions is a per-ASPSP question.
    balance_type text        not null,
    amount_minor bigint      not null,
    currency     char(3)     not null check (currency ~ '^[A-Z]{3}$'),
    scale        smallint    not null default 2 check (scale between 0 and 4),
    -- When the bank says it was true. Null on Erste today, which is itself
    -- worth knowing: without it, `observed_at` is the only ordering available.
    reference_date timestamptz null,
    observed_at  timestamptz not null default clock_timestamp(),
    raw          jsonb       not null default '{}'::jsonb
);
-- A snapshot is a point in time, so the same type may be recorded many times;
-- what must not happen is two rows for one observation.
create unique index balances_observation_uidx
    on finance.balances (account_id, balance_type, observed_at);
create index balances_account_idx on finance.balances (account_id, observed_at desc);

alter table finance.balances enable row level security;

create policy balances_visible on finance.balances
    using (
        current_setting('app.user_id', true) is null
        or current_setting('app.user_id', true) = ''
        or account_id in (
            select a.id from finance.accounts a
             where a.party_id in (
                 select party_id from public.party_access
                  where user_id = current_setting('app.user_id', true)::uuid
                    and (expires_at is null or expires_at > now())
             )
        )
    );

grant select, insert, update, delete on finance.balances to tbd_finance;
