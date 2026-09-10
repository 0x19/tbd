# 007 — Identity anchors

Status: decided — supersedes D2 in [005-decisions.md](005-decisions.md)

What each verification source actually provides, verified against primary specs.
**This document overturns the wallet-first ordering.**

---

## Definition

An **anchor** is anything that lets us recognise the same human on a later visit. It
is the input to `person_id`. Not every verification source is an anchor — most
deliberately are not.

Two properties matter, and they are independent:

- **Assurance** — how confident are we this is a real, distinct person?
- **Persistence** — will the same human produce the same value next time?

**Persistence is the scarce one.** Nearly every modern identity system is
engineered to destroy it.

## What each source actually provides

| Source | Assurance | Persistent? | Coverage for us | Verdict |
|---|---|---|---|---|
| **Apple Verify with Wallet** | High | ❌ **No — by design** | Not eligible | Age assurance only |
| **Google Digital Credentials** | High | ❌ No | Unclear eligibility | Age assurance only |
| **World ID nullifier** | Very high (Orb) | ✅ **Yes, rp_id-scoped** | ❌ Not Croatia | Opportunistic |
| **EUDI Wallet pseudonym** | High, state-backed | ✅ **Yes, RP-scoped, mandated** | ✅ Croatia, Dec 2026 | **Strategic** |
| **Vendor IDV document hash** | Medium-high | ✅ Yes | ✅ Everywhere, now | **Primary at launch** |
| **Vendor biometric 1:N** | High | ✅ Yes | ✅ Everywhere | Escalation |
| **Device / SIM attestation** | Low | Partial | ✅ Everywhere | Signal only |

### Wallets cannot anchor. This is deliberate.

ISO 18013-5 mdoc achieves unlinkability through **batch-issued one-time
credentials**: the wallet holds many credentials, each with freshly salted attribute
hashes, and uses each once. Two presentations by the same person are
cryptographically unlinkable.

Apple states it directly: *"there is no persistent identifier returned that would
allow a business to recognize the same person across separate verifications."*

`document_number` **is** technically available in the identity-verification data
set — but requesting it to build a persistent identity graph runs against the stated
design intent, and see eligibility below.

### We are not eligible for Apple's API

Approved categories: Access (Physical Security), Air Travel, Alcohol Purchase, Car
Rental, Financial Services, Gig Economy, Government Services, Healthcare,
Hospitality, Insurance, Scooter Rentals, Ticketing.

**Dating is not on the list. Neither is social, nor identity provider.** Entitlements
are granted per bundle ID and bound to a single binary — an awkward fit for an IdP
serving many organisations.

*Route if we need it later:* the **Verifier Registrar flow** lets an IDV provider or
aggregator act as its own CA and manage onboarding for downstream relying parties.
Persona already accepts Apple Wallet mDLs on behalf of entitled customers. So wallet
acceptance is reachable *through a vendor* without our own category approval.

### World ID is our mechanism, already built

The World ID **nullifier** is deterministic per (person, context), unlinkable across
contexts. In World ID 4.0 nullifiers are scoped to `rp_id`. That is
`HMAC(rp_salt, person_id)` with a zero-knowledge proof around it — our design,
shipped, at 38M users.

Blocked on coverage, not merit. The World ID 4.0 passport credential covers
Argentina, Chile, Colombia, Costa Rica, Japan, Malaysia, Mexico, Panama, South Korea,
Taiwan, UK, US. **Croatia is absent**, and Orb density in the EU is thin. Orb Mini is
slated for 2026 and may change this.

### eIDAS 2.0 mandates exactly what we need

Two binding requirements in the regulation:

1. Wallet Units **must** be able to generate pseudonyms and store them encrypted
2. **Pseudonyms must be unique to each Relying Party**

Relying parties must accept pseudonymous identification where no legal
identification requirement exists. Every member state must ship a certified wallet
by **December 2026**.

State-backed, free, pairwise pseudonymity, in our home market, on a known date.

## The strategic consequence

If an organisation receives a stable per-RP pseudonym directly from the EUDI wallet,
**it can enforce its own bans without us.** Uniqueness gets commoditised in the EU,
much as attribute verification already has been.

That would be fatal, except for one structural fact:

> **Every one of these systems is engineered to prevent portability.**
> Apple, Google, World ID and EUDI all scope identity per relying party *precisely
> so it cannot travel between them.* Portability requires correlation, and
> correlation is the thing they are built to prevent.

Portability-with-consent is the one capability none of them can offer without
abandoning their own privacy model. It is not a feature they have failed to ship —
it is one they have deliberately excluded.

**Revised positioning:**

- **Uniqueness is the wedge.** Urgent, painful, demonstrably unsolved
  ([004-prior-art.md](004-prior-art.md)) — and with a shelf life in the EU once
  EUDI lands.
- **Portability is the moat.** Structural, durable, and unavailable to the
  incumbents by their own design.

Personas, portable profile, and the Engine representation are the expression of the
moat. Uniqueness is how we get in the door.

## Revised anchor ordering — supersedes D2

Try in order, take the strongest available, record which fired:

1. **EUDI Wallet pseudonym** — once available (target Dec 2026, Croatia). Highest
   assurance, zero cost, state-backed, natively RP-scoped.
2. **Vendor IDV document hash** (Didit) — **the primary anchor at launch.** The only
   option with both persistence and coverage today.
3. **Vendor biometric 1:N** — escalation. Closes the multi-document gap.
4. **World ID nullifier** — accept opportunistically where a user already has one.
   Free assurance; never a dependency.
5. **Device / SIM attestation** — confidence signal only, never an anchor.

**Wallet credentials (Apple/Google) are used for age assurance only**, and only
through a vendor holding the entitlement.

This inverts D2. The document hash is not a hedge against vendor lock-in — **it is
the mechanism.** Everything else is either unavailable to us, not yet shipped, or
not persistent.

## Design consequence: anchor-agnostic person_id

Since anchors differ per user and change over time (a user gets an EUDI wallet in
2027 who joined on a document hash in 2026), `person_id` must not be tied to one
anchor type:

```
person_id ← resolve(anchors[])       // any anchor may resolve to an existing person
sub_for_org = HMAC(org_salt, person_id)
```

Anchors attach to a person; the person is the stable thing. Adding a stronger anchor
later must **raise** the uniqueness confidence without changing any existing `sub` —
otherwise every org's ban list breaks on upgrade.

**This is the single most important structural requirement to get right in v1.**
Getting it wrong is not recoverable later.

## The Ghost Trilemma — a choice, not an oversight

[SoK: The Ghost Trilemma](https://arxiv.org/pdf/2308.02202) holds that sentience,
location and uniqueness cannot all be verified in a **fully decentralised** setting.

We escape it only by **not being decentralised**: we are a trusted centre holding a
minimal registry. That is a deliberate trade — it buys uniqueness and portability at
the cost of requiring users to trust us. Every privacy commitment elsewhere in these
documents exists to make that trust defensible.

Related work worth tracking: *Proof-of-Uniqueness* (eprint 2026/1725) achieves
sybil-resistance via threshold-OPRF and a zk-SNARK registry — no central biometric
store. Enrolment costs ~615k gas and ~65s proof construction, so not viable for us
now, but it is the shape of a future where we would not need to be the trusted
centre.

## Sources

- [Apple: Get started with Verify with Wallet](https://developer.apple.com/wallet/get-started-with-verify-with-wallet/)
- [Google: Verify with Google Wallet FAQ](https://developers.google.com/wallet/identity/verify/faq)
- [Persona: Verify Apple Wallet driver's licenses](https://withpersona.com/blog/new-at-persona-mobile-drivers-license-mdl-verification-via-apple-wallet/)
- [EUDI ARF — Topic E: Pseudonyms](https://eudi.dev/1.6.0/discussion-topics/e-pseudonyms-including-user-authentication-mechanism/)
- [World ID 4.0 specs](https://github.com/worldcoin/world-id-protocol/blob/main/docs/world-id-4-specs/README.md)
- [World ID passport credential launch](https://world.org/blog/announcements/new-world-id-passport-credential-launches-access-wld-tokens)
- [Invisible Traces: Subversion Attacks on Batch-Issued Credentials](https://eprint.iacr.org/2026/1229.pdf)
- [SoK: The Ghost Trilemma](https://arxiv.org/pdf/2308.02202)
- [Proof-of-Uniqueness (eprint 2026/1725)](https://eprint.iacr.org/2026/1725)
