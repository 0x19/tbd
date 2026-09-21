# docs/nda

The confidentiality agreement the accountant signs before receiving the finance
data: `nda.hr.md` (Croatian, the governing version) and `nda.en.md` (English, a
translation kept clause for clause with it).

What it covers, and why it is not a generic NDA:

- **Two disclosing parties.** The finance system holds the company's books and the
  director's personal accounts side by side, and the accountant will see both. The
  director therefore signs in his own name too, and the personal financial data is
  named as Confidential Information outright, with a clause (2.4) forbidding its use
  for anything but the services and a clause (3.5) keeping it out of the company's
  filings unless the law or the director asks.
- **Statutory disclosure is allowed.** An accountant files with the Tax
  Administration, FINA and the insurance institutes; clause 2.2.b) permits exactly
  that, so the agreement does not put them in breach by doing their job.
- **GDPR processor terms** (Article 3) in the shape of Article 28 GDPR, so a separate
  data processing agreement is optional.
- **Credentials.** Clause 2.3.a) makes an account in the finance UI personal and asks
  for multi-factor authentication, which matches how delegated readers are granted.
- **Term**: five years after the engagement, unlimited for personal data and trade
  secrets.
- **Contractual penalty** (7.3): EUR 10,000 per breach of the confidentiality, data
  protection or return obligations, plus any damage above that (Obligations Act,
  art. 355). The figure is the one that recurs in European NDA practice (the German
  federal BMWK portal reports about EUR 5,000 as the everyday figure and up to
  EUR 25,000 for larger counterparties; Croatian sample clauses use EUR 10,000), and
  it sits well inside what a court leaves untouched under art. 354, which reduces a
  penalty "disproportionate to the value and significance of the object of the
  obligation" on the debtor's request. A six-figure penalty against a small
  bookkeeping firm for a micro company's books would be cut down on request, and the
  accountant would likely refuse to sign it; the damages route in 7.1 and 7.3 covers
  the large-loss case anyway. Sources: zakon.hr (ZOO art. 350-356),
  existenzgruendungsportal.de (BMWK), katastor.hr, ipdraughts.wordpress.com.

The company data (full name, seat, MBS, court) is filled from the court register as
mirrored by companywall.hr and fininfo.hr on 2026-09-21; check the MBS against the
company's own registration decision before signing.

Before sending:

1. Fill the remaining blanks: the director's private address and OIB, the
   accountant's details, and the date; change the place of signing if it is not
   Viškovo.
2. Have a Croatian lawyer read it once. It was drafted with care from the Croatian
   Obligations Act, the GDPR and the trade-secrets act in mind, but it is not legal
   advice, and the accountant may have their own engagement terms to reconcile.
3. Render to PDF: `mise run nda:pdf` writes `target/nda/nda.hr.pdf` and
   `target/nda/nda.en.pdf` (A4, numbered pages) through `docs/nda/render.mjs`, which
   uses the Playwright Chromium the finances UI's e2e tests already install and
   fetches `marked` once through npx. `NDA_OUT=<dir>` writes somewhere else. The
   PDFs are build output and are not committed.

Both versions are signed; the Croatian one prevails in a dispute (8.2).
