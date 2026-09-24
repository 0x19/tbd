-- The books: a general ledger per company party. Five tables, all money in
-- bigint minor units (see 0003_finance.sql for why never numeric or float).
--
--   ledger_accounts   the party's chart: RRiF numbering, class and kind from the
--                     first digit, analytics under a synthetic parent
--   periods           one row per (year, month): open, closed or locked
--   journal_entries   one posting event, pointing at its source and the rule
--                     that produced it; the source key makes re-posting idempotent
--   journal_lines     the debit and credit lines of an entry, one side each,
--                     never negative; a deferred constraint trigger holds
--                     sum(debit) = sum(credit) per entry at commit
--   opening_balances  the trial balance the year starts from, imported as filed:
--                     both sides per account and *signed*, because the column
--                     an accountant hands over at a mid-year hand-over carries
--                     year-to-date P&L balances, accounts with both sides, and
--                     reversals booked as negative postings (docs/accountant/
--                     findings.md §1). A trial balance is opening plus lines.

create table finance.ledger_accounts (
    party_id    uuid        not null references public.parties(id) on delete cascade,
    code        text        not null check (code ~ '^[0-9]{3,6}$'),
    name        text        not null check (length(name) between 1 and 200),
    -- The synthetic account this one sits under; null for a top-level group.
    parent_code text        null,
    -- RRiF class, the first digit: 0 long-term assets, 1 cash and receivables,
    -- 2 liabilities, 3 inventories, 4 expenses, 7 revenue, 8 result, 9 equity.
    class       smallint    not null generated always as (left(code, 1)::smallint) stored,
    kind        text        not null generated always as (
                    case left(code, 1)
                        when '0' then 'asset'
                        when '1' then 'asset'
                        when '3' then 'asset'
                        when '2' then 'liability'
                        when '4' then 'expense'
                        when '7' then 'revenue'
                        when '8' then 'result'
                        when '9' then 'equity'
                        else 'other'
                    end) stored,
    -- A group or synthetic account postings never land on directly.
    synthetic   boolean     not null default false,
    archived_at timestamptz null,
    created_at  timestamptz not null default clock_timestamp(),
    primary key (party_id, code),
    constraint ledger_accounts_parent_fk foreign key (party_id, parent_code)
        references finance.ledger_accounts (party_id, code) on delete restrict
);

create table finance.periods (
    party_id    uuid        not null references public.parties(id) on delete cascade,
    fiscal_year integer     not null check (fiscal_year between 2000 and 2100),
    month       smallint    not null check (month between 1 and 12),
    -- open: postings land; closed: a person closed the month, it reopens;
    -- locked: filed, final.
    status      text        not null default 'open' check (status in ('open', 'closed', 'locked')),
    changed_at  timestamptz not null default clock_timestamp(),
    changed_by  uuid        null references public.users(id) on delete set null,
    primary key (party_id, fiscal_year, month)
);

create table finance.journal_entries (
    id           uuid        primary key,
    party_id     uuid        not null references public.parties(id) on delete cascade,
    entry_date   date        not null,
    fiscal_year  integer     not null generated always as (extract(year from entry_date)::integer) stored,
    month        smallint    not null generated always as (extract(month from entry_date)::smallint) stored,
    -- What produced the entry. Postings are derived, never typed: opening,
    -- bank (a transaction), invoice, document, payroll (a month), judgement
    -- (a declared decision), closing (year end), chaos (the tool's own).
    source_kind  text        not null check (source_kind in ('opening', 'bank', 'invoice', 'document', 'payroll', 'judgement', 'closing', 'chaos')),
    source_id    text        not null,
    rule_id      text        not null,
    rule_version text        not null,
    memo         text        not null default '',
    reversal_of  uuid        null references finance.journal_entries(id) on delete restrict,
    created_at   timestamptz not null default clock_timestamp(),
    -- Re-running a rule for a source is a no-op, not a second entry.
    constraint journal_entries_source_uk unique (party_id, source_kind, source_id, rule_id, rule_version),
    constraint journal_entries_id_party_uk unique (id, party_id)
);
create index journal_entries_party_date_idx on finance.journal_entries (party_id, entry_date);

create table finance.journal_lines (
    entry_id          uuid     not null,
    party_id          uuid     not null,
    position          smallint not null check (position >= 1),
    account_code      text     not null,
    debit_minor       bigint   not null default 0 check (debit_minor >= 0),
    credit_minor      bigint   not null default 0 check (credit_minor >= 0),
    -- Exactly one side, and not zero.
    constraint journal_lines_one_side check ((debit_minor > 0) <> (credit_minor > 0)),
    currency          char(3)  not null default 'EUR' check (currency ~ '^[A-Z]{3}$'),
    -- The amount in the document's currency when that is not the books'.
    original_minor    bigint   null,
    original_currency char(3)  null check (original_currency is null or original_currency ~ '^[A-Z]{3}$'),
    counterparty      text     null,
    vat_code          text     null,
    primary key (entry_id, position),
    -- The party rides on the line so the account FK and the RLS policy hold
    -- without a join; the composite FK keeps it equal to the entry's.
    constraint journal_lines_entry_fk foreign key (entry_id, party_id)
        references finance.journal_entries (id, party_id) on delete cascade,
    constraint journal_lines_account_fk foreign key (party_id, account_code)
        references finance.ledger_accounts (party_id, code) on delete restrict
);
create index journal_lines_party_account_idx on finance.journal_lines (party_id, account_code);

-- sum(debit) = sum(credit) per entry, checked at commit so an entry's lines
-- can be inserted one by one inside a transaction. The code checks the same
-- before it writes, for the message; this is what no code path can bypass.
create function finance.assert_entry_balanced() returns trigger
language plpgsql as $$
declare
    entry uuid := coalesce(new.entry_id, old.entry_id);
    diff  bigint;
begin
    select coalesce(sum(debit_minor), 0) - coalesce(sum(credit_minor), 0)
      into diff
      from finance.journal_lines
     where entry_id = entry;
    if diff <> 0 then
        raise exception using
            errcode = 'check_violation',
            message = format('journal entry %s is unbalanced: debit exceeds credit by %s minor units', entry, diff);
    end if;
    return null;
end
$$;
grant execute on function finance.assert_entry_balanced() to tbd_finance;

create constraint trigger journal_entry_balanced
    after insert or update or delete on finance.journal_lines
    deferrable initially deferred
    for each row execute function finance.assert_entry_balanced();

create table finance.opening_balances (
    party_id     uuid        not null references public.parties(id) on delete cascade,
    fiscal_year  integer     not null check (fiscal_year between 2000 and 2100),
    -- The date the balances stand at: 1 January, or the hand-over date when
    -- the books changed hands mid-year.
    as_of        date        not null,
    account_code text        not null,
    -- Signed on purpose, see the header.
    debit_minor  bigint      not null default 0,
    credit_minor bigint      not null default 0,
    -- filed: the accountant's column as handed over; imported: another
    -- system's export; derived: computed by this service from a prior year.
    source       text        not null check (source in ('filed', 'imported', 'derived')),
    imported_at  timestamptz not null default clock_timestamp(),
    imported_by  uuid        null references public.users(id) on delete set null,
    primary key (party_id, fiscal_year, account_code),
    constraint opening_balances_account_fk foreign key (party_id, account_code)
        references finance.ledger_accounts (party_id, code) on delete restrict
);

alter table finance.ledger_accounts   enable row level security;
alter table finance.periods           enable row level security;
alter table finance.journal_entries   enable row level security;
alter table finance.journal_lines     enable row level security;
alter table finance.opening_balances  enable row level security;

create policy ledger_accounts_visible on finance.ledger_accounts using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy periods_visible on finance.periods using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy journal_entries_visible on finance.journal_entries using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy journal_lines_visible on finance.journal_lines using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy opening_balances_visible on finance.opening_balances using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));

grant select, insert, update, delete on finance.ledger_accounts  to tbd_finance;
grant select, insert, update, delete on finance.periods          to tbd_finance;
grant select, insert, update, delete on finance.journal_entries  to tbd_finance;
grant select, insert, update, delete on finance.journal_lines    to tbd_finance;
grant select, insert, update, delete on finance.opening_balances to tbd_finance;
