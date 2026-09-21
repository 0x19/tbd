# Moj-eRačun: the company's incoming e-invoices as receipts

Domestic suppliers deliver e-invoices through an information intermediary since January
2026. The company's intermediary is Moj-eRačun (`moj-eracun.hr`). The `mojeracun`
connector kind pulls what arrived there into the same receipts ledger the mailboxes
fill (`/documents/`), with supplier, number, date and total read from the e-invoice
itself and written as facts, not guessed from a PDF.

## What is pulled, and how it becomes a receipt

Moj-eRačun's API (`https://www.moj-eracun.hr/apis/v2`, the "API V2" reference) is
documented with its request and response bodies, which is what makes this the certain
source. A pull:

1. `POST queryInbox` for status `30` (sent to the company) and `40` (delivered), over a
   window reaching seven days back past the last pull (`From`/`To`); each row carries
   `ElectronicId`, `DocumentNr`, `SenderBusinessName`, `Sent`, `Delivered`.
2. `POST receive` per `ElectronicId` not stored before: the document as the **UBL 2.1
   XML** it was delivered as.
3. The XML is read structurally: `AccountingSupplierParty` (the legal name), `ID`,
   `IssueDate`, `LegalMonetaryTotal/PayableAmount` with its currency. A Croatian
   e-invoice usually embeds its visual PDF (`AdditionalDocumentReference/Attachment/
   EmbeddedDocumentBinaryObject`, `mimeCode application/pdf`): that PDF becomes the
   document a person opens. Without one, the XML itself is stored as the document, and
   the facts still stand.

At most 50 documents a round (the run is `partial` and the next continues); one request
a second. Nothing is written back: the import is **not** confirmed to Moj-eRačun
(`notifyimport`), so the accountant's own software still sees the invoice as new, and
nothing is marked paid, rejected or archived from here.

On `/documents/` such a receipt shows `from the intermediary` beside its vendor, number,
date and amount; the reader still runs over the PDF and fills only what the UBL did not
give, and a re-read never overrules a fact. A person's correction still outranks it.

## Linking

`/connectors/` → Link a mailbox → **Moj-eRačun** → Continue → the values:

| Field | What it is |
|---|---|
| API user | the company's Moj-eRačun login (`Username`) |
| Password | its password |
| Company OIB | the company's OIB (`CompanyId`) |
| Business unit | only if one is registered at Moj-eRačun (`CompanyBu`); otherwise leave empty |
| SoftwareId | the identifier Moj-eRačun's integration department issues to the integrator |

The `SoftwareId` is the one thing the company does not have by default: it is issued
per integrating software by `integracije@moj-eracun.hr`, and the Moj-eRačun web
application's *Integratori* page lists the integrators a company has authorised. Ask
for one for "tbd finance" (or the name you prefer), and for the demo environment too if
you want to try against `https://demo.moj-eracun.hr/apis/v2` first: set the row's
`config.base_url` to that.

The link is proven by one inbox query before the values are sealed; a wrong login is
refused on the spot with Moj-eRačun's own words. The values are sealed at rest like every
connector credential and never shown again.

## Not done

Confirming the import back (`notifyimport`), and marking paid or rejecting through
Moj-eRačun's Fiscalization 2.0 operations, are deliberate omissions: the connector is a
reader. Parsing the UBL's VAT lines (rates, bases) is F7 proper in
`docs/plans/finance-accountant.md`. Sending e-invoices is the outgoing half of the same
plan.
