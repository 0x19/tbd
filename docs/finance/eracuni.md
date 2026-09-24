# e-računi: the company's incoming e-invoices as receipts

Since 1 January 2026 domestic suppliers send the company e-invoices through an
information intermediary, not as PDFs by mail. The company's intermediary is e-računi
(E-RAČUNI d.o.o., `e-racuni.hr`). The `eracuni` connector kind pulls what e-računi holds
into the same receipts ledger the mailboxes fill (`/documents/`), with the fields
e-računi itself knows -- supplier, number, date, total -- written as facts, not read
from the PDF as guesses.

## What is pulled

| Source | API methods | Becomes |
|---|---|---|
| **Received invoices** (booked in e-računi) | `ReceivedInvoiceList` over a date window, then per new one `ReceivedInvoiceGet` (the facts) and `ReceivedInvoiceAttachmentList` / `ReceivedInvoiceAttachmentGet` (the file) | one document per invoice: the first PDF attachment, else the first attachment there is (the e-invoice XML) |
| **The e-invoice inbox** (arrived, not yet booked) | `DocumentInboxList` with `type = PurchaseInvoice` over `receivalDateFrom/To`, then `DocumentInboxEntryGet` | one document per entry |

The connector's *Source* select on `/connectors/` chooses received invoices, the inbox,
or both (the default). A pull reaches back seven days past the last one, stores at
most 50 documents a round (the run is `partial` and the next one continues), and never
fetches an id it stored before. The same bytes seen twice are one document with two
sources.

A document from e-računi shows `from the intermediary` beside its vendor, number, date
and amount on `/documents/`; the reader still runs over the PDF and fills only what
e-računi did not give, and a re-read never overrules a fact. A person's correction
still outranks everything.

## Linking

`/connectors/` → Link a mailbox → **e-računi** → Continue → the three values:

| Field | Where it is in e-računi |
|---|---|
| API user | Postavke → Web servisi: the API user account (not your login) |
| API secret key | the same page, the user's API key |
| Web-services token | the same page, the company's token |

Web services must be enabled for the company in e-računi first; e-računi issues the
company token when they are. The link is proven by one call (the last day's received
invoices) before the values are sealed; a wrong value is refused on the spot. The
values are sealed at rest like every connector credential and never shown again.

## What is known, and what the first pull settles

The API is `POST https://e-racuni.hr/WebServicesHR/API` with a JSON envelope
`{ "username", "secretKey", "token", "method", "parameters" }` and a rate limit of four
requests in four seconds (the connector keeps to one a second). The method names above
are documented. **The shape of the answers is not documented publicly** beyond "the PDF
is returned with BASE64 encoding", so the connector is deliberately tolerant: the
payload is looked for under the method's name, `result`, `data`, `response`, then at
the top level; a list is the first array in it; ids under `documentID`, `id`,
`sequentialNumber`; file bytes under `contents`, `content`, `data`, `file`, `pdf`; the
facts under the usual names (`supplierName`, `documentNumber`, `date`, `totalAmount`,
`currency`, …). An answer that fits none of that fails the pull with a sentence naming
the keys it saw (`no file in the answer (object with …)`), which the connector's Test
and the run's error show. One round with real credentials settles the names; the
tolerance is there so that round is short, not so a guess becomes a document.

## Not done

Parsing the UBL XML structurally (VAT by rate, lines) is F7 proper in
`docs/plans/finance-accountant.md`; sending e-invoices through e-računi is the outgoing
half of the same plan; other intermediaries (Moj-eRačun, FINA e-Račun) are one kind file
and one registry line each.
