# Humans plane

> **Status, 2026-09-11:** proposed, awaiting approval. The `ledger` service exists as
> a scaffolded stub (`crates/ledger`, [docs/ledger](../../ledger/README.md)): health,
> metrics, tracing, one labelled `Ping`; none of the facts model below is built, and
> Q2/Q3 still gate it. This
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
- **Two crates, one process.** `ledger` is the generic substrate: append-only facts
  about an opaque subject, with provenance, scope ids, tombstones, the erasure
  cascade and the outbox, on Postgres, with no dependency on identity concepts,
  personas, Redis or anything human. `humans` embeds it and adds everything
  human-specific: the path registry for people, presence, per-persona projections,
  resolution of `sub` against the id plane, and the JSON surface. The `humans`
  service (behind Envoy like the engine) is the one process; the protocol forwards
  to it; the engine reads and writes through it and owns no storage. It stores no
  grants and no pseudonyms: both are resolved from the id plane per read.
- **Promotion rule.** `ledger` becomes its own service when a non-human subject
  type exists or a consumer other than `humans` needs it directly. Not before: a
  network hop between the two on every cache miss would sit on top of the key
  calls [005](005-encryption.md) already adds to the 10 ms budget in
  [002](002-storage.md), for no gain, and the two reasons the id plane earned its
  own process (compliance blast radius, different hardware,
  [id/README](../id/README.md)) do not apply between them. The crate API is the
  trait a gRPC implementation satisfies later, so promotion changes no caller.
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
- **Grants are data the id plane owns, and keys.** A read is scoped by the
  organisation's grant and persona, resolved from the id plane, never from a token
  scope alone ([003](003-consent-and-erasure.md)); every fact value is encrypted
  under a per-fact key wrapped per `(human, scope, persona)`, released by the id
  plane against the grant, so the ledger and its backups are ciphertext to anyone
  without one ([005](005-encryption.md)).
- **Erasure is a cascade the database proves in Postgres**, asynchronous and
  verified elsewhere, with the residue stated store by store
  ([003](003-consent-and-erasure.md)).
- **Q2 and Q3 still gate the Engine.** This plane is the substrate that makes the
  proof measurable (observed facts are the outcomes); it does not answer whether
  matching works. Nothing here is built before
  [../001-open-questions.md](../001-open-questions.md) Q2 and Q3 are decided.
  Approving [002](002-storage.md) closes the storage line of Q6 there, and that
  document gets the note in the same change.

## Asks of the id plane

Recorded here rather than edited into `../id/`, which is that plane's record:

- A **key service**: holds human keys and scope private keys wrapped by the one
  KMS root, releases a scope private key to `humans` for a valid
  `(principal, human, scope)` against its permission graph, offers a batch release
  for the engine principal and the `analytics` principal (the outbox consumer
  inside `humans`), re-wraps DEKs on rotation without releasing the retiring key,
  and destroys a human key on request ([005](005-encryption.md)). Its key table's
  backup retention is 24 hours.
- A resolution call returning `(human_id, grant, persona)` for a `sub`
  ([003](003-consent-and-erasure.md), [004](004-surface.md)).
- The engine recorded as Client #1 in the permission graph, with per-person
  `engine.<capability>` grants ([003](003-consent-and-erasure.md)).

## Documents

| # | Document | Status |
|---|----------|--------|
| 000 | [Facts ledger](000-facts-ledger.md) — a Human is a ledger of facts with provenance | open, proposed |
| 001 | [Sources](001-sources.md) — the five sources and what each may assert | open, proposed |
| 002 | [Storage](002-storage.md) — Postgres ledger, Redis projection, ClickHouse later | open, proposed |
| 003 | [Consent and erasure](003-consent-and-erasure.md) — scoping reads, cascading deletes | open, proposed |
| 004 | [Surface](004-surface.md) — JSON only, provenance on every field | open, proposed |
| 005 | [Encryption](005-encryption.md) — a value is readable only with the key its scope grants | open, proposed |
| 006 | Lenses — the symbolic readings and their one vocabulary | not written |

## Crate boundary

| | `ledger` | `humans` |
|---|---|---|
| Knows | subject id (opaque), path, source, origin, scope ids, time, tombstones, erasure cascade, outbox, envelope encryption per (subject, scope) | the human path registry, presence, projections, personas, the id plane, the JSON surface |
| Stores | Postgres | the Redis projection and presence |
| Depends on | nothing in the domain; the path registry and scope ids are injected | `ledger`, the id plane |
| Reusable for | any subject type: a venue, an event, an organisation | people only |
