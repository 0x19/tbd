# 004 — Prior art & competitive landscape

Status: decided — research complete, conclusions binding until contradicted

Researched August 2026. What already exists, what it means for us, and what
assumptions it invalidates.

---

## The short version

Pieces of what we designed exist. **Nobody has assembled them.** But two doors are
closing fast, and one assumption we were operating on was simply wrong.

## What exists

### World ID — the incumbent, and already inside Tinder

| | |
|---|---|
| Scale | ~38M users, largest proof-of-personhood network |
| Mechanism | Orb iris scan → ZK proof of humanity, **unlinkable per-app pseudonyms** |
| Dating | Match Group piloted it in Japan for age verification, **expanded to the US April 2026**. Verified users get a Tinder badge and five free Boosts |
| Elsewhere | Razer, Mythical Games, Docusign, Zoom, Vercel, Okta, Browserbase, Exa |

⚠️ **This invalidates an assumption.** We previously reasoned that Match Group has
no incentive to federate identity because they own the network. **They already
federate.** Do not plan around Match Group being closed.

**Strategic conclusion: World ID is a provider, not a competitor.** It proves
personhood and nothing else — no profile, no personas, no model of the person. For
the 38M people who already have one, accepting a World ID proof is free
personhood-assurance we do not have to build. Orb friction and the privacy optics
are their problem, not ours. See [002-providers.md](002-providers.md).

### GuyID — closest to us in spirit, opposite in architecture

Ottawa startup (founder Ravi Shankar), covered by Biometric Update in August 2026.
IDV via Didit, plus peer vouching and social-presence review, producing a portable
"trust profile" shared by link or QR across Tinder, Bumble, Hinge, Instagram,
WhatsApp.

**But it is consumer-to-consumer.** You send your GuyID to a date. It is not an
identity provider that organisations integrate, and it carries no per-org identity
continuity. Deliberate design choice on their part — "friction as a feature" to
filter for intent.

The org-side of this market is unoccupied.

### Reusable KYC as a category

Yoti (consumer identity wallet, reuse across participating businesses), Dock/Truvera,
Indicio, Didit, Veriff. All general-purpose infrastructure. **None dating-specific,
none with personas, none with an intelligence layer.**

### Academic ground already laid

- **Personhood credentials** (arXiv 2408.07892, OpenAI/Microsoft authors) formalises
  our design: unlinkable pseudonymity, one credential per person per issuer,
  service-specific pseudonyms uncorrelatable *even if issuers and services collude*.
- **BISON** — "Blind Identification with Stateless scOped pseudoNyms"
  (arXiv 2406.01518) — is our derive-don't-store scoped pseudonym scheme, formally
  treated.

**Adopt their vocabulary rather than inventing our own.** Speaking the same language
as the literature makes security review, regulator conversations and hiring easier.

## What we missed

### 1. Apple and Google have commoditised attribute verification

- Apple shipped **Digital ID** in iOS 26 (Sept 2025); US passports in Wallet
- **Verify with Wallet API extended to age verification for apps and websites,
  April 2026**
- Google's **Digital Credentials API** live on Android and Chrome; Aadhaar
  Verifiable Credentials in Google Wallet (April 2026)
- Both on **ISO/IEC 18013-5 mdoc** with selective disclosure, via the
  **W3C Digital Credentials API**
- ~41% of Americans in states with live mDL programmes; ~76% live or in development

**Together this covers roughly 99% of the smartphone market, free, at OS level.**

→ **Tier 1 of our data ladder is not a product.** "We verify you are 18+" is
becoming a platform feature. Anyone whose business is attribute verification is
being disintermediated by the OS vendors. See [003-data-sharing.md](003-data-sharing.md).

→ **But it is an input.** Accepting wallet credentials is cheaper, faster and
higher-assurance than document upload. It should be the *preferred* verification
path, not an afterthought.

### 2. Match Group already built our core feature, and it publicly failed

**Sentinel**, operating since 2019: a cross-brand safety database logging phone
numbers, email addresses, IP addresses, photos and birth dates, explicitly designed
to stop banned users returning. By 2022 it was recording hundreds of incidents
weekly.

The Markup's 18-month investigation created 50+ test accounts across Tinder, Hinge,
OkCupid and Plenty of Fish (tests run April–May 2024 and January–February 2025) and
found banned users **could re-register without changing their name, birthday, or
profile photos.**

This is the strongest validation available:

- The problem is real, expensive, and reputationally severe — Match Group faces
  active litigation over it
- It is **unsolved at the largest player in the market**
- It fails for exactly the reason our design predicts: **Sentinel matches on PII,
  not on a cryptographic identity.** Change an email address and you are through.

Derive-don't-store ([001-uniqueness.md](001-uniqueness.md)) is precisely the fix.

⚠️ Counter-consideration: Match Group building this internally means they may never
buy it. Treat them as validation of the problem, not as a prospect.

### 3. The regulatory floor moved

- **UK Online Safety Act** in force since 25 July 2025. Dating platforms squarely in
  scope. Self-declared date of birth explicitly insufficient.
- **US state patchwork**: Texas, Louisiana, Utah, Virginia, Arkansas, Tennessee,
  Florida, New York and others. No federal law. Some states additionally require
  background-check disclosure for dating platforms.
- **Google Play**: since 28 January 2026, dating apps must use the "Restrict
  Declared Minors" setting.
- **EU**: eIDAS 2.0 (Reg. 2024/1183) in force 20 May 2024. **Every member state must
  offer a certified EUDI Wallet by December 2026.** Private-sector acceptance
  obligation for regulated sectors lands December 2027. The EU Age Verification
  Blueprint reached feature-ready status April 2026.

→ The EUDI question from [001-open-questions.md](../001-open-questions.md) is
answered: **tailwind, not threat** — provided we consume wallet credentials rather
than compete with them. Croatia being in the EU is an advantage here, not a
constraint.

## Correction: the economics were wrong

We previously estimated vendor IDV at $1–3 per check. **Didit charges $0.33, with
500 free every month.** So "verify once, reuse across five apps" saves roughly
**$1.65** — not $10. The cost-arbitrage pitch is far weaker than claimed.

**The real value is conversion, not cost.** Verification is a funnel killer — users
abandon at document upload. Removing that step for the 2nd through Nth app is worth
vastly more than the fee saved. The same argument applies to profile re-entry, which
is the single largest source of drop-off in dating onboarding.

Rewrite the pitch accordingly: **we are not cheaper, we are frictionless.**

## The resulting position

Attribute verification is going to Apple and Google. Proof of personhood is going to
World ID. Both doors are closing.

What nobody owns:

> **Continuity.** Remembering, per organisation, that this is the same human — and
> carrying a portable presentation and understanding of them across organisations.

- **Apple / Google** verify an attribute and forget you. No persistent
  per-relying-party identity, by design.
- **World ID** proves personhood. No profile, no personas, no model of the person.
- **Match Group** attempted continuity internally and it broke, publicly.
- **GuyID** is consumer-side sharing, with no org integration and no continuity.

Continuity is the defensible position, and it is what the architecture in
[000-account-model.md](000-account-model.md) was already pointed at. The three
layers that express it — persistent per-org pseudonyms, personas, and the Engine's
representation — are each things the incumbents structurally do not have.

## Sources

- [World ID × Match Group](https://world.org/blog/announcements/experience-real-connections-with-world-id-and-match-group)
- [World ID new partners](https://world.org/blog/announcements/the-new-world-id-and-the-partners-bringing-proof-of-human-to-the-internet)
- [GuyID combines IDV with reputation — Biometric Update](https://www.biometricupdate.com/202608/guyid-combines-idv-with-reputation-for-online-dating)
- [Reusable digital identity vendors 2026 — Dock](https://www.dock.io/post/reusable-digital-identity-verification-vendors)
- [Didit pricing](https://didit.me/pricing/)
- [Investigating systemic failures enabling abuse on dating apps — GIJN](https://gijn.org/stories/investigating-systemic-failure-enabling-abuse-dating-apps/)
- [Wallet-based age verification: Apple & Google Digital Credentials API](https://xident.io/blog/wallet-based-age-verification-apple-google-digital-credentials-api-2026/)
- [Digital ID going mainstream in 2026 — Authsignal](https://www.authsignal.com/blog/articles/digital-id-is-going-mainstream-in-2026)
- [eIDAS 2.0 & EUDI Wallet timeline — Gataca](https://www.gataca.io/resources/blog/eIDAS2-timeline/)
- [Age verification for dating platforms — operator guide](https://whitelabeldating.com/trust-safety/age-verification-dating)
- [Personhood credentials (arXiv 2408.07892)](https://arxiv.org/pdf/2408.07892)
- [BISON: Blind Identification with Stateless scOped pseudoNyms (arXiv 2406.01518)](https://arxiv.org/pdf/2406.01518)
