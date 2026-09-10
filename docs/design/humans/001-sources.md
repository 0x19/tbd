# 001 — Sources

Status: open — proposed for approval

Five sources, kept apart. What each is allowed to assert, and what `confidence`
means for each.

---

## The five

| Source | What it is | `confidence` | Example |
|---|---|---|---|
| **verified** | A vendor, or the EUDI wallet once it ships, asserted it under [id/002](../id/002-providers.md) and [id/007](../id/007-identity-anchors.md); we hold the normalised outcome, never the evidence | absent in the row; `origin.assurance_level` holds the id plane's grade and the path registry maps grades to a number at read time (illustrative: Low 0.6, Substantial 0.8, High 0.95, in whichever vocabulary id/002 settles on) | `identity.age.over_18 = true`, Didit outcome, registry-mapped |
| **declared** | The person typed it | 1.0: it is what they said, not a claim that it is true | `profile.intentions = [long_term]` |
| **inferred** | A model produced it from facts or media inputs | the model's own, calibrated per model version | `traits.warmth = 0.71`, engine/trait-v3, inputs listed |
| **symbolic** | A deterministic reading from birth data or name | absent; it is a reading | `readings.natal.sun = { sign: "leo", degree: 12.4 }` |
| **observed** | Something happened | 1.0 for events the system produced; 0.5 per reporting party for person-reported outcomes, 1.0 when both report | `outcomes.met = true`, both reported |

`confidence` is the producer's confidence in the assertion. It is never a
probability that a person *is* something, and a verified `over_18` is a vendor's
grade, not certainty: borrowed and synthetic documents exist
([id/001](../id/001-uniqueness.md), "What we cannot close"). Mapping grades at read
time keeps the bands changeable; a number frozen into every row would not be.

## Rules per source

**verified.** Written only from the id plane's provider outcome, with `origin`
holding `provider_id`, `assurance_level`, `expires_at` and the vendor `evidence_ref`
pointer from [id/002](../id/002-providers.md). Never a copy of a document.
`origin.expires_at` is mirrored into the clear `expires_at` column at write time;
a fact past it is excluded from `current` and the projection, and a job appends
its tombstone ([000](000-facts-ledger.md)). Served to organisations by the
id plane, not from here ([003](003-consent-and-erasure.md)).

**declared.** The person owns it and can change or retract it at any time
(retraction: [000](000-facts-ledger.md)). `journal.*` facts hold the reference to an
entry (content hash, kind, written_at) and any structured signal; the text itself
is an input, encrypted at rest under the `inputs` scope key
([005](005-encryption.md)), readable by the engine for inference and by the
person, never by an organisation, never in any projection.
This is the "Signals to the algorithm" idea from apex
(`apex/docs/system/profile-system.md`, section 3.2) that
[../000-premise.md](../000-premise.md) keeps, given a home.

**inferred.** Every inferred fact records the model version and its inputs, both
kinds: the facts it read and the raw inputs it read (`origin.inputs`, mirrored
relationally by `fact_inputs` in [002](002-storage.md)). It is recomputed when the
model changes and tombstoned when its last input is retracted or deleted. Two
rules the previous project broke:

1. No inference is written with `stub: false` unless the model ran. apex's RPCs
   returned `0.5` per trait with `success: true` when no engine was loaded
   (`apex/backend/src/main.rs:314-337`); here that value is absent or `stub: true`.
2. Inference of any special category under
   [GDPR Art. 9(1)](https://eur-lex.europa.eu/eli/reg/2016/679/art_9/oj), and
   anything face-derived, is **off by default** and exists only as an explicit,
   revocable opt-in the person can see the output of
   ([003](003-consent-and-erasure.md), engine capabilities). apex's
   `authenticity_score` and emotion analysis (`apex/docs/system/engine-image.md`)
   were the least defensible part of that design and are not carried over unless a
   later document argues for them.

No model is trained on personal inputs in v1; inference only. If that changes, it
is a new document, and it is the "Journal tension" Q5 in
[../001-open-questions.md](../001-open-questions.md) leaves open.

**symbolic.** Computed by deterministic code from `birth.*` and `profile.name`,
never by a model. The lens inventory carried over is apex's "cosmic data"
(`apex/docs/system/profile-system.md:359-364`: birth date, time and place to chart;
life path from the date) plus the frameworks 006 defines. Chart
positions are tested against an independent ephemeris as the oracle (Swiss
Ephemeris, whatever library computes ours). Presented everywhere as a reading.
Weighted into matching only for people who opted the lens in.

**observed.** The only source that can validate anything. Outcomes are the ground
truth Q2 and Q3 in [../001-open-questions.md](../001-open-questions.md) need;
without this source the matcher is apex again, a formula nobody can evaluate.

## Raw inputs

Photos and journal text are inputs, not facts. They are stored encrypted, one key
per input wrapped under the `inputs` scope key ([005](005-encryption.md)),
referenced from facts by content hash, and go with the human at erasure, the
human key destroyed after the grace window ([003](003-consent-and-erasure.md)). They are kept after inference so that a better
model can recompute `traits.*` later.

## Unrecoverable if wrong

- **The `confidence` contract per source** above. Every stored fact carries it
  forever.
- **`origin.inputs` on every inferred fact** from the first write. Without it,
  neither recomputation nor tombstoning-on-input-deletion can be applied.
- **Keeping raw inputs.** Keeping them is reversible (delete later); discarding
  them is not (no model upgrade can ever reach existing people). Proposed: keep,
  encrypted.

## Not decided here

Which models produce `inferred` facts and how they are evaluated. That is the
Engine question, and it stays behind Q2 and Q3.
