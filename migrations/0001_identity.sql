-- Identity, shared by every service (docs/plans/finance-service.md, Part 1).
--
-- Every object here is schema-qualified, and that is not style. Postgres'
-- default search_path is `"$user", public`: on a database whose role is named
-- `ledger` and which also has a `ledger` schema, an unqualified `create table`
-- lands in `ledger`, not `public`. That happened -- the identity tables were
-- created twice, once in each -- and the foreign keys then pointed at an empty
-- copy. Qualify everything; never rely on the path.
--
-- Migrations are per project, not per service: foreign keys cross service
-- boundaries, so one ordered set and one database. These tables live in
-- `public`; each service owns a schema of its own and points its keys here.
--
-- The model is a supertype. `parties` is anyone money can belong to or flow to
-- -- you, the company, a client, a supplier. `users` and `orgs` are the two
-- kinds, each keyed on the party it is. Every ownership column downstream is
-- then a single `party_id` that works for a person or an organisation, and a
-- projection over "just the company", "just me" or "both" is a filter rather
-- than a schema change.

create table public.parties (
    id           uuid        primary key,
    kind         text        not null check (kind in ('person', 'org')),
    display_name text        not null check (length(display_name) between 1 and 200),
    country_code char(2)     null check (country_code ~ '^[A-Z]{2}$'),
    created_at   timestamptz not null default clock_timestamp(),
    archived_at  timestamptz null
);
create index parties_kind_idx on public.parties (kind) where archived_at is null;

-- Parties that can sign in.
--
-- `subject` is the OIDC `sub` Envoy verified and forwarded in `x-jwt-payload`.
-- Safe as a natural key because devops/k8s/auth/config/hydra.yml sets
-- `subject_identifiers.supported_types: [public]`, so the value is the stable
-- Kratos identity id and is identical for every client. If that is ever changed
-- to `pairwise` this column silently starts meaning something else -- the same
-- human would arrive under a different `sub` per client -- so the setting and
-- this table have to change together.
create table public.users (
    id           uuid        primary key references public.parties(id) on delete restrict,
    subject      text        not null unique check (length(subject) between 1 and 255),
    email        text        null,
    last_seen_at timestamptz null
);
create unique index users_email_uidx on public.users (lower(email)) where email is not null;

-- Parties that are organisations: ours, and our clients' and suppliers'.
create table public.orgs (
    id         uuid    primary key references public.parties(id) on delete restrict,
    legal_name text    not null check (length(legal_name) between 1 and 200),
    oib        text    null check (oib ~ '^[0-9]{11}$'),   -- Croatian tax id
    vat_id     text    null check (length(vat_id) between 4 and 20),
    address    jsonb   not null default '{}'::jsonb,
    -- true when we invoice *as* this org, rather than merely trading with it.
    internal   boolean not null default false
);
create unique index orgs_oib_uidx on public.orgs (oib) where oib is not null;
create index orgs_internal_idx on public.orgs (internal) where internal;

-- Who belongs to which organisation. This answers "is this person part of the
-- company", which is a different question from "may this person read that
-- money" -- see party_access.
create table public.memberships (
    user_id  uuid        not null references public.users(id) on delete cascade,
    org_id   uuid        not null references public.orgs(id)  on delete cascade,
    role     text        not null check (role in ('owner', 'admin', 'member', 'viewer')),
    added_at timestamptz not null default clock_timestamp(),
    primary key (user_id, org_id)
);
create index memberships_org_idx on public.memberships (org_id);

-- Who may read whose money.
--
-- Deliberately separate from `memberships`. A personal account belongs to a
-- person party, not to an org, so a membership cannot express "the accountant
-- reads the company's books and nothing else" without inventing a fake org to
-- hold the personal data. Conflating the two is exactly how personal data leaks
-- into a business grant.
--
-- Absence is the denial. Someone who may not see a party has *no row* for it,
-- not a row with a lesser capability.
create table public.party_access (
    user_id    uuid        not null references public.users(id)   on delete cascade,
    party_id   uuid        not null references public.parties(id) on delete cascade,
    capability text        not null check (capability in ('own', 'read')),
    granted_at timestamptz not null default clock_timestamp(),
    granted_by uuid        null references public.users(id) on delete set null,
    expires_at timestamptz null,
    primary key (user_id, party_id)
);
create index party_access_party_idx on public.party_access (party_id);
create index party_access_live_idx  on public.party_access (user_id)
    where expires_at is null;

-- Who read whose data, for the delegated-access case. Money plus a second
-- reader means "who saw what" has to be answerable after the fact.
create table public.access_log (
    id         bigint      generated always as identity primary key,
    user_id    uuid        not null references public.users(id) on delete cascade,
    party_id   uuid        not null references public.parties(id) on delete cascade,
    route      text        not null,
    rows_seen  integer     null,
    trace_id   text        not null default '',
    at         timestamptz not null default clock_timestamp()
);
create index access_log_at_idx      on public.access_log (at desc);
create index access_log_party_idx   on public.access_log (party_id, at desc);
