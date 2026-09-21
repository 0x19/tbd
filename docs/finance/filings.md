# Filings: the company's ePorezna forms in the service

Everything the accountant files for the company under its OIB sits in ePorezna, whoever
pressed the button: the corporate tax return (PD), the related-persons form (PD-IPO),
the monthly VAT returns (PDV, PDV-S, ZP), the pay reports (JOPPD) and the tourist-board
form (TZ). Each downloads as XML against a schema the Tax Administration publishes
(`ePorezna_Schemas.zip` under e-porezna.porezna-uprava.hr/Upute/G2B/). The `/filings/`
page takes those files in; the service reads them into figures, so the year's payroll,
VAT and tax figures come from what was filed rather than from someone typing.

## What is taken in

Upload one or several `.xml` files on `/filings/` (or `UploadDocument` with
`text/xml`). The form is read before it is stored:

| Form | Root element | Schema (2025 → 2026) | Read as |
|---|---|---|---|
| PD | `ObrazacPD` | v9-0 → v10-0 | `1`…`59`, the loss table (`Godina00`, `PG00`…), donations `110`…`150` and their recipients |
| PDV | `ObrazacPDV` | v10-0 → v11-0 | `000` all supplies, `100`…`111` the non-taxable section, pairs `200.Vrijednost`/`200.Porez` (taxable, II) and `300.…` (input VAT, III), `400` the signed result (II − III: positive to pay, negative a refund), `500` the deduction pro-rata in %, `610`…`660` the other data (`630` services received from abroad, `640` services provided abroad) |
| PDV-S | `ObrazacPDVS` | v1-0 | one row per EU supplier, `IsporukeUkupno.I1` goods acquired, `I2` services received |
| ZP | `ObrazacZP` | v1-0 | one row per EU customer, `IsporukeUkupno.I1` goods, `I2` goods under procedures 42 and 63, `I3` triangular trade, `I4` services |
| JOPPD | `ObrazacJOPPD` | v1-1 | page A as `A.…` (`A.PredujamPoreza.P1` income tax and surtax, `A.Doprinosi.….P1` the contribution totals), page B one row per recipient (`P1`…`P17`: `P11` gross, `P141` tax, `P162` net paid); the headline adds page B up as `B.P11`, `B.P162`, `B.P141`, since page A has no gross |
| PD-IPO | `ObrazacPDIPO` | v1-0 | one row per related person and section (loans received, loans given, goods and services), each with its `Obveze`/`Potrazivanja` tranches; the section totals sit beside the persons and flatten to `Podaci.Podaci2.Osobe.Sveukupno.S1` (principal), `S3` (balance), `S4` (interest) |
| TZ | `ObrazacTZ` | v1-1 | `01`…`07` (`02` is a rate with four decimals) |

Keys are the element names with the `Podatak` prefix dropped, joined with a dot below a
block. Repeated blocks (recipients, supplies, persons) are rows, each tagged `_kind`
with the container it came from. Every amount stays the decimal string the form
carries; nothing is rounded on the way in, and the page converts for display.

The header is the form's own: the period (a JOPPD's report date), the taxpayer's OIB
and name, the preparing software's timestamp and author (`prepared_at`: when the file
was made, not when ePorezna accepted it, which the XML does not carry), the schema
(`Metapodaci/Uskladjenost`), and a JOPPD's report mark (`OznakaIzvjesca`, `yyDDD`).

## What is refused, and what is kept with an error

Refused on upload (nothing stored): bytes that are not XML, XML whose root is not one
of the forms above, a form with no OIB, a form whose OIB is not the party's registered
OIB (the message names both), and an upload to a person rather than a company. That
is deliberate: a file from another company must bounce, not sit in the list with a
warning.

Kept with `error` set: a recognised form whose body the reader could not finish (a
newer schema, a missing block). The row shows the error and whatever was read;
**Read again** after a parser upgrade clears it. The parser stamps `parser_version`
(`filings/2` since the PD-IPO totals moved out of the rows) on every row, so a later reader can tell what it wrote.

A filing never moves to another party (`UpdateDocument` refuses), and the same bytes
uploaded twice are one document. Several JOPPDs in a month, or a corrected PDV beside
the original, sit side by side; `identifier` (the software's id for the form) tells
them apart.

## On the wire

`ListFilings` (`GET /v1/finance/filings?form=&year=`) lists newest period first with a
few headline figures per form; `GetFiling` (`GET /v1/finance/filings/{id}`) adds every
value and the rows as JSON. The file itself is the document: `GetDocument` gives the
bytes. A filing is a document of kind `filing`; the receipts listing does not show it,
`ListDocuments` with no kind does, and search reaches its one-line summary.

## Not done

Zip upload (ePorezna downloads one file at a time; the page takes several files at
once instead), the PKK export, FINA's RGFI PDFs (they upload as ordinary PDFs), and
any posting of the figures into books: that is `docs/accountant/plan.md`.
