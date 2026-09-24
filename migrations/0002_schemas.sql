-- Schema per service, one database (docs/plans/finance-service.md, Part 1).
--
-- Foreign keys cross schemas freely in Postgres, which is the whole reason the
-- migrations are per project: a finance account points at a party in `public`
-- and the database enforces it.
--
-- Roles are NOT created here. A migration runs as whoever holds the admin
-- credential, and `create role` needs privileges a managed Postgres may not
-- grant; more importantly a login role needs a password, and a password does
-- not belong in a file under version control. `mise run db:roles` creates the
-- login roles from generated secrets and grants them the group roles below.
-- What lives here is the part that should be reviewable: who may touch what.

create schema if not exists ledger;
create schema if not exists finance;

-- Group roles carry the privileges; login roles inherit them. Created here
-- without LOGIN so they hold no credential, and `if not exists` so a database
-- that already has them is not disturbed.
do $$
begin
    if not exists (select 1 from pg_roles where rolname = 'tbd_ledger') then
        create role tbd_ledger nologin;
    end if;
    if not exists (select 1 from pg_roles where rolname = 'tbd_finance') then
        create role tbd_finance nologin;
    end if;
exception when insufficient_privilege then
    -- A managed Postgres that forbids CREATE ROLE still gets its schemas; the
    -- operator grants by hand and `mise run db:roles` says so.
    raise notice 'cannot create roles here; run mise run db:roles as an admin';
end $$;

-- Identity is shared and read-only to services. They resolve a caller and read
-- grants; they never rewrite somebody else's access.
do $$
begin
    grant usage on schema public to tbd_ledger, tbd_finance;
    grant select on public.parties, public.users, public.orgs,
                    public.memberships, public.party_access to tbd_ledger, tbd_finance;
    -- ensure_user provisions on first sight, and a delegated read is audited,
    -- so those two are the only writes a service makes to shared identity.
    grant insert, update on public.parties, public.users to tbd_ledger, tbd_finance;
    grant insert on public.access_log to tbd_ledger, tbd_finance;

    -- Each service owns its own schema and cannot reach the other's.
    grant usage, create on schema ledger  to tbd_ledger;
    grant usage, create on schema finance to tbd_finance;
    grant all on all tables    in schema ledger  to tbd_ledger;
    grant all on all sequences in schema ledger  to tbd_ledger;
    grant all on all tables    in schema finance to tbd_finance;
    grant all on all sequences in schema finance to tbd_finance;

    -- Tables a later migration adds are covered without revisiting this file.
    alter default privileges in schema ledger  grant all on tables    to tbd_ledger;
    alter default privileges in schema ledger  grant all on sequences to tbd_ledger;
    alter default privileges in schema finance grant all on tables    to tbd_finance;
    alter default privileges in schema finance grant all on sequences to tbd_finance;
exception when insufficient_privilege or undefined_object then
    raise notice 'cannot grant here; run mise run db:roles as an admin';
end $$;
