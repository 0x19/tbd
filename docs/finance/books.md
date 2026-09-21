# Books: the general ledger of a company party

The finance service keeps the books of a company: a chart of accounts, fiscal periods,
a journal of debit and credit lines, the opening balances a year starts from, and the
trial balance that follows. This is phase 1 of `docs/accountant/plan.md`; postings
derived from bank transactions, invoices and documents, the statements and the tax
return follow in later phases. `/books/` is the page.

## What the ledger is

- **Money is integer minor units** in the books' currency, EUR. Nothing is rounded on the
  way in; a foreign document is valued into EUR on its line and carries the original
  beside it (`original_minor`, `original_currency`).
- **The chart is data.** `configs/finance/hr/chart-rrif-2025.toml` holds the RRiF-numbered
  accounts the company's accountant used in 2025 (the 77 of `docs/accountant/gfi-2025.md`
  §1) and the synthetic accounts they sit under. It is compiled into the service and
  seeded into a company's chart on its first books call. Class and kind come from the
  first digit (0, 1, 3 asset; 2 liability; 4 expense; 7 revenue; 8 result; 9 equity). A
  company adds an analytic account of its own with `UpsertLedgerAccount`; its parent is
  the longest existing code that is a prefix, and a code with nothing above it is
  refused. A synthetic account takes no posting.
- **Periods** are the twelve months of a calendar fiscal year, each `open`, `closed` or
  `locked`. `ClosePeriod` closes a month (`reopen` opens it again); `LockPeriod` is final.
  A month that is not open refuses a posting; a year with any month not open refuses an
  opening (re)import.
- **Postings are derived, never typed.** A journal entry names its source (`opening`,
  `bank`, `invoice`, `document`, `payroll`, `judgement`, `closing`, or `chaos` for the
  tool's own) and the rule and rule version that produced it; the same key twice is a
  conflict, not a second entry. A line has exactly one positive side. An entry balances
  or does not land: the code checks it and names both sums, and a deferred constraint
  trigger (`finance.assert_entry_balanced`) checks it again at commit for anything that
  writes the table directly. There is no RPC to post an entry; phase 2 brings the rules.
- **The opening is kept as handed over.** `ImportOpeningBalances` takes a year's opening
  trial balance with both sides per account and *signed* amounts, at the date it stands
  (`as_of`: 1 January, or the hand-over date when the books changed hands mid-year), and
  replaces an earlier import of the same year in one transaction. The set must balance
  to the cent and every code must be an analytic account of the chart. Signed and
  two-sided on purpose: the column an accountant hands over at a mid-year change carries
  year-to-date P&L balances, accounts with both sides (2200: 3.988,71 / 399,54) and
  reversals booked as negative postings (2211: a credit of −96,27), and only reproducing
  it as filed makes the year's totals independent of where the import date falls
  (`docs/accountant/findings.md` §1). Openings are therefore their own table, not journal
  lines.
- **The trial balance** (`TrialBalance`, a year through a month) lists every account with
  an opening row or a posting in code order, with opening, movement, totals and balance
  (total debit − total credit; negative is a credit balance), class totals, the grand
  totals and `balanced`. The 2025 hand-over column (`docs/accountant/expected/
  opening-2025.csv`) imported at 2025-07-01 comes back as 77 rows, 184.988,01 on both
  sides, class by class equal to `gfi-2025.md` §1; the finance crate's tests hold that.

## Who may do what

Books belong to a company party; a person's party is refused. Anyone the party is
granted to may read the chart, the periods and the trial balance. Importing an opening,
changing the chart and closing or locking a month need `own` on the party; a reader gets
`PERMISSION_DENIED`. A party outside the grant reads as `NOT_FOUND`, as everywhere in
the service.

## On the wire

| RPC | REST | Notes |
|---|---|---|
| `ListLedgerAccounts` | `GET /v1/finance/books/accounts?party_id=` | seeds the chart on first call |
| `UpsertLedgerAccount` | `POST /v1/finance/books/accounts` | 4 to 6 digits, parent by prefix |
| `ListPeriods` | `GET /v1/finance/books/periods?party_id=&fiscal_year=` | twelve months, `changed_at` empty for a month never touched |
| `ClosePeriod`, `LockPeriod` | `POST /v1/finance/books/periods/close`, `.../lock` | `FAILED_PRECONDITION` on a locked month |
| `ImportOpeningBalances` | `POST /v1/finance/books/opening` | `INVALID_ARGUMENT` names the sums or the code; `FAILED_PRECONDITION` names the month |
| `TrialBalance` | `GET /v1/finance/books/trial-balance?party_id=&fiscal_year=&through_month=` | `fiscal_year` 0 is the current year, `through_month` 0 the whole year |

Amounts are `int64` minor units, rendered as JSON strings by the transcoder. Dates are
`YYYY-MM-DD`.

## Not done

Posting rules and `PostPeriod` (phase 2), the statements and the PD return (phase 3),
exports (phase 4), FX rates, a fiscal year that is not the calendar year, deleting an
account, and a Postgres for the chaos scenarios in CI (`docs/accountant/chaos.md`).
