-- Invoicing: what we issue, to whom, and the proof of who approved it.
--
-- Three decisions, each made once here so no code has to remember it:
--
-- 1. A number is allocated at approval, never at draft creation, enforced by
--    `(ordinal is null) = (approved_at is null)`. Croatian numbering is
--    gapless per year and legally binding; a discarded draft that had taken
--    a number would leave a hole. The counter is a row updated inside the
--    approving transaction -- never a sequence, whose `nextval` survives a
--    rollback and leaves exactly that hole.
-- 2. Money is bigint minor units, and `total = subtotal + vat` is a check,
--    so a rounding bug is a constraint violation and never a wrong invoice.
-- 3. What was approved is what was rendered: the draft's content hash is
--    stored with the approval, and the PDF is content-addressed, so a later
--    re-render that differs is a detected change, not a quiet one.

-- The issuing side: one row per internal org party, with every line the
-- footer of a Croatian invoice must carry. Data, not template text, so a
-- change of address or bank is one row and every later invoice follows.
create table finance.issuers (
    party_id        uuid        primary key references public.parties(id) on delete cascade,
    legal_name      text        not null,
    address_lines   text[]      not null,
    oib             text        not null,
    vat_id          text        not null,
    iban            text        not null,
    swift           text        not null,
    bank_name       text        not null,
    court           text        not null,
    registration_no text        not null,
    share_capital   text        not null,
    board_member    text        not null,
    issued_by       text        not null,
    place_of_issue  text        not null,
    operator_id     text        not null default '1',
    -- The legal triple's fixed parts: broj računa - poslovni prostor - naplatni uređaj.
    premises        text        not null default '1',
    device          text        not null default '1',
    -- Days from issue to due, the default for a new draft.
    due_days        integer     not null default 15,
    updated_at      timestamptz not null default clock_timestamp()
);

-- Who we invoice. Belongs to the issuing party; a client of the company is
-- not a client of the person.
create table finance.clients (
    id              uuid        primary key,
    party_id        uuid        not null references public.parties(id) on delete cascade,
    name            text        not null,
    address_lines   text[]      not null default '{}',
    country_code    char(2)     not null,
    -- Their tax identifier, whatever the jurisdiction calls it.
    tax_id          text        not null default '',
    -- Decides the VAT line and the note the invoice must carry.
    vat_treatment   text        not null check (vat_treatment in
                       ('standard_hr', 'reverse_charge_eu', 'outside_scope_non_eu', 'exempt_issuer')),
    -- Where invoices go, once sending exists. Recorded now so an invoice
    -- carries its recipients from the day it is approved.
    recipients      text[]      not null default '{}',
    currency        char(3)     not null default 'EUR',
    created_at      timestamptz not null default clock_timestamp(),
    archived_at     timestamptz null
);
create index clients_party_idx on finance.clients (party_id) where archived_at is null;

-- The gapless counter, one per issuer and year. Updated under its row lock
-- inside the approving transaction.
create table finance.invoice_numbers (
    party_id     uuid    not null references public.parties(id) on delete cascade,
    year         integer not null,
    next_ordinal integer not null default 1 check (next_ordinal >= 1),
    primary key (party_id, year)
);

create table finance.invoices (
    id              uuid        primary key,
    party_id        uuid        not null references public.parties(id) on delete cascade,
    client_id       uuid        not null references finance.clients(id) on delete restrict,
    status          text        not null check (status in ('draft', 'approved', 'sent', 'paid', 'cancelled')),
    -- The legal number: ordinal-premises-device-year, once approved.
    year            integer     not null,
    ordinal         integer     null,
    premises        text        not null,
    device          text        not null,
    number          text        generated always as
                        (case when ordinal is null then null
                              else ordinal::text || '-' || premises || '-' || device || '-' || year::text end) stored,
    issued_at       timestamptz null,
    delivery_date   date        not null,
    due_date        date        not null,
    place_of_issue  text        not null,
    currency        char(3)     not null,
    subtotal_minor  bigint      not null default 0,
    vat_minor       bigint      not null default 0,
    total_minor     bigint      not null default 0,
    vat_treatment   text        not null,
    -- Copied onto the invoice at approval, never looked up at render time:
    -- a law change next year must not rewrite an issued invoice.
    vat_note        text        not null default '',
    note            text        not null default '',
    -- The hash of the canonical draft the approver saw. Approval carries the
    -- hash from the preview; a draft changed since is refused.
    content_hash    text        null,
    approved_at     timestamptz null,
    approved_by     uuid        null references public.users(id),
    document_id     uuid        null,
    -- The invoice this draft was pre-filled from, so the screen can diff.
    prefilled_from  uuid        null references finance.invoices(id),
    cancelled_at    timestamptz null,
    created_at      timestamptz not null default clock_timestamp(),
    updated_at      timestamptz not null default clock_timestamp(),
    check ((ordinal is null) = (approved_at is null)),
    check (total_minor = subtotal_minor + vat_minor),
    unique (party_id, year, premises, device, ordinal)
);
create index invoices_party_idx on finance.invoices (party_id, status, created_at desc);

create table finance.invoice_lines (
    id               uuid    primary key,
    invoice_id       uuid    not null references finance.invoices(id) on delete cascade,
    position         integer not null,
    description      text    not null,
    -- Thousandths, so 1.5 hours and 0.25 days are exact.
    quantity_milli   bigint  not null check (quantity_milli > 0),
    unit_price_minor bigint  not null,
    amount_minor     bigint  not null,
    unique (invoice_id, position)
);

-- PDFs, content-addressed. Bytes in a side table so listing documents never
-- drags them off disk.
create table finance.documents (
    id           uuid        primary key,
    party_id     uuid        not null references public.parties(id) on delete cascade,
    kind         text        not null,
    sha256       text        not null,
    content_type text        not null,
    size_bytes   bigint      not null,
    created_at   timestamptz not null default clock_timestamp(),
    unique (party_id, sha256)
);
create table finance.document_blobs (
    document_id uuid  primary key references finance.documents(id) on delete cascade,
    bytes       bytea not null
);
alter table finance.invoices
    add constraint invoices_document_fk foreign key (document_id) references finance.documents(id);

-- Every state change, by whom, with what the caller presented.
create table finance.invoice_events (
    id         uuid        primary key,
    invoice_id uuid        not null references finance.invoices(id) on delete cascade,
    at         timestamptz not null default clock_timestamp(),
    user_id    uuid        null references public.users(id),
    event      text        not null,
    detail     jsonb       not null default '{}'::jsonb
);
create index invoice_events_invoice_idx on finance.invoice_events (invoice_id, at);

grant select, insert, update, delete on finance.issuers, finance.clients, finance.invoice_numbers,
    finance.invoices, finance.invoice_lines, finance.documents, finance.document_blobs,
    finance.invoice_events to tbd_finance;

alter table finance.issuers  enable row level security;
alter table finance.clients  enable row level security;
alter table finance.invoices enable row level security;
alter table finance.documents enable row level security;
create policy issuers_visible on finance.issuers using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy clients_visible on finance.clients using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy invoices_visible on finance.invoices using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy documents_visible on finance.documents using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
