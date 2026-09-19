-- Mail sent from a linked mailbox, and what came back.
--
-- A connector that may send (Gmail with the send scope) carries the mail of a
-- party: to the accountant, mostly. Every message sent is kept verbatim, with
-- who it went to and what was attached; a reply that arrives in the same
-- thread is imported as an inbound row on the next pull, and its attachments
-- become documents. Templates hold the recurring shape ("Račun {{Month}}/
-- {{Year}}") with default recipients; the page renders the helpers and the
-- service stores what was actually sent.

create table finance.mail_templates (
    id         uuid        primary key,
    party_id   uuid        not null references public.parties(id) on delete cascade,
    name       text        not null check (length(name) between 1 and 120),
    subject    text        not null default '',
    body       text        not null default '',
    to_addrs   text[]      not null default '{}',
    cc_addrs   text[]      not null default '{}',
    bcc_addrs  text[]      not null default '{}',
    created_at timestamptz not null default clock_timestamp(),
    updated_at timestamptz not null default clock_timestamp(),
    unique (party_id, name)
);

create table finance.mails (
    id            uuid        primary key,
    party_id      uuid        not null references public.parties(id) on delete cascade,
    connector_id  uuid        null references finance.connectors(id) on delete set null,
    -- `out`: sent by us; `in`: a reply that arrived.
    direction     text        not null check (direction in ('out', 'in')),
    -- The provider's ids: the thread the conversation lives in, and the message.
    thread_key    text        null,
    provider_id   text        null,
    -- RFC 5322 ids, for threading across providers.
    message_id    text        null,
    in_reply_to   text        null,
    -- The outgoing mail a reply answers.
    parent_id     uuid        null references finance.mails(id) on delete set null,
    template_id   uuid        null references finance.mail_templates(id) on delete set null,
    from_addr     text        not null default '',
    to_addrs      text[]      not null default '{}',
    cc_addrs      text[]      not null default '{}',
    bcc_addrs     text[]      not null default '{}',
    subject       text        not null default '',
    body          text        not null default '',
    body_html     text        null,
    -- `sent`, `failed` (with `error`), `received`.
    status        text        not null check (status in ('sent', 'failed', 'received')),
    error         text        null,
    sent_at       timestamptz null,
    received_at   timestamptz null,
    created_at    timestamptz not null default clock_timestamp()
);
create index mails_party_idx on finance.mails (party_id, created_at desc);
create index mails_thread_idx on finance.mails (connector_id, thread_key);
create unique index mails_provider_idx on finance.mails (connector_id, provider_id) where provider_id is not null;

-- What went with a mail (documents we hold) or came with a reply (stored as
-- documents of kind `attachment` when not a receipt).
create table finance.mail_documents (
    mail_id     uuid not null references finance.mails(id) on delete cascade,
    document_id uuid not null references finance.documents(id) on delete cascade,
    primary key (mail_id, document_id)
);

-- Whether a connector's credential may send, decided by the kind at link time.
alter table finance.connectors add column can_send boolean not null default false;

grant select, insert, update, delete on finance.mail_templates, finance.mails, finance.mail_documents to tbd_finance;

alter table finance.mail_templates enable row level security;
alter table finance.mails enable row level security;
create policy mail_templates_visible on finance.mail_templates using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
create policy mails_visible on finance.mails using (
    current_setting('app.user_id', true) is null or current_setting('app.user_id', true) = ''
    or party_id in (select party_id from public.party_access
                     where user_id = current_setting('app.user_id', true)::uuid
                       and (expires_at is null or expires_at > now())));
