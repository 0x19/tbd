# Humans plane

> **Status, 2026-09-10:** proposed, awaiting approval. Nothing here is built. This
> directory is the Engine plane the top-level [README](../README.md) listed as
> "not yet", under the name we actually use for it. Reviewed once against every
> other design doc; the findings are folded in.

The layer that holds what is known about one human, from many sources, for many
first-party consumers, and answers "everything about this person, now" fast enough
to sit on a request path.

## What a Human is

Not a profile row. **A ledger of facts about one person**, where every fact says
where it came from, how sure the producer was, when it was true, and under which
grants it may be read. The dating app reads it. The matcher reads it. A KYC verdict
writes to it. A journal entry, after inference, writes to it. A later first-party
surface adds kinds of facts; nothing adds columns.

```
Identity (id plane, one per verified human)
   │  one link row, held by the id plane
   └── Human (opaque id, minted at enrolment)
         facts[]      one row per (path, source, time)            ← many to one
         current      the latest fact per (path, source), materialised
         relations[]  facts about a pair, one row on each side, cascading together
         projections  what a persona exposes to an organisation; derived, never stored as truth
```

## Where it sits

- **Identity** ([`../id/`](../id/)) establishes *that* someone is one real human
  and holds anchors, verified attributes, personas and the permission graph. It
  holds no profile content ([id/008](../id/008-federation.md)). This ledger is the
  store of **Resource Server #1, our own app**; other organisations' data is
  brokered under id/008 and is never written here. The Human is one per identity
  ([id/005 D9](../id/005-decisions.md)), keyed by an opaque id; the derived
  pseudonym never enters this plane ([id/006](../id/006-legal-erasure.md),
  requirement 1).
- **The `humans` service** (a new crate, behind Envoy like the engine) owns the
  ledger and presence, and enforces scoping on every read. The protocol forwards
  to it; the engine reads and writes through it and owns no storage. It stores no
  grants and no pseudonyms: both are resolved from the id plane per read.
- **Engine** (the crate) computes facts: inference from journal and photos, the
  symbolic readings, compatibility. It writes `inferred` and `symbolic` facts.
- **API** exposes the Human as JSON only, with provenance on every field
  ([004-surface.md](004-surface.md)).

## Standing constraints

- **Every fact carries provenance.** No value without a source, an origin and a
  time. A placeholder says `stub: true` on the wire, as
  `crates/engine/src/service.rs` already does for scores.
- **Sources never blend silently.** Verified, declared, inferred, symbolic and
  observed are five different claims about the same path and are returned as such.
- **A reading is not a measurement.** Symbolic facts carry no confidence and are
  never presented as probabilities.
- **Grants are data the id plane owns.** A read is scoped by the organisation's
  grant and persona, resolved from the id plane, never from a token scope alone
  ([003](003-consent-and-erasure.md)).
- **Erasure is a cascade the database proves in Postgres**, asynchronous and
  verified elsewhere, with the residue stated store by store
  ([003](003-consent-and-erasure.md)).
- **Q2 and Q3 still gate the Engine.** This plane is the substrate that makes the
  proof measurable (observed facts are the outcomes); it does not answer whether
  matching works. Nothing here is built before
  [../001-open-questions.md](../001-open-questions.md) Q2 and Q3 are decided.
  Approving [002](002-storage.md) closes the storage line of Q6 there, and that
  document gets the note in the same change.

## Documents

| # | Document | Status |
|---|----------|--------|
| 000 | [Facts ledger](000-facts-ledger.md) — a Human is a ledger of facts with provenance | open, proposed |
| 001 | [Sources](001-sources.md) — the five sources and what each may assert | open, proposed |
| 002 | [Storage](002-storage.md) — Postgres ledger, Redis projection, ClickHouse later | open, proposed |
| 003 | [Consent and erasure](003-consent-and-erasure.md) — scoping reads, cascading deletes | open, proposed |
| 004 | [Surface](004-surface.md) — JSON only, provenance on every field | open, proposed |
| 005 | Lenses — the symbolic readings and their one vocabulary | not written |
