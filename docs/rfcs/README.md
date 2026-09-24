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
lab: llm
summary: One sentence for the index and the search engines.
supersedes: 0000-an-older-slug
---
```

- `status`: `open` while it is being argued with, `decided` once it is settled,
  `superseded` when a later RFC replaces it (`supersedes` on the newer one points back).
- `public`: the publish switch. A `false` page is a draft: it is rendered only into an
  admin-only file and read at `/lab/draft/?doc=rfc/<slug>`, before and after the lab is
  published, is not redaction-checked, and may not be referenced by a public page.
- `date`: the day it was first written; the status log at the end carries every later
  change.
- `lab`: the lab it belongs to, an id in `ui/www/src/data/labs.ts`; the lab's page lists
  it, and the build refuses an id that is not there.

The status log's bullets are `- YYYY-MM-DD: text` (continued on indented lines); the
lab pages read them as the timeline, so a bullet in any other shape fails the build.

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
| 0001 | [The llm service, an L2 over two engines](0001-the-llm-service.md) | decided |
| 0002 | [Memory and tools](0002-memory-and-tools.md) | open |
| 0003 | [The shield](0003-the-shield.md) | open |
| 0004 | [The workbench and the arena](0004-the-workbench-and-the-arena.md) | open |
| 0005 | [The platform over MCP](0005-the-platform-over-mcp.md) | open |
| 0006 | [Publication](0006-publication.md) | open |
| 0007 | [Quiet Pager, an InOrbit lab](0007-quiet-pager.md) | open, draft |
| 0008 | [The grader](0008-the-grader.md) | open, draft |
| 0009 | [Subscriptions](0009-subscriptions.md) | open, draft |
| 0010 | [The sandbox](0010-the-sandbox.md) | open |
