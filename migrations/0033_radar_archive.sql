-- The radar's archive: past weeks written by the backfill from each source's
-- own archive (2025 and 2026 at first). A digest says where it came from:
-- `live` from the weekly run and the owner's review, `archive` from the
-- backfill, published at once and labelled on the page as not individually
-- reviewed.

alter table radar.digests
    add column origin text not null default 'live' check (origin in ('live', 'archive'));
