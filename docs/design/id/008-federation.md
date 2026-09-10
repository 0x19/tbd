# 008 — Federation

Status: decided — architecture. **v1 scope deliberately minimal.**

Organisations are not only consumers of identity. They are also sources of data,
queryable through us with the user's permission.

---

## The model

Standard OAuth 2.0, used properly. Nothing invented.

| Role | Who |
|---|---|
| **Authorization Server** | Us. We hold the permission graph and broker every request. |
| **Client** | Any org consuming identity or data |
| **Resource Server** | Any org exposing data behind its own API |

Every organisation is **both** — it consumes identity to onboard a user, and it
exposes what it holds about that user for the user to share elsewhere. Our own app
is Client #1 and Resource Server #1, and its job is to be the reference
implementation of both roles.

This matters for adoption: "we're an OAuth authorization server" is a sentence every
backend engineer already understands. The moment federation needs a bespoke protocol
it stops being adoptable.

## Why this makes us hold *less*

Counter-intuitive, and it is the strongest argument for it.

The model in the original [003-data-sharing.md](003-data-sharing.md) had us storing
a canonical profile and pushing projections outward. That makes us a database, with
the retention, staleness and liability that implies.

Under federation we store:

- Verified attributes (canonical — they are facts, see [003](003-data-sharing.md))
- Persona definitions
- The permission graph
- Identity anchors ([007](007-identity-anchors.md))

We do **not** store profile content. Photos, bios and preferences live at the
resource server that owns them. We are infrastructure, not a warehouse.

Three consequences, all good:

1. **Revocation becomes real.** In the storage model, revoking access stopped future
   reads but we still held the data. Here, revoking access stops the data flowing at
   all, because we never had a copy. This fixes the weakest part of
   [003](003-data-sharing.md).
2. **Staleness disappears.** No sync problem, no conflict resolution, no
   "edits flow outward only" rule to enforce — each org owns its own data by
   construction.
3. **Breach exposure drops again.** We already held no documents and no biometrics.
   Now we hold no profile content either.

## The constraint that makes it safe

> **Organisations never talk to each other. Ever. We broker every request and
> translate every identifier.**

If org A calls org B's API directly, both learn they share that user — which
destroys the pairwise pseudonymity [000-account-model.md](000-account-model.md)
depends on. Different subject identifiers are worthless when the query itself
reveals the overlap.

```
  Org A                    Us                     Org B
    │                       │                       │
    │  GET scope for sub_A  │                       │
    ├──────────────────────▶│                       │
    │                       │ resolve sub_A → person│
    │                       │ check consent         │
    │                       │ translate → sub_B     │
    │                       ├──────────────────────▶│
    │                       │◀──────────────────────┤
    │◀──────────────────────┤                       │
    │                       │                       │
  never learns B exists                     never learns A asked
```

**Blind broker.** A learns nothing about which orgs served the request. B learns
nothing about who asked. Only we hold the correlation, and only because the user
granted it.

This is the third thing Apple, Google, World ID and EUDI structurally cannot do —
it requires exactly the trusted middle they are designed to eliminate. See
[007-identity-anchors.md](007-identity-anchors.md).

## Personas become the assembly point

This unifies federation with the persona model rather than bolting it on.

A **persona is a view assembled from sources the user chooses.** "My Proximity
persona uses these three photos from Proximity and my verified age. My other persona
uses different photos from somewhere else."

The user thinks in terms of *what this org can see*, never *which backend serves it*.
Source selection is our problem, not theirs.

## Scope rules

**Queryable scopes are user-owned data only. Never third-party judgements about
the user.**

| Allowed | Forbidden |
|---|---|
| Profile content the user authored | Ban and report history |
| Preferences and intentions | Trust or risk scores |
| Verified attributes | Moderation decisions |
| Engine representation (tier 4, separate consent) | Anything one org concluded *about* the user |

Without this rule, federation becomes the cross-app blacklist we deliberately
refused in [001-uniqueness.md](001-uniqueness.md) — someone will expose ban history
within a month of launch. **Enforce it in the scope registry, in code. Not in a
contract.**

## Framing: user portability, not a data deal

**GDPR Article 20** gives a user the right to have their data transmitted directly
from one controller to another where technically feasible. A *user-initiated* pull is
on far firmer ground than a business-to-business data arrangement.

Build it, describe it and instrument it as the user exercising portability. That
framing is both more defensible legally and more honest about whose data it is.

## The honest objection

**Reciprocity is asymmetric, and incumbents will not participate.**

Small apps gain enormously from querying a rich network. Large apps gain nothing
from exposing data to competitors. This is precisely why Open Banking required PSD2
to compel it — banks did not volunteer, and Match Group will not either.

Plan for a network that grows bottom-up among willing participants, seeded by our
own app. If it becomes large enough that incumbents need it, that is upside, not the
plan. Do not build anything whose value depends on a big platform reciprocating.

## v1 scope — deliberately small

**Architect for federation now, build the minimum.**

apex died of starting four surfaces and finishing none. Federation is exactly the
kind of idea compelling enough to consume a year, so the discipline is explicit:

| Build in v1 | Defer |
|---|---|
| Scope registry with the user-owned-data rule enforced | Multi-source resolution |
| Blind-broker indirection in the API path | Third-party resource servers |
| One resource server (our own app) | Any source-selection UI |
| One scope, end to end | Caching or replication |

The indirection and the scope model must be right from the start, because retrofitting
them means changing every relying-party integration. Everything else can wait.

## Open questions

- **Caching.** Any persistent cache turns us back into a database and reopens
  staleness and breach exposure. Leaning: no persistent cache, short TTL at most,
  accept dependency on resource-server availability. Needs a decision before the API
  design is fixed.
- **Availability.** What does a client see when a resource server is down? Federation
  makes our uptime a function of other people's uptime.
- **Where does a brand-new user's content live** before they have joined any org?
  Either our app is the default resource server, or we hold a minimal home profile
  and stop being purist about holding nothing.
- **Liability for brokered data.** If we pass wrong or defamatory data from A to B,
  who is controller? Interacts with question 3 for counsel in
  [006-legal-erasure.md](006-legal-erasure.md).
- **Rate limiting and abuse.** A client could enumerate scopes to profile users
  across the network. Needs a budget model per client.
