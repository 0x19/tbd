# Invoices

How an invoice goes from a draft to a numbered PDF, and what the template is
given. The code is `crates/finance/src/invoice/`; the template is
`crates/finance/assets/invoice.typ`; the UI is `ui/finances/src/app/invoices/`.

## The flow

```
draft ──preview──▶ (hash, PDF)          nothing written
draft ──approve(hash)──▶ approved        number taken, PDF stored, hash recorded
approved ──cancel──▶ cancelled           keeps its number
draft ──discard──▶ cancelled             never had one
```

- A **number** is `ordinal-premises-device-year`, e.g. `10-1-1-2026`, allocated
  at approval from `finance.invoice_numbers` under its row lock, inside the
  approving transaction. Gapless per year. Never a sequence.
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
