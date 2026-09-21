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
  secrets. The contractual penalty (7.3) is optional; strike it or fill the amount.

Before sending:

1. Fill the blanks: the company's registered address, the director's address and
   OIB, the accountant's details, the court's seat (normally where the company sits),
   the penalty or its deletion, place and date.
2. Have a Croatian lawyer read it once. It was drafted with care from the Croatian
   Obligations Act, the GDPR and the trade-secrets act in mind, but it is not legal
   advice, and the accountant may have their own engagement terms to reconcile.
3. Render to PDF if the accountant wants one. There is no converter in this tree;
   `pandoc nda.hr.md -o nda.hr.pdf` works where pandoc and a PDF engine are installed,
   and any Markdown editor prints it.

Both versions are signed; the Croatian one prevails in a dispute (8.2).
