# 000 — Facts ledger

Status: open — proposed for approval

A Human is an append-only ledger of facts, not a record with columns.

---

## The decision

Every piece of knowledge about a person is one **fact**:

```
Fact {
    human_id,          // opaque uuid minted at enrolment; never person_id, never sub
    path,              // "traits.warmth", "identity.age.over_18", "birth.time", "relations.match.<id>"
    value,             // JSON, encrypted under a per-fact key wrapped per (human, scope, persona) (005); null is a tombstone
    source,            // verified | declared | inferred | symbolic | observed   (001)
    origin,            // who or what produced it; encrypted with value; for inferred: { model, inputs: { facts: [id], inputs: [hash] } }
    confidence,        // the producer's confidence in the assertion, 0..1; absent for symbolic and verified (001)
    counterparty_id,   // the other human for relations.* and pair facts; null otherwise
    observed_at,       // when it was true
    recorded_at,       // when we learned it
    consent,           // registry scope ids under which it may be read (003)
    stub: bool,        // true when the producer could not do the real computation
}
```

The ledger itself is a generic crate that knows the subject only as an opaque id;
`humans` is what makes the subject a person (plane [README](README.md), "Crate
boundary"). The ledger key never leaves the `humans` service; the identifier on
the public surface is the caller's pairwise `sub` ([004](004-surface.md)). The
service holds no `sub` of its own: resolution is a call to the id plane.

This ledger holds what **our own app** (Resource Server #1 under
[id/008](../id/008-federation.md)) knows. Other organisations' data is brokered,
never copied here.

Facts are never updated in place. A new value is a new fact; `current` is the
latest fact per `(human_id, path, source)`, materialised, excluding facts whose
clear `expires_at` column (mirrored from `origin.expires_at` at write time) has
passed. History is the ledger.

**Retraction is the one exception to append-only.** When a person deletes a
declared fact, or an inferred fact loses its last input, the rows are physically
deleted and a **tombstone** fact (`value: null`, same path and source, new
`recorded_at`) is appended in the same transaction. Our reading of Regulation
(EU) 2016/679: erasure of a single value under
[Art. 17(1)](https://eur-lex.europa.eu/eli/reg/2016/679/art_17/oj) requires the
value to be gone, while rectification under
[Art. 16](https://eur-lex.europa.eu/eli/reg/2016/679/art_16/oj) may be met by the
newer fact or a supplementary statement. Pending the counsel
[id/006](../id/006-legal-erasure.md) requires.

## Why a ledger and not a schema

apex modelled a person three ways that never agreed: the proto
(`apex/proto/apex.proto:219-242`, `PersonalityProfile` and `TraitScores`), the Dart
`ProfileData` (`apex/mobile/lib/screens/profile/profile_data.dart:5-184`) and the
design doc (`apex/docs/system/profile-system.md`). Nothing carried provenance, so
the trait heuristic that read fixed embedding dimensions
(`apex/backend/src/engine/pipeline/profile_analyzer.rs:190-218`) and the fallback
that returned `0.5` for every trait with `success: true` and no stub marker
(`apex/backend/src/main.rs:314-337`) were indistinguishable from measurements.

A ledger fixes both:

- **Many to one.** A first-party surface, a lens or a KYC vendor adds paths, not
  migrations. Two sources can hold the same path at once.
- **Provenance is structural.** A reader that asks for `age` gets the facts with
  their sources and decides, or asks for one source.
- **Time is structural.** "What did we know on the day of the match" is a query on
  `recorded_at` over the facts still held, which is what evaluating the matcher
  (Q3) needs. Retracted values are gone from every such cut; see below.

## Paths

Paths are the contract. They are namespaced and versioned in the name when the
value schema changes (`traits.v2.warmth`), never rewritten in place. A registry in
the repository lists every path, its value schema, which sources may write it and
its default scopes. Unknown paths are rejected at write time.

Reserved namespaces for v1:

| Namespace | Holds | Writers |
|---|---|---|
| `identity.*` | mirror of the id plane's verified attributes, for the engine: `age.over_18`, `liveness`, `uniqueness_confidence` (the enum plus evidence, never a boolean) | verified |
| `birth.*` | date, time, place as the person gives them; feeds the lenses; a strong re-identifier. A vendor-verified date of birth stays in the id plane and is not mirrored | declared |
| `profile.*` | photo references, bio, basics, intentions | declared |
| `journal.*` | references to entries (content hash, kind, written_at) and signals; the text lives in `inputs` ([002](002-storage.md)) | declared |
| `traits.*` | what the engine infers: disposition, communication style, warmth | inferred |
| `readings.*` | chart positions, numerology, the lens outputs (006, not written) | symbolic |
| `relations.*` | matches, blocks, sparks, pair values, one fact per side with `counterparty_id` | observed, declared, inferred |
| `outcomes.*` | what happened: met, rated, returned | observed |

Current location is **not** a path. It is presence, held only in the projection
with an expiry ([002](002-storage.md)); a ledger of locations would be the history
apex promised never to keep ([../000-premise.md](../000-premise.md), "What we drop").

## Relations are facts too

A match between A and B is two facts, `relations.match.<id>` on each human, each
with the other as `counterparty_id` and the same `origin`. Pair values
(compatibility and whatever 006 defines) are facts on both sides with
`source: inferred`. `counterparty_id` is a foreign key that cascades, so erasing A
removes the pair from both sides in Postgres at commit, with a tombstone appended
on B's side (`counterparty_id: null`). B's Redis projection is rebuilt through the
outbox afterwards; the [003](003-consent-and-erasure.md) scenario asserts both.

## Unrecoverable if wrong

- **The key.** Every fact is keyed by the opaque `human_id`. Keying by identity
  ([id/005 D9](../id/005-decisions.md)) is decided; keying by the derived pseudonym
  would put `person_id` into the engine, which [id/006](../id/006-legal-erasure.md)
  forbids, and is not reversible once written.
- **Path names in the public API.** Renaming a path breaks every consumer;
  version in the name from day one.
- **Provenance columns.** `source`, `origin`, `recorded_at` and `origin.inputs` on
  every inferred fact are required from the first write. A fact written without
  them is unsourced forever.
- **The meaning of `confidence` per source** ([001](001-sources.md)). Facts are
  never rewritten, so the day-one contract is the contract for every stored row.
- **Physical deletion on retraction.** A retracted value is absent from every
  history cut. Q3's evaluation must be designed on what survives (observed outcomes
  and pair facts still attached to both humans), or retaining retracted values
  under Art. 17(3) goes to counsel as a new question in id/006. Decide before the
  first retraction. Proposed: delete; privacy over evaluation.

## Not decided here

Which sources may write which paths ([001](001-sources.md)); where it is stored
([002](002-storage.md)); how grants scope a read and what erasure removes
([003](003-consent-and-erasure.md)); how it is exposed ([004](004-surface.md));
how values are encrypted ([005](005-encryption.md)); the lenses (006, not written).
