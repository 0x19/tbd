# Inputs available to replay 2025

The filings are the expected output. To replay the year the service needs the inputs the
accountant had. This is what exists on the server under `~/finance-data/` as of
2026-09-21, and what is missing.

## Issued invoices (revenue side)

`~/finance-data/Firma/` holds the company's own invoices as PDF (and ODT sources),
numbered `ordinal-1-1-year`:

| Number | Date | Total EUR | Note |
|---|---|---:|---|
| 1-1-1-2025 | 31.01.2025 | 16.470,89 | the file also shows the HRK equivalent |
| 2-1-1-2025 | 28.02.2025 | 9.882,00 | |
| 3-1-1-2025 | 31.03.2025 | 9.167,00 | |
| 4-1-1-2025 | 11.04.2025 | 4.801,76 | |
| 1-1-1-2024 to 14-1-1-2024 | 2024 | 9.167,00 each from March | the 2024 series, present for comparatives |

Only invoices 1 to 4 of 2025 are in the folder (40.321,65 of the 173.644,37 filed).
Invoices 5 to 14 of 2025 must come from the finance service's own invoicing (the
`finance.invoices` table) or from the customers' portals. Revenue by customer residence
(35.519,89 EU, 138.124,48 third countries) has to be reproduced from the client
records. USD-denominated invoices need the exchange rate of the invoice date (HNB
middle rate) to reproduce 7724/4750.

Also in the folder: `inorbit-03-27-2025-proforma-1.pdf`, `inorbit-04-03-2025-4-1-1-tenderly.pdf`
(a client-specific rendering), `putni_nalog_1.*` (a travel order, the basis for 4607),
`Scanned Document.pdf`, `Eiger/`, `Tenderly/` (client folders), `a1_ht_racuni/`
(telecom invoices behind 4100).

## Supplier documents (expense side)

Monthly folders `~/finance-data/04-2025` to `12-2025` (and `Firma/01_2025` to
`04_2025`) hold what was sent to the accountant each month: Hetzner, Adobe, Namecheap,
Apple/Google receipts (`Receipt-*.pdf`), Medium, Paddle, HRT, insurance, travel tickets,
WhatsApp photos of paper receipts, plus the accountant's own items (`INORBIT_102-KPNS-01.pdf`,
`K.P.N.service_otvorene stavke INORBIT 14.05.2025..pdf`, `Faktura_000002696_38846238650_1-2.pdf`).
`01-2026` to `08-2026` continue the series. The finance service already ingests the
same documents from the mailbox connector, so the golden set can be built from
`finance.documents` rather than from these folders, with the folders as a cross-check.

## Bank statements

Erste statements exist as HTML exports for 2024 (`Firma/*/IZV_*.html`, `izvodi.html`)
and one for June 2025 (`IZV_2025_06_09 ...html`). The 2025 movements are otherwise in
the finance service's `finance.bank_transactions` (imported through Enable Banking; the
importer was audited against 2.861 Erste rows) and in `prototype/bank/data/raw/*.json`.
The trial balance says the account had 111.446,90 of credits and 90.498,88 of debits in
the second half plus an opening of 7.261,68, and the 31.12.2024 balance was 4.699,41:
the replay must hit 28.209,70 at year end.

Cash: 1.950,00 withdrawn and spent; the cash-box entries need the receipts behind them.

## Payroll

No payroll documents are in the folders. The monthly JOPPD and payslips are with the
accountant; the annual totals are known from the trial balance (findings.md §6) and
suffice to replay 2025 as twelve identical months. For 2026 the service needs the
JOPPD input.

## Fixed assets

No register is in the folders. The trial balance gives cost by group (03100 1.724,22,
03110 26.370,80, 03120 921,31, 0311 1.199,99, 0371 1.407,28) and the year's
depreciation; the per-asset list, dates and rates are a question for the accountant.

## Opening balances at 1.1.2025

The filed 31.12.2024 balance sheet gives the opening per AOP position; the trial balance
gives the migration-date balances per account. The account-level opening at 1.1.2025
must be derived from both (findings.md §1), or obtained as the previous accountant's
closing trial balance.

## What the golden dataset therefore consists of

1. Opening trial balance at 1.1.2025 per account (derived or obtained).
2. Every 2025 bank transaction with its classification.
3. Every issued invoice (14) with customer residence and currency.
4. Every supplier document with net, VAT treatment and account.
5. Twelve payroll months.
6. The asset register with rates.
7. The accountant's judgement entries as a list (member loan and interest, catch-up
   depreciation, distribution decision, prior-year costs, small-inventory write-off,
   tax accrual and netting).
8. Expected outputs: the trial balance, POD-BIL, POD-RDG, POD-DOP and PD figures in
   gfi-2025.md, as machine-readable fixtures.

Items 2 to 4 come out of the finance service's database and the mailbox; items 1, 5, 6
and 7 need the accountant's cooperation once (the NDA in `docs/nda/` covers that
exchange).
