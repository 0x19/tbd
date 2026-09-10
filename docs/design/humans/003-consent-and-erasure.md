# 003 — Consent and erasure

Status: open — proposed for approval

Grants live in the id plane. The `humans` service resolves them on every read.
Erasure is a soft mark now and one Postgres transaction at the end of a disclosed
window, asynchronous and verified elsewhere.

---

## Who may read what

A read is `(reader, persona, grant, human)`:

- **reader**: the person, the engine, or an organisation. Today
  `crates/protocol/src/subject.rs` yields the `sub` claim only; the caller kind
  (`azp` or `client_id` for an organisation, a service identity for the engine on
  its internal route) is new work on the protocol and on the `envoy.yaml` JWT
  requirements.
- **persona**: for an organisation, the persona it is connected through
  ([id/000](../id/000-account-model.md), [id/005 D9](../id/005-decisions.md)).
  Projections are per persona: a `tier2` reader sees the photos *this* persona
  exposes, not every `profile.*` fact.
- **grant**: the scopes the id plane's permission graph records for
  `(organisation, persona)` ([id/008](../id/008-federation.md)). The engine is
  Client #1 in that graph: `engine.base` is its standing grant and each
  `engine.<capability>` opt-in is a grant a person adds for it there. **The token's
  scope claim is not the grant.** The service asks the id plane for
  `(human_id, grant, persona)` in one call and caches it for at most 30 seconds,
  so revocation takes effect within 30 seconds; that number is the guarantee and
  the scenario. No grant is stored in this plane.

Every fact carries `consent`, a list of registry scope ids; the store returns only
facts whose ids intersect the grant. Batch jobs read under `engine.base` with the
same intersection. There is no other read path.

| Scope id | Who may read | Default for |
|---|---|---|
| `self` | the person | everything |
| `engine.base` | the engine, for matching and inference | `profile.*`, `journal.*`, `birth.*`, `traits.*`, `relations.*`, `outcomes.*`, `identity.*` |
| `engine.<capability>` | the engine, one capability at a time, by explicit opt-in the person sees the output of ([001](001-sources.md) rule 2; each lens's `readings.*`) | none until opted in |
| `tier2`, `tier3` | organisations, through the persona | `profile.*` projections |
| `engine_representation` | organisations granted tier 4 through the persona by the **separate consent flow** [id/005 D8](../id/005-decisions.md) requires; never a token scope, and a token carrying it is rejected | `traits.*` |

What never reaches an organisation, and why:

- `relations.*`, `outcomes.*`: [id/008](../id/008-federation.md)'s rule that
  queryable data is user-owned data, never a judgement about the user.
- `journal.*`, `birth.*`: this document's decision; both are user-owned, and both
  are the most re-identifying and most sensitive things we hold
  ([000](000-facts-ledger.md), `birth.*`).
- `identity.*`: served by the id plane, where verified attributes are canonical
  ([id/003](../id/003-data-sharing.md)); `self` and `engine.base` only here.
- `readings.*`: the one exception. A per-lens opt-in adds a `tier2` scope to that
  lens's paths so a badge can show; nothing else.

Journal text in `inputs` is readable by the person and, under `engine.base`, by
the engine; it has no other reader.

## Erasure

A person deletes their account:

1. **At the request.** `humans.erased_at` is set and every read is denied; the
   Redis hash and geo member are deleted (presence does not wait); an `erasures`
   row is written ([002](002-storage.md)); the per-human key stays in escrow. The
   window is **7 days, disclosed**, so that a hijacked account can be recovered:
   recovery is clearing `erased_at` and rebuilding the projection. This is a
   choice against instant shredding and is flagged below.
2. **At window end.** One Postgres transaction cascades from `humans` through every
   fact, both sides of every relation via `counterparty_id`, `fact_inputs`,
   `facts_current`, `inputs` and `outbox`, and appends a tombstone on the surviving
   side of each relation with `counterparty_id: null` (the erasure cascade is the
   one place a tombstone has no counterparty).
3. The key is destroyed. `human.erased` is published from the `erasures` row,
   delivered at least once and retried, not guaranteed. Counterparties' Redis
   projections are rebuilt through the outbox.
4. What survives is the residue table in [002](002-storage.md) for its stated
   retentions, and what [id/006](../id/006-legal-erasure.md) provisionally allows
   pending counsel on its questions 1 and 3: the anchor hash and derived pseudonym
   in the id plane.

## What a test must prove

Privacy claims are enforced by code and covered by a test that fails on regression
([../001-open-questions.md](../001-open-questions.md) Q5). Chaos scenarios, next to
the latency ones, on every route in [004](004-surface.md), at load:

- A `tier2` reader receives nothing outside `profile.*` and opted-in `readings.*`,
  and never an `origin.inputs` entry.
- A `tier2` or `engine_representation` reader connected through persona P receives
  only the projection P exposes.
- A token carrying `engine_representation` is rejected.
- A revoked grant stops reads within 30 seconds.
- No response, event or webhook body contains a ledger uuid, `person_id` or anchor
  value.
- At the request: reads denied, presence gone. After the window: zero rows for the
  human in every Postgres table, no key, and after the outbox drains no Redis hash
  holds a fact with the erased human as counterparty. The id plane's suite must
  prove that re-verifying with the same anchor yields the same `sub`
  ([id/005](../id/005-decisions.md), next action 2).
- An inferred fact is tombstoned when its last input is retracted or deleted.
- A silent person's location is gone from Redis after the presence retention.

## Not a boolean

None of this makes the person anonymous or the data gone from every copy in the
world: an organisation that fetched a projection may still hold it (O3 in
[id/005](../id/005-decisions.md)); backups age out on their schedule; no model is
trained on personal inputs in v1 ([001](001-sources.md)).

## Unrecoverable if wrong

- **Scope ids on every row.** `consent` stores registry scope ids forever, frozen
  like path names; a scope is added, never renamed. Splitting a tier later means a
  new id, not a rewrite.
- **The grace window.** Instant key destruction would make an account takeover
  plus `DELETE` final; the window trades seven days of unreadable, still-stored
  rows for recoverability. Proposed: 7 days.
