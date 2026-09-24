-- The llm service: every generation a caller ran (who, which tier and engine,
-- how many tokens, how it ended) and the sessions they belong to. The budget
-- is a sum over this table per subject per UTC day. No prompt or completion
-- text is stored: the record is what happened, not what was said. Its own
-- schema, keyed by the verified subject; nothing here references public.
-- The group role follows 0002: no login, privileges only.

create schema if not exists llm;

do $$
begin
    if not exists (select 1 from pg_roles where rolname = 'tbd_llm') then
        create role tbd_llm nologin;
    end if;
exception when insufficient_privilege then
    raise notice 'cannot create roles here; run mise run db:roles as an admin';
end $$;

do $$
begin
    grant usage, create on schema llm to tbd_llm;
    alter default privileges in schema llm grant all on tables    to tbd_llm;
    alter default privileges in schema llm grant all on sequences to tbd_llm;
exception when insufficient_privilege or undefined_object then
    raise notice 'cannot grant here; run mise run db:roles as an admin';
end $$;

-- A session is a record a caller may continue, not a memory: every request
-- carries the whole conversation. It groups generations for the caller's own
-- history and nothing else.
create table llm.sessions (
    id            uuid primary key default gen_random_uuid(),
    subject       text not null,
    title         text not null default '',
    created_at    timestamptz not null default now(),
    last_used_at  timestamptz not null default now()
);
create index sessions_subject_idx on llm.sessions (subject, last_used_at desc);

-- One row per Generate call that got past admission: started when the engine
-- was asked, finished with the outcome. `running` rows that never finish are
-- a crash's trace, not a state the service reads.
create table llm.generations (
    id                 uuid primary key,
    session_id         uuid references llm.sessions (id) on delete set null,
    subject            text not null,
    tier               text not null check (tier in ('fast', 'deep')),
    engine             text not null,
    model              text not null,
    stub               boolean not null default false,
    status             text not null check (status in ('running', 'ok', 'failed', 'cancelled')),
    error              text not null default '',
    prompt_tokens      integer not null default 0,
    completion_tokens  integer not null default 0,
    -- Milliseconds from admission to the first text chunk; null if none came.
    first_token_ms     integer,
    started_at         timestamptz not null default now(),
    finished_at        timestamptz
);
-- The budget query: this subject's tokens since the start of the UTC day.
create index generations_budget_idx on llm.generations (subject, started_at desc);
create index generations_session_idx on llm.generations (session_id, started_at desc);
