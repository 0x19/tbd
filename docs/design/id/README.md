# Identity plane

The portable, verified identity that outlives any single app.

## What this is

A person verifies once. That verification, and the profile built on top of it,
becomes reusable across every organisation they choose to connect to — Proximity
first, other platforms later.

Concretely: **"Sign in with [us]"** — a standards-compliant OIDC flow, plus verified
attributes and a portable profile. Every app already knows how to integrate that
flow, which is the single largest adoption lever available to us.

**We are not a verification vendor.** Platform wallets (Apple, Google, EUDI) and
IDV vendors (Didit first) do the verification. We orchestrate them, normalise their
results, and make the resulting assertion portable. We never hold documents or raw
biometric templates. Wallet-first, vendor-second — see
[005-decisions.md](005-decisions.md) D2 and [002-providers.md](002-providers.md).

## How it relates to the other planes

```
                    ┌──────────────────────────┐
   third parties ──▶│   API plane              │◀── our own app
                    │   OIDC/OAuth 2.1 + REST  │
                    └────────┬─────────────────┘
                             │  gRPC, internal
              ┌──────────────┴──────────────┐
              ▼                             ▼
     ┌─────────────────┐          ┌──────────────────┐
     │  ID service     │          │  Engine          │
     │  ◀── you are    │          │  representation  │
     │      here       │          │  + policy        │
     └────────┬────────┘          └──────────────────┘
              │
              ▼
     ┌─────────────────────────────────┐
     │  providers crate                │
     │  Wallets · World ID · Didit · … │
     └─────────────────────────────────┘
```

The **API plane** is the single public contract. The **Engine** models who a person
is and decides how the system behaves toward them. Identity is the foundation both
sit on: it establishes *that* someone is a real, unique human, and holds what is
known about them.

ID is a separately deployable service for two reasons that are not about scale:

1. **Compliance blast radius.** Its own datastore, audit log and access control makes
   the DPIA and any future certification work dramatically cheaper.
2. **Different hardware.** Engine wants GPU. ID wants a KMS/HSM.

## Why this exists at all

**Not cost — friction.** Vendor verification is cheap (Didit: $0.33/check, 500 free
monthly), so reuse across five apps saves under $2. That is not a business.

Verification and profile re-entry are the two largest drop-off points in dating
onboarding. Removing both for every app after the first is worth vastly more than
the fee saved. See the correction in [004-prior-art.md](004-prior-art.md).

And the obstacle that killed reusable KYC in fintech does not apply here. In banking
each institution carries its own regulatory duty to verify and cannot delegate it.
Dating and social platforms have no equivalent obligation, so reusability is legally
straightforward in this sector in a way it never was in finance.

**What we actually sell is continuity** — remembering, per organisation, that this
is the same human, and carrying a portable presentation and understanding across
organisations. Apple, Google and World ID all verify and forget. See
[005-decisions.md](005-decisions.md) D1.

## Documents

| # | Document | Status |
|---|----------|--------|
| 000 | [Account model](000-account-model.md) — identity, personas, organisations | decided |
| 001 | [Uniqueness & re-registration](001-uniqueness.md) — how we recognise a returning human | decided |
| 002 | [Providers](002-providers.md) — the vendor abstraction | decided |
| 003 | [Data sharing](003-data-sharing.md) — what travels to an organisation | decided, mechanism superseded by 008 |
| 004 | [Prior art](004-prior-art.md) — competitive landscape, what it invalidates | decided |
| 005 | [Decisions](005-decisions.md) — every open question resolved | decided, D2 superseded |
| 006 | [Legal: erasure vs recognition](006-legal-erasure.md) — GDPR Art. 17 analysis | decided, needs counsel |
| 007 | [Identity anchors](007-identity-anchors.md) — what can actually anchor a person | decided |
| 008 | [Federation](008-federation.md) — orgs as resource servers, brokered access | decided |

## Standing constraints

These apply across every document here.

- **We never hold documents or raw biometric templates.** Not in v1, not later.
- **Uniqueness is a confidence, never a boolean.** We cannot promise what we cannot
  guarantee, and the previous project's defining habit was presenting a stub as a
  guarantee.
- **The user sits at the centre, not the organisations.** They can see every
  connected org and what each can access, and revoke any of them.
- **Prefer the standard.** OIDC, OAuth 2.1, W3C VC, NIST 800-63, eIDAS LoA. Invent
  nothing that already has a specification.
