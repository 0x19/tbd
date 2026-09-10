# Design

> **Status, 2026-09-09:** idea material from the identity-layer exploration, kept for
> reference. Not a spec for the code in `crates/`. See the root `ARCHITECTURE.md`.

Where we work out what we're building, before we build it.

## How this works

Each document decides **one thing**. Numbered in the order written, never
renumbered. A document is a record of a decision and the reasoning behind it —
not a specification of a system that doesn't exist yet.

Every document opens with a status line:

```
Status: open | decided | superseded by NNN
```

`open` means we're still arguing with ourselves. `decided` means it's settled
and the code should match. `superseded` means a later document overrode it —
the original stays, so the reasoning trail survives.

## Rules

- **Decide, don't describe.** If a doc doesn't close a question, it's notes,
  and notes go in the open-questions doc until they're ready to become a decision.
- **Keep it short.** apex had a 1,178-line matching spec and no matching code.
  Length was a symptom, not a feature.
- **Cite reality.** When a decision rests on something in apex, in the running
  Proximity deployment, or in a model's actual behaviour, point at it.
- **Correct after building.** When implementation teaches us the doc was wrong,
  the doc gets fixed. A stale design doc is worse than no design doc.

## Index

| # | Document | Status |
|---|----------|--------|
| 000 | [Premise](000-premise.md) — why we're starting over, what we keep, what we drop | open |
| 001 | [Open questions](001-open-questions.md) — what must be decided before any code | superseded in part by id/005 |
| 002 | [Repository layout](002-repository-layout.md) — workspace, tooling, CI, README conventions | decided, implemented |

## Planes

Design that belongs to one plane of the system lives in its own subdirectory, with
its own README and its own numbering.

| Plane | Directory | Covers |
|-------|-----------|--------|
| **Identity** | [`id/`](id/) | Portable verified identity: accounts, personas, uniqueness, vendor orchestration, data sharing |
| **Engine** | _not yet_ | User representation and system behaviour policy |
| **API** | _not yet_ | The single public contract |
