# docs/accountant

The accountant's 2025 annual filings for INORBIT d.o.o., taken apart line by line, and
the plan to make the finance service produce the same filings itself, proven by chaos
against these documents as the source of truth.

Source files (not in git, personal financial data): `~/finance-data/gfi-2025/` on the
server, copied from the accountant's package on 2026-09-21. Everything quoted here was
extracted from those files; nothing is estimated.

| Page | What it holds |
|---|---|
| [gfi-2025.md](gfi-2025.md) | Every figure in the six documents: the trial balance per account, the balance sheet and P&L per AOP position, the additional data, the tax return, the FINA receipt and the reference page of the FINA workbook. |
| [mapping.md](mapping.md) | How the trial balance becomes each AOP position and each tax-return row, verified to the cent, including the two closing entries the accountant made outside the printed trial balance. |
| [findings.md](findings.md) | What the accountant actually did in 2025 beyond posting invoices: the mid-year migration, the owner's private spending turned into a loan with interest, the catch-up depreciation, the dividend and its 12% tax, the pre-closing trial balance. Each is a behaviour the service must support. |
| [inputs.md](inputs.md) | The input documents that exist for 2025 (issued invoices, supplier invoices, bank statements) and what is missing to replay the year. |
| [rules.md](rules.md) | Croatian rules the numbers obey, with primary sources: chart of accounts, GFI-POD, corporate tax, loans to members, depreciation, payroll, VAT, levies, 2026 changes. |
| [system.md](system.md) | What the finance service, ledger and chaos tool can do today and the gaps to a double-entry ledger with statutory reporting. |
| [plan.md](plan.md) | The architecture and the phased plan: ledger core, posting rules, statements, tax return, exports (GFI-POD xls, PDF), dashboard, chaos replay. |
| [chaos.md](chaos.md) | The chaos scenario families that replay 2025 and compare every position to the filings, and how the tool is extended to carry them. |
| [email.md](email.md) | The email to the accountant asking only for what nobody else holds, and the list of what we fetch ourselves from ePorezna and FINA first. |
| [product.md](product.md) | What this becomes for other companies and their accountants, and what has to be true before it is sold. |

Conventions: amounts are EUR with two decimals as filed; account numbers are the
accountant's (RRiF-style chart, classes 0 to 9); AOP is the position code on the FINA
forms. "PS" in an account name is the accountant's suffix for balances imported at the
software migration (see findings).
