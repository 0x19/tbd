-- Every time an invoice went out: the mail that carried it, to whom, when.
-- The invoice is `sent` from the first; a second send is refused unless the
-- person says so, and then it is one more row here.
create table finance.invoice_deliveries (
    id         uuid        primary key,
    invoice_id uuid        not null references finance.invoices(id) on delete cascade,
    party_id   uuid        not null references public.parties(id) on delete cascade,
    mail_id    uuid        null references finance.mails(id) on delete set null,
    to_addrs   text[]      not null default '{}',
    sent_at    timestamptz not null default clock_timestamp()
);
create index invoice_deliveries_invoice_idx on finance.invoice_deliveries (invoice_id, sent_at);
alter table finance.invoices add column sent_at timestamptz null;
grant select, insert, update, delete on finance.invoice_deliveries to tbd_finance;
alter table finance.invoice_deliveries enable row level security;
create policy invoice_deliveries_visible on finance.invoice_deliveries using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
