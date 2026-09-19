-- What the accountant needs for each outgoing transaction, and which receipt
-- covers it.
--
-- A receipt links to the transaction that paid it. `inferred` links come
-- from the matcher; `declared` ones a person made and the matcher never
-- touches; `rejected` is a person undoing an inferred one, kept so the
-- matcher does not make it again.
create table finance.transaction_documents (
    transaction_id uuid        not null references finance.bank_transactions(id) on delete cascade,
    document_id    uuid        not null references finance.documents(id) on delete cascade,
    source         text        not null check (source in ('declared', 'inferred', 'rejected')),
    -- 0..100, how sure the matcher was; 100 for a person.
    confidence     smallint    not null check (confidence between 0 and 100),
    -- What matched, for the page: "amount 75,00 USD · vendor · 2 days".
    reason         text        not null default '',
    created_at     timestamptz not null default clock_timestamp(),
    primary key (transaction_id, document_id)
);
create index transaction_documents_document_idx on finance.transaction_documents (document_id);

-- A person's word on a counterparty: what the accountant needs from us for
-- it. `eracun`: the supplier delivers an e-invoice, nothing to send.
-- `receipt`: we must supply the document. `none`: no document exists or
-- is needed (tax, salary, bank fee, cash). `personal`: not a business cost.
-- Matched the way counterparty_aliases are: on the normalised name, exactly
-- or as a substring.
create table finance.counterparty_policies (
    id               uuid        primary key,
    party_id         uuid        not null references public.parties(id) on delete cascade,
    match_normalised text        not null check (length(match_normalised) between 2 and 200),
    exact            boolean     not null default false,
    policy           text        not null check (policy in ('eracun', 'receipt', 'none', 'personal')),
    note             text        not null default '',
    created_at       timestamptz not null default clock_timestamp(),
    unique (party_id, match_normalised)
);
create index counterparty_policies_party_idx on finance.counterparty_policies (party_id);

alter table finance.transaction_documents enable row level security;
alter table finance.counterparty_policies enable row level security;
create policy counterparty_policies_visible on finance.counterparty_policies using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
-- A link is visible when its transaction is: the transaction's own policy
-- decides, so a row outside the grant has no links either.
create policy transaction_documents_visible on finance.transaction_documents using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or transaction_id in (select id from finance.bank_transactions));

grant select, insert, update, delete on finance.transaction_documents, finance.counterparty_policies to tbd_finance;
