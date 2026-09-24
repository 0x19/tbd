-- Which agent a generation spoke as (RFC 0011): its id, or empty for a
-- conversation with the bare model. Empty on rows from before this column.

alter table llm.generations
    add column agent text not null default '';
