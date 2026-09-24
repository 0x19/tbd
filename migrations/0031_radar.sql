-- The radar service: every item it has read from the Go and Rust sources, and
-- the weekly digests the llm service wrote from them. Items are public facts
-- (a title, a link, a date, the source's own summary); digests are model-written
-- text, marked as such on every surface. Its own schema; nothing here
-- references public. The group role follows 0002: no login, privileges only.

create schema if not exists radar;

do $$
begin
    if not exists (select 1 from pg_roles where rolname = 'tbd_radar') then
        create role tbd_radar nologin;
    end if;
exception
    when insufficient_privilege then
        raise notice 'cannot create roles here; run mise run db:roles as an admin';
    -- Roles are cluster-wide: two databases migrating at once (the tests do)
    -- can both see the role missing and race to create it.
    when duplicate_object or unique_violation then
        null;
end $$;

do $$
begin
    grant usage, create on schema radar to tbd_radar;
    alter default privileges in schema radar grant all on tables    to tbd_radar;
    alter default privileges in schema radar grant all on sequences to tbd_radar;
exception when insufficient_privilege or undefined_object then
    raise notice 'cannot grant here; run mise run db:roles as an admin';
end $$;

-- One thing a source published. A source names its items by a guid (the feed
-- entry's id, the issue's URL); the same guid read twice is the same item.
create table radar.items (
    id           bigserial primary key,
    source       text        not null,
    language     text        not null check (language in ('go', 'rust')),
    guid         text        not null,
    title        text        not null,
    url          text        not null,
    summary      text        not null default '',
    published_at timestamptz not null,
    fetched_at   timestamptz not null default now(),
    unique (source, guid)
);

create index items_language_published on radar.items (language, published_at desc);

-- One week's digest for one language in one reader language. A rewrite
-- (RunDigest with force) replaces the row; there is no history of drafts.
create table radar.digests (
    id          bigserial primary key,
    week        text        not null,
    language    text        not null check (language in ('go', 'rust')),
    lang        text        not null check (lang in ('en', 'hr')),
    created_at  timestamptz not null default now(),
    changed     text        not null,
    why         text        not null,
    drill       text        not null,
    script      text        not null,
    item_count  integer     not null,
    model       text        not null,
    stub        boolean     not null default false,
    unique (week, language, lang)
);

create index digests_week on radar.digests (week desc);
