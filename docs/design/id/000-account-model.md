# 000 — Account model

Status: decided

One verified human, many personas, many organisations.

---

## The model

```
Identity ─────────────────────────────────────────────────────
   one per verified human
   holds: verified attributes (18+, liveness, uniqueness confidence)
   never exposed to an organisation directly
   │
   ├── Persona ──────────────────────────────────────────────
   │      many per identity, user-created, one per context
   │      holds: presentation — photos, bio, interests, intentions
   │
   └── OrgConnection ────────────────────────────────────────
          identity ←→ organisation
          holds: the pairwise `sub`, granted scopes, active persona
```

An organisation never sees the `Identity`. It sees a `sub` and whatever the
connected `Persona` exposes.

## The rule that makes it safe

**`sub` is derived per (identity, organisation) — never per persona.**

```
person_id  = HMAC(pepper,  identity_evidence)
sub_for_rp = HMAC(rp_salt, person_id)
```

A persona decides **what an organisation sees**, never **who they see**.

Four consequences, all of them wanted:

| | |
|---|---|
| Two accounts at one org | **Impossible.** Both personas resolve to the same `sub`. |
| Ban evasion via a second persona | **Impossible.** Same human, same `sub`, still blocked. |
| Verification effort | **Paid once.** Every persona inherits the identity's verified attributes. |
| Cross-org correlation | **Still prevented.** Different orgs get different `sub`s. |

## Why one identity per human

It is what makes everything else work. Uniqueness is the assertion organisations
actually want — it is what makes a ban stick and what kills serial re-registration.
An identity that can be minted freely asserts nothing.

See [001-uniqueness.md](001-uniqueness.md) for how a returning human is recognised
without us ever holding their documents.

## Why personas

Two independent reasons arrived at the same answer, which is usually a sign the
answer is right.

### 1. Safety

Mandatory single-identity is fine for most people and dangerous for some. A trans
person, an abuse survivor, or a sex worker may have a genuine need to keep contexts
apart. A system that structurally forbids that pushes those users either off the
platform or into unsafe situations.

Personas give separation of presentation while preserving the single verified human
underneath — so the safety accommodation costs us nothing in ban enforcement.

### 2. Personas are what make pairwise pseudonymity real

This one is not obvious and it matters.

Pairwise subject identifiers are supposed to stop two organisations working out that
they share a user. **They do not, if both organisations receive the same content.**
Identical photos and an identical bio are trivially correlated — perceptual hashing
alone is sufficient. Different `sub` values are irrelevant when the payload gives it
away.

Per-persona content separation is what actually closes that hole. Without personas,
pairwise `sub`s are security theatre the moment we start sharing profile data.

> Pairwise identifiers prevent *lazy* correlation. Personas prevent *determined*
> correlation. You need both.

## Consent dashboard

A first-class product surface, not a settings page:

- Which organisations are connected
- Which persona each one sees
- What data each one can access, per [003-data-sharing.md](003-data-sharing.md)
- Revoke any of them

This is the concrete expression of "the user sits at the centre, not the
organisations." It is also the thing that makes the model explicable to a regulator
in one screenshot.

## Open sub-questions

Deliberately not decided here.

- **Persona limits.** Cap the number? Unlimited personas is an abuse surface
  (content-farming, catfishing at scale) even though it cannot defeat ban
  enforcement.
- **Switching personas at a connected org.** Presumably allowed, and visible to the
  org as a profile change. Needs a rule.
- **Default persona.** Does a new identity get one implicitly, or is creating a
  persona an explicit step during onboarding?
- **Does the Engine representation attach to identity or persona?** The human is one
  human, which argues for identity. But presentation differs per persona, which
  argues the projection should be persona-scoped. See
  [003-data-sharing.md](003-data-sharing.md).
