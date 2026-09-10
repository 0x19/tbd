# 001 — Uniqueness & re-registration

Status: decided — layers resolved in [005-decisions.md](005-decisions.md) (D3, D4); GDPR question O1 still open

How we recognise a returning human without ever holding their documents.

---

## The requirement

> If someone is rejected or banned by a particular organisation, they must not be
> able to re-register at *that* organisation under a new account.

Explicitly **not** global cross-app uniqueness, and explicitly **not** a shared
blacklist. Scoping it this way is what keeps the design tractable and keeps us out
of the reputation business.

## What we hold: nothing about bans

The organisation holds its own ban list, keyed on its own `sub`. We hold the
identity ↔ `sub` mapping and nothing else.

- Banned at an org → new signup attempt → same identity → **same `sub`** → blocked.
- We carry no reputation data, and no liability for another party's moderation
  decisions.
- Two organisations still cannot compare notes: different `sub`s.

This is why the per-org scoping matters so much. The version of this feature that
holds a cross-app blacklist is a completely different company with a completely
different legal exposure.

## Derive, don't store

```
person_id  = HMAC(pepper,  identity_evidence)
sub_for_rp = HMAC(rp_salt, person_id)
```

`person_id` is **computed** from the verification evidence, not assigned at signup.
That single property is what makes the mechanism hold:

- Delete the account, come back, re-verify → the same value falls out → the org's
  ban still matches. **Ban-evasion-by-deletion fails.**
- Recognition requires almost no stored state. The pepper and per-RP salts are
  secrets in a KMS, not personal data.

> ⚠️ **Needs legal sign-off.** A deterministic pseudonym that singles out an
> individual is personal data under GDPR, so Article 17 applies. Fraud prevention is
> a recognised legitimate interest and Art. 17(3) carries exemptions — but *"we can
> still recognise you after you asked to be erased"* is a lawyer's call, not an
> engineering one. Resolve before launch, not after.

## ⚠️ The pepper can never be rotated

The sharpest operational trap in this design, and it is unfixable if missed in v1.

`person_id = HMAC(pepper, anchor)`. Rotate the pepper and **every** `person_id`
changes, therefore every `sub` changes, therefore **every organisation's ban list
silently becomes worthless.** Banned users walk back in. Nothing alerts anyone —
there is no error, just a quiet loss of every enforcement decision ever made.

And we cannot re-derive after rotation, because "derive, don't store" means we do
not keep the input.

**Mitigation: versioned KMS keys, never destroyed.**

- Each `person_id` records which key version derived it
- New enrolments use the current version
- Lookups try each live version, newest first
- Old versions are retained forever — retired for *writing*, never for *reading*

**Consequence for the "we store nothing" claim.** To resolve a returning person we
must compare against something, so we **do** store anchor hashes. The accurate
statement is:

> We store anchor hashes and derived pseudonyms. We store no documents, no biometric
> templates, no ban lists, and no reputation data.

This is still minimal and single-purpose, but it is not "nothing" and we must never
say otherwise. [006-legal-erasure.md](006-legal-erasure.md) is amended accordingly.

**Treat the pepper as the crown jewel.** HSM-backed, split knowledge, never
exported, never destroyed. We get one shot at it.

## Identity resolution — the hard part

None of the above works unless we can answer *"is this the same human?"* at
enrolment. We never see the documents, so we need another route. Four layers,
weakest to strongest:

| # | Layer | Strength | Cost / exposure |
|---|-------|----------|-----------------|
| 1 | **Vendor duplicate detection** within our tenant — every user goes through our vendor account, so the vendor can say "this applicant matches an existing one" | Good, but single-vendor only | Near-free. We store nothing. |
| 2 | **Salted document hash** — vendor returns the document number or a hash; we pepper it with a KMS-held secret and derive | Works *across* vendors, fixing layer 1's weakness. Breaks when the same person uses a passport once and a national ID next time | Low. No Article 9 data. |
| 3 | **Biometric 1:N as a vendor capability** — the vendor runs a face search against its own tenant and returns match/no-match | Closes the multi-document gap, which is the main realistic evasion path | Article 9: explicit consent + DPIA. **Template storage stays with the vendor under our DPA — we still hold nothing.** |
| 4 | **Device / SIM attestation** — Play Integrity, App Attest, SIM-based phone verification | Trivially defeated alone; raises the cost of casual re-registration and adds signal to the confidence score | Low |

**Leaning: layers 1 + 2 as the baseline, layer 3 as a paid escalation** that
high-assurance organisations can require. Not yet decided — see open questions.

Vendor duplicate-detection capabilities (Sumsub, Onfido, Persona, Veriff) need
confirming during selection rather than assuming.

## Confidence, never a boolean

`unique: true` is a promise we cannot keep. We emit:

```
uniqueness_confidence: high | medium | low
evidence: [which layers fired, which vendor, when]
```

Each organisation sets its own threshold. This maps directly onto the normalised
assurance vocabulary in [002-providers.md](002-providers.md).

## What we cannot close

Stated plainly, in the product and in the docs:

- **Borrowed or stolen documents** — a friend verifies on someone's behalf
- **A genuine second legal identity** — rare and expensive, but real
- **Synthetic identity** — sufficiently well-constructed fakes

The goal is **raising cost, not perfection.** apex's defining habit was presenting a
placeholder as a guarantee; this is the single worst place in the system to repeat
it. Every uniqueness claim we make must be one we would defend in front of a
customer whose ban was evaded.

## Roadmap hook: cross-org ban pools

OIDC's pairwise derivation keys off a `sector_identifier`. Organisations that opt
into a shared sector receive the **same** `sub` — which is the specification's own
answer to "a consortium of apps that agree to share safety signals."

Build per-org now, and cross-org ban pools later become a configuration change
rather than a re-architecture. No custom protocol needed.

Note that enabling a sector re-opens the pseudonymity-vs-reputation tension: within
a sector, member orgs *can* correlate users. That is acceptable precisely because it
becomes an explicit, consented, opt-in decision rather than an architectural
default.

## Open questions

- **Which layers ship in v1?** Leaning 1 + 2, with 3 available as an escalation.
- **One vendor at launch, or multi-vendor?** Layer 1 only works within a single
  vendor tenant; layer 2 is what makes multi-vendor viable. This decision and the
  layer decision are entangled.
- **Confidence thresholds.** What actually constitutes `high`?
- **Re-verification cadence.** Assertions go stale. Documents expire. What triggers
  a re-check, and who pays for it?
- **Vendor migration.** If we drop a vendor, does the whole user base need
  re-verification? Layer 2 is the hedge against this — worth weighting in the
  decision above.
