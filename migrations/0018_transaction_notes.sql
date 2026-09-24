-- A person's note on a transaction for the accountant: why a receipt is
-- missing, what a charge was for. One per transaction; it goes into the
-- month's README and summary, and with the mail.
create table finance.transaction_notes (
    transaction_id uuid        primary key references finance.bank_transactions(id) on delete cascade,
    party_id       uuid        not null references public.parties(id) on delete cascade,
    note           text        not null check (length(note) between 1 and 2000),
    updated_at     timestamptz not null default clock_timestamp()
);
create index transaction_notes_party_idx on finance.transaction_notes (party_id);
grant select, insert, update, delete on finance.transaction_notes to tbd_finance;
alter table finance.transaction_notes enable row level security;
create policy transaction_notes_visible on finance.transaction_notes using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
