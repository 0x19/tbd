# What the accountant did in 2025

Everything below is read off the trial balance and the forms; where an interpretation is
made it is marked as one. Each item ends with what the service has to be able to do.
Items to confirm with the accountant are collected at the end.

## 1. The books changed hands or software on 1 July 2025

Every P&L account with activity in both halves of the year shows the same amount in the
"opening" column and the "period" column: 4200 net salaries 6.453,60 + 6.453,60, 4210
1.426,80 as 713,40 + 713,40, 4220 and 4230 likewise, 7518 revenue 75.947,47 in the opening
column. P&L accounts have no opening balances in a normal ledger. The opening column is
the first half of 2025 imported as balances into the new software ("-PS" accounts:
03100, 03110, 03120, 03910), and the period column is the second half posted natively.

Consequences visible in the numbers:

- 7518 "Prihodi od programskih usluga" (the previous chart) carries the H1 revenue
  75.947,47 and is reversed with a negative credit of −75.947,47, then the full year is
  re-posted on 75402 (EU, 35.519,89) and 75411 (third countries, 138.124,48). The
  reversal is booked as a negative credit, not a debit, so the "total" columns of that
  account are 0,00/0,00 rather than 75.947,47/75.947,47.
- 1331 "private expenses of members" 72.515,51 is reversed the same way (negative
  debit of −72.515,51) when it is reclassified to 11506.
- The opening column of balance-sheet accounts is the balance at the migration date, not
  31.12.2024: 1000 bank 7.261,68 versus 4.699,41 filed for 31.12.2024; 1331 72.515,51
  versus AOP 050 60.311,04; 94010 62.082,14 versus 21.012,23 + 65.047,18 = 86.059,41.
- Only the "Ukupno" and "Saldo" columns are authoritative. The split is an artefact.

What the service must support: importing an opening trial balance at an arbitrary date
(including P&L year-to-date balances), negative postings on either side, and reporting
year totals that do not depend on where the import date falls.

## 2. The owner's spending from the company account became a loan with interest

At migration the company held a receivable of 72.515,51 from the member for private
costs (1331). In the second half that receivable was reversed and 97.493,01 was posted
to 11506 "Zajmovi članovima uprave i zaposlenicima", 105,33 was repaid, leaving a loan of
97.387,68 at year end. Interest of 1.721,40 was accrued (Dr 1243 / Cr 7710) and is
unpaid at year end. The balance sheet shows the loan as short-term financial assets
(AOP 061) and the additional data as a loan to households (AOP 231), whereas at
31.12.2024 the same kind of balance sat in AOP 050 "receivables from employees and
members".

Interpretation: 97.493,01 − 72.515,51 = 24.977,50 of further private spending in 2025
went into the loan; 1.721,40 on an average balance of that size is a rate of about 2%,
which is the floor in the income-tax act for loans to an employee (below it the
difference is salary in kind; rules.md §4). The 4,38% rate the corporate-tax act
prescribes for 2025 applies to loans between related entities, not to this one. The
company account paid for both business and private items through the year, which is
also why the reconciliation policy in the finance service has a `personal` need.

What the service must support: a member-loan sub-ledger (drawdowns from bank payments
marked personal, repayments, prescribed-rate interest accrual with day count, year-end
receivable for interest), the reclassification between AOP 050 and AOP 061, and the tax
consequence if interest is not charged (hidden distribution, PD row 25).

## 3. Equipment was depreciated to zero

At 31.12.2024 equipment stood at 9.745,73 net after 631,57 of depreciation for the whole
of 2024. In 2025 depreciation is 12.821,80: 9.422,54 in the imported half plus 3.399,26
in the second half, of which 3.249,26 finishes the imported assets (cost 29.016,33,
accumulated 29.016,33, net 0,00) and 150,00 starts the new computer bought in the year
(0311, cost 1.199,99; 150,00 is three months at 50% a year). A further 1.407,28 sits in
0371 "equipment in preparation" (2.607,27 acquired, 1.199,99 put into use) and is not
depreciated. Small inventory 402,81 (3500 → 3600 → 3630) was expensed in full on 4040.

Interpretation: the new accountant applied the prescribed 50% rate for computers to the
whole remaining book value, twenty times the previous year's charge. This is the single
largest expense of the year and is entirely a judgement, not a transaction.

What the service must support: a fixed-asset register with acquisition date, cost, rate,
monthly straight-line depreciation with a start-of-next-month convention, assets in
preparation, small-inventory write-off, and a catch-up depreciation entry.

## 4. The whole 2024 result was distributed, in two steps

Equity at 31.12.2024 was 2.654,46 + 21.012,23 + 65.047,18 = 88.713,87. At year end 2025
retained earnings are 0,00. The trial balance shows one distribution: gross 62.082,14
(Dr 94010), of which 7.449,87 withholding tax on capital income (Cr 24311, 12%) and
54.632,27 net to the member (Cr 2010); 38.000,00 of the net and 5.181,83 of the tax were
paid in the year, leaving 16.632,27 (AOP 122) and 2.268,04 (in AOP 121) owed. The
remaining 86.059,41 − 62.082,14 = 23.977,27 does not appear in this trial balance at
all: it was distributed before the migration and is netted into the imported opening
balances.

What the service must support: a profit-distribution decision as a journal entry
(gross, 12% tax, net), payment tracking against it, the JOPPD-side report of the tax,
and the equity roll-forward check that catches distributions hidden in an import.

## 5. The trial balance is pre-closing

No income tax expense, no tax payable and no result account appear; the forms include
them (mapping.md §3). The tax is 10% of 121.406,54 = 12.140,65; prepayments of 7.451,61
(1430) are netted, leaving 4.689,04 to pay, and next year's monthly advance is
1.011,72. A separate 542,84 on 2399 is a prepayment for the next year booked as a
liability.

What the service must support: the PD computation as a first-class step of year-end
close, producing the accrual entry and the netting entry, then the statements.

## 6. Payroll for one part-year-looking salary

Net salaries 12.907,20 a year (1.075,61 a month, the same figure as the unpaid December
salary in AOP 120), income tax 1.426,80, contributions from salary 3.583,56 (pension
pillar I 2.687,64, pillar II 895,92), contributions on salary 2.956,44 (health 16,5%).
Gross is therefore 17.917,56 a year, 1.493,13 a month. The year-end liabilities on 2420,
2421, 2423 are one month's contributions; 2410 income tax is settled. A refund
receivable of 176,10 on 1410 and 0,05 on 1424 exist.

What the service must support: a payroll journal from a JOPPD-shaped input (gross, tax,
each contribution, net), the monthly liability and payment matching, and the RDG split
into AOP 141, 142, 143.

## 7. VAT is a flow-through, not a cost

All revenue is to foreign business customers, so no output VAT. Input VAT on domestic
invoices (140012, 1.757,75), reverse-charged EU services (14032/24032, 138,69 and
171,45), reverse-charged third-country services (14042/24042, 522,04) and import VAT
(14052, 105,33) all net to zero within the year; the surplus input VAT claimed back
accumulates on 1407 (5.803,59 at year end, of which 2.789,36 arose in 2025). The
difference between 14032 (138,69) and 24032 (171,45) means some EU services were
self-assessed without a matching deduction (interpretation: a car-related or
representation item with restricted deduction, or a rounding of periods).

What the service must support: input VAT split on every supplier document, reverse
charge with self-assessment and deduction, monthly PDV return figures, and the refund
receivable.

## 8. Other judgements

- 4850 "costs from previous years" 236,46: invoices dated 2024 booked in 2025 as a
  separate expense line so they can be excluded from the year's comparisons.
- 4741 late interest on taxes 5,82 is the only non-deductible item, added back on PD
  row 9.
- 4860 donation 500,00 is within 2% of prior-year revenue, so fully deductible (PD
  section X).
- 4616 "prigodne nagrade" 1.900,00 is a non-taxable bonus to the employee within the
  yearly limit, booked in the first half.
- Foreign-currency purchases produced 146,24 of exchange losses (4750) and other
  financial income of 2.367,70 (7724), interpretation: exchange gains on USD receipts
  and card settlements; the mapping to AOP 165 confirms they are FX gains.
- Cash: 1.950,00 was withdrawn through the transit account (1009) into the cash box
  (1020) and spent the same year; the cash box is empty at year end.
- Tourist board membership fee: 192,31 was paid in 2025 and 257,62 sits on 1450 as a
  receivable. NKD 62 has not been liable for this fee since 2020 (rules.md §8), so the
  balance is an overpayment to reclaim, and the 2025 payment should not have been made.
- Additional data AOP 278 "compensation to management" reports 2.878,75, which is
  account 4686. In the accountant's ledger 4686 is named "software licences", but in
  the RRiF chart 4686 is "compensation to external board members" and the AOP map
  follows the number, not the name (rules.md §1). The same collision puts the HRT fee
  (4684) under "other rights of use". The filed figure is wrong by naming, not by
  amount; the service maps by account meaning and reports 4686 under AOP 145 only.
- Deposits 79,63 and a loan to a natural person 80,00 carried from earlier years.

## Questions for the accountant

1. Confirm the migration date (1 July 2025?) and whether the previous accountant's
   closing trial balance exists; the 23.977,27 distribution and the 2024 versus opening
   differences (bank 7.261,68 vs 4.699,41; equipment accumulated depreciation) should
   reconcile to it.
2. Loan to the member: the interest rate used, the day count, and whether a loan
   agreement with a repayment schedule exists (it matters for the hidden-distribution
   test and for 2026 interest).
3. Depreciation: the rate and start date per asset for the 2025 charge of 12.821,80,
   and the list of assets behind 0371 (1.407,28 not yet in use).
4. 14032 vs 24032: which EU service had a restricted input-VAT deduction.
5. AOP 279: 2.597,45 filed vs 2.597,48 on 4199; AOP 278: why 4686 (software) was
   reported as management compensation; and the tourist-board fee paid for an
   activity that is exempt since 2020.
6. The 2024 comparatives (AOP 133 other operating income 4.095,52, AOP 156 3.932,29,
   AOP 279 4.140,00): which accounts, so the 2024 opening for a full replay can be
   rebuilt.
7. Copies of the notes to the statements and the two decisions as filed, to reproduce
   their content.
