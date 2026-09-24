# Inorbit finance — the platform layer and the `finance` service

## Context

Inorbit d.o.o. is a one-person Croatian company. Today the owner has no consolidated view of
where money goes, collects supplier invoices by hand from email *and* individual vendor
portals (Hetzner, Cloudflare, Google Cloud, Medium, Excalidraw), hands that pile to the
accountant ad hoc, and writes client invoices in **Google Docs**, emailing each PDF manually.

Goals, in priority order:

1. **Understand the money** — every transaction, personal *and* business, categorized and
   queryable, with projections across both. "Ways I can improve my financial status" needs
   the whole picture, not the company half.
2. **Deliver clean data to the accountant** — supplier invoices collected automatically,
   matched to the transactions that paid them, exported as a monthly bundle.
3. **Create, change and see every invoice ever issued** — replace Google Docs with a
   reviewed, auditable end-of-month generate → approve → send flow.

Out of scope: the socializing/dating ideas discussed previously.

### What the real files settled

`~/finance-data/08-2026/` answered several questions outright:

| Finding | Consequence |
|---|---|
| Invoice no. `9-1-1-2026` = `ordinal-premises-device-year`; July was `8-1-1` | The Croatian legal triple *broj računa – oznaka poslovnog prostora – oznaka naplatnog uređaja*. Gapless per-year numbering is a **legal constraint**. Store the parts, derive the string. |
| **"Poziv na broj" == the invoice number** | Invoice→payment matching is *deterministic*, not fuzzy — Erste carries it in the remittance field. |
| Client is **Tenderly, Belgrade, Serbia** — non-EU; VAT 0,00 under čl. 17. st. 1. | No domestic B2B → **Fiskalizacija 2.0 / eRačun does not apply to outgoing invoices**. PDF+email is Tenderly's *permanent* path, not a stopgap. |
| All 9 supplier PDFs have **extractable text** (verified with `pdftotext`) | No OCR. Ingestion is text extraction + per-vendor rules. |
| Producer `Skia/PDF Google Docs Renderer`; **612×1008pt (US Legal)**; logo a **292×140px bitmap** | The template is a Google Doc. New one is A4; the logo needs a vector original. |
| IBAN `HR9224020061100925189`, Erste & Steiermärkische Bank d.d. | Confirms the account to connect. |

### Provider decision

**Enable Banking** is the AISP. GoCardless/Nordigen is dead — signups closed, docs go dark
24 Aug 2026. Enable Banking is licensed (no eIDAS/QWAC needed, ≈€3–8k/yr avoided), Erste is
listed for Croatia, `psu_type: "business"` exists, and the free **restricted production**
tier works precisely on accounts you link yourself. Constraints: RS256 JWT signed locally;
sessions ≤180 days; **most ASPSPs allow only 4 fetches per account per day** when the user is
offline (429 `ASPSP_RATE_LIMIT_EXCEEDED`, resume after 6h).

---

# Part 0 — `prototype/bank` — DONE, gate passed

Built and run. Full evidence in **`prototype/bank/FINDINGS.md`**; this is the summary.

**Verdict: Enable Banking works for this use case, at zero cost.** Erste Croatia exposes
business *and* personal accounts, both authorized under one application
(`fd18716b-2132-49c6-89b6-384b99c92fd4`, PRODUCTION, restricted), consents valid to
2027-03-15, no card and no contract. 2,861 real transactions pulled into
`prototype/bank/data/bank.sqlite`.

| Question | Answer |
|---|---|
| Business accounts readable? | **Yes** — `HR9224020061100925189`, INORBIT d.o.o. |
| Personal accounts readable? | **Yes** — two IBANs, same application, simultaneously |
| `maximum_consent_validity` | 15552000s = **180 days**, as assumed |
| `required_psu_headers` (e.g. `Psu-Ip-Address`) | **Absent** — unattended syncing is viable. This was the finding most likely to break the design |
| History depth | personal ~21 months (2024-12-30 →), business ~6 (account age, not a limit) |
| `entry_reference` | present and **unique on all 2,657 rows** |
| *Poziv na broj* | **not** in the structured field — in free-text remittance. See the matching section above |
| Rate limiting | no 429 in 60+ calls during an active session. The 4/day cap applies to *unattended* access and is **still unmeasured** |
| Cost | €0 |

**Built:** `prototype/bank/eb/{config,token,aspsps,link,pull,load}.py` — Python, excluded from
cargo (no `Cargo.toml` under `prototype/`, so no workspace `exclude` was needed after all).
`data/` is gitignored and `0600`; the RSA key lives in `~/.config/enablebanking/` at `0600`.

**Also built, and these are production, not throwaway:**

| URL | Purpose |
|---|---|
| `https://finance.proximity.is/connect/callback` | the registered redirect URL; open route, static page |
| `https://chaosadmin.proximity.is/privacy.html` | required by registration |
| `https://chaosadmin.proximity.is/terms.html` | required by registration |

Envoy gained a `finance.*` vhost and two open routes on `chaosadmin.*`; Caddy gained a
`finance.{$BASE_DOMAIN}` block; Let's Encrypt issued the certificate.

**Open, worth closing before P4 relies on it:** whether `entry_reference` is stable across
pulls on different days. One repeat pull answers it.

---

# Part 1 — the platform layer

This is the part that isn't about finance. Migrations become **per project, not per service**,
because foreign keys need to cross service boundaries, and `users` has to exist as something
other tables can point at.

## Today

Only `crates/ledger` has migrations or sqlx at all. Two Postgres instances: `ledger-postgres`
(pgvector, db `ledger`) and `postgres` (Ory's hydra+kratos). No `users` table anywhere —
identity stops at `Principal.sub` from `x-jwt-payload`.

## One app database, schema per service

`/migrations/` at the repo root, one ordered set, one database. Schemas namespace it:
`public` for shared identity, `ledger.*`, `finance.*` — several tables per service schema,
and foreign keys cross schemas freely in Postgres.

```
migrations/
  0001_identity.sql     parties, users, orgs, memberships
  0002_ledger.sql       moved from crates/ledger/migrations, renumbered
  0003_finance.sql      the finance domain
```

Ledger's existing migration moves in and is renumbered. Confirmed: no data worth preserving,
so no `_sqlx_migrations` compatibility shim — anyone with a local database drops and
recreates it.

**Ory's database stays separate.** That schema is Kratos's and Hydra's, owned by them and
migrated by their own tooling. Merging it would mean owning someone else's schema.

Compose service names become `tbd-postgres` (the app) and `auth-postgres` (Ory) — the volume
is already called `auth-postgres`, so this just makes the service match.

**Per-service Postgres roles against the same database.** Each service keeps its existing
`<NAME>_DATABASE_URL` convention, but the URLs carry *different roles* with grants scoped to
their schema and a `search_path` set per role. This preserves the env-var convention, keeps
the blast radius of a compromised service bounded, and costs one `grant` block per schema in
the migration. A shared database does not have to mean shared credentials.

## The `tbd-db` crate

`crates/db`, holding what is currently trapped inside `crates/ledger/src/store/pg.rs`:

- `MIGRATOR` — `sqlx::migrate!("$CARGO_MANIFEST_DIR/../../migrations")`
- `connect_lazy(&PgOptions)` over `PgPoolOptions`, `pool_counts()`, `max_connections()`
- `map_err(sqlx::Error) -> StoreError` and the shared `StoreError` type
- `Clock` / `SystemClock` / `ManualClock`
- `Cursor` (base64url `v1:<micros>:<id>`) and `Page::cut`
- the identity row types and `ensure_party(&Principal) -> PartyId`

Every service with a store depends on `tbd-db`. It is **not** `tbd-common`: ARCHITECTURE
invariant 3 keeps `tbd-common` transport-free, and pulling sqlx into every binary through it
would violate the spirit of that.

## Migrations run once, not per service

`migrate_on_start` per service races when two pods start together and gets worse with a
shared schema. Instead: **one migrator**, a `tbd migrate` subcommand run as a Kubernetes Job
(and a `mise run db:migrate` task locally), with every service **refusing to start if the
schema version is behind what its binary expects**. That turns a silent half-migrated cluster
into a clean startup failure naming the gap.

## Identity: parties, users, orgs, memberships

The requirement is mixing personal and business — a projection over just the company, just
the person, or both together. The clean way is a **supertype table** so every ownership
foreign key is a single column that works for either.

```sql
-- Anyone money can belong to or flow to: you, Inorbit, a client, a supplier.
create table parties (
    id           uuid primary key,
    kind         text not null check (kind in ('person','org')),
    display_name text not null,
    country_code char(2) null,
    created_at   timestamptz not null default clock_timestamp(),
    archived_at  timestamptz null
);

-- Parties that can log in. Keyed on the Kratos identity id.
create table users (
    id           uuid primary key references parties(id) on delete restrict,
    subject      text not null unique,          -- Principal.sub
    email        text null unique,
    last_seen_at timestamptz null
);

-- Parties that are organisations: ours and our clients' and suppliers'.
create table orgs (
    id         uuid primary key references parties(id) on delete restrict,
    legal_name text not null,
    oib        text null unique,                -- Croatian tax id
    vat_id     text null,                       -- EU VAT id
    address    jsonb not null default '{}'::jsonb,
    internal   boolean not null default false   -- true = we invoice *as* this org
);

create table memberships (
    user_id  uuid not null references users(id) on delete cascade,
    org_id   uuid not null references orgs(id)  on delete cascade,
    role     text not null check (role in ('owner','admin','member','viewer')),
    added_at timestamptz not null default clock_timestamp(),
    primary key (user_id, org_id)
);
```

**`users.subject` is safe as the natural key.** `devops/k8s/auth/config/hydra.yml` sets
`subject_identifiers: supported_types: [public]`, so `sub` is the stable Kratos identity UUID,
identical across every OAuth client. Worth noting that `Principal.sub`'s doc comment in
`crates/protocol/src/principal.rs` says *"a person's pairwise id"* — **that comment is stale
and should be fixed in this work**, because if it were ever made true, keying on `sub` would
silently break the day a second client is registered.

Users are **JIT-provisioned**: `ensure_party(&Principal)` does
`insert … on conflict (subject) do update set last_seen_at = now() returning id`, called by
each service's `admit()`. No Kratos webhook, no sync job.

### What this buys the finance domain

Every ownership column becomes `party_id uuid not null references parties(id)`:

- A bank account is owned by *you* (personal) or by *Inorbit* (business) — one column.
- An invoice has an `issuer_party_id` (an `internal` org) and a `client_party_id`.
- A transaction's counterparty is a nullable `party_id`, resolved when identifiable.
- **The personal/business projection is a filter, not a schema change.** "Just the company"
  is one party id; "my whole picture" is my user's party plus the orgs I'm a member of;
  "everything" is no filter.

Note the conceptual overlap with `ledger.subjects` — deliberately left alone. Ledger's
subject is opaque by design ("this schema knows nothing about people"); `public.parties` is
the application's identity. If they ever need linking, a nullable `parties.subject_id` is one
column, not a dependency.

## Access control: the personal accounts must be unreachable, not merely hidden

**The requirement.** Only `nevio@inorbit.hr` sees any finance data at all. If access is later
given to someone else — the accountant is the obvious case — they must not see the personal
accounts *on the UI, through the API, or anywhere else*.

"Anywhere else" is what makes this a data-layer problem. A UI filter is cosmetic; a `WHERE`
clause the handler remembers to add is one forgotten query away from a leak. Three layers,
each sufficient on its own, because this is money and the requirement is absolute.

### 1. The allowed set is derived from the Principal, never supplied by the caller

```sql
create table party_access (
    user_id    uuid not null references users(id)    on delete cascade,
    party_id   uuid not null references parties(id)  on delete cascade,
    capability text not null check (capability in ('own','read')),
    granted_at timestamptz not null default clock_timestamp(),
    granted_by uuid null references users(id),
    expires_at timestamptz null,
    primary key (user_id, party_id)
);
```

- Nevio: `('own', his person party)` **and** `('own', the Inorbit org party)`.
- An accountant: `('read', the Inorbit org party)` — and **no row at all** for the person
  party. Not a row with reduced capability. No row.

`memberships` models who belongs to an org; `party_access` models who may *read whose money*.
They are different questions and conflating them is how personal data leaks into a business
grant. A person party can be granted without inventing a fake org to hold it.

Every finance query filters on `party_id in (select party_id from party_access where user_id =
$caller and (expires_at is null or expires_at > now()))`. The caller's id comes from
`ensure_party(&Principal)`, i.e. from the JWT Envoy verified. **No RPC accepts a `party_id`
filter from the client** — the UI's personal/business/combined toggle narrows a set the server
already computed, and asking for a party outside the grant is `NOT_FOUND`, never
`PERMISSION_DENIED`, which would confirm it exists.

### 2. Postgres row-level security, as defence in depth

Application code can forget a filter. RLS means the database refuses anyway.

```sql
alter table finance.accounts          enable row level security;
alter table finance.bank_transactions enable row level security;
-- and every table carrying a party_id

create policy party_visible on finance.bank_transactions
  using (party_id in (select party_id from party_access
                      where user_id = current_setting('app.user_id')::uuid
                        and (expires_at is null or expires_at > now())));
```

`set local app.user_id = $1` runs at the start of every transaction — **`local`, and
per-transaction**, because sqlx hands out pooled connections and a session-level setting would
leak across callers. The migration owner needs `bypassrls`; the service role must not have it.

Trade-off: RLS costs a `set local` on every transaction and makes some query plans harder to
read. Worth it here — it converts "we always remember the filter" from a promise into an
invariant the database enforces.

### 3. Envoy gates the host separately from the chaos admin

Today `chaosadmin.*` admits anyone with `role == admin` **or** any token carrying scope
`tbd.api`. Reusing that for finance would mean every platform admin and every machine token
could read the company's books. `finance.*` and the `/v1/finance/*` routes get their own JWT
requirement and a dedicated **`tbd.finance`** scope, granted to one person.

Envoy authenticates; the service authorizes. Neither is sufficient alone — a machine token
with `tbd.finance` still only sees what `party_access` grants it.

### Audit

Money plus delegated access means "who read what" has to be answerable. An `access_log` row
per read that returns another party's data — caller, party, route, `trace_id`, timestamp —
and the `approvals` table already records who approved each invoice. The accountant's monthly
export writes a row naming exactly what left the system.

### Current exposure: none

There is no finance service, no finance UI and no finance data in any database yet. The only
copy of real transactions is `prototype/bank/data/bank.sqlite` on the server, now `0600` in a
`0700` directory and gitignored. The Enable Banking private key is `0600` in `~/.config/
enablebanking/` (`0700`). The two published pages carry no data, and the callback route
returns a static page that reads its own URL.

---

# Part 2 — the `finance` service

```
mise run tbd -- new service finance --port 50054 --metrics-port 9468 --bacon-key f
```

Legal per `crates/cli/src/service.rs`; 50054 and 9468 are the next free ports.

**Finance keeps its own relational tables — it does not become ledger facts.** The ledger's
job is unifying fact-shaped, provenance-carrying data; the money domain's core operations are
mutable and relational (a draft you edit, a status machine, a gapless counter under a row
lock). An append-only log cannot produce 1..n with no holes — appending is precisely what
leaves holes when a transaction rolls back.

Two things from the ledger are worth *copying* rather than reinventing, because they are
better than what I'd have invented: the `source` enum
(`verified | declared | inferred | symbolic | observed`) with `confidence` beside it — a bank
feed is `verified`, an LLM-extracted invoice total is `inferred`, a category you set by hand
is `declared` — and the `Envelope`/`stub` labelling discipline.

**One crate for both halves** (ingestion and invoicing). ARCHITECTURE invariant 6 forbids
services addressing each other directly, so splitting them would make payment matching a gRPC
hop instead of one transaction.

```
crates/finance/src/
  lib.rs        serve / serve_on / serve_with / serve_store; JoinSet + CancellationToken
  main.rs       the only file that may print: config, import, template
  config.rs     [store] [provider] [sync] [render] [mail] [invoicing] [documents]
  service.rs    FinanceService; admit() / RequestTimer / status_of
  store/        mod, pg, memory, instrumented, faulty, validate, sql, dedup
  banking/      enablebanking/{auth,client,models,error}, provider.rs, syncer.rs
  invoice/      numbering, lifecycle, totals, prefill
  render/       mod (Renderer + Mock), typst.rs, doc.rs
  mail/         mod (Mailer + Capture + Mock), smtp.rs, outbox.rs
  documents/    mod (Blobs trait), pg.rs
```

## Workspace changes

| Change | Why |
|---|---|
| `ring = "0.17"` | RS256 JWT signing. Already in the lockfile as rustls's provider. |
| `lettre`, `default-features = false`, features `smtp-transport, tokio1, tokio1-rustls, ring, webpki-roots, builder, pool` | Defaults pull `aws-lc-sys`, **banned in `deny.toml`**. Catch it here, not in CI. |
| `chrono-tz` | The invoicing period is `Europe/Zagreb`. A run opened 23:30 CEST on 31 Aug is a *September* run in UTC — a real bug for someone working late on the last day of the month. |
| `typst-as-lib` (MIT), default-features off, behind a default-on crate feature | PDF rendering. No `packages`/`reqwest`/`ureq` — remote resolution would make renders non-hermetic. |
| Move `Principal`/`CallerKind`/`Key` to `crates/common/src/principal.rs`, leaving the axum `attach` middleware re-exported | Approvals record the identity **Envoy verified**, read from `x-jwt-payload` on `tonic::Request::metadata()`. Don't duplicate the parser. |
| Chaos: `fault: bool` / `store_fault: bool` → `fault_surfaces: &'static [&'static str]` | Finance needs a third surface (mail); eRačun will want a fourth. Keep `/behavior` and `/store_behavior` as aliases. `kinds::markdown()` gains a column and `docs/chaos/kinds.md` is drift-checked by `chaos:docs:check`. |

## Data model

`migrations/0003_finance.sql`, schema `finance`. **Money is `bigint` minor units +
`char(3)` currency + `smallint` scale. Never a float.** The workspace's sqlx features don't
include `bigdecimal`/`rust_decimal` so `numeric` isn't decodable today; proto has no decimal
type so the wire is `int64` regardless; and the transcoder renders int64 as a JSON string,
which is what money wants in a browser. Every aggregate is `sum(minor) group by currency`; a
cross-currency total is never computed. `exchange_rate` is `numeric`, display-only.

**Banking**: `providers`, `connections`, `accounts`, `balances`, `categories`, `rules`,
`bank_transactions`, `sync_runs`.

- **Dedup**: `unique (account_id, dedup_key)`. **Measured on real Erste data: 2,657
  transactions, 2,657 distinct `entry_reference`, zero nulls** — so for Erste the key is
  `entry_reference` alone and the content-hash path is never taken. Keep the fallback, because
  this is multi-bank and other ASPSPs are not this well behaved: SHA-256 over length-prefixed
  labelled fields, the content tuple plus an **ordinal**, since two identical €4.50 charges on
  one day must stay two rows. But it is the cold path — the plan should not carry it as though
  every row needs it. Still proptested. **Not yet verified:** whether `entry_reference` is
  stable across pulls on different days. One repeat pull settles it, and it should happen
  before P4 relies on it.
- **Pending vs booked**: booked rows are a log; pending rows are a per-account snapshot
  **replaced wholesale** each fetch. A pending entry has no stable identity anywhere in PSD2,
  so treating it as a log invents one the bank never gave.
- `accounts` carries the sync worker's state — `sync_budget_day`, `sync_budget_used`,
  `backoff_until`, `last_booked_through` — **in the row, never in the process**, so a
  restart, a second replica and a manual sync all see one truth.

**Invoicing**: `client_recipients`, `subscriptions`, `subscription_lines`,
`document_templates`, `invoice_numbers`, `invoice_runs`, `invoices`, `invoice_lines`,
`invoice_events`, `approvals`, `invoice_deliveries`, `email_outbox`. There is no separate
`clients` table — a client is a `public.parties` row, which is also what makes a supplier and
a counterparty the same kind of thing.

**Numbering — gapless, the load-bearing decision:**

1. A number is allocated **at approval, never at draft creation**, enforced by
   `check ((ordinal is null) = (approved_at is null))`. A discarded draft cannot consume a
   number. This is what makes gaplessness possible at all.
2. `update invoice_numbers set next_ordinal = next_ordinal + 1 … returning`, **inside the
   transaction** that flips `draft → approved`. A Postgres `sequence` is explicitly *wrong*:
   `nextval` is non-transactional, so a rolled-back transaction leaves a permanent gap.
3. The row lock serialises concurrent approvals of the same `(series, year)`. A real
   bottleneck, and for one person invoicing five clients monthly it costs nothing.
4. Enforced, not intended: `unique (series, year, ordinal)`.
5. A cancelled *approved* invoice keeps its number. Gapless means "1..n with no holes", not
   "every number belongs to a live invoice".

**VAT** is a closed set on both the client party and the invoice: `standard_hr` (2500bp),
`reverse_charge_eu`, `outside_scope_non_eu` ← Tenderly, `exempt_issuer`. `vat_note` is
**copied onto the invoice at approval**, not looked up at render time — a law change next
year must not rewrite an issued invoice. Line VAT is `round_half_up(net * rate_bp / 10000)`;
`check (total = subtotal + vat)` makes a rounding bug **a constraint violation instead of a
silently wrong invoice**.

**Status machine**, every transition a compare-and-swap (`where id = $1 and status =
$expected`; zero rows is a lost race, and the caller gets current state, not a retry):

```
draft ──approve──▶ approved ──send──▶ sending ──delivered──▶ sent ──match──▶ paid
  └──cancel──▶ cancelled           └─all failed─▶ approved          └──UnmatchPayment──┘
```

`paid → sent` is the **one non-monotone edge**, reachable only through explicit
`UnmatchPayment` with its own audit row (the bank corrected a transaction). Name it, test it,
forbid it everywhere else.

**Documents**: `documents` (content-addressed, `sha256 unique`) + `document_blobs` (payload
in a side table so metadata queries never drag bytes off disk) + `document_sources` (the same
supplier PDF arriving by email *and* in the folder is one document with two source rows).
Bytes live in Postgres behind a `Blobs` trait, `storage_key` reading `pg:<sha256>`. One
backup, one transaction, one erasure path, at the cost of a row per PDF — correct up to a few
GB, and ~360 documents/year at 50–500KB is two orders of magnitude away. Rejected: a
filesystem PVC creates a second durability domain where the backup and the volume snapshot
disagree; S3 — **there is no object store in this tree** and `aws-lc-sys` is banned. The
trait is the seam for `S3Blobs` later. `~/finance-data/` is an import *source*, not storage.

**Two link tables, different things**: `transaction_documents` (supplier invoices ↔ the
transaction that paid them, many-to-many, with confidence) and `payment_matches` (**our**
issued invoices ↔ incoming transactions, primary key `(invoice_id, bank_transaction_id)` as
the duplicate-payment guard).

**How a payment is matched to an invoice — corrected against real data.** The plan originally
assumed *poziv na broj* arrives in a structured reference field. It does not, for the payments
that matter. Every Tenderly settlement carries `reference_number: "HR99"` — model 99, meaning
*no structured reference* — and puts the invoice number in free text:

```
2026-09-04  14,500.82  TENDERLY D.O.O.  remittance: ["HR99", "BROJ RACUNA 9-1-1-2026"]
2026-08-06  25,021.67  TENDERLY D.O.O.  remittance: ["HR99", "BROJ RACUNA 8-1-1-2026"]
…seven consecutive months, identical format
```

So the matcher tries two paths in order: the structured `reference_number` when its model is
not `HR99` (real for outgoing tax and utility payments — `HR68 1910-38846238650-26253`), then
a regex for `BROJ RACUNA <ordinal>-<premises>-<device>-<year>` over `remittance_information`.
Matching stays deterministic — amount and invoice number agree — it just reads text rather
than a field. Amount alone is never sufficient: two months could bill the same figure.

`remittance_information` is an **array of strings**, not a string; element 0 is the reference
model. Counterparty names need an alias table before they can be grouped — the same person
arrives as `VESIĆ NEVIO`, `Vesic Nevio` and `Nevio Vesić`, and Anthropic bills as both
`ANTHROPIC* CLAUDE SUB` and `CLAUDE.AI SUBSCRIPTION`. Case and diacritic normalisation is
necessary but does not fix word order or vendor aliasing; `parties` is where the aliases live.

## Riding the protocol: one gateway, every surface

**Nothing is written in `crates/protocol` for finance.** Verified by reading
`docs/protocol/README.md` and `crates/protocol/src/transcode/`. The gateway reads
`tbd_proto::DESCRIPTOR_SET_ALL` at startup, and a service in package `tbd.<name>.v1`
forwards to the registry backend `<name>`. Two things make finance appear on every surface:

1. `[services.finance]` in `configs/protocol/base.toml` — **the scaffolder inserts this**
   before the `# tbd:services-end` marker, along with `PROTOCOL_FINANCE_URL` in
   `.env.example`, `compose.yaml`, the ConfigMap and the ansible template.
2. `option (google.api.http)` on each RPC in `proto/tbd/finance/v1/finance.proto`.

From those two, the following come for free and **cannot drift apart**, because they are read
from the same annotations:

| Surface | How finance gets it |
|---|---|
| **REST + SSE** | Every annotated RPC becomes a route. Unary → JSON; server-streaming → server-sent events. |
| **Multiplexed WebSocket** `/v1/ws` | Every annotated RPC is callable by name: `{"type":"call","id":"1","method":"tbd.finance.v1.FinanceService/ListTransactions","body":{…}}`. Same callable set as REST, by construction. |
| **gRPC** | Through Envoy's edge to the protocol, routed by service name. |
| **OpenAPI** | `docs/protocol/openapi.json`, served at `/openapi.json`. A committed-diff test fails if it is stale, so `mise run protocol:openapi` runs in the same commit. |
| **Health** | `/readyz` probes finance's `grpc.health.v1` and re-reports it under `tbd.finance.v1.FinanceService`. |
| **Client metrics** | `tbd_engine_client_requests_total{backend="finance",route,status}` and the duration histogram. The metric name predates the registry; `backend` is the `[services]` name. |

`required = false` in the registry, deliberately: a required backend gates `/readyz`, so a
finance outage would take the whole edge out of rotation. The ledger sets the precedent and
the reasoning is spelled out in the protocol README.

**GraphQL is the one surface that is not automatic.** `crates/protocol/src/graphql.rs` is a
hand-written, engine-only schema — `Query { evaluate, engine_ready }`, `EmptyMutation`,
`EmptySubscription`, resolvers calling the typed engine client. It is not descriptor-driven,
so finance gets nothing there for free.

**Recommendation: make GraphQL descriptor-driven like everything else, as its own phase, and
do not block finance on it.** Hand-writing finance resolvers would be faster but reintroduces
exactly the drift the transcoder exists to prevent — two hand-maintained surfaces over one
contract. Doing it generically fixes engine, ledger, humans and every future scaffolded
service in one change.

It is feasible because the hard parts already exist in `crates/protocol/src/transcode/`:
JSON↔proto conversion, calling a backend by method name over the traced transport, and gRPC
status → error envelope mapping. The new code is schema construction, not plumbing.

- `async-graphql` 7.2.1 is already a workspace dependency; add its **`dynamic-schema`**
  feature (currently only `playground` is on).
- `crates/protocol/src/graphql/dynamic.rs`: walk `tbd_proto::DESCRIPTOR_SET_ALL`; message →
  `dynamic::Object`, enum → `dynamic::Enum`, repeated → `TypeRef::List`; resolver converts
  GraphQL args → `serde_json::Value` → the existing transcode call → JSON → `FieldValue`.
- **Query vs Mutation comes from the HTTP annotation** — `get:` is a Query field, everything
  else a Mutation. Reuses metadata that already exists rather than inventing a second rule,
  and keeps the three surfaces describing the same contract.
- Server-streaming RPCs become Subscriptions over the existing WebSocket transport.
- Field naming: `tbd.finance.v1.FinanceService/ListTransactions` → `financeListTransactions`.

Gotchas to settle when it is built, none of them blocking: proto `oneof` has no clean GraphQL
input equivalent (flatten, or expose as a JSON scalar); `Timestamp`, `Struct` and `Any` need
custom scalars; and field casing diverges — REST and the WS mux use proto snake_case names,
while GraphQL clients expect camelCase. Diverging is the right call there, but it must be
deliberate and documented, not accidental.

Once the dynamic schema works, the hand-written engine schema is deleted rather than kept
alongside it. One path, or the drift comes back.

### What the proto must satisfy

The transcoder **refuses these at startup**, each with its reason, so the contract is
constrained before it is written:

- `*` and `**` path segments, `{a=b/*}` sub-paths, `:verb` suffixes, custom verbs
- `body: "*"` on `GET` or `DELETE`
- client or bidirectional streaming
- a streaming template that does not end in `/events`, or a unary one that does
- a duplicate `(verb, path)`, including collisions with hand-written routes

Input is strict and that is a feature: an unknown body field, an unknown query key, a
singular field given twice, or a value that will not convert is `400 bad_request` with a
`field` detail. Bodies must be `application/json` (else `415`) and ≤2 MiB (else `413`);
responses are capped at tonic's 4 MiB — which is why `GetInvoiceDocument` returns a PDF as
base64 `bytes` and a 50 KB invoice is comfortable, but a future bulk export must stream or
paginate rather than return one blob.

JSON conventions the UI will see: proto field names, **64-bit integers as strings** (which is
exactly what the `bigint` minor-units money representation wants in a browser), enums by
name, `bytes` as base64, `Timestamp` as RFC 3339, and every field emitted including defaults
— so `stub` is present on every response.

### The RPCs

Two streams only: `/v1/finance/transactions/events` and
`/v1/finance/invoice-runs/{run_id}/events`. Connection and sync progress are *fields* on
`Connection`/`Account`, not a third stream — an event stream for something that changes four
times a day is ceremony.

## The three hard mechanisms

**1. Approval you cannot fake.** `PreviewInvoice` writes nothing and returns
`number_preview: { ordinal: 10, preview: true }` — the stub-labelling rule applied to a number
that has *not* been allocated. `ApproveInvoice` carries `content_hash`, the SHA-256 of the
canonical draft from the preview the human saw. **Draft changed since? `FAILED_PRECONDITION`.**
That is what makes "nothing goes out without my say-so" testable rather than aspirational.
One transaction does all of: CAS the status, allocate the ordinal, render the PDF, insert the
document, freeze `vat_note`, write the `approvals` row (the whole `Principal` plus
`trace_id` and `content_hash`), append `invoice_events`.

**2. Send that cannot double-decide, and is honest about the rest.** `SendInvoice` CASes
`approved → sending`, then in one transaction inserts one `invoice_deliveries` row and N
`email_outbox` rows, and returns. The intent is committed before any network call. Three
independent guards: `unique index (invoice_id, channel) where cancelled_at is null` — Postgres,
not application code; the status CAS; and `unique (invoice_id, idempotency_key)`.

The drainer reuses `crates/ledger/src/outbox.rs`'s shape — `SKIP LOCKED` claim, lease,
batch-then-ack — but **not its protocol**. The ledger's claim → publish → ack is at-least-once
by design, and ClickHouse dedupes on `event_id`. **An SMTP server does not dedupe.** So:
claim → write attempt intent → send → ack, with the RFC 5322 `Message-ID` minted *before the
first attempt* as `uuidv5(delivery_id, recipient)`, so a resend after a lost ack carries the
same id and clients suppress it. Lease > SMTP timeout + slack. Bounded attempts with backoff.

**Say plainly in the docs:** "never send twice" is not achievable against SMTP. What *is*
guaranteed is (a) the system never *decides* to send twice, and (b) any duplicate carries the
same `Message-ID`, is counted, and is visible — a claimed row with `attempts > 0`,
`sent_at is null` and an expired lease becomes `uncertain`, a gauge, an SSE frame and a UI
banner. That is the stub-labelling ethic applied to guarantees. Set `Return-Path`/VERP now so
a bounce processor later needs no schema change.

**3. Consent whose callback never touches an open route.** The bank's redirect lands on the
authenticated UI host. Envoy's API vhost requires a JWT and scope `tbd.api`; the only open
routes are `/healthz`, `/readyz` and the chaos health path. `finance.<domain>` uses the
existing `*ui_gated` OAuth2 browser filter, so the browser is already signed in. The page
reads `code` + `state` from its own URL and POSTs them to `CompleteConnection`, code **in the
body**. Rejected: a dedicated open callback route — an unauthenticated endpoint minting a
180-day banking consent for whoever calls it first, with Envoy's access log writing `path`
verbatim into VictoriaLogs. The code is never a query parameter on our API, never a `tracing`
field, never a DB column, and is excluded from every `Debug` impl. `state` is 32 bytes from
`rand`, unique, single-use; reuse is `ALREADY_EXISTS`; mismatched owner is `NOT_FOUND`, not
`PERMISSION_DENIED`, which would confirm it exists.

## Sync worker and the 4/day budget

`banking/syncer.rs`, modelled line for line on `crates/ledger/src/sweeper.rs`: `tick()`
testable alone, `run(cancel)` with `tokio::select!`, spawned into the same `JoinSet` as
`health::run`.

**The budget is a database row, not a token bucket** — incremented inside the same
transaction that claims the account `for update skip locked`. Costs a round trip; it's the
only version that survives a restart or a second replica, and spending it wrongly costs a
*day* of stale data. A 429 writes `backoff_until = now + retry_after` (header, else the
documented 6h), and `claim_due_accounts` already filters on it — **backoff is one `where`
clause, not a sleeping task**.

`[sync]`: `interval = "15m"` (how often the loop wakes, not how often a bank is called),
`min_interval = "5h"`, `budget_per_day = 4`, `scheduled_budget = 3`, `overlap_days = 7`,
`initial_history = "730d"`, `reconsent_lead = "14d"`, `default_backoff = "6h"`.
**One call a day is reserved for the human** — pressing *Refresh* must not answer "come back
tomorrow" because a cron job spent the quota.

Retries, ~20 lines, no crate: 429 and 5xx on a data call are **never** retried in-process
(retrying a 429 against a 4/day quota is the worst possible behaviour); connect/timeout on a
GET retries twice with jitter, only when the request was provably not sent; `POST /sessions`
is never retried because the code is single-use.

## PDF rendering

**Typst as an embedded library**, behind a default-on crate feature so chaos/CI builds can
skip the compile. Real typesetting in-process — no browser, no TeX distribution, no
`openssl`, no `aws-lc-sys`, no `unsafe` — paid for with a heavy compile and a small template
language. Rejected: pure-Rust PDF crates (the layout *is* Rust code, so "supply your own
template" degrades to "file a ticket"); headless Chromium (~300MB browser in a distroless
image, a second process to supervise, OOM as a routine failure mode); LaTeX (TeX Live in the
image plus shell-escape exposure).

The contract is one versioned JSON document, `render::InvoiceDoc`, documented in
`docs/finance/template.md` and dumped by `finance template sample`; the template reads it via
`sys.inputs`. **Templates are rows, not files** — source plus assets stored as `documents`,
rendered over an in-memory resolver seeded with exactly those. Hermetic, no filesystem, no
package downloads. `template_id` + `template_version` pinned onto the invoice, so re-rendering
in 2031 yields the same document. Pin the PDF `/ID` and `document.date` from the issue date so
renders are byte-identical — `documents.sha256` then dedupes for free and a differing
re-render is a *detected* tamper. Guardrails: `spawn_blocking` behind a semaphore, with
timeout, `max_pages`, `max_bytes`; a timeout is a clean `DEADLINE_EXCEEDED`, never `Internal`.
Ships with a `Mock` and a `stub` engine emitting a watermarked "STUB — NOT AN INVOICE" page;
`SendInvoice` **refuses** to send a stub outside tests.

## Chaos: the Fintech section and severe tests

Extend the generated kind adapter like `crates/chaos/src/kinds/ledger.rs`: fields for
`database_url`, `mail` (capture|off), `render` (typst|stub), `clock`, `timezone`; three fault
surfaces; real checks replacing `grpc_finance_ping` — `grpc_finance_invoice_cycle`,
`grpc_finance_numbering_gapless`, `grpc_finance_stale_hash_refused`. **A pinned clock is a
prerequisite**, not a nicety: month-boundary and skew tests are unwritable without it.

Three pages in `ui/chaos`: `/fintech/` (the money-safety board — not a chart, rules with a
state and a count: "Numbering gapless (HR 2026): ordinals 1–9, 0 gaps", "0 approved-but-unsent
> 24h", "0 email rows uncertain", "0 invoices with >1 live delivery"), `/fintech/mail/` (the
capture sink, **two rows with the same `Message-ID` flagged red** — the screen that makes
"did it double-send" visible), `/fintech/clock/` (month-boundary presets).

New API: `PUT /stack/{name}/faults/{surface}`, `PUT /stack/{name}/clock`,
`GET /fintech/invariants`, `GET|DELETE /fintech/mail`. New SSE frames `mail_captured`,
`fintech_invariant_broken`. `docs/chaos/{api,config,kinds,scenarios,stress,ui}.md` and
`ui/chaos/src/lib/api/schema.ts` update **in the same commit** — that is the tool's contract.

**Access-control scenarios — these are the ones that must exist, because a leak is
silent and a unit test cannot drive the real gRPC surface.** They need the finance
service to exist (P2), which is why none of them can be written during P1:

| Scenario | What it drives | Fails when |
|---|---|---|
| `finance_access_denied` | two principals against one running service: the owner and a reader granted only the company | the reader's `ListTransactions` returns a single row belonging to the personal party |
| `finance_access_narrow` | the reader calls every read RPC passing party ids it was never granted, in path, query and body | any RPC honours a client-supplied party filter instead of intersecting it |
| `finance_access_revoked` | revoke mid-run while the reader is streaming `/v1/finance/transactions/events` | the open stream keeps delivering after the grant is gone |
| `finance_access_expired` | a grant lapsing during a run, clock pinned to cross the boundary | rows arrive after `expires_at` |
| `finance_access_enumeration` | ask for 1,000 random and 10 real-but-ungranted party ids | any answer distinguishes "exists but not yours" from "does not exist" — a forbidden where there should be a not-found |
| `finance_access_scope` | a machine token carrying `tbd.api` but not `tbd.finance` | it reaches any `/v1/finance/*` route. **This is the gap measured on the live cluster**: a `tbd.api` token reads the chaos admin API today, and finance must not inherit that |
| `finance_access_rls` | the same reads with the service's SQL filter deliberately removed, RLS left on | the database returns rows the filter would have excluded — proves the second layer is real rather than decorative |

The corresponding stress invariants, which run against a model built only from requests
and responses: `no_row_outside_grant`, `filter_never_widens`, `revocation_is_immediate`,
`absence_is_indistinguishable_from_denial`.

**Other scenarios**: `finance_baseline`, `finance_month_boundary` (clock pinned to 23:59:30 CEST on
31 Aug, pushed past local midnight mid-run — the period must still be August),
`finance_send_storm` (200 concurrent sends on one invoice → exactly one delivery row;
`ALREADY_EXISTS` counted as a *clean* answer, not an error), `finance_mail_fault`,
`finance_store_fault` (lost-ack mode — `store/faulty.rs` already produces exactly this),
`finance_render_timeout`.

**Stress** needs a second worker class and client in `tbd-stress` — the largest single piece
of hardening. Generalise rather than fork: add `client::FinanceClient` and
`model/invoice_invariants.rs` beside the ledger's; `trace.rs`, `shrink.rs`, `replay.rs`,
`stats.rs`, `sweep.rs` reused **unchanged**. The founding rule holds: `tbd-stress` never
depends on `tbd-finance`; the model is built only from requests and responses. Invariants:
`numbering_gapless`, `numbering_unique`, `number_only_after_approval`, `approved_is_immutable`,
`content_hash_gates_approval`, `one_live_delivery`, `send_requires_approval`,
`idempotent_approve`/`idempotent_send`, `totals_are_exact`, `money_is_conserved`,
`status_monotone`, `no_duplicate_message_id_with_different_bytes`. Campaigns:
`finance_numbering` (**must run on Postgres** — gaplessness is a row-lock property),
`finance_send_dedupe`, `finance_lost_ack`, `finance_rounding` (hostile money: 33.333
quantities, 0.005 unit prices, JPY with no minor unit, VAT on a net rounding at exactly .5,
negative lines — **this is where the currency-rounding bug lives**), `finance_clock_skew`,
`finance_render_pressure`.

## UI

**A new `ui/finance` app**, separate from `ui/chaos`. `ui/chaos` is served *by the chaos
binary*, and `configs/chaos/production.toml` says outright that `serve` never runs in
production — invoices must not be reachable from the pod whose job is injecting faults. Envoy
gates by host, so `finance.<domain>` is one more virtual host. `ui/auth` is the precedent.

Screens: **spend dashboard** with a personal/business/combined toggle backed by the party
filter; **invoices** list; **monthly review and send** (one card per draft, editable lines,
live totals, a *what changed since last month* diff against `prefilled_from`, inline PDF
preview, per-invoice Approve carrying the displayed preview's `content_hash`, and a run-level
Send disabled until everything is approved); **invoice detail** (the immutable record — PDF,
approval, deliveries, payment match); **parties**; **templates**.

Drafts pre-fill from `subscription_lines`, then **overlaid with the last `sent` invoice** —
what you want is "same as last month", not "same as the contract". `prefilled_from` records
which, so the screen can diff.

`mise run ci` gains `ui:finance:check` mirroring `ui:auth:check`, plus a `ui-finance` job in
`.github/workflows/ci.yml` and a row in `docs/ci.md`.

---

## Phasing

**P0 is done and the gate passed.** Erste Croatia business *and* personal accounts are
readable through Enable Banking's free restricted-production tier, under one application,
with 180-day consents and no contract. 2,861 real transactions pulled. Full evidence in
`prototype/bank/FINDINGS.md`; the corrections it produced are folded into the sections above.

One sequencing lesson worth keeping: P0 was written to run `GET /aspsps?country=HR` as a cheap
pre-consent gate. That is impossible — an application in restricted production is inactive
until own accounts are linked, and an inactive application gets 403 on *every* endpoint,
including the read-only ASPSP list. Linking **is** the gate. A future provider evaluation
should assume the same and budget for it.

| Phase | Work | Deliverable |
|---|---|---|
| **P0. Prototype** | `prototype/bank/`, excluded from the workspace. Steps 1–2 first, then stop and read the ASPSP list before building 3–6 | `FINDINGS.md` answering: business + personal both readable, history depth, whether *poziv na broj* survives, `entry_reference` stability, the real rate limit, and the real cost |
| **P1. Platform** | `/migrations` at the root, `crates/db` with the migrator and pool, ledger's migration moved and renumbered, `0001_identity.sql`, per-service roles and grants, `tbd migrate` + the startup version check, compose/k8s renames | `mise run db:migrate` builds the whole schema; ledger still passes its suite on the shared database; `users` JIT-provisions from a real login |
| **P2. Scaffold** | `tbd new service finance`, then its `CHECKLIST`: `cargo check`, `mise run chaos:docs`, `kustomize build && docker compose config -q && mise run envoy:validate`, `mise run ci` | `GET /v1/finance/ping` answers through Envoy |
| **P3. Store** | `0003_finance.sql`, the `Store` trait + memory/pg/instrumented/faulty/validate/sql/dedup, the `conformance_suite!` macro from the first commit | Suite green on memory **and** real Postgres |
| **P4. Real data, no APIs** | `finance import --csv` (bank, through the production upsert path) and `finance import --dir ~/finance-data` (documents, plus backfilling subscriptions and the `invoice_numbers` row from last year's invoices) | **Real transactions and every past invoice in the database within days**, de-risking the schema before any API work |
| **P5. Read + dashboard** | Unary read RPCs, `service.rs`, `[services.finance]` in `configs/protocol/base.toml`, `ui/finance` dashboard with the personal/business toggle, `finance.<domain>` vhost | **Goal 1 met**, with Enable Banking not yet existing |
| **P6. Invoice workflow** | Numbering, status machine, approvals, `OpenInvoiceRun`/`UpdateInvoiceDraft`/`PreviewInvoice`/`ApproveInvoice`, Typst renderer, the real template and new logo. Scenarios `finance_baseline`, `finance_month_boundary` | **On 30 September: open the screen, see Tenderly pre-filled from August, edit, approve, download a correct PDF.** Sending still by hand — value before risk |
| **P7. Bank automation** | Enable Banking client (`ring` JWT, fixture stub server), `Provider` trait, consent flow and callback page, `syncer.rs`, budget and backoff, `StreamTransactions` | Transactions arrive with nobody pressing anything |
| **P8. Sending** | `SendInvoice`, deliveries, `email_outbox`, the mailer, `lettre`, `finance:smtp`, capture sink, chaos `mail` surface and the Fintech UI section. Scenarios `finance_send_storm`, `finance_mail_fault`; campaigns `finance_send_dedupe`, `finance_lost_ack` | **The Send button — with the double-send proofs green in CI before the first real send** |
| **P9. Reconciliation + accountant** | `payment_matches` auto-matched on amount + *poziv na broj*, `UnmatchPayment`, `ApplyRules` at scale, `MonthlySummary` across parties, the export bundle | **Goal 2 met** — one download for the accountant |
| **P10. Document ingestion** | Gmail collector (read-only scope), portal fetchers, text extraction (no OCR), matching into `transaction_documents` with confidence, reviewed in the UI | No more digging through mail and portals |
| **P11. Hardening** | The `tbd-stress` finance worker class and remaining campaigns, Grafana dashboard, metrics into `crates/common/src/metrics.rs` `names` + `docs/observability/metrics.md` | |
| **PG. GraphQL, descriptor-driven** | `dynamic-schema` feature, `crates/protocol/src/graphql/dynamic.rs`, Query/Mutation split from the HTTP annotation, Subscriptions for streams, custom scalars, then delete the hand-written engine schema. Independent of the finance phases — schedule it whenever, it lands GraphQL for **every** service at once | `financeListTransactions` and `engineEvaluate` resolve from one schema built at startup; `docs/protocol/README.md` gains a GraphQL section; the chaos `graphql_evaluate` check still passes |

**Deferred on purpose: the eRačun / Fiskalizacija 2.0 channel.** Tenderly is non-EU and there
are no domestic clients, so nothing needs it. What's already in place so it stays a serialiser
and not a migration: `invoice_deliveries.channel` is a column not an assumption; parties carry
OIB and EU VAT id; VAT is line-level with a rate and treatment code. **Open question when it
comes due: which *informacijski posrednik*** — `FiskAplikacija` is browser-only via
NIAS/ePorezna and is not an integration target. Evaluate on: a real HTTP API with a sandbox;
whether they also *receive* on your behalf (mandatory from 1.1.2026 regardless, and Inorbit is
VAT-registered); per-document pricing at ~60/year; whether they accept your UBL.

**Side track: the logo.** Fresh explorations, 3–4 directions as SVG (wordmark, mark+wordmark,
monogram). On the critical path for P6 — the template references it as an asset. The chosen
one becomes a `document_templates.assets` entry in light/dark/mono, reused by `ui/finance` and
the favicon. The 292×140 bitmap is retired then.

---

## Verification

**Every phase:** `mise run ci` — fmt, typos, `cargo deny`, clippy `-D warnings`, buf lint,
nextest + doctests, `tbd:selfcheck`, docs, `chaos:docs:check`, `chaos:run`, `stress:run`, UI
checks. It must pass before anything is called done, and its output gets shown.

| Phase | How it's proven |
|---|---|
| P1 | `mise run db:migrate` from empty; ledger's full suite including `pg::` passes against the shared database; a real browser login creates exactly one `users` row; starting a service against a behind schema **fails with a message naming the gap** |
| P2 | `mise run tbd -- service check finance` exits 0; `local:up && local:deploy`, then `grpcurl` the ping through Envoy on :18080 |
| P3 | `cargo nextest run -p tbd-finance` on memory; `FINANCE_TEST_DATABASE_URL` or testcontainers for `pg::`. The Postgres-only cases matter most: concurrent upserts insert once, `claim_due_accounts` never hands one account to two callers |
| P4 | Import a real Erste CSV, then **run it twice** — the second inserts zero rows. Same for `--dir`. `mise run db:psql` to eyeball |
| P5 | Open `finance.<domain>`, sign in through Ory, see real August transactions, categorize one, toggle personal/business/combined and watch the totals change |
| P6 | Open a run for the current period, confirm it pre-fills from the real August invoice, edit a line, preview, approve, and **diff the PDF against `inorbit-31-08-2026-9-1-1-tenderly.pdf`** — same fields, same number format, same reverse-charge note. Then approve with a stale `content_hash` → `FAILED_PRECONDITION`; approve three and cancel the middle → ordinals 1,2,3, no gap |
| P7 | `finance providers --country HR` against the live API (one harmless call proving signing, TLS, decoding). Link the real Erste account once. Then chaos scenarios for 429 and expired consent, asserting the 4th scheduled call in a day is refused **without an HTTP request** |
| P8 | `finance_send_storm` and `finance_mail_fault` green in CI *first*. Then a real send to your own address, checking `/fintech/mail/` shows one `Message-ID`. Only then a real client |
| P9 | Issue an invoice, pay it with the *poziv na broj*, confirm it auto-matches to `paid`. Then `UnmatchPayment` and confirm `paid → sent` is the only backwards edge allowed |
| P10 | Point the collector at a month already reconciled by hand; compare against `~/finance-data/08-2026/` — the nine known PDFs, no more, no fewer |
| P11 | `mise run stress:run`; `finance_rounding` and `finance_numbering` find zero violations |
