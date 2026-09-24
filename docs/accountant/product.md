# What this is for other companies

Written as the pitch it would have to survive, with the conditions that must hold before
anyone is charged for it.

## The offer

Books that write themselves from the bank feed and the invoices, in the chart of
accounts the accountant already uses, with every statutory statement computed and every
posting traceable to its source, and a replay that proves the result against a filed
year. The accountant reviews, declares the judgements, and signs; the company sees its
real position every day instead of once a year in April.

Who it is for first: Croatian micro and small d.o.o. with a handful of customers,
foreign revenue, one or two employees, and an owner who wants to see the books. INORBIT
is the template: IT services exported to the EU and the US, VAT registered, one salary,
a member loan, a yearly dividend. There are tens of thousands of such companies, and
from 1 January 2026 every one of them must receive and issue e-invoices, so each is
choosing software this year anyway.

Who pays: the company (subscription per party), or the accounting firm (per client,
white-labelled) which gets a client whose books arrive posted, reconciled and checked
against FINA's own rules before the firm opens them.

## Why it would be believed

- **Replay-proven.** The chaos family `year_replay` reproduces a real filed year to the
  cent, account by account, and runs on every change. A prospect's accountant can be
  shown their own prior year replayed before they trust the current one.
- **Provenance on every number.** A statement position opens to its accounts, an
  account to its journal lines, a line to the bank row or the document it came from,
  with the rule and rule version that produced it. Nothing is typed in.
- **Judgements are declared, not hidden.** The loan to the member, the depreciation
  rate, the distribution, the prior-year invoice are first-class declarations with a
  who and a when, which is what an inspector asks for.
- **Deterministic documents.** Invoice PDFs, statements and exports are pure functions
  of their inputs and hash-checked, as the invoicing already is.
- **The accountant stays the accountant.** The product never files; it prepares. The
  signature and the liability remain with a licensed person, which is also what keeps
  accounting firms as a channel rather than a competitor.

## What it would cost to build versus buy

Croatian market software (Synesis, Pantheon, Minimax, e-računi, Saop) is a ledger with a
data-entry UI; the bank feed, the receipt reading and the reconciliation are afterthoughts
or absent, and none replays a year against a filing. Our position is the reverse: the
feeds and the reconciliation exist and are audited on real data; the ledger is what is
being added. The moat is the replay and the provenance, not the ledger.

## Conditions before selling

1. **Two filed years replayed exactly** for two different companies, one of them not
   ours, with their accountant's written agreement that the replay matched.
2. **Rule packs reviewed yearly** by a named accountant under contract (chart, AOP map,
   PD rules, contribution rates, prescribed interest, depreciation rates), with the
   review recorded as a version in the pack.
3. **Multi-tenancy proven** by the existing access scenarios (`finance_access`) extended
   to the books: a firm's user sees only their clients' ledgers; the owner sees only
   their own.
4. **eRačun in and out** live, since from 2026 a product without it is not usable.
5. **Retention and erasure** per the Accounting Act (eleven years for books) reconciled
   with the GDPR erasure the ledger crate already implements for the humans plane.
6. **Terms** that say what the product does (prepares) and does not (file, advise), a
   professional-liability policy, and the NDA in `docs/nda/` generalised for
   accountant onboarding.
7. **Price** set from the accountant's own invoice: this company paid 2.610,00 in 2025
   for bookkeeping (account 4164). The product plus a reviewing accountant must land
   below that for a micro company, or above it with a visibly better result.

## Sequence

INORBIT's 2025 first (this plan), 2026 live with e-invoicing, then one friendly
accounting firm with three clients on the white-label, then the public offer.
