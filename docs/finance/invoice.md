# Invoices

How an invoice goes from a draft to a numbered PDF, and what the template is
given. The code is `crates/finance/src/invoice/`; the template is
`crates/finance/assets/invoice.typ`; the UI is `ui/finances/src/app/invoices/`.

## The flow

```
draft ──preview──▶ (hash, PDF)          nothing written
draft ──approve(hash)──▶ approved        number taken, PDF stored, hash recorded
approved ──cancel──▶ cancelled           keeps its number
draft ──delete──▶ (gone)                 never had one; nothing remains
any ──duplicate──▶ draft                 the header and lines, dated today
approved ──sent by mail──▶ sent          a delivery row: the mail, to whom, when
approved ──payments cover it──▶ paid     matched from the bank, or recorded
paid ──last payment undone──▶ approved   the one step back
```

- A **number** is `ordinal-premises-device-year`, e.g. `10-1-1-2026`, allocated
  at approval from `finance.invoice_numbers` under its row lock, inside the
  approving transaction. Gapless per year **and per series**: the counter is one
  per `(party, year, premises, device)`, so a second premises numbers from 1 on
  its own. The year is the year of approval, whatever year the draft was written
  in. Never a sequence.
- A company has one **default client** (`clients.is_default`, `SetDefaultClient`, chosen on
  the Clients page): the invoices list opens on it and a new draft is for it.
- A **draft's header** may change until it is approved: the client (of the same
  party), the currency, the VAT treatment (its note follows) and the series. A
  draft that is not wanted is deleted; only an issued invoice is cancelled.
- **Sent** is a fact, not a button: a mail sent through `SendMail` with
  `invoice_id` set (the composer sets it when the page handed it an invoice)
  records a row in `finance.invoice_deliveries` (the mail, the recipients, the
  time), flips `approved` to `sent` and stamps `sent_at` with the first send
  (`paid` stays `paid`), and writes a `sent` event. A draft or a cancelled
  invoice is refused before the provider is asked. Sending the same invoice a
  second time is refused with the date and recipients of the first send unless
  `force`, which the page asks for as "Send again"; every send is one more row,
  listed on the invoice page with a link to its thread. `Invoice.deliveries` and
  `sent_at` carry them.
- **Age** is the server's word (`invoice/aging.rs`): an invoice is open while
  `approved` or `sent`, `outstanding_minor` is what the payments do not cover,
  `days_overdue` counts from the due date in Zagreb and is zero until then.
  `AgingReport` sums the open invoices into the five buckets (not yet due,
  1–30, 31–60, 61–90, over 90 days) per currency and per client, most overdue
  first; the invoices page draws it and a client row narrows the list.
- A **reminder** is a mail with `reminder` set beside `invoice_id`: recorded as
  a delivery of kind `reminder` (never refused for the invoice having gone out,
  refused for a paid one), `reminded_at` on the invoice is the last one, and the
  event is `reminded`. The page offers it on an overdue invoice; the composer
  fills `{{Outstanding}}` and `{{DaysOverdue}}` beside the invoice helpers, takes
  a template named "reminder" or "opomena" when the company has one, and has a
  default text in the page's language otherwise.
- A **preview** renders the draft with the number it *would* take and the current
  minute, watermarked `PREVIEW`, and returns the SHA-256 of the canonical document.
  The minute is truncated so an approval that follows within it computes the same
  hash; a preview older than that, or a draft edited since, is refused with
  `FAILED_PRECONDITION` and takes no number.
- **Approve** rebuilds the document under the draft's lock, compares the hash,
  then allocates, renders without the watermark, stores the PDF content-addressed
  (`finance.documents` + `document_blobs`), freezes the VAT note onto the invoice,
  and records who approved what hash (`finance.invoice_events`).
- The **PDF is a pure function of the document**: fonts and the mark are compiled
  into the binary, the PDF's id and creation time are pinned to the invoice, so the
  same document renders to the same bytes (tested).

## Money

Minor units (`bigint`) everywhere; quantities in thousandths. A line amount is
`quantity_milli × unit_price_minor / 1000` rounded half up; VAT is
`subtotal × rate_bp / 10000` the same way; `total = subtotal + vat` is a check
constraint. Nothing is ever a float.

VAT treatment is a closed set on the client, copied onto the invoice at creation
and its note frozen at approval:

| treatment | rate | note |
|---|---|---|
| `standard_hr` | 25% | — |
| `reverse_charge_eu` | 0 | Article 17(1), bilingual |
| `outside_scope_non_eu` | 0 | Article 17(1), bilingual (Tenderly) |
| `exempt_issuer` | 0 | Article 90 |

## The template contract

`sys.inputs.doc` is one dictionary; every value is pre-formatted, so the template
holds layout and nothing else.

| key | example |
|---|---|
| `number`, `number_preview` | `"10-1-1-2026"`, `false` |
| `issued_at` | `"30.09.2026. 13:55"` (Europe/Zagreb) |
| `delivery_date`, `due_date` | `"30.09.2026."` |
| `place_of_issue`, `currency` | `"Viškovo"`, `"EUR"` |
| `issuer.*` | `legal_name`, `address_lines[]`, `oib`, `vat_id`, `iban`, `swift`, `bank_name`, `court`, `registration_no`, `share_capital`, `board_member`, `issued_by`, `operator_id` |
| `client.*` | `name`, `address_lines[]`, `country`, `tax_id` |
| `lines[]` | `position`, `description`, `quantity` (`"1.5"`), `unit_price`, `amount` (`"13,750.00"`) |
| `subtotal`, `vat`, `vat_label`, `total` | `"14,500.82"`, `"0.00"`, `"VAT / PDV"`, `"14,500.82"` |
| `vat_note`, `note`, `watermark` | text; `""` when none |

The mark is `image("mark.svg")` from a static resolver; the wordmark is set in
Inter next to it (Typst does not shape text inside SVG). Fonts: Inter Regular,
Medium, SemiBold, Bold (OFL), under `crates/finance/assets/fonts/`.

For eyes: `INVOICE_SAMPLE_OUT=/tmp/sample.pdf cargo nextest run -p tbd-finance -E 'test(invoice::render::)'`
writes the August 2026 sample.

## Invoices issued before this service

`finance import-invoices --dir ~/finance-data --party <uuid>` reads every PDF under the
directory and shows what it found: number, issue time, client, total, lines. Nothing is
written without `--apply`; `--lines` prints each parsed line. It needs poppler's
`pdftotext` (the positions are what tell a wrapped description which row it belongs to),
so it runs on a laptop against the cluster's database, never in the service.

What it writes: an approved invoice with the printed number, dates, lines and totals; the
PDF as the invoice's document; the client found by name or made from the page; an
`imported` event naming the file; and the counter raised past the number. A file that
does not add up (lines, subtotal, VAT, total) is refused with the arithmetic; a copy of an
invoice (the same content in a second file) is one invoice; two files claiming one number
with different content are both refused and named, because only a person can say which
was issued. A second run finds everything present and writes nothing.

## Money in

`finance.invoice_payments` holds what settled an invoice. The matcher
(`invoice/payments.rs`, `settle`) runs before every read of the invoices and looks
only at booked credits no payment row names yet: the payer writes the invoice
number in the remittance (`HR99 | BROJ RACUNA 9-1-1-2026`, no structured reference,
measured on every Tenderly settlement) or in the structured *poziv na broj*; the
invoice with that number, in the same currency, gets the transaction as an
`inferred` payment for the amount that arrived. A part pays a part. The invoice's
`paid_minor` is the sum; it is `paid` once that covers the total, with `paid_at`
the day of the last payment. `RecordPayment` is a person's word (`declared`): a
transaction of the party, or an amount and a day the bank has not shown.
`UnlinkPayment` undoes one; a match the matcher made is kept as `rejected` so it is
not remade, and a paid invoice with nothing left covering it goes back to
`approved`, the one step back the machine has. Amount alone never matches: two
months can bill the same figure.
