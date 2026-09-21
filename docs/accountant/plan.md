# Plan: the finance service keeps the books

Goal: from the inputs the service already holds (bank feed, issued invoices, supplier
documents) plus a small set of declared judgements, produce the trial balance, balance
sheet, P&L, additional data and corporate tax return for a fiscal year, identical to
what a licensed accountant filed, and prove it by replaying 2025 against the filings in
`gfi-2025.md`. Then render the same on the dashboard and as the FINA workbook.

One recommendation per decision, trade-off stated. Anything touching more than one
crate or the proto goes through plan mode before code.

## Architecture

```
sources                     posting rules                 books                    statements
bank_transactions  ─┐                                    journal_entries          trial balance
invoices            ├─► rule packs (data, per year) ─►    journal_lines     ─►    BIL / RDG / DOP (AOP map, data)
documents           │   + declared judgements             periods, balances       PD (tax rules, data)
payroll months      │                                     fixed_assets            exports: xlsx, FINA xls, PDF
judgement entries  ─┘                                     member_loans             dashboard
```

**A real general ledger inside the finance crate, as a new module `books`.** The earlier
plan (`docs/plans/finance-accountant.md`) refused to own a ledger so that the accountant
stayed the owner of the numbers. That line moves: the accountant becomes the reviewer,
the ledger is ours, and the replay against their filings is the proof. Trade-off: we take
on year-specific tax and reporting rules that change every year; they are kept as data
(rule packs per fiscal year) so a new year is a new file, not a code change.

**Postings are derived, never typed.** Every journal line points at its source (a bank
transaction, an invoice, a document, a payroll month, a declared judgement) and the rule
that produced it. Re-running the rules for a period is idempotent (source id + rule
version is the key) until the period is locked. Trade-off versus free-form manual
journals: a person cannot "just fix" a number; they declare the judgement (a loan
drawdown, a depreciation rate, a distribution) and the posting follows. That is what
makes the result reproducible and what chaos can replay.

**Money stays integer minor units.** Percentages (tax rate, contribution rates,
depreciation rates, interest) are basis points; each computation names its rounding
(half up to the cent, at which step) in one place, `books::money`, with tests taken from
the 2025 figures (12.140,65; 1.011,72; 7.449,87; 150,00).

**Chart of accounts is data.** `configs/finance/hr/chart.toml` seeds the RRiF-style
chart the accountant uses (the 77 accounts in `gfi-2025.md` plus the groups the AOP map
needs); a party may add analytic accounts under a synthetic one. The AOP map
(`configs/finance/hr/gfi-2025.toml`) and the PD rules (`configs/finance/hr/pd-2025.toml`)
are versioned by year in the same way. Trade-off: no UI to invent accounts freely; that
is deliberate, an unmapped account would silently drop out of a statement, so adding an
account requires adding its AOP mapping and a check enforces it.

**Reporting currency is EUR with HNB middle rates.** A rate table (`finance.fx_rates`,
loaded from the HNB API, date and currency) values foreign invoices and card
settlements on the booking date; realised differences post to 7724 or 4750. Trade-off:
one more external feed; it is small, daily, and cacheable.

## Data model (new tables, schema `finance`)

- `ledger_accounts(party_id, code, name, class, group, kind: asset|liability|equity|revenue|expense|off, aop_map_version, parent_code)`
- `periods(party_id, fiscal_year, month, status: open|closed|locked, closed_at, closed_by)`
- `journal_entries(id, party_id, entry_date, period, source_kind, source_id, rule_id, rule_version, memo, reversal_of, created_at)`
- `journal_lines(entry_id, position, account_code, debit_minor, credit_minor, currency, fx_rate_bp, original_minor, counterparty_id, vat_code)` with a check that exactly one of debit/credit is non-zero and a deferred trigger that Σdebit = Σcredit per entry.
- `opening_balances(party_id, fiscal_year, account_code, debit_minor, credit_minor, source: filed|imported|derived)`
- `fixed_assets(id, party_id, account_code, description, acquired_on, in_use_from, cost_minor, rate_bp, method, disposed_on, document_id)`
- `member_loans(id, party_id, member, opened_on, rate_bp, day_count)` and `member_loan_events(loan_id, on, kind: drawdown|repayment|interest, amount_minor, source_id)`
- `payroll_months(party_id, year, month, gross_minor, tax_minor, pension1_minor, pension2_minor, health_minor, net_minor, paid_on, joppd_ref)`
- `judgements(id, party_id, fiscal_year, kind, params jsonb, declared_by, declared_at)` for distribution decisions, catch-up depreciation, prior-year costs, small-inventory write-off, tax accrual.
- `documents` gains `net_minor, vat_minor, vat_rate_bp, vat_treatment, supplier_id, account_code`; `suppliers(id, party_id, name, oib, country, vat_id)`.
- `statements(party_id, fiscal_year, kind: tb|bil|rdg|dop|pd, computed_at, rules_version, values jsonb, inputs_hash)` so a filed statement is frozen with the inputs it came from.

## RPCs (all also over REST through the protocol's transcoding)

Books: `ListLedgerAccounts`, `ImportOpeningBalances`, `ListJournal`, `GetJournalEntry`,
`PostPeriod` (run the rules for a period), `ClosePeriod`, `LockPeriod`, `TrialBalance`.
Judgements: `DeclareJudgement`, `ListJudgements`, `RevokeJudgement`.
Assets and loans: `UpsertFixedAsset`, `ListFixedAssets`, `UpsertMemberLoan`,
`RecordLoanEvent`. Payroll: `UpsertPayrollMonth`. Statements: `ComputeStatements`,
`GetStatement`, `ExportStatement` (xlsx, FINA xls, PD PDF, journal CSV).
Every handler: span with trace id, `RequestTimer`, `Principal` gate; the accountant role
(a delegated reader today) gains `books:declare` and `books:close` scopes.

## Exports

- **xlsx** with `rust_xlsxwriter`: the trial balance, the three statements in FINA's
  layout with AOP codes, the PD rows, the journal. Deterministic bytes (fixed
  timestamps) so chaos can hash them.
- **The official FINA .xls.** FINA keeps the BIFF8 `.xls` format on purpose (control
  108). No Rust crate writes BIFF8 while preserving a template's controls. Recommendation:
  ship the values as JSON and fill the official template with a small Python tool
  (`tools/gfi/fill.py`, xlrd + xlutils) run by `mise run finance:gfi`, verified by
  reading the result back with `calamine` in a test. Trade-off: one Python step in the
  toolchain; the alternative (re-implementing FINA's workbook) would produce a file FINA
  may reject.
- **PD as PDF** through the existing Typst renderer, laid out like the ePorezna form,
  labelled "prepared" until the accountant signs; the ePorezna XML upload is a later
  step once the schema is confirmed (`rules.md`).
- **Notes and the two decisions** as Typst documents from a template with the year's
  figures.

## Dashboard (ui/finances)

New pages: `/books` (trial balance per period with drill-down to journal lines and to
the source transaction or document), `/statements` (BIL, RDG, DOP, PD side by side with
the prior year, every position clickable to its accounts), `/close` (the year-end
checklist: judgements declared, periods locked, checks green, exports). Charts follow
the dataviz skill; numbers are BigInt minor units as today.

## Phases (each its own PR on `feat/finance-prototype`, each closed by `mise run ci`)

| # | Delivers | Proof |
|---|---|---|
| 0 | This documentation | reviewed by you |
| 1 | `books` core: chart seed, periods, journal with the balance invariant, opening balance import, `TrialBalance` RPC and page | chaos: `grpc_finance_books_balanced`, `finance_trial_balance` operation; trial balance from the imported 2025 opening equals the migration column |
| 2 | Posting rules for existing sources: categorised bank transactions, invoices (receivable, revenue by residence, FX), documents (expense, payable, input VAT, reverse charge), payroll months, member loan, small inventory | chaos replay of 2025 H2 sources reaches the "period" column of the trial balance account by account |
| 3 | Year-end: fixed assets and depreciation, distribution, interest accrual, PD computation, tax accrual and netting, BIL/RDG/DOP mapping, consistency checks from FINA's Kont sheet | chaos replay of 2025 equals every filed AOP and PD row; stress invariants |
| 4 | Exports (xlsx, FINA xls fill, PD PDF, notes, decisions) and the three dashboard pages | export bytes hashed in chaos; FINA workbook re-read shows zero errors |
| 5 | 2026 readiness: monthly PDV figures, eRačun inbound and outbound (UBL 2.1 through an intermediary), JOPPD input, NKD 2025 | validate checks per surface |

Phase 1 starts with plan mode: it touches `crates/finance`, `proto`, `migrations`,
`crates/lab` (the finance kind) and `ui/finances`.

## Risks stated up front

- The 2025 opening balances at account level are not in hand (findings.md §1). Phase 3's
  replay can start from the migration column (which the trial balance gives) and prove
  the second half plus year-end exactly; the full-year replay needs the previous
  accountant's closing balance or the accountant's answers.
- Tax rules change yearly; the rule packs must be reviewed by an accountant each year
  before the year's close. The product page treats that as part of the offer.
- The FINA `.xls` step is the one part that is not pure Rust.
