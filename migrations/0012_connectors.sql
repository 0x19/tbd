-- Connectors: linked external accounts the system pulls documents from.
--
-- The bank link (finance.connections) was the first of these; this is the
-- general shape for everything after it -- three mailboxes today, vendor
-- portals tomorrow. One row per linked account, owned by a party like any
-- other finance row, so a delegated reader never sees a mailbox they were
-- not granted.
--
-- Credentials are stored encrypted with a key the service holds and Postgres
-- never sees (FINANCE_CONNECTOR_KEY, a Kubernetes Secret). A refresh token in
-- plaintext would turn a database backup into mailbox access.

create table finance.connectors (
    id              uuid        primary key,
    party_id        uuid        not null references public.parties(id) on delete cascade,
    -- The registry name: gmail, cloudflare, ...
    kind            text        not null,
    -- What a person sees: the mailbox address, the account name.
    label           text        not null default '',
    status          text        not null check (status in ('pending', 'linked', 'expired', 'failed', 'disabled')),
    -- Ours, single-use, for an OAuth callback; null for a token typed in.
    state           text        null unique,
    -- AEAD ciphertext of the kind's credential JSON; null while pending.
    credentials     bytea       null,
    -- Non-secret settings the kind reads (a label filter, a vendor list).
    config          jsonb       not null default '{}'::jsonb,
    -- The account identity the provider reported at link time (an email,
    -- an account id), for de-duplication and for the label.
    external_id     text        null,
    linked_at       timestamptz null,
    last_sync_at    timestamptz null,
    -- ok, error; null before the first sync.
    last_sync_status text       null,
    last_sync_error text        null,
    failure         text        null,
    created_at      timestamptz not null default clock_timestamp(),
    updated_at      timestamptz not null default clock_timestamp(),
    -- One link per external account per party: linking the same mailbox
    -- twice updates the row rather than pulling everything twice.
    unique (party_id, kind, external_id)
);
create index connectors_party_idx on finance.connectors (party_id, kind);

-- What each sync did, for "why is this receipt missing".
create table finance.connector_runs (
    id            uuid        primary key,
    connector_id  uuid        not null references finance.connectors(id) on delete cascade,
    started_at    timestamptz not null default clock_timestamp(),
    finished_at   timestamptz null,
    trigger       text        not null check (trigger in ('scheduled', 'manual', 'test')),
    outcome       text        null,
    found         integer     not null default 0,
    stored        integer     not null default 0,
    skipped       integer     not null default 0,
    error         text        null
);
create index connector_runs_connector_idx on finance.connector_runs (connector_id, started_at desc);

grant select, insert, update, delete on finance.connectors, finance.connector_runs to tbd_finance;

alter table finance.connectors enable row level security;
create policy connectors_visible on finance.connectors using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));

-- What a connector pulls: receipts and invoices from suppliers, as documents.
-- The bytes go in document_blobs like an issued invoice; these columns are
-- what a receipt is *about*, filled by the kind's extractor and corrected by
-- a person. Null means "not known yet", never a guess.
alter table finance.documents
    add column filename     text        null,
    add column vendor       text        null,
    add column doc_date     date        null,
    add column total_minor  bigint      null,
    add column currency     char(3)     null,
    add column extracted    jsonb       not null default '{}'::jsonb;

-- Where a document came from. The same receipt arriving by two mailboxes is
-- one document with two sources.
create table finance.document_sources (
    document_id   uuid        not null references finance.documents(id) on delete cascade,
    connector_id  uuid        null references finance.connectors(id) on delete set null,
    -- The provider's id for the thing: a Gmail message id, a portal invoice id.
    external_ref  text        not null,
    subject       text        not null default '',
    sender        text        not null default '',
    received_at   timestamptz null,
    created_at    timestamptz not null default clock_timestamp(),
    primary key (document_id, external_ref)
);
create index document_sources_connector_idx on finance.document_sources (connector_id, external_ref);
-- A message already pulled is never pulled twice.
create unique index document_sources_ref_uidx on finance.document_sources (connector_id, external_ref)
    where connector_id is not null;

grant select, insert, update, delete on finance.document_sources to tbd_finance;
