-- A rule needs a name you can refer to.
--
-- Rules are seeded from a file and later edited in the UI, and both need to
-- address an existing rule rather than append a second copy of it. Without a
-- key, re-running a seed either duplicates every rule or has to delete the
-- party's rules wholesale -- which would take the hand-written ones with it.
--
-- The empty default was the real problem: it let any number of nameless rules
-- exist, so there was nothing to conflict on.

update finance.rules set name = 'rule-' || left(id::text, 8) where name = '';

alter table finance.rules
    alter column name drop default,
    add constraint rules_name_len_ck check (length(name) between 1 and 120),
    add constraint rules_party_name_uk unique (party_id, name);
