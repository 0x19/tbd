# Inorbit finance — the accountant handoff

Extends `finance-service.md`. That plan gets money in, categorised and queryable, and gets
invoices out. This one closes goal 2 properly: **the accountant receives, once a month, one
bundle that answers every question they would otherwise email about**, in a form their
software imports. It does not replace the accountant. The computations they still own
(payroll, the corporate tax return, the annual statements) are listed at the end so the line
is explicit.

## Context

There is no licence for who keeps a d.o.o.'s books in Croatia; the director is responsible
either way. What the accountant sells is a set of recurring computations and the liability
for them. Today they get the raw material ad hoc: a pile of supplier PDFs, questions about
what a transfer was for, and invoices they have to key in by hand.

Measured on the real data (`prototype/bank/FINDINGS.md`), the questions they ask every month
have the same shape:

| Question | What answers it | Exists today |
|---|---|---|
| What is this payment for, where is the invoice? | a document linked to the transaction | `transaction_documents` planned in P10; no "missing" report |
| Is this supplier foreign, do we self-assess VAT? | a VAT treatment on the incoming document | no |
| What was this transfer to your personal account? | a classification with a document behind it | `transactions_enriched.internal` says *that* it is a transfer, not *what* |
| Which account code does this go to? | the chart-of-accounts code | `categories.account_code`, null everywhere |
| Which tax did this state-budget payment settle? | the revenue-type code in the payment reference | `reference_number` is stored, never decoded |
| What did you buy that we depreciate? | a fixed-asset register | no |
| Can I import this instead of typing it? | a file in their software's format | no |

Every one of these is data the system already holds or will hold. What is missing is the
layer that turns it into the accountant's vocabulary.

## Deliverable: the monthly bundle

One download per period, produced by `ExportPeriod` and recorded in `exports` so "what did
the accountant get for August" is a query. The contents, in the order they will open them:

1. **`cover.pdf`** — period, party, counts, totals per currency, and the list of things the
   system could not resolve: transactions without a document, transfers without a
   classification, documents with no VAT treatment. **The unresolved list is the point.**
   A bundle that hides its gaps is a bundle the accountant cannot trust.
2. **`ira/`** — every invoice we issued in the period: `UBL 2.1` XML plus the PDF the client
   received, and `ira.csv` (the *Knjiga IRA* columns: number, date, client, OIB or VAT id,
   net, VAT by rate, total, VAT treatment, paid-on date and the matching transaction).
3. **`ura/`** — every supplier document: the original file, and `ura.csv` (the *Knjiga URA*
   columns: supplier, country, their invoice number, date, net, VAT, total, VAT treatment,
   reverse-charge flag, the transaction that paid it).
4. **`statements/`** — one CSV per account with every booked transaction, category,
   account code, counterparty canonical name, and the document or classification it links
   to. The bank's own PDF statement beside it where the provider gives one.
5. **`transfers.csv`** — every internal transfer with its classification and supporting
   document.
6. **`assets.csv`** — the fixed-asset register with the period's depreciation.
7. **`journal.csv`** — one row per proposed posting, in the accountant's import format.
   Proposed: the accountant reviews and imports; the system never claims these are the
   books.

Every row that came from a rule rather than a person carries `source = inferred`, in the CSV
as a column, in the PDF as a mark. The ledger's provenance vocabulary applies to the export
exactly as it applies to the store: nothing inferred is presented as declared.

## Features

### F1. The missing-document list

A booked expense with no `transaction_documents` row, aged by booking date, excluding
categories flagged `no_document_expected` (bank fees, state-budget payments, cash
withdrawals). Surfaces in the UI as a queue, in the bundle as the first table on the cover.

Schema: `categories.document_expected boolean not null default true`. The seed flips it for
`bank_fees`, `tax_*` and `cash`.

This is the feature with the best ratio of value to size. It is one query and it removes the
monthly email thread.

### F2. VAT treatment on incoming documents

The suppliers are almost all foreign. Hetzner and Google invoice from the EU, Anthropic,
Cloudflare, Medium and Excalidraw from outside it. A service bought from abroad is
self-assessed: the company books output VAT and deducts the same amount as input VAT in
the same return. The accountant needs to know which line of the return each document
belongs on, and the return distinguishes EU from non-EU.

`documents` gains:

```sql
vat_treatment  text null check (vat_treatment in (
    'domestic_standard',          -- Croatian supplier, VAT on the invoice
    'reverse_charge_eu',          -- EU supplier, service, self-assess
    'reverse_charge_non_eu',      -- non-EU supplier, service, self-assess
    'exempt',                     -- VAT-exempt supply
    'out_of_scope'                -- not a VAT event: fees, taxes, fines
)),
vat_rate_bp    integer null,
net_minor      bigint  null,
vat_minor      bigint  null,
supplier_party_id uuid null references public.parties(id)
```

Treatment is **derived from the supplier party** by default (`parties.country_code`,
`orgs.vat_id`) and recorded per document so a supplier moving jurisdictions does not
rewrite history. Source is `inferred` until a person confirms it; the extractor in P10 fills
net and VAT from the PDF text with `confidence`.

### F3. Owner transfers, classified

116 transfers between personal and company accounts move more money than everything else
combined. To the books, every company-to-person euro is exactly one of these, and each
needs its own paper:

| Kind | Document required | Tax event |
|---|---|---|
| `salary` | payslip, JOPPD reference | contributions and income tax, paid separately to the state budget |
| `profit_distribution` | board decision (*odluka o isplati dobiti*) | capital-income tax withheld, JOPPD |
| `loan_to_owner` / `loan_repayment` | loan agreement with the year's minimum tax-recognised interest | interest income for the company |
| `expense_reimbursement` | the supplier receipt in the owner's name | none, if documented |
| `travel` | travel order (*putni nalog*) | none within the non-taxable caps |
| `capital_contribution` (person to company) | shareholder decision | none |

```sql
create table finance.transfer_classifications (
    bank_transaction_id uuid primary key references finance.bank_transactions(id) on delete cascade,
    kind         text not null check (kind in ('salary','profit_distribution','loan_to_owner',
                    'loan_repayment','expense_reimbursement','travel','capital_contribution')),
    document_id  uuid null references finance.documents(id),
    source       text not null check (source in ('declared','inferred')),
    note         text not null default '',
    classified_at timestamptz not null default clock_timestamp(),
    classified_by uuid null references public.users(id)
);
```

Rules can infer `salary` (the same amount on the same day each month, paired with
state-budget payments in the same week) but a transfer is **never exported as classified
unless it is `declared`**. An inferred classification is a suggestion in the UI, and an
unclassified transfer is on the cover page. This is money leaving the company; a guess is
worse than a gap.

Both sides of a transfer are one fact. When both accounts are ours, the classification
attaches to the debit side and the view resolves the credit side to the same row.

### F4. State-budget payments, decoded

Payments to *Državni proračun* carry model `HR68` references whose first group is the
revenue-type code (*brojčana oznaka vrste prihoda*): which contribution, which tax, which
period. The data has them already (`HR68 1910-38846238650-26253`, `FINDINGS.md`). A lookup
table, seeded from the Ministry of Finance's published list and versioned by year, turns
`reference_number` into "pension contribution, pillar I, August" without a rule per code.

```sql
create table finance.revenue_codes (
    code        text not null,
    valid_from  date not null,
    valid_to    date null,
    name        text not null,
    category_slug text not null,
    primary key (code, valid_from)
);
```

The categoriser consults it before the rules: a decoded code beats a name match, because
the same payee (the budget) receives every tax.

### F5. Chart-of-accounts mapping

`categories.account_code` becomes required for every non-archived category once the
accountant supplies their chart. Until then the bundle's `journal.csv` leaves the column
empty and the cover says so. The mapping is theirs to give; inventing codes would produce a
file that looks importable and is not.

A `postings` view, not a table: derived from categorised transactions and their documents
at export time, one debit and one credit row each, with VAT split out by treatment. It is a
proposal for the accountant's import and is labelled as such. Making it a table would mean
owning a general ledger, which is the line this plan does not cross.

### F6. Fixed-asset register

Anything on the depreciation threshold or above (the accountant confirms the figure; the
tax law sets it) is an asset, not an expense. Laptops and monitors are the realistic cases.

```sql
create table finance.fixed_assets (
    id            uuid primary key,
    party_id      uuid not null references public.parties(id),
    document_id   uuid not null references finance.documents(id),
    name          text not null,
    acquired_on   date not null,
    cost_minor    bigint not null,
    currency      char(3) not null,
    rate_bp       integer not null,       -- annual depreciation rate, basis points
    method        text not null default 'straight_line',
    disposed_on   date null,
    archived_at   timestamptz null
);
```

The bundle's `assets.csv` carries the period's depreciation per asset, straight-line,
computed at export. The accountant may post something different; this gives them the
register and the arithmetic to check against.

### F7. eRačun inbox

*2026-09-21: the cheap form landed -- the `mojeracun` connector pulls the inbox as
documents with the facts read from each UBL and its embedded PDF
(`docs/finance/mojeracun.md`; `eracuni` likewise for e-računi); the VAT lines, below,
remain.*

Since 1 January 2026 every VAT-registered company must receive domestic e-invoices and
fiscalise their receipt. Croatian suppliers now arrive as UBL 2.1 or CII XML through an
information intermediary, not as PDF. The P10 collector gains a third source beside Gmail
and the vendor portals: the intermediary's inbox. A UBL document is parsed structurally
(supplier OIB, number, lines, VAT by rate), so it enters `documents` with `vat_treatment`,
net and VAT already `verified` rather than extracted.

Which intermediary is the open question the main plan already carries for outgoing eRačun;
receiving makes it due now rather than later. `MIKROeRAČUN`, the Tax Administration's free
application, is for companies *outside* the VAT register, so it does not apply to Inorbit.

### F8. The export itself

`ExportPeriod(party, year, month)` renders the bundle into `documents` as one zip
(content-addressed like everything else), writes an `exports` row (period, who, when,
`sha256`, the unresolved counts from the cover), and returns the document id. Re-exporting
the same period produces a new row, never overwrites: the accountant may already hold the
old one.

```sql
create table finance.exports (
    id           uuid primary key,
    party_id     uuid not null references public.parties(id),
    period       date not null,          -- first day of the month
    document_id  uuid not null references finance.documents(id),
    unresolved   jsonb not null,         -- counts from the cover page
    exported_at  timestamptz not null default clock_timestamp(),
    exported_by  uuid not null references public.users(id)
);
```

The `access_log` row the main plan specifies for reads that leave the system is written
here: this is precisely the case it was designed for.

**The journal format is decided by asking the accountant which software they run**, then
matching its import specification. Minimax, e-Računi, Pantheon and Synesis cover most
Croatian firms and each has a documented CSV or XML import. Until answered, `journal.csv`
ships in a neutral layout (date, document number, account code, debit, credit, description,
counterparty OIB, VAT base, VAT amount, treatment). Guessing the target format would waste
the one thing the accountant is being asked to do, which is import it once and say whether
it worked.

### F9. Deadline calendar

A static, year-versioned table of obligations (monthly VAT return, JOPPD on the salary
payment day, corporate tax advances, the annual statements to FINA, the tax return) and a
`/finance/calendar` view that shows what is due, what the bundle for the period looks like,
and whether it has been exported. Not a notification system; a page.

## Where it lands in the phases

| Phase | Gains |
|---|---|
| **P9. Reconciliation + accountant** | F1 missing-document list, F3 transfer classification, F4 revenue codes, F5 account codes, F8 `ExportPeriod` and the bundle with `cover`, `ira`, `statements`, `transfers`, `journal` |
| **P10. Document ingestion** | F2 VAT treatment, F7 eRačun inbox, `ura/` in the bundle |
| **P12. Accountant handoff** (new) | F6 fixed assets, F9 calendar, the accountant's real import format after the first bundle round-trips, `assets.csv` |

P12 is deliberately after a bundle has been sent and imported once. Its content depends on
what the accountant says was wrong.

## What stays with the accountant

Listed so nobody mistakes the bundle for the books:

- **Payroll**: the salary calculation, contributions, income tax at the municipal rate, the
  JOPPD filing, non-taxable receipts and their yearly caps.
- **The VAT return** (*PDV obrazac*): the books above feed it; filing it and answering for
  it is theirs.
- **Corporate income tax** (*PD*): non-deductible items, depreciation as posted, loss
  carry-forward, next year's advances.
- **Annual statements** (*GFI-POD*) and the board decisions that accompany them.
- **Profit distribution** as a tax event.

Each of these changes every January and carries liability. For one director's salary and
one annual return they cost less to buy than to maintain. If, after a year of bundles, the
accountant's postings match `journal.csv` line for line, taking them in-house is a decision
backed by data rather than a bet; that is the point at which a second plan is worth writing.

## Verification

| Feature | Proof |
|---|---|
| F1 | Seed one month of real transactions and no documents: every expense is listed; link one document and it leaves the list; a `bank_fees` row never appears |
| F2 | Every supplier in `~/finance-data/08-2026/` resolves to a treatment from its party's country; a Hetzner invoice is `reverse_charge_eu`, an Anthropic one `reverse_charge_non_eu`; `net + vat = total` is a constraint |
| F3 | Classify the August salary transfer; the credit side on the personal account resolves to the same row; an `inferred` classification never reaches `transfers.csv` |
| F4 | Decode every `HR68` reference in the real data; each maps to a code in the table or is listed as unknown, never silently categorised by the payee name |
| F5 | Export before the chart is supplied: `journal.csv` has empty codes and the cover says so; after: no empty codes for categorised rows |
| F6 | One asset, twelve monthly exports: the depreciation rows sum to the annual rate times cost, in minor units, with the rounding remainder on the last month |
| F7 | A sample UBL 2.1 invoice lands with `verified` net and VAT and needs no extraction |
| F8 | Export August twice: two `exports` rows, two document ids, identical `sha256` when nothing changed; an `access_log` row per export |
| F9 | Every obligation for the year renders with its date; the page shows the August bundle as exported once it is |

The whole plan has one external gate: **the accountant imports one bundle and says what was
wrong.** Everything in P12 is shaped by that answer, so the first bundle should go out as
soon as P9 produces one, before the format is polished.

## Open questions

1. Which software does the accountant use, and does it import CSV, XML or both?
2. How is Inorbit receiving domestic eRačuni today? The obligation began 1 January 2026;
   if the answer is "by email as PDF", a supplier is not compliant and the company's receipt
   is not being fiscalised.
3. Does *eIzvještavanje* (the payment-status reporting introduced with Fiskalizacija 2.0)
   impose anything on invoices issued to a non-EU client? The guidance says fiscalisation
   does not apply to invoices issued abroad; the reporting side is newer and worth one
   question to the accountant.
4. The depreciation threshold and the minimum interest on a loan to the owner for the
   current year, both to be stored in the year table rather than in code.
