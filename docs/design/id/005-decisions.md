# 005 — Decisions

Status: decided — **D2 superseded by [007-identity-anchors.md](007-identity-anchors.md);
D1 and D7 sharpened; O1 resolved by [006-legal-erasure.md](006-legal-erasure.md)**

Every open question from 001–003 and `../001-open-questions.md`, resolved against
the evidence in [004-prior-art.md](004-prior-art.md). Where a decision is
provisional or needs outside input, it says so.

> **Revision, after primary-spec research.** Wallet credentials cannot anchor
> identity — mdoc is deliberately unlinkable — and we are not eligible for Apple's
> API. Read [007-identity-anchors.md](007-identity-anchors.md) before acting on D2,
> D3 or D4.

---

## D1 — What we sell is continuity

**Decision.** The product is **identity continuity**, not verification.

Verification is an input we buy for cents. Attribute assertion is being given away
free by Apple and Google. Proof of personhood belongs to World ID. What no one
provides is a persistent, per-organisation identity that remembers a human, carries
their presentation, and grows a model of them.

Three layers express it, and each is something the incumbents structurally lack:

1. Persistent per-org pseudonyms → ban enforcement that actually works
2. Personas → context separation with one verified human underneath
3. Engine representation → an org understands a brand-new user on day one

**Consequence:** we never position as an IDV vendor, never compete on verification
price, and never build verification ourselves.

**Sharpened after research.** Uniqueness is the *wedge*; **portability is the
moat**. Apple, Google, World ID and EUDI all scope identity per relying party
precisely so it cannot travel — portability requires correlation, which is the thing
they are built to prevent. Consented portability is the one capability they cannot
add without abandoning their own privacy model. See
[007-identity-anchors.md](007-identity-anchors.md).

## D2 — Verification sources ⚠️ SUPERSEDED

> **Superseded by [007-identity-anchors.md](007-identity-anchors.md).** Wallet-first
> was wrong. mdoc uses batch-issued one-time credentials and is unlinkable by
> design, so wallets return no persistent identifier; and dating is not among
> Apple's twelve approved categories. **Vendor IDV document hash is the primary
> anchor at launch;** EUDI wallet pseudonym becomes primary once Croatia ships
> (target Dec 2026). Wallets are for age assurance only, via a vendor holding the
> entitlement.
>
> Original decision retained below for the reasoning trail.

**Original decision.** Four tiers, tried in order. All behind the `providers`
abstraction in [002-providers.md](002-providers.md).

| Order | Source | Why first |
|---|---|---|
| 1 | **Platform wallet credentials** — Apple Verify with Wallet, Google Digital Credentials API, EUDI Wallet | Free, instant, no document upload, highest assurance, ~99% smartphone coverage, selective disclosure built in |
| 2 | **World ID** | Free personhood proof for ~38M people who already have one. Their Orb friction, not ours |
| 3 | **Vendor IDV** — Didit first | $0.33/check, 500 free monthly, reusable KYC and biometric matching in one API. Covers everyone the above miss |
| 4 | **Age estimation** — document-free facial | Cents. Sufficient where only an age gate is required |

Wallet-first is the strategic call. It inverts the usual model: instead of treating
OS-level identity as a threat, we consume it and add the layer it deliberately omits.

## D3 — Uniqueness layers for v1

**Decision.** Ship layers 1 + 2. Layer 3 available as a paid escalation. Layer 4
as signal only.

- **Layer 1** — vendor duplicate detection within our tenant
- **Layer 2** — salted document hash, peppered with a KMS-held secret, derived not stored
- **Layer 3** — biometric 1:N as a *vendor capability*; templates stay with the vendor
- **Layer 4** — device / SIM attestation, feeds the confidence score only

Rationale: Sentinel failed because it matched PII. Layers 1+2 are cryptographic and
survive the exact attack that beat Match Group — changing name, email and photos.
Layer 3 closes the multi-document gap but costs a DPIA, so it waits for a customer
who requires it.

Uniqueness is emitted as **confidence, never a boolean**. Unchanged.

**Revised weighting.** Layer 2 is no longer a hedge against vendor lock-in — it is
**the mechanism**. Wallets cannot anchor, World ID does not cover Croatia, and EUDI
is months away. The document hash is the only anchor with both persistence and
coverage on day one. See [007-identity-anchors.md](007-identity-anchors.md).

## D4 — One vendor at launch

**Decision.** **Didit** as the single IDV vendor, with the abstraction built for
many from day one.

$0.33/check with 500 free monthly means development and early operation cost
essentially nothing. Reusable KYC and biometric matching are in the same API, so
layer 3 needs no second integration. Layer-1 dedupe works within a single tenant,
which is exactly what a single vendor gives us.

Layer 2 (document hash) is what prevents lock-in — it makes the identity graph
survive a vendor switch. **Build layer 2 in v1 even though layer 1 alone would
work**, precisely so the vendor decision stays reversible.

## D5 — Provider capability scope

**Decision.** In scope for v1:

- `WalletCredentialProvider` — Apple / Google / EUDI
- `PersonhoodProvider` — World ID
- `VerificationProvider` — document + liveness
- `AgeEstimationProvider` — document-free

Out of scope, deferred: `ScreeningProvider` (AML/PEP — irrelevant to dating),
`BackgroundCheckProvider` (heavy jurisdiction-specific liability; a prior Tinder
attempt did not stick; revisit only on customer demand).

## D6 — Trust model: centralised registry, standards everywhere else

**Decision.** We hold a **minimal central uniqueness registry** and nothing more.
Everything else runs on open standards: OIDC + OAuth 2.1 for the relying-party
surface, ISO 18013-5 / W3C Digital Credentials for input, SD-JWT VC where we issue.

Pure self-sovereign identity is rejected: a decade of weak adoption, and it makes
uniqueness and ban enforcement close to impossible — which is the entire product.

Full centralisation is rejected too: we hold no documents, no biometric templates,
no ban lists, no reputation.

The registry stores derived scoped pseudonyms, not identities. Align terminology
with the personhood-credentials literature.

## D7 — The wedge is ban enforcement

**Decision.** Lead with **re-registration prevention**, not age verification.

Age assurance looked like the funded wedge until we found that Apple and Google are
giving it away. Selling a commodity that ships free in the OS is a losing position.

Ban enforcement is the opposite: publicly broken at the largest player, actively
litigated, and impossible to solve with the PII matching everyone currently uses.
It is also the thing only we can do, because it requires exactly the continuity
layer from D1.

**Sequence:** our own app as relying party #1 and reference implementation →
long-tail dating and social apps → larger platforms once the network has weight.
Match Group is validation of the problem, **not a prospect** — they build in-house.

⚠️ **The wedge has a shelf life.** eIDAS 2.0 requires every EUDI wallet to emit
RP-scoped pseudonyms, which lets an EU organisation enforce its own bans without us
once adoption is real. Uniqueness gets us in the door; it does not keep us there.
Build the portability layer — personas, portable profile, Engine representation —
while the door is open.

## D8 — Data tiers in v1

**Decision.** Ship tiers 1–3. Tier 4 (Engine representation) is a separate,
explicit, revocable opt-in with its own consent flow — never an OAuth scope.

Tier 1 alone is not a product (D1). Tiers 2–3 — portable profile and preferences —
are where the conversion win lives, and conversion is the real economics (see the
correction in [004-prior-art.md](004-prior-art.md)).

## D9 — Engine attaches to identity, projects per persona

**Decision.** The Engine models the **identity** — one human, one model. Projections
to an organisation are **persona-scoped**.

A person's underlying disposition does not fork when they create a second persona,
so modelling per persona would fragment the signal and weaken every prediction. But
what an organisation receives must respect the persona boundary, or personas stop
providing the unlinkability they exist for.

## D10 — EUDI is a tailwind

**Resolved by research.** eIDAS 2.0 in force since 20 May 2024. Every member state
must offer a certified EUDI Wallet by **December 2026**; private-sector acceptance
obligations for regulated sectors land **December 2027**.

Not an existential threat, because we consume wallet credentials rather than
competing with them (D2). Croatia's EU membership is an advantage: we are inside
the regime, early, with the standards maturing exactly as we build.

## D11 — Repricing the pitch

**Decision.** Stop saying "cheaper." Say **"frictionless."**

Verification runs $0.33, so reuse across five apps saves about $1.65. The saving is
not the point. Verification and profile re-entry are the two largest drop-off points
in dating onboarding; removing both for every app after the first is worth far more
than the fee.

---

## Still genuinely open

Not resolvable from research. Listed so they are not mistaken for settled.

| # | Question | Blocked on |
|---|----------|-----------|
| ~~O1~~ | ~~Does derive-don't-store survive GDPR Art. 17?~~ → **provisionally yes**, see [006-legal-erasure.md](006-legal-erasure.md). Suppression-list precedent + Recital 47 + Art. 17(3). Needs DPIA, LIA, and counsel on 5 named questions | Lawyer, before launch |
| O2 | Persona limits — cap the count? Unlimited is a content-farming and catfishing surface even though it cannot defeat bans | Product judgement |
| O3 | What does revocation oblige an org to do — stop future reads, or delete what it holds? | Relying-party agreement drafting |
| O4 | Photo hosting: do we serve images, hand over URLs, or transfer copies? Determines whether revocation is meaningful, and whether we become a CDN | Architecture, after O3 |
| O5 | Pricing shape — per verification, per connection, per tier? Tier 4 is worth far more than tier 1 | Needs a first design partner |
| O6 | Re-verification cadence and who pays for it | Needs a first design partner |
| O7 | Does accepting World ID create a dependency we regret if their terms change? | Read their RP terms — **lower stakes now**: World ID is opportunistic, not a dependency (no Croatia coverage) |
| O8 | Can we reach wallet acceptance via the Verifier Registrar flow, or must a vendor hold the entitlement for us? | Read Google/Apple VR terms |
| O9 | What is the EUDI integration shape, and when does Croatia's wallet actually ship? | Track Croatian implementation; ARF 2.x |

## Next actions — revised after research

Actions 1 and 2 from the previous revision are **done**: see
[006-legal-erasure.md](006-legal-erasure.md) and
[007-identity-anchors.md](007-identity-anchors.md). Action 3 changed shape — there is
no wallet path to spike, because wallets cannot anchor identity.

1. **Counsel on the five questions in [006](006-legal-erasure.md).** Q1 (does the
   suppression-list analogy hold for a third party's decision?) and Q3 (controller
   vs processor for the ban decision) gate everything. Do not build past the
   `providers` crate without them.
2. **Spike Didit end to end** — enrol, receive webhook, derive a document hash,
   detect the same person returning after deletion. This is now *the* mechanism, so
   it is the thing that must be proven first. Free tier covers it entirely.
3. **Design `person_id` anchor-agnostically** — see
   [007](007-identity-anchors.md). Adding a stronger anchor later must raise
   confidence without changing any existing `sub`, or every org's ban list breaks
   on upgrade. Not recoverable if we get it wrong.
4. **Track EUDI**: when does Croatia's wallet ship, and what is the pseudonym API?
   It becomes the primary anchor the moment it exists.
5. **Then** the `providers` crate proper, with Mock and Didit.
6. **Then** OIDC surface + registry.
7. **Engine last** — and Q2/Q3 in `../001-open-questions.md` are still unanswered
   and still blocking it.
