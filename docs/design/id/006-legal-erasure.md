# 006 — Legal: erasure vs. recognition

Status: decided — provisionally. **Requires external legal sign-off before launch.**

Resolves O1 from [005-decisions.md](005-decisions.md): does derive-don't-store
survive GDPR Article 17?

---

## The question

Our re-registration mechanism ([001-uniqueness.md](001-uniqueness.md)) derives a
person's identifier from verification evidence rather than assigning it at signup.
Consequence: **deleting an account does not stop us recognising the person when
they return.** That is the point — it is what makes ban-evasion-by-deletion fail.

Is that lawful?

## Provisional answer: yes, with paperwork

Three independent supports.

### 1. The suppression-list precedent

The closest established analogue, and it is directly on point.

When someone objects to direct marketing, the ICO explicitly permits retaining a
**minimal suppression entry — typically a hashed identifier — whose sole function is
to prevent future contact.** The controller keeps "just enough information about the
individual to ensure that you respect the restriction in future."

Structurally identical to ours:

| Suppression list | Our registry |
|---|---|
| Retain a hashed identifier | Retain a derived pseudonym |
| Sole function: enforce a decision | Sole function: enforce a ban |
| Minimum data necessary | Anchor hashes + derived pseudonyms, nothing else |
| Segregated from active data | Separate service, separate datastore |
| Erasure would *defeat* the user's own intent | Erasure would defeat the org's moderation decision |

**Important distinction, honestly stated:** in a suppression list the retained data
enforces *the data subject's own* request. Ours enforces *a third party's* decision
against them. That is a materially weaker position and is exactly why this needs a
lawyer rather than a confident engineer.

### 2. Recital 47 — fraud prevention is named in the text

> "The processing of personal data strictly necessary for the purposes of preventing
> fraud constitutes a legitimate interest of the data controller concerned."

Ban evasion is fraud in the ordinary sense: circumventing an access decision by
misrepresenting identity. Art. 6(1)(f) applies, subject to the three-part test:

- **Purpose** — preventing banned users, including people banned for sexual assault,
  from returning to a platform under a new account. Legitimate, and see the Match
  Group evidence in [004-prior-art.md](004-prior-art.md) for why it matters.
- **Necessity** — we hold anchor hashes and derived pseudonyms, and nothing else.
  (Anchor hashes are unavoidable: resolving a returning person requires something to
  compare against, and key rotation requires a stored input — see the pepper section
  in [001-uniqueness.md](001-uniqueness.md).) There is no less intrusive way to
  recognise a returning human without holding *more* data, not less.
- **Balancing** — favourable: no documents, no biometrics, no ban list, no
  reputation data, no cross-org correlation, no profiling from this value. It cannot
  be reversed to an identity and it is useless for any other purpose. Set against
  the safety interest of other users, including people at risk of assault.

### 3. Article 17(3) exemptions

Art. 17(3) disapplies erasure where processing is necessary for, among others,
compliance with a legal obligation, or the establishment/exercise/defence of legal
claims. Both are arguable — the second especially, given active litigation in this
sector over exactly this failure.

## The complication

**EDPB Guidelines 01/2025 on Pseudonymisation** (adopted 16 January 2025) confirm
that pseudonymised data **remains personal data** as long as the controller or any
third party has the means to reidentify. Our derived pseudonym is therefore in
scope. It is not anonymous data and we must never describe it as such.

Consequences:

- **A DPIA is required.** Large-scale processing, vulnerable data subjects,
  systematic monitoring adjacent.
- **A documented Legitimate Interest Assessment is required**, covering the
  three-part test above.
- Data subject rights other than erasure still apply — access, rectification,
  objection. We must be able to answer "what do you hold about me?" with a true,
  intelligible answer.

## Requirements this places on the design

Binding, not advisory.

1. **Single purpose.** The derived pseudonym is used for uniqueness and ban
   enforcement only. Never analytics, never the Engine, never a growth signal.
   Enforce in code, not in policy.
2. **Tell the user plainly, at enrolment.** Not buried in a privacy policy: *"If you
   delete your account and return, organisations that blocked you will still
   recognise you. Here is why."* Reasonable expectations are part of the balancing
   test — surprise is what loses it.
3. **Erasure means erasure of everything else.** Profile, personas, connections,
   Engine representation — all genuinely deleted. Only the anchor hash and derived
   pseudonym survive, and the user is told exactly that. Under
   [008-federation.md](008-federation.md) most content is not ours to begin with,
   which shrinks this surface further.
4. **Retention limit.** The pseudonym is not kept forever by default. Bans expire or
   are reviewable; the derivation should have a defined lifetime.
5. **No reversal path.** Keep the pepper in an HSM-backed KMS; store only hashes,
   never raw document numbers. If we cannot reverse it, neither can an attacker or a
   subpoena. Note the tension with key rotation — see
   [001-uniqueness.md](001-uniqueness.md).
6. **Written record before launch**, not after: DPIA, LIA, retention schedule,
   the enrolment disclosure text.

## Still needs a lawyer

Specific questions to put to counsel, ideally one with Croatian/EU data-protection
practice:

1. Does the suppression-list analogy hold when the retained value enforces a *third
   party's* decision rather than the data subject's own request?
2. Is Art. 17(3)(e) (legal claims) available pre-emptively, or only once a claim
   exists?
3. Who is controller and who is processor between us and a relying organisation for
   the ban decision? This determines who owes the DPIA and who carries liability.
4. Does the DSA's repeat-offender regime (Art. 23, misuse) supply an independent
   basis for relying parties, and does that help or hinder us as intermediary?
5. Does a defined retention period for the pseudonym strengthen the balancing test
   enough to be worth the reduced enforcement?

**Do not build past the `providers` crate without an answer to Q1 and Q3.**
