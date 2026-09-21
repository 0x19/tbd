# From the trial balance to every filed position

This is the derivation the service must implement. Every line was recomputed from the
77 trial-balance rows with a script and compared to the filed forms; "OK" means equal
to the cent. Signs: a balance is debit minus credit; liabilities, equity and revenue are
negated when they feed a form.

## 1. Balance sheet

| AOP | Position | Accounts (balances summed) | Computed | Filed |
|---|---|---|---:|---:|
| 013 | Postrojenja i oprema | 031x + 039x | 1.049,99 | OK |
| 017 | Materijalna imovina u pripremi | 037x | 1.407,28 | OK |
| 049 | Potraživanja od kupaca | 121x | 2.464,26 | OK |
| 051 | Potraživanja od države i drugih institucija | 140x + 141x + 142x + 145x (**not** 143x, see §3) | 6.237,36 | OK |
| 052 | Ostala potraživanja | 124x + 125x | 1.735,28 | OK |
| 061 | Dani zajmovi, depoziti i slično | 115x | 97.547,31 | OK |
| 063 | Novac u banci i blagajni | 100x + 102x | 28.209,70 | OK |
| 065 | Ukupno aktiva | classes 0 + 1 + 3, less 1430 | 138.651,18 | OK |
| 068 | Temeljni kapital | −900x | 2.654,46 | OK |
| 085 | Zadržana dobit | −940x | 0,00 | OK |
| 088 | Dobit poslovne godine | RDG 186 | 109.260,07 | OK |
| 118 | Obveze prema dobavljačima | −(220x + 221x) | 983,98 | OK |
| 120 | Obveze prema zaposlenima | −230x | 1.075,61 | OK |
| 121 | Obveze za poreze, doprinose i slična davanja | −(240x + 241x + 242x + 243x) + tax payable (§3) | 2.812,91 + 4.689,04 = 7.501,95 | OK |
| 122 | Obveze s osnove udjela u rezultatu | −201x | 16.632,27 | OK |
| 124 | Ostale kratkoročne obveze | −239x | 542,84 | OK |
| 126 | Ukupno pasiva | 067 + 110 | 138.651,18 | OK |

Subtotals are sums of their children as printed on the form (002 = 003+010+020+031+036
and so on); the FINA workbook computes them itself.

## 2. Profit and loss

| AOP | Position | Accounts | Computed | Filed |
|---|---|---|---:|---:|
| 130 | Prihodi od prodaje | −(751x + 754x) | 173.644,37 | OK |
| 137 | Troškovi sirovina i materijala | 401x + 404x + 405x | 7.152,84 | OK |
| 139 | Ostali vanjski troškovi | 410x + 416x + 417x + 419x | 8.498,07 | OK |
| 141 | Neto plaće i dnevnice | 420x | 12.907,20 | OK |
| 142 | Taxes and contributions from salaries | 421x + 422x | 5.010,36 | OK |
| 143 | Doprinosi na plaće | 423x | 2.956,44 | OK |
| 144 | Amortizacija | 431x | 12.821,80 | OK |
| 145 | Ostali troškovi | 460x + 461x + 464x + 465x + 468x + 469x | 6.023,15 | OK |
| 156 | Ostali poslovni rashodi | 485x + 486x | 736,46 | OK |
| 164 | Prihodi s osnove kamata | −771x | 1.721,40 | OK |
| 165 | Tečajne razlike i ostali financijski prihodi | −772x | 2.367,70 | OK |
| 171 | Rashodi s osnove kamata | 474x | 26,25 | OK |
| 172 | Tečajne razlike i drugi rashodi | 475x | 146,24 | OK |
| 175 | Ostali financijski rashodi | 479x | 53,94 | OK |
| 180 | Ukupni prihodi | −class 7 | 177.733,47 | OK |
| 181 | Ukupni rashodi | class 4 | 56.332,75 | OK |
| 182 | Dobit prije oporezivanja | 180 − 181 | 121.400,72 | OK |
| 185 | Porez na dobit | PD row 55 | 12.140,65 | OK |
| 186 | Dobit razdoblja | 182 − 185 | 109.260,07 | OK |

The grouping is by account group (first three digits), which is the RRiF convention
FINA's instructions assume. Two groups need the fourth digit: 474 splits into 4740
(commercial late interest) and 4741 (late interest on taxes), both in AOP 171 but only
4741 in PD row 9; 486 (donations) is AOP 156 but also PD section X.

## 3. The two closing entries that are not in the printed trial balance

The trial balance balances at zero with no income-tax expense and no result account, so
it was printed before closing. The filed statements include two entries made afterwards:

1. **Income tax accrual.** Dr 8xx income tax expense 12.140,65 / Cr 2400 tax payable
   12.140,65. This gives RDG 185 and reduces the result to 109.260,07.
2. **Netting of prepayments.** Dr 2400 / Cr 1430 for 7.451,61, so the balance sheet
   shows one liability of 4.689,04 in AOP 121 and nothing in AOP 051 for account 1430.
   Without this netting AOP 051 would be 13.688,97 and total assets 146.102,79.

The service must therefore compute the tax return before the balance sheet, and present
the prepaid tax net of the liability (or the liability net of prepayments when
prepayments exceed it: a receivable in AOP 051).

## 4. Tax return

| PD row | Source | Value |
|---|---|---:|
| 1 | RDG 180 | 177.733,47 |
| 2 | RDG 181 | 56.332,75 |
| 3 | row 1 − row 2 | 121.400,72 |
| 9 | account 4741 | 5,82 |
| 26 | sum of rows 5 to 25 | 5,82 |
| 36 | row 3 + row 26 − row 35 | 121.406,54 |
| 43 | 10 % because row 1 < 1.000.000,00 | |
| 44 | row 42 × 10 %, rounded half up to the cent | 12.140,65 |
| 56 | balance of 1430 | 7.451,61 |
| 57 | row 55 − row 56 | 4.689,04 |
| 59 | row 55 / 12, rounded to the cent | 1.011,72 |
| X.1 | account 4860 | 500,00 |
| X.1.1 | X.1 / prior-year RDG 180 × 100, two decimals | 0,43 |
| X.1.2 | X.1 / RDG 180 × 100, two decimals | 0,28 |

Rows not used this year but needed in general: 6 (50% of 4xx representation), 7 (private
car), 18 (depreciation above prescribed rates), 20 to 23 (write-downs and provisions),
25 (other increases including hidden distributions), 37 (loss carried forward), 45 to
52 (reliefs).

## 5. Additional data

| AOP | Source |
|---|---|
| 231 | 11506 (loans to households: the director is a natural person) |
| 232 | 115091 |
| 252 | AOP 130, all revenue is NKD 62 |
| 255 | AOP 130, every customer is abroad (75402 EU + 75411 third countries) |
| 274, 275 | 4649 |
| 278 | 4686 |
| 279 | 4199 (filed 2.597,45 against a balance of 2.597,48; a 0,03 difference to raise with the accountant) |
| 285 | credit turnover of 2210 + 2211 |
| 286 | AOP 164 |
| 289 | AOP 171 |
| 290, 292, 296 | debit turnover of 0371 (acquisitions in the year) |

## 6. Consistency rules to carry as checks

From FINA's Kont sheet and the arithmetic above:

- Σ debit = Σ credit over the whole ledger, and per journal entry.
- AOP 065 = AOP 126 within 0,14.
- BIL 088 = RDG 187; BIL 067 = 068 + 084 + 087 for a micro entity.
- Retained earnings roll-forward: 085 (t) = 085 (t−1) + 088 (t−1) − distributions declared in t. For 2025: 21.012,23 + 65.047,18 − 86.059,41 = 0,00, so the whole 2024 result plus prior retained earnings was distributed (see findings on the 23.977,27 not visible in the trial balance).
- PD row 1 = RDG 180, row 2 = RDG 181, RDG 185 = PD row 55.
- AOP 254 + 255 = AOP 129 + 130; AOP 231 + 232 + 233 ≤ AOP 061 (+ others listed in control 91).
- Dividend tax: 24311 credits = 12 % of the gross distribution (62.082,14 × 12 % = 7.449,86; filed 7.449,87 from rounding per payout).
