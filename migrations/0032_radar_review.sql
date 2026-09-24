-- The radar's review gate and the change-per-block shape. A digest is a draft
-- until the owner publishes it (the page says issues are human-reviewed; this
-- is the review), and carries a short summary and its changes as JSON, each
-- with a fixed impact category and a link that was one of the week's items.
-- `changed` and `why` stay for the digests written before this migration.

alter table radar.digests
    add column status       text        not null default 'draft'
                                        check (status in ('draft', 'published')),
    add column published_at timestamptz,
    add column published_by text,
    add column summary      text        not null default '',
    add column changes      jsonb       not null default '[]';

-- The four digests written before the gate existed were read by the owner
-- before this migration: they stay public.
update radar.digests
   set status = 'published', published_at = created_at, published_by = 'migration:0032'
 where status = 'draft';

-- The old sections are optional from now on.
alter table radar.digests alter column changed set default '';
alter table radar.digests alter column why set default '';

create index digests_status_week on radar.digests (status, week desc);
