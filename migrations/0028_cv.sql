-- The cv service: who asked for the full CV, what the owner decided, and every
-- download. Its own schema beside finance's, on the shared identity tables in
-- public (a request is keyed by the verified subject, never by an id from a
-- request body). The group role follows 0002: no login, privileges only.

create schema if not exists cv;

do $$
begin
    if not exists (select 1 from pg_roles where rolname = 'tbd_cv') then
        create role tbd_cv nologin;
    end if;
exception when insufficient_privilege then
    raise notice 'cannot create roles here; run mise run db:roles as an admin';
end $$;

do $$
begin
    grant usage on schema public to tbd_cv;
    grant select on public.parties, public.users, public.orgs,
                    public.memberships, public.party_access to tbd_cv;
    grant insert, update on public.parties, public.users to tbd_cv;
    grant insert on public.access_log to tbd_cv;
    grant usage, create on schema cv to tbd_cv;
    alter default privileges in schema cv grant all on tables    to tbd_cv;
    alter default privileges in schema cv grant all on sequences to tbd_cv;
exception when insufficient_privilege or undefined_object then
    raise notice 'cannot grant here; run mise run db:roles as an admin';
end $$;

-- One row per person (the OIDC subject); a person asks again by renewing the
-- same row, so the history of a decision is the row's timestamps, not a pile
-- of rows.
create table cv.requests (
    id            uuid primary key default gen_random_uuid(),
    subject       text not null unique,
    email         text not null,
    name          text not null default '',
    note          text not null default '',
    status        text not null check (status in ('requested', 'approved', 'refused', 'revoked')),
    requested_at  timestamptz not null default now(),
    decided_at    timestamptz,
    decided_by    text,
    -- When the owner was told; null until the mail went out.
    notified_at   timestamptz,
    updated_at    timestamptz not null default now()
);
create index requests_status_idx on cv.requests (status, requested_at desc);

-- Every render handed out, with what the gateway saw of the client.
create table cv.downloads (
    id          uuid primary key default gen_random_uuid(),
    request_id  uuid not null references cv.requests (id) on delete cascade,
    at          timestamptz not null default now(),
    user_agent  text not null default '',
    ip          text not null default ''
);
create index downloads_request_idx on cv.downloads (request_id, at desc);
