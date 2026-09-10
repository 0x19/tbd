# 004 — Surface

Status: open — proposed for approval

The Human is exposed as JSON only, with provenance on every field. Identifiers on
the public surface are always the caller's pairwise `sub`. The ledger key travels
only on the internal gRPC contract.

---

## The decision

**Outside: JSON over HTTP, one versioned surface.** REST for facts and commands,
Server-Sent Events for a human's changes, webhooks for consumers that cannot hold a
connection. Requests and responses are JSON; nothing else is offered to an
integrator.

**Inside.** The protocol translates and forwards to the `humans` service over gRPC,
the way it forwards to the engine today; a `Fact` message will be added to the
proto contract (plan mode and `buf lint`, per `CLAUDE.md`). The internal contract
carries the ledger `human_id`; that is what the engine uses. The `humans` gRPC
service is reachable only from the protocol and engine clusters and gets no edge
route in `envoy.yaml`. The protocol's other edge surfaces (gRPC over h2c, GraphQL,
WebSocket) stay for first-party use and are outside this contract; a later
API-plane document decides whether they are retired.

**Identifiers.** `{sub}` in every route and `sub` in every body is the caller-scoped
pseudonym from [id/000](../id/000-account-model.md). Resolution is one call to the
id plane returning `(human_id, grant, persona)`, cached as
[003](003-consent-and-erasure.md) states; the `humans` service stores no `sub` and
never emits the ledger key or any identity value. A person calling as themselves
gets their own `sub`; `counterparty_id` on the wire is the counterparty's `sub` for
the same caller, translated by the same lookup.

## Shape

Every field is a fact, never a bare value:

```json
GET /v1/humans/{sub}/facts?path=traits.*&source=inferred

{
  "sub": "<sub>",
  "facts": [
    {
      "path": "traits.warmth",
      "value": 0.71,
      "source": "inferred",
      "origin": { "model": "engine/trait-v3" },
      "confidence": 0.62,
      "observed_at": "2026-09-10T18:20:00Z",
      "recorded_at": "2026-09-10T18:20:03Z",
      "stub": false
    }
  ],
  "next": null
}
```

Field map against [000](000-facts-ledger.md): on the wire as-is: `path`, `value`,
`source`, `confidence`, `observed_at`, `recorded_at`, `stub`; translated:
`counterparty_id` (as `counterparty`, a `sub`); scoped: `origin` (full, with
`inputs`, to `self` and the engine; `origin.model` or `origin.provider_id` only to
an organisation); internal only: `human_id`, `consent`.

`at=` returns the facts still held whose `recorded_at` is at or before that
instant; a retracted value is absent for every `at`, and its tombstone is visible
from the tombstone's own `recorded_at` ([000](000-facts-ledger.md)).

Writes are commands: `POST /v1/humans/{sub}/facts` with one fact and an
`Idempotency-Key`
([draft-ietf-httpapi-idempotency-key-header](https://datatracker.ietf.org/doc/draft-ietf-httpapi-idempotency-key-header/)),
scoped per caller and human and remembered for 24 hours. A rejected path is
`400 bad_request`; a source the caller may not write is `403 forbidden`. Today
`crates/protocol/src/error.rs` maps downstream `InvalidArgument` to 400 and
`PermissionDenied` to 403 but labels both `engine_error` (lines 41, 44, 56-66) and
names its variant after the engine; a downstream-agnostic variant with per-cause
codes is the new work that makes those codes true.

| Route | Purpose |
|---|---|
| `GET /v1/humans/{sub}` | current facts the caller may read, grouped by path |
| `GET /v1/humans/{sub}/facts` | filtered, paginated, `at=` for history |
| `POST /v1/humans/{sub}/facts` | declare or observe one fact |
| `DELETE /v1/humans/{sub}/facts/{path}` | retract a declared fact ([000](000-facts-ledger.md)) |
| `PUT /v1/humans/{sub}/presence` | current location, first-party only; Redis with expiry, never the ledger ([002](002-storage.md)) |
| `GET /v1/humans/{sub}/events` | SSE: `fact.recorded`, `fact.retracted`, `human.erased`; needs its own `timeout: 0s` route in `envoy.yaml` like `/v1/subjects/` |
| `DELETE /v1/humans/{sub}` | erasure request ([003](003-consent-and-erasure.md)), with the grace window |
| `POST /v1/webhooks` | register a consumer endpoint; deliveries signed per [Standard Webhooks](https://www.standardwebhooks.com/) |

The `humans` service resolves the grant and persona and scopes every read
([003](003-consent-and-erasure.md)); the protocol adds nothing. The 003 scenarios
run against these routes, including the one that asserts no body ever carries a
ledger uuid or identity value.

## Developer experience

- **OpenAPI generated from the Rust types** with `utoipa` and `utoipa-axum` (new
  work), committed next to the protocol and diffed in CI against the generated
  output, so it cannot drift.
- **The path registry is the documentation.** Every path, its value schema, sources
  and default scopes, rendered into the docs from the file the store validates
  against.
- **Versioning in the path, not the header.** `/v1/` for the surface, `traits.v2.*`
  for a value schema change; both additive. Fields are added, never renamed.
- **Errors say what to do.** `code` is stable and lives in the generated OpenAPI;
  `error` is a sentence.

## Unrecoverable if wrong

- **The public identifier.** Consumers store it. It is the pairwise `sub` and
  nothing else, from the first response; this is where id/008's blind-broker rule
  becomes concrete.
- **The envelope field names, the `code` vocabulary and the event `type` names.**
  Frozen the day an SDK is generated from them.
- **The webhook signature scheme.** Every consumer implements it; changing it is a
  coordinated break, hence a published standard rather than our own.
- **The idempotency scope and window.** Narrowing either later changes which
  retries are replays for every existing SDK; frozen with the envelope.

## What this is not

Not the federation broker of [id/008](../id/008-federation.md); that sits in the id
plane and calls this surface as Resource Server #1 with the translated `sub`. Not a
query language: consumers filter by path and source, and anything cleverer belongs
in ClickHouse behind an internal tool.
