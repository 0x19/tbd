# 002 — Providers

Status: decided — scope and vendor resolved in [005-decisions.md](005-decisions.md) (D2, D4, D5)

The abstraction over third-party verification vendors.

---

## Position

**We are not a verification vendor and will not become one.**

Persona, Veriff, Sumsub, Onfido and Yoti already do document verification and
liveness well and cheaply. That market is crowded and well-funded, and entering it
buys us nothing. We orchestrate, normalise, and make the result portable.

Two consequences worth being explicit about:

1. **Our breach exposure collapses.** We never hold passport images or biometric
   templates — the vendor does, under its own compliance regime. Without this
   constraint we would be the most attractive target in the sector: Ashley Madison
   with passports attached.
2. **IDV is a commodity; portability is the product.** Our differentiator is the
   network and the reuse, never the verification itself.

## Prior art worth reusing

apex's `backend/src/providers/` — trait-per-capability with swappable
implementations (`LocationProvider`, `MatchRepository`, `NotificationProvider`) —
was the single best-designed thing in that codebase. Same shape here, except now
it is load-bearing rather than incidental.

## Trait per capability

Routing selects on *capability*, not vendor:

```
VerificationProvider     document + liveness
AgeEstimationProvider    facial age estimation, no document
ScreeningProvider        AML / PEP / sanctions          — if in scope
BackgroundCheckProvider  criminal records               — if in scope
```

Capability rather than vendor is what lets us route by jurisdiction, cost or
document type without the call sites knowing which vendor answered.

## Normalised outcome

Vendors differ wildly in result schema, flow style (hosted / redirect / SDK) and
webhook semantics. One internal representation:

```
VerificationOutcome {
    assurance_level,     // normalised vocabulary — see below
    attributes,          // age_over(n), name, jurisdiction, …
    evidence_ref,        // pointer to vendor-held evidence, for disputes
    provider_id,
    verified_at,
    expires_at,
}
```

`evidence_ref` is a pointer, never a copy. We can resolve a dispute by asking the
vendor; we cannot leak what we do not store.

## The assurance vocabulary is the actual product

**This is what makes us an identity layer rather than a vendor proxy.**

Every provider result maps into one normalised scale — NIST 800-63 IAL, or eIDAS
LoA (Low / Substantial / High). Without this, an organisation integrating us has to
understand each vendor's idiosyncratic result format, and we have added no value
over them integrating the vendor directly.

With it, an organisation says *"I require Substantial"* and never thinks about
vendors again. That is the whole pitch, compressed into one field.

## Routing

A real product feature, not plumbing:

- **Jurisdiction** — vendor coverage and accuracy varies sharply by country
- **Document type** — not every vendor handles every document
- **Cost** — cheapest vendor that meets the required assurance level
- **Fallback** — vendor down, or verification failed for a recoverable reason

## Cross-cutting mechanics

Every vendor integration needs these, and every vendor does them differently. They
belong in the abstraction, not in each implementation:

- **Webhook signature verification** — different scheme per vendor
- **Idempotency** — webhooks retry; the same outcome must not be applied twice
- **Session lifecycle** — start, poll, expire, resume
- **Sandbox credentials** — every vendor's test environment behaves differently
- **Rate limits and backoff**

## The Mock provider is mandatory

From the first commit, not "when we get around to testing."

apex's defining failure was integrations that were never exercised end to end — a
Flutter app that never called the backend, a gRPC client that was generated and
never used, analyzers that returned hardcoded values while an accuracy report was
run against them. A `Mock` provider that exercises the full flow (session, webhook,
outcome, failure modes) is how we avoid discovering the abstraction is wrong six
months in.

It also means local development needs no vendor account and costs nothing.

## Open questions

**Capability scope.** IDV only? IDV plus age estimation? Screening? Background
checks?

Age estimation is worth calling out: it is document-free, costs cents rather than
dollars, and serves the age-assurance mandates that are currently forcing platforms
to buy compliance. That funded, non-speculative demand may well be the commercial
wedge, even though it is the least glamorous slice of the vision.

Background checks are the dating-safety angle that would most interest a large
platform — and carry heavy, jurisdiction-specific legal exposure. A prior Tinder
attempt at third-party background checks did not stick; worth studying before
committing.

**Crate layout.** Single crate with per-vendor feature flags, or `providers-core`
(traits) plus `providers-{vendor}` crates? Feature flags are simpler; separate
crates avoid compiling every vendor SDK. **Leaning feature flags** until there is
evidence the compile cost matters.

**Vendor selection.** Blocked on capability scope, on pricing at our expected
volume, and on which vendors actually expose duplicate detection —
see [001-uniqueness.md](001-uniqueness.md), layer 1.

**How many vendors at launch?** One vendor makes layer-1 dedupe work out of the box
but creates lock-in. Multi-vendor forces the abstraction to be honest early and
proves layer 2 works, at the cost of doubling sandbox and webhook work before
anything ships.
