# 001 — Open questions

Status: superseded in part by [id/005-decisions.md](id/005-decisions.md)

> **Q1, Q4, Q5 and the identity half of Q6 are answered.** See
> [id/004-prior-art.md](id/004-prior-art.md) for the research and
> [id/005-decisions.md](id/005-decisions.md) for the decisions.
> **Q2 and Q3 — the smallest proof, and how we measure the Engine — remain open
> and are now the blocking questions for the Engine plane.**

What has to be decided before any code is written. Ordered by how much the
answer constrains everything downstream.

Each question gets its own numbered document once we're ready to settle it.
Until then, this is the parking lot. Nothing here is a decision.

---

## Q1. What is the product, exactly? — ANSWERED

**Not a dating app. An identity continuity layer for dating and social platforms**,
with our own app as relying party #1. See [id/005-decisions.md](id/005-decisions.md) D1, D7.

The cold-start worry below is resolved: as an identity provider we bootstrap from
our own app rather than needing density per square kilometre. Croatia-first remains
right — the EU is the favourable regime here (EUDI, D10), not a limitation.

<details><summary>Original framing, kept for the reasoning trail</summary>

### Q1 (original). What is the product, exactly?

apex was "Tinder but compatibility-first and proximity-gated, in Croatia".
That's a positioning, not a product definition.

- Is this still dating? Or is the proximity + compatibility mechanic better
  aimed somewhere with less brutal cold-start economics — meeting people at
  conferences, in a new city, co-working, hobby groups?
- Dating apps need dense two-sided liquidity in a small geographic radius or
  the app is empty and dies. Proximity matching makes this *harder*, not
  easier: you need density per square kilometre per hour, not per city.
  **Does the proximity mechanic survive contact with the cold-start problem?**
  If not, everything else is moot.
- Croatia-first: still right? It was chosen (Nominatim runs a Croatia-only OSM
  extract, the app geo-fences to a Croatian bounding box) but never argued.

**Constrains:** everything.

</details>

## Q2. What's the smallest thing that proves the premise?

The central claim is that compatibility can be predicted well enough that
people accept matches sight-unseen and enjoy meeting.

That claim is testable *without* an app. Possibly without a backend.

- What's the cheapest experiment that produces evidence? A batch script over
  volunteer profiles, scored, then asked "would you want to meet this person?"
- What result would make us confident? What result would kill the idea?
- Can this run before we choose a database, a language, or a platform?

**Constrains:** whether we build a product next or a research harness next.

## Q3. How do we know matching works?

apex never answered this and it's why the matcher was never real. It shipped a
weighted formula (35% personality / 25% communication / 20% interests / 15%
vibe / 5% values) with hand-picked constants, fed by placeholder traits, with
no way to evaluate it.

- What's the ground truth? There is no labelled dataset of "these two people
  got along".
- Can we bootstrap one? Self-reported compatibility between people who already
  know each other is a plausible cheap proxy.
- **Design the evaluation before the model.** This is non-negotiable and it's
  the single biggest process change from apex.

**Constrains:** the entire ML approach.

## Q4. ML approach — DEFERRED to the Engine plane

Unchanged by identity research. Qwen3-VL 8B and BGE-M3 are on disk at
`/mnt/raid0/proximity/models/`. Blocked on Q2 and Q3 below, not on stack choice.

<details><summary>Original framing</summary>

### Q4 (original). VLM-first, or task-specific models?

Already on this machine at `/mnt/raid0/proximity/models/`:

| Model | Size | Role |
|---|---|---|
| Qwen3-VL 8B Instruct (Q4_K_M, GGUF) | 5.0 GB | Vision-language, served by Ollama |
| BGE-M3 (ONNX + data) | 2.3 GB | Multilingual text embeddings |

One VLM plausibly replaces apex's face detection + emotion + age/gender + face
parsing with a single prompt. BGE-M3 replaces EmbeddingGemma and handles
Croatian natively.

- Is a VLM's read of a photo actually *better* for our purpose than
  purpose-built classifiers, or just fewer moving parts?
- Latency and cost: 8B Q4 on CPU vs. GPU. What's the real per-profile budget?
- Structured output reliability — can we get consistent, parseable traits out
  of it, or do we end up writing a parser that fails on 3% of profiles?
- Do we even want photo-derived personality signals? apex's `authenticity_score`
  and emotion analysis were the least defensible parts of the design, both
  technically and ethically.

**Constrains:** hardware, latency budget, and how much of the product is ML at all.

</details>

## Q5. Privacy contract — ANSWERED for identity, open for the Engine

Identity side settled: we hold no documents, no biometric templates, no ban lists,
no reputation. Only derived scoped pseudonyms. See
[id/001-uniqueness.md](id/001-uniqueness.md) and [id/005-decisions.md](id/005-decisions.md) D6.

⚠️ One question gates the whole mechanism and needs a lawyer, not an engineer:
**does derive-don't-store survive GDPR Art. 17?** Tracked as O1 in
[id/005-decisions.md](id/005-decisions.md).

The Journal tension below is unresolved and belongs to the Engine plane.

<details><summary>Original framing</summary>

### Q5 (original). What is the honest privacy contract?

apex's public privacy table and its actual design were in tension, and one item
was outright contradicted by the code (see 000, "What we drop").

The sharpest unresolved tension: `profile-system.md` describes a private
Journal — free-text entries about struggles, patterns, what you actually want —
that "AI reads for deep matching". That is the most sensitive data in the
product and the most useful. It is also nowhere in the public privacy
disclosures.

- What do we actually collect, and what does the user see about that?
- Where does inference run, and does that answer change the promise we can make?
- What's the deletion story? A model trained or an embedding derived from
  deleted data is not deleted.
- **Rule going in:** every privacy claim must be enforced by code and
  covered by a test that fails if it regresses.

**Constrains:** data model, storage, hosting, and legal posture.

</details>

## Q6. Architecture and stack — ANSWERED for identity

OIDC + OAuth 2.1 public surface, ISO 18013-5 / W3C Digital Credentials as input,
minimal central registry, `providers` crate behind it. See
[id/005-decisions.md](id/005-decisions.md) D2, D6.

Storage, language and client choices for the Engine remain deliberately open.

<details><summary>Original framing</summary>

### Q6 (original). Architecture and stack

Deliberately last. These are the questions that feel most urgent and matter
least, and answering them first is how apex ended up with SurrealDB and Ollama
running empty and a backend that doesn't compile.

- Storage: Redis-only was apex's choice and it broke on the first real
  requirement (expiring geo members). SurrealDB is running here with zero data.
  Postgres + PostGIS is the boring answer and boring is currently winning.
- Rust again? The apex backend was clean and the crate split in
  `/opt/proximity/backend/crates` was genuinely good work. But if Q2 says the
  next thing is a research harness, Python is the obvious tool for that phase.
- Client: Flutter again, or web-first? apex's Flutter app was 22k lines of
  polished UI wired to nothing. Web-first is faster to a testable loop.
- Protocol: apex ran gRPC + HTTP + WebSocket simultaneously from day one, for
  a client that used none of them.

**Constrains:** nothing yet. Decide after Q1–Q3.

</details>

---

## Sequencing

```
Q1  what is it          ── ANSWERED: identity continuity layer
Q5  privacy contract    ── ANSWERED for identity (O1 needs a lawyer)
Q6  stack               ── ANSWERED for identity
                             │
                             ▼
Q2  smallest proof      ─┐
Q3  how we measure      ─┘  ← BLOCKING, and now Engine-only
                             │
                             ▼
                        Q4  ML approach
```

The identity plane is designed. **Q2 and Q3 are the next conversation**, and they
are now purely about the Engine: what is the smallest thing that proves a machine
can predict who you will enjoy meeting, and how would we know it worked.

Do not open Q4 until they are `decided`. That ordering is the single biggest
process change from apex, where the matcher was built before anyone could measure
whether matching worked.
