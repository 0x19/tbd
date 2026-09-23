# RFCs

What was decided before it was built, and why. Each document is one decision or one
design, numbered in the order written and never renumbered. The site's Lab renders
every RFC marked public (`ui/www`, `/lab/rfc/<slug>/`), so a page here is written for
two readers at once: the engineer who will build it, and a visitor who has never seen
this repository.

## The file

`NNNN-slug.md`, four digits, a slug that never changes once published (the URL is the
file stem). YAML front matter, scalars only:

```
---
title: The llm service, an L2 over two engines
status: open
date: 2026-09-23
public: false
summary: One sentence for the index and the search engines.
supersedes: 0000-an-older-slug
---
```

- `status`: `open` while it is being argued with, `decided` once it is settled,
  `superseded` when a later RFC replaces it (`supersedes` on the newer one points back).
- `public`: the publish switch. `false` pages are not rendered, not checked, not linked.
  A public page may not reference a private one.
- `date`: the day it was first written; the status log at the end carries every later
  change.

The body is Markdown with these sections, in this order: Problem, Proposal,
Alternatives considered, Decision, Publication (what changes on the site or in the
platform when this ships), Status log.

## Redaction

A public page never carries a security detail: an internal host name, an address, a
port, a path, a Kubernetes name, a Secret's name, the shape of an auth flow, a rate
limit. The build refuses a public page that does (`ui/www/tool/lab-redaction.ts`; the
rules are listed in `ui/www/CLAUDE.md`). Where the fact belongs in the narrative, it is
withheld in the open rather than paraphrased away:

- inline: `the balancer answers on [REDACTED: the internal port] behind the edge`
- a whole block:

  ````
  ```redacted
  the routing table, six lines
  ```
  ````

Both render as a black bar with the reason on hover. The reason is public text, so it
is checked too. Words the rules cannot infer (resource names) go in
`docs/lab/redaction.json`.

## A published RFC is a living document

Once public, an RFC describes something that exists. Any change that alters what the
page says, a decision reversed, a component replaced, a number remeasured, updates the
page in the same commit and adds a line to the status log; a decision that no longer
holds is superseded by a new number, never rewritten in place. A stale RFC on the site
is a false claim.

## Index

| # | Document | Status |
|---|---|---|
| 0001 | [The llm service, an L2 over two engines](0001-the-llm-service.md) | open |
