# 003 — Data sharing

Status: decided — tiers resolved in [005-decisions.md](005-decisions.md) (D8, D9).
**Delivery mechanism superseded by [008-federation.md](008-federation.md).**

> **What changed.** The tier ladder, canonical-vs-projection split and per-persona
> control below all stand. What changed is *how* data reaches an organisation: we no
> longer store profile content and push projections outward — each org owns its data
> behind its own API and we broker access. This resolves the weak revocation story
> and removes the sync problem entirely. Read
> [008-federation.md](008-federation.md) alongside this document.

What travels from an identity to an organisation, and under what consent.

---

## The tier ladder

"Re-use the information we have about the user" spans an enormous range. Naming the
tiers separately is what stops the most sensitive one being shipped by accident
alongside the least.

| Tier | What travels | Value to org | Sensitivity |
|---|---|---|---|
| **1** | Verified attributes — 18+, liveness, uniqueness confidence | High, immediate | Low |
| **2** | Base profile — photos, bio, basics | High | Medium |
| **3** | Preferences and intentions | High | Medium |
| **4** | **Engine representation** — the model of who this person is | Enormous | Very high |

Tier 1 is the floor and is uncontroversial. Tier 4 is the real moat — an
organisation plugs in and instantly understands a brand-new user, with no cold
start — and it is also a psychological profile following a person between
platforms.

**Tier 4 needs its own consent model, not an OAuth scope flag.** A checkbox in a
consent screen is not informed consent for "this app now knows how you behave in
relationships." Undecided what the right mechanism is; a scope flag is definitely
the wrong one.

## Canonical facts vs per-org projection

Not everything should behave the same way.

| | Scope | Why |
|---|---|---|
| **Verified attributes** (18+, liveness, uniqueness) | **Canonical** — one truth, identical everywhere | It is a fact about a human, not a presentation choice. Divergence here would be a bug. |
| **Profile content** (photos, bio, interests) | **Per-org projection**, via the connected persona | People deliberately present differently on different apps. Forcing one canonical bio is wrong for the product *and* maximises the correlation leak below. |

## ~~Edits flow outward only~~ — dissolved by federation

This rule existed to solve a sync and conflict-resolution problem that federation
removes entirely. Each organisation owns the data it holds, behind its own API.
There is no canonical copy of profile content to keep in sync, so there is nothing
to conflict.

What we still hold canonically: **verified attributes** (they are facts, not
presentation), persona definitions, and the permission graph. See
[008-federation.md](008-federation.md).

## Per-persona sharing control

The mechanism, not merely a mitigation:

> *"Tinder sees these three photos. Proximity sees all six."*

This exists because **pairwise subject identifiers do not prevent correlation when
the payload is identical.** Two organisations receiving the same photos and the same
bio can correlate users trivially — perceptual hashing is sufficient, and different
`sub` values are irrelevant.

So per-persona, per-org content control is what makes the privacy guarantee in
[000-account-model.md](000-account-model.md) actually true rather than nominal.

Two honest options existed here: accept content-based correlation and disclose it
plainly, or give the user real control. We chose control — it is both the stronger
privacy position and a genuine user-facing feature.

## Consent

Per organisation, per tier, per persona. Surfaced in the consent dashboard
(see [000-account-model.md](000-account-model.md)), revocable at any time.

**Revocation is now meaningful.** Under federation we hold no copy, so revoking a
grant stops the data flowing rather than merely stopping future reads of something
we still possess.

What remains open is what an organisation must do with data it *already* fetched.
That is a relying-party agreement question (O3), not an architecture one — but the
architectural exposure is far smaller than it was.

## Open questions

- **Which tiers ship in v1?** Leaning 1–3 by default, tier 4 as a separate explicit
  opt-in with its own consent flow.
- **Does the Engine representation attach to identity or persona?** The human is one
  human, arguing for identity. But presentation and context differ per persona,
  arguing the projection should be persona-scoped. Likely: model at identity,
  project per persona.
- **What does revocation oblige an organisation to do?** See above.
- **Photo hosting.** Do we serve the images, or hand over URLs, or transfer copies?
  This determines whether we can revoke access to a photo in any meaningful sense —
  and whether we become a CDN.
- **Pricing shape.** Per verification, per active connection, per tier? Tier 4 is
  worth far more than tier 1 and probably should not be priced the same way.
- **Data residency.** EU users, EU storage. Interacts with vendor selection in
  [002-providers.md](002-providers.md).
