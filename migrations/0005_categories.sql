-- Categories, rules, and the counterparty aliases they need.
--
-- Three things the real data forced into this design, none of them obvious
-- before looking at it (prototype/bank/FINDINGS.md):
--
-- 1. Normalising counterparty names is necessary and not sufficient. The same
--    person arrives as `VESIĆ NEVIO`, `Vesic Nevio` and `Nevio Vesić`;
--    Anthropic bills as both `ANTHROPIC* CLAUDE SUB` and
--    `CLAUDE.AI SUBSCRIPTION`. Case and diacritics are code. Word order and
--    vendor aliasing are data, so they need a table.
--
-- 2. Transfers between accounts the same person owns are not spending. There
--    are 116 of them moving EUR 235,849, and counting them inflated the
--    company's monthly outgoings by 88-163%. They are derived, never assigned:
--    a category you can forget to apply is one somebody will forget.
--
-- 3. A rule and a human can disagree about one transaction. The human wins,
--    always, and the rule must not quietly reassign it on the next run.

create table finance.categories (
    id           uuid        primary key,
    party_id     uuid        not null references public.parties(id) on delete cascade,
    parent_id    uuid        null references finance.categories(id) on delete restrict,
    slug         text        not null check (slug ~ '^[a-z0-9]+(_[a-z0-9]+)*$'),
    name         text        not null check (length(name) between 1 and 120),
    -- What the category does to the books, not what it is called. `transfer`
    -- exists so an owner draw can be categorised on both sides without either
    -- side being counted as trade.
    kind         text        not null check (kind in
                   ('expense', 'income', 'transfer', 'tax', 'capital')),
    -- The accountant's chart-of-accounts code, when they give us one. Null
    -- until then rather than invented.
    account_code text        null,
    deductible   boolean     not null default true,
    created_at   timestamptz not null default clock_timestamp(),
    archived_at  timestamptz null,
    unique (party_id, slug)
);
create index categories_party_idx on finance.categories (party_id) where archived_at is null;

-- One real counterparty, however the bank spelled it.
--
-- `match_normalised` is compared against the same normalisation the importer
-- applies (uppercase, collapse whitespace, strip to [A-Z0-9 ]), so a rule
-- written against one spelling catches the others. `exact` distinguishes a
-- whole-name match from a substring: `ANTHROPIC` should catch
-- `ANTHROPIC* CLAUDE SUB`, but `A1` must not catch `A1 HRVATSKA` and every
-- other name containing those two characters.
create table finance.counterparty_aliases (
    id               uuid        primary key,
    party_id         uuid        not null references public.parties(id) on delete cascade,
    match_normalised text        not null check (length(match_normalised) between 2 and 200),
    exact            boolean     not null default false,
    -- What to call it once recognised.
    canonical        text        not null check (length(canonical) between 1 and 200),
    -- Optional: recognising a counterparty is often enough to categorise it.
    category_id      uuid        null references finance.categories(id) on delete set null,
    created_at       timestamptz not null default clock_timestamp(),
    unique (party_id, match_normalised, exact)
);
create index counterparty_aliases_party_idx on finance.counterparty_aliases (party_id);

-- Rules, in explicit priority order.
--
-- `priority` ascending, then `id`: two rules matching one transaction resolve
-- the same way on every run, on every replica. Leaving that to insertion order
-- is how a categorisation quietly changes between two runs over the same data.
create table finance.rules (
    id                      uuid        primary key,
    party_id                uuid        not null references public.parties(id) on delete cascade,
    priority                integer     not null default 100,
    name                    text        not null default '',
    -- Every condition is optional; those that are set must all match.
    match_counterparty_like text        null,
    match_counterparty_iban text        null,
    match_remittance_like   text        null,
    match_currency          char(3)     null,
    match_credit_debit      text        null check (match_credit_debit in ('CRDT', 'DBIT')),
    match_amount_min_minor  bigint      null,
    match_amount_max_minor  bigint      null,
    category_id             uuid        not null references finance.categories(id) on delete cascade,
    enabled                 boolean     not null default true,
    -- How many transactions it has claimed. A rule at zero after a full run is
    -- either wrong or obsolete, and both are worth seeing.
    hits                    bigint      not null default 0,
    created_at              timestamptz not null default clock_timestamp(),
    check (match_amount_min_minor is null
        or match_amount_max_minor is null
        or match_amount_min_minor <= match_amount_max_minor)
);
create index rules_party_priority_idx on finance.rules (party_id, priority, id) where enabled;

-- What a transaction was categorised as, and by whom.
--
-- `source` borrows the ledger's vocabulary rather than inventing one: a rule is
-- `inferred`, a person is `declared`. The distinction is the whole point --
-- applying rules must never overwrite `declared`.
alter table finance.bank_transactions
    add column category_id     uuid        null references finance.categories(id) on delete set null,
    add column category_source text        null check (category_source in ('declared', 'inferred')),
    add column category_rule_id uuid       null references finance.rules(id) on delete set null,
    add column categorised_at  timestamptz null,
    -- Both or neither: a category with no provenance cannot be reasoned about.
    add constraint bank_transactions_category_source_ck
        check ((category_id is null) = (category_source is null));

create index bank_transactions_uncategorised_idx
    on finance.bank_transactions (party_id, booking_date desc)
    where category_id is null;
create index bank_transactions_category_idx
    on finance.bank_transactions (category_id) where category_id is not null;

-- Everything a summary needs, with the two derived facts it must not get wrong.
--
-- `internal` is computed, never stored: a transfer is internal exactly when its
-- counterparty is an account we hold, and that becomes true the moment another
-- account is linked. A stored flag would be right on the day it was written and
-- wrong afterwards.
create view finance.transactions_enriched as
select
    t.*,
    exists (
        select 1 from finance.accounts a
         where a.iban is not null
           and a.iban = t.counterparty_iban
    ) as internal,
    coalesce(al.canonical, t.counterparty_name) as counterparty_canonical
from finance.bank_transactions t
left join finance.counterparty_aliases al
       on al.party_id = t.party_id
      and (
        (al.exact and upper(coalesce(t.counterparty_name, '')) = al.match_normalised)
        or (not al.exact and upper(coalesce(t.counterparty_name, '')) like '%' || al.match_normalised || '%')
      );

grant select, insert, update, delete on finance.categories, finance.rules,
    finance.counterparty_aliases to tbd_finance;
grant select on finance.transactions_enriched to tbd_finance;

alter table finance.categories            enable row level security;
alter table finance.rules                 enable row level security;
alter table finance.counterparty_aliases  enable row level security;

create policy categories_visible on finance.categories
    using (
        current_setting('app.user_id', true) is null
        or current_setting('app.user_id', true) = ''
        or party_id in (
            select party_id from public.party_access
             where user_id = current_setting('app.user_id', true)::uuid
               and (expires_at is null or expires_at > now())
        )
    );

create policy rules_visible on finance.rules
    using (
        current_setting('app.user_id', true) is null
        or current_setting('app.user_id', true) = ''
        or party_id in (
            select party_id from public.party_access
             where user_id = current_setting('app.user_id', true)::uuid
               and (expires_at is null or expires_at > now())
        )
    );

create policy counterparty_aliases_visible on finance.counterparty_aliases
    using (
        current_setting('app.user_id', true) is null
        or current_setting('app.user_id', true) = ''
        or party_id in (
            select party_id from public.party_access
             where user_id = current_setting('app.user_id', true)::uuid
               and (expires_at is null or expires_at > now())
        )
    );
