# Invoicing: from one company's tool to something an accountant would sign off

Written 2026-09-21 after a full read of `crates/finance/src/invoice/`, `service_invoices.rs`,
migrations 0010 and 0011, the three UI pages, the identity layer and the two earlier plans.
Every "missing" below was verified absent in code, not inferred from the UI.

## 1. What the screen is telling you

The invoices list shows five rows: one draft and four "stornirano" drafts with the number
"nacrt". Those four are not cancelled invoices. They are drafts that were *discarded*, and
discard is implemented as `CancelInvoice`, which flips the status and keeps the row forever
(`invoice/store.rs:1123-1152`). Nothing deletes an invoice row; there is no RPC for it.

The KPI strip reads zero everywhere because:

- **Nine real 2026 invoices are not in the database.** The counter `invoice_numbers` sits at
  10 for 2026, so the next number is right, but the invoices 1..9 exist only as Google Docs
  PDFs under `~/finance-data/`. "Invoiced 2026", the year filter, and the pre-fill from the
  last approved invoice all work off an empty set.
- **`sent` and `paid` are unreachable.** The schema allows them; no code path writes them.
  "Send by mail" hands the PDF to the composer and never touches the invoice. There is no
  payment model at all: no `payment_matches`, no `MarkPaid`, no partial payment. The Paid tab
  and the Outstanding / Overdue tiles can never be right.

## 2. What is there, and is good

Keep all of it. The core is stricter than most commercial tools:

- Gapless numbering allocated at approval under a row lock, never a sequence; number is a
  generated column; a cancelled invoice keeps its number. This is the legal rule, done right.
- Approval by content hash: what you saw in the preview is what gets the number, byte for byte.
- Integer money, VAT as a closed set frozen onto the invoice at approval, `total = subtotal + vat`
  as a constraint.
- Deterministic Typst rendering, PDF stored content-addressed.
- Every state change written to `invoice_events` with who and what.
- Party-scoped access with "absence is not found", proven over the real gRPC surface.

## 3. What is missing, seen as the accountant

Grouped by how loudly an accountant would complain.

### 3.1 Day one

| Gap | Where it bites |
|---|---|
| Delete a draft | four immortal rows in the Cancelled tab already |
| Change a draft's client, currency, VAT treatment, series (premises/device) | all fixed at `CreateInvoice(client_id)`; the only way out is discard and start over |
| Duplicate an invoice | monthly work for a stable client is copy-and-adjust |
| Historical invoices | the 2026 books start at number 10; every yearly figure is wrong |
| Mark paid, record a payment date, partial payment | receivables are the accountant's first question every month |
| Storno / credit note (odobrenje) | an issued invoice is never edited or silently cancelled; the correction is a numbered document referencing the original. Today "cancel" on an approved invoice produces no document at all, which is not defensible in an audit |
| Readable audit trail | events are written, nothing reads them; the cancel reason typed into the prompt is invisible afterwards |
| Sent state and delivery record | you cannot tell which invoices went out, when, to whom, or that one went twice |
| Year rollover | a draft made on 31 Dec and approved on 2 Jan takes the old year's counter and prints the old year (`store.rs:599-601, 1033`) |

### 3.2 What a Croatian invoice is expected to carry

- **HUB-3 2D barcode (PDF417)** with model and *poziv na broj* for domestic clients, and a
  SEPA QR for EU ones. Today the reference is a text line and there is no `model` field.
- **Per-line VAT rate** (25 / 13 / 5 / 0) and per-line discount; today one treatment per
  invoice, no discounts, no units (h, kom, mjesec). Fine for Tenderly, wrong for a shop.
- **Non-EUR invoices** need the HNB middle rate on the invoice date and the EUR equivalent
  printed for the VAT amount. No rate source exists.
- **Advance invoice / final invoice** pair, and **proforma (ponuda/predračun)**, which is not an
  invoice and has its own numbering. Neither exists.
- **Invoice language** per client (HR, EN, bilingual). Hard-coded bilingual today.
- **Issuer branding**: the InOrbit mark and wordmark are compiled into the template
  (`assets/invoice.typ`). A second issuer would print InOrbit's logo.

### 3.3 Monthly and yearly outputs

- KIR (knjiga izlaznih računa) export by month: number, date, client, OIB, base, VAT, treatment.
- Receivables aging (0-30, 31-60, 61-90, 90+) and a reminder (opomena) from a mail template.
- OPZ-STAT-1: the quarterly statistical report of overdue unpaid receivables. Impossible
  without paid tracking.
- Bulk PDF / CSV export of a period; the journal in the accountant's software format
  (still an open question in `finance-accountant.md`).

### 3.4 eRačun and Fiskalizacija 2.0

Its own section (§8), to be discussed before anything is planned. Nothing in phases 1 to 6
depends on it, and nothing in them may make it harder: invoice kinds, line-level VAT and the
delivery record are all shaped so an e-invoice channel is one more serialiser and one more
channel value.

## 4. What a SaaS with hundreds of issuers is missing

The access layer was designed for this and is half-wired:

- **No way to create an org, invite a user, or grant and revoke access** except SQL. No RPC,
  no CLI, no page (`crates/db/src/identity.rs` has the functions; every caller is a test).
- **`read` and `own` are not distinguished for writes.** A reader may draft, approve and cancel
  for the party they can read (`tests/it/invoices.rs:441-443` says so). Roles on `memberships`
  are a CHECK constraint and nothing else.
- **RLS is inert at runtime**: the policies let an unset `app.user_id` see everything, and
  `bind_rls_user` is never called by the service. Layer two of the three-layer design protects
  nothing today.
- **`access_log` is never written.** No record of who read whose invoices.
- **No tenant entity.** An accounting firm with 300 client companies would be one user with 300
  `party_access` rows and one flat list. The firm itself, its staff, and who in the firm handles
  which client cannot be expressed.
- Per-issuer template, logo, language, bank accounts and mail settings; a delivery outbox with
  `Message-ID` dedupe; per-tenant rate limits; data export and deletion for a leaving customer.

## 5. The plan

Six phases. The first three are what you would want even if the SaaS idea never happens; they
are also what makes the accountant trust the numbers. Sizes are relative: S is a session, M a
few, L a week of sessions.

### Phase 1: drafts, history, numbering (M)

1. **Delete drafts.** `DeleteInvoice` for status `draft` only, hard delete with cascade; the
   Discard button becomes delete. The four discarded rows are deleted by a one-off. "Cancelled"
   then means an issued invoice, only.
2. **Editable draft header.** `UpdateInvoice` gains client, currency, VAT treatment, premises,
   device, and re-derives the VAT note; the page shows them as selects.
3. **Duplicate.** `CreateInvoice{from_invoice_id}` copies header and lines into a new draft,
   dated today; the row gets a Duplicate action.
4. **Import the nine 2026 invoices** (and 2025's) from the PDFs into the database as approved
   invoices with their real numbers, dates, lines, totals and the original PDF as the document,
   flagged `imported` in the events. A `finance import invoices --dir` subcommand that reads the
   Typst-era and Docs-era PDFs, previews, and asks before writing. After it the year is whole and
   the counter is verified against the rows, not set by hand.
5. **Numbering fixes.** Year taken at approval, not draft creation (a draft crossing the year is
   approved into the new year and says so). Counter keyed `(party, year, premises, device)` to
   match the uniqueness constraint. Optional: a `start_at` on the counter so a new issuer can
   continue from their previous tool's last number, once, before the first approval.
6. **List hygiene.** Status colours, cancelled drafts gone, a Number column that never says
   "draft", the `n` shortcut that the comment promises.

### Phase 2: money in (M)

1. **Payments table**: `invoice_payments(invoice_id, bank_transaction_id null, amount_minor,
   paid_on, method, note, source declared|inferred)`. Partial and over-payments are rows; the
   invoice's `paid_minor` is a sum; `paid` when covered.
2. **Matcher** for incoming transactions, as designed in `finance-service.md`: structured
   reference when the model is not HR99, else `BROJ RACUNA <n>-<p>-<d>-<y>` in the remittance,
   amount must agree. Runs with the sync; a hand link and an unlink with a reason; the one
   backwards edge `paid → sent` only through unlink, with an event.
3. **Sent** becomes real: sending through the composer records `invoice_deliveries(invoice_id,
   mail_id, channel, sent_at)`, sets `sent`, and refuses a second send of the same PDF unless
   forced. The mail thread is reachable from the invoice.
4. **Overdue and aging** computed server-side; a reminder action that opens the composer with an
   `{{Invoice}}`, `{{DueDate}}`, `{{Outstanding}}` template.

### Phase 3: corrections and the record (M)

1. **Document kinds**: `invoices.kind in (invoice, credit_note, advance, proforma)`, with
   `credits_invoice_id` for a credit note and `settles_advance_id` for a final invoice. A credit
   note takes the next number in the same series (or its own series, an issuer setting), negative
   lines, the original's number printed. Proformas number separately and never touch the counter.
2. **Cancel policy**: an approved invoice that was never delivered may be cancelled as today
   (number kept, reason recorded, a "storno" stamp on the PDF); one that was delivered can only
   be corrected by a credit note. The accountant should confirm this rule; it is the usual one.
3. **Audit trail read path**: `ListInvoiceEvents` and a timeline on the invoice page: created,
   edited, previewed, approved (by whom, which hash), sent (to whom), paid, cancelled (reason).
4. **Client archive** and merge; VAT treatment changes on a client take effect on new drafts only.

### Phase 4: the Croatian invoice, complete (L)

HUB-3 PDF417 barcode and model/reference fields; SEPA QR; per-line VAT rate, discount and unit;
HNB rate on the invoice date for non-EUR with the EUR VAT line; invoice language per client;
per-issuer logo, footer text and up to three bank accounts; an item catalogue shared across
clients (line templates stay as the per-client override). Template stays one Typst file with
data-driven branding; a per-issuer template upload waits until someone needs it.

### Phase 5: accounting outputs (M)

KIR export per month; receivables aging report; OPZ-STAT-1 extract; bulk PDF zip and CSV for a
period; the journal in the accountant's software format once they name it. All reuse the
reconciliation bundle's zip and mail path.

### Phase 6: multi-tenant (L, and a product decision first)

1. **Tenant model**: an `orgs` row is a tenant when it has members; `memberships.role` becomes
   real: `owner` (everything), `admin` (settings, access), `accountant` (drafts, approve, send,
   payments), `viewer` (read). `party_access` stays the money boundary; a firm-level grant is
   a membership on the firm org plus grants on each client party it handles.
2. **Access RPCs and pages**: create org, invite by email (Kratos identity, JIT on first login),
   list members, change role, grant and revoke a party with expiry, and an access log page.
   Enforce capability on every write; call `bind_rls_user` in every transaction; write
   `access_log` on delegated reads; add the seven chaos access scenarios the plan lists.
3. **Per-tenant everything**: issuer branding and language, mail connectors (already per party),
   numbering (already per party), a delivery outbox with `Message-ID` dedupe, rate limits, data
   export and deletion.
4. eRačun is not part of this phase; see §8.

## 6. What I recommend doing next, in order

1. Phase 1 now. It is what the screenshot complains about and it fixes the books for 2026.
2. Phase 2 straight after. Paid tracking is the difference between a PDF generator and invoicing,
   and the matcher's design already exists.
3. Phase 3, with one question to the accountant (the cancel rule).
4. Then decide the SaaS question. Two things from Phase 6 are worth doing before any second
   user, even the accountant: enforce `read` versus `own` on writes, and turn RLS on in the
   service. They are small and they close the two findings that matter most in
   `finance-service.md`'s own threat model.

## 7. Questions only you can answer

1. **Who is the SaaS customer**: an accounting firm running invoicing for its client companies
   (tenant = firm, many issuers, staff assignment) or companies invoicing for themselves (tenant
   = company, one issuer, maybe an invited accountant)? The tenant model differs, and Phase 6
   waits on this.
2. **Domestic clients in the plan?** That decides when §8 is opened.
3. **Cancel rule** for issued invoices: confirm with the accountant that an undelivered approved
   invoice may be cancelled with its number kept, and a delivered one only by credit note.
4. **Which software the accountant runs**, for the journal and KIR formats.

## 8. eRačun and Fiskalizacija 2.0: to be discussed

Not planned yet, on purpose. What is known, so the discussion starts from facts:

- Since 1 Jan 2026 a VAT-registered company issuing to a domestic business issues the invoice
  as an e-invoice (UBL 2.1, the Croatian CIUS) through an *informacijski posrednik*, and the
  collections are reported (eIzvještavanje). Receiving domestic e-invoices is mandatory for
  every VAT-registered company, Inorbit included, already.
- Invoices to a non-EU client (Tenderly) are outside fiscalisation; whether eIzvještavanje
  imposes anything on them is the one open question for the accountant
  (`finance-accountant.md`, open question 3).
- The intermediary is unchosen. `FiskAplikacija` is browser-only and not an integration
  target. Criteria noted earlier: a real HTTP API with a sandbox, receiving on our behalf,
  per-document pricing at our volume, accepting our UBL.
- Everything above is shaped so this becomes one serialiser (invoice → UBL), one delivery
  channel value, and one inbox (UBL → `documents` with verified fields), not a migration.

To discuss: whether domestic clients are coming at all, for Inorbit or for a SaaS customer;
which intermediary; and whether the inbox comes first, since that obligation already applies.
