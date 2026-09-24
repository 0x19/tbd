# What the system can do today, and the gap

Surveyed on 2026-09-21 (finance crate 18.290 lines, chaos and lab crates, ui/finances,
migrations 0001 to 0021). File references are to that state.

## Finance service

What exists:

- **Bank feed.** Enable Banking PSD2 client, budgeted sync worker (4 fetches per account
  per day), the single provider-JSON-to-rows importer (`crates/finance/src/import.rs`),
  `finance.accounts`, `finance.bank_transactions` (amount in minor units, currency,
  scale, booking and value date, counterparty, remittance, reference), `finance.balances`
  (the bank's own balances).
- **Categorisation.** `finance.categories` with five kinds (expense, income, transfer,
  tax, capital), rules, aliases, declared versus inferred provenance. `account_code` on a
  category exists as a nullable column and is never set.
- **Outgoing invoices.** Gapless numbering per `(party, year, premises, device)` under a
  row lock, four VAT treatments (standard 25%, EU reverse charge, outside scope, exempt
  issuer), integer arithmetic in minor units, Typst PDF with a content hash, approve by
  hash, cancel, duplicate. VAT is one number per invoice, not per line.
- **Supplier documents.** Mailbox connectors pull receipts, PDF text extraction, regex
  field rules per vendor (vendor, date, total, invoice number), manual correction with
  provenance.
- **Reconciliation.** Per month: what each transaction needs (eRačun, receipt, none,
  personal, income, internal), receipt-to-transaction scoring and linking, counterparty
  policies, notes, and a browser-built zip (three CSVs plus PDFs) for the accountant.
- **Mail.** Templates, sending through the linked mailbox, threads, bundle attachment.
- **UI.** Overview with a 24-month summary, accounts, transactions, categories,
  documents, reconciliation, invoices, clients, issuer, connections, connectors, mail.
  Money is BigInt minor units in the browser. No spreadsheet or report generation.
- **Chaos coverage.** Two validate checks (`grpc_finance_ping`, `grpc_finance_books_balanced`)
  and five load operations (`finance_ping`, `finance_access`, `finance_money` over two
  seeded transactions; `finance_trial_balance` and `finance_import_opening` against a
  Postgres, `scenarios/finance_books.toml`, skipped in CI).

What does not exist anywhere in the tree:

| Needed for the filings | State |
|---|---|
| Chart of accounts (classes 0 to 9, groups, synthetic and analytic accounts) | absent; `categories.kind` is a five-word spending taxonomy |
| Journal entries with debit and credit lines, Σdebit = Σcredit enforced | absent; the plan in `docs/plans/finance-accountant.md` deliberately stopped short of a general ledger |
| Account balances, trial balance | absent |
| Fiscal periods, period lock, opening balances, year-end close | absent; months exist only as a `YYYY-MM` string |
| Receivables and payables as ledgers (an approved invoice creates no receivable; `sent`/`paid` are never set) | absent |
| Input VAT on documents (net, VAT, rate, treatment), reverse charge, VAT books (URA/IRA), PDV return | absent; documents carry a total only |
| Fixed-asset register and depreciation | absent |
| Payroll journal, member loan sub-ledger, profit distribution | absent |
| Balance sheet, P&L, additional data, PD return, GFI-POD workbook, notes | absent |
| FX rates and a reporting-currency view | absent; every aggregate is per currency |
| Supplier master with OIB | absent; suppliers are free text |
| Decimal type in the database | absent by decision (`migrations/0003_finance.sql`): money is `bigint` minor units, sqlx has no `numeric` feature |

The money representation (integer minor units, no floats, per-currency sums) is the
right foundation and stays. Depreciation, interest and percentages need a documented
rounding rule on top of it, not a decimal type.

## Ledger crate

`crates/ledger` is the append-only facts store for the humans plane (subject, path,
source, value). It has nothing to do with money and the finance crate does not depend
on it. The name is a coincidence; the accounting ledger is a new thing inside the
finance crate.

## Chaos tool

The seams for a replay family exist and are documented in `docs/chaos/extending.md`:

- A **kind** is data (`Kind` in `crates/lab/src/kind.rs`) with a `parse` function, a
  `Service` implementation that boots the real service on port 0, and validate checks.
  The finance kind boots `tbd_finance::serve_with` or `serve_seeded`; a Postgres URL is a
  field, an empty one means the in-memory store (transactions only).
- A **validate check** returns `Result<String, String>`; a **load operation** returns
  `Ok(())` or one `OpError`; an **assertion** produces `{name, passed, expected: String,
  actual: String}`; a **stress finding** carries `expected` and `actual` as JSON and a
  trace. None of them carries a table today; the UI renders assertions as a four-column
  table and findings as JSON.
- Input comes only from scenario TOML, kind fields (free text), the load seed, and a
  database URL. There is no fixture-file mechanism and no golden-output comparison.
- `[paths]` in `configs/chaos/base.toml` is the directory registry; the
  `scenarios_seed`/`campaigns_seed` pattern (read-only in the image, writable copy on a
  volume) is the idiom for shipped data.
- Docs are the contract: `kinds.md` is generated (`mise run chaos:docs`), `scenarios.md`,
  `api.md` (routes and SSE frames, mirrored by Zod in `ui/chaos`), `config.md`,
  `commands.md`, `extending.md`. `tbd:selfcheck` guards the anchors in shared files.
- Stale spots the survey found (`docs/chaos/architecture.md` listed modules under
  `crates/chaos/src` that moved to `crates/lab`; `README.md` said three kinds and eleven
  operations; `scenarios.md` `[stack]` example omitted finances, humans and playgrounds)
  were fixed in the first chaos commit of this work, with the books operations.
