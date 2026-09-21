-- What settled an invoice: a payment in the bank matched to it by the
-- reference the payer wrote (the invoice number, as printed), or one a person
-- recorded. An invoice is paid when its payments cover its total; less is a
-- part, and the invoice stays open with what is still owed.
create table finance.invoice_payments (
    id             uuid        primary key,
    invoice_id     uuid        not null references finance.invoices(id) on delete cascade,
    party_id       uuid        not null references public.parties(id) on delete cascade,
    -- The bank transaction, when the payment is one; a person may record a
    -- payment the bank has not shown yet (cash, a foreign account).
    transaction_id uuid        null references finance.bank_transactions(id) on delete set null,
    amount_minor   bigint      not null check (amount_minor <> 0),
    currency       char(3)     not null,
    paid_on        date        not null,
    -- `inferred`: the matcher, from the reference and the amount. `declared`: a
    -- person. `rejected`: a match a person undid, kept so it is not made again.
    source         text        not null check (source in ('declared', 'inferred', 'rejected')),
    -- What matched: "reference 9-1-1-2026", "amount".
    reason         text        not null default '',
    note           text        not null default '',
    created_at     timestamptz not null default clock_timestamp()
);
create index invoice_payments_invoice_idx on finance.invoice_payments (invoice_id);
create index invoice_payments_party_idx on finance.invoice_payments (party_id);
-- A transaction settles one invoice.
create unique index invoice_payments_transaction_idx
    on finance.invoice_payments (transaction_id) where transaction_id is not null;

-- The sum on the invoice, so a list does not join; `paid_at` the day the
-- last payment that covered it arrived.
alter table finance.invoices
    add column paid_minor bigint not null default 0,
    add column paid_at    timestamptz null;

grant select, insert, update, delete on finance.invoice_payments to tbd_finance;
alter table finance.invoice_payments enable row level security;
create policy invoice_payments_visible on finance.invoice_payments using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
