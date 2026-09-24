-- Which build answered, and which weights: a generation records the engine's
-- version and the model's revision (a digest, a file name) as the service's
-- probe last read them, so a benchmark can be rerun against the same pair.
-- Empty on rows from before this column, and until the first probe answered.

alter table llm.generations
    add column engine_version text not null default '',
    add column model_revision text not null default '';
