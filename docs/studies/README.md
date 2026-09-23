# Studies

What was measured, what surprised us, what we would do differently. A study follows an
RFC (`rfc:` names it) and reports on the thing the RFC decided to build. Same file
rules as `docs/rfcs/README.md`: `NNNN-slug.md`, never renumbered, front matter with
scalars only, the redaction syntax, the publish switch, and the living-document rule.

```
---
title: What a 16 GB card really does
status: running
date: 2026-09-30
public: false
summary: One sentence.
rfc: 0001-the-llm-service
headline: 72 tok/s
headline_note: the fast tier at batch one, median over ten runs
---
```

- `status`: `running` while numbers are still coming in, `measured` once the
  measurements are final, `published` once the learning section is written in the
  owner's own words.
- `headline` and `headline_note`: the one number the index shows, and what it is. A
  number earns its place only when attached to a claim; leave both out rather than
  invent one.

The body, in this order: The question, What we believed, What we built, What we
measured (tables, with the method next to them), What surprised us, What we would do
differently, Glossary (the terms that were new that week, in one line each).

## Index

| # | Study | Status |
|---|---|---|
