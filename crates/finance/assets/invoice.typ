// The InOrbit invoice. Data comes in through `sys.inputs.doc`; nothing in this
// file knows a client, a number or a bank. See crates/finance/src/invoice/render.rs
// for the contract, and docs/finance/invoice.md for what each field means.
#import sys: inputs
#let doc = inputs.doc

#set page(paper: "a4", margin: (top: 18mm, bottom: 16mm, x: 18mm), footer: context [
  #set text(size: 7.2pt, fill: rgb("#6a6a64"))
  #grid(columns: (1fr, auto), align: (left, right),
    [#doc.issuer.legal_name · #doc.issuer.address_lines.join(", ") · OIB #doc.issuer.oib],
    [#counter(page).display() / #counter(page).final().first()])
])
#set text(font: "Inter", size: 9.2pt, fill: rgb("#1a1a18"), lang: "hr")
#set par(leading: 0.55em)

#let muted(body) = text(fill: rgb("#6a6a64"), body)
#let label(en, hr) = [#text(weight: "medium", en) #muted[\/ #hr]]
#let mono(body) = text(font: "Inter", features: ("tnum",), body)

// Watermark for anything that is not an issued invoice.
#if doc.watermark != "" [
  #place(center + horizon, rotate(-30deg, text(size: 64pt, weight: "bold", fill: rgb("#00000012"), doc.watermark)))
]

// ---- head: brand left, number and dates right ----------------------------
#grid(columns: (1fr, auto), align: (left + top, right + top), gutter: 12pt,
  [
    #box(height: 22pt, baseline: 20%, image("mark.svg", height: 22pt))
    #h(6pt)
    #box(baseline: 20%, text(size: 17pt, weight: "semibold", tracking: -0.4pt, "InOrbit"))
    #v(6pt)
    #text(weight: "medium", doc.issuer.legal_name) \
    #doc.issuer.address_lines.join("\n") \
    #muted[OIB / VAT-ID: #doc.issuer.oib · #doc.issuer.vat_id]
  ],
  [
    #text(size: 20pt, weight: "semibold", tracking: -0.5pt, if doc.number_preview [Račun / Invoice] else [Račun / Invoice]) \
    #text(size: 13pt, weight: "medium", mono(doc.number)) #if doc.number_preview [#muted[(next)]] \
    #v(4pt)
    #set text(size: 8.6pt)
    #grid(columns: (auto, auto), column-gutter: 10pt, row-gutter: 3.5pt, align: (right, left),
      muted[Date and time / Datum i vrijeme], mono(doc.issued_at),
      muted[Delivery date / Datum isporuke], mono(doc.delivery_date),
      muted[Due date / Rok dospijeća], mono(doc.due_date),
      muted[Place of issue / Mjesto izdavanja], doc.place_of_issue,
    )
  ],
)

#v(16pt)
#line(length: 100%, stroke: 0.5pt + rgb("#e3e3df"))
#v(10pt)

// ---- bill to ----------------------------------------------------------------
#grid(columns: (1fr, 1fr), gutter: 16pt,
  [
    #muted[#text(size: 7.6pt, weight: "medium", tracking: 0.6pt, upper[Bill to / Kupac])] \
    #v(3pt)
    #text(weight: "medium", size: 10.5pt, doc.client.name) \
    #doc.client.address_lines.join("\n") \
    #doc.client.country \
    #if doc.client.tax_id != "" [#muted[Tax ID / Porezni broj:] #mono(doc.client.tax_id)]
  ],
  [
    #muted[#text(size: 7.6pt, weight: "medium", tracking: 0.6pt, upper[Payment / Plaćanje])] \
    #v(3pt)
    #set text(size: 8.6pt)
    #grid(columns: (auto, 1fr), column-gutter: 10pt, row-gutter: 3.5pt,
      muted[Method / Način], [Bank transfer / Transakcijski račun],
      muted[IBAN], mono(doc.issuer.iban),
      muted[SWIFT / BIC], mono(doc.issuer.swift),
      muted[Bank / Banka], doc.issuer.bank_name,
      muted[Reference / Poziv na broj], mono(doc.number),
    )
  ],
)

#v(18pt)

// ---- lines --------------------------------------------------------------------
#table(
  columns: (auto, 1fr, auto, auto, auto),
  stroke: (x, y) => if y == 0 { (bottom: 0.8pt + rgb("#1a1a18")) } else { (bottom: 0.4pt + rgb("#e3e3df")) },
  inset: (x: 6pt, y: 7pt),
  align: (right + horizon, left + horizon, right + horizon, right + horizon, right + horizon),
  table.header(
    muted[\#],
    label[Service][Usluga],
    label[Qty][Količina],
    label[Price][Cijena],
    label[Amount][Iznos],
  ),
  ..doc.lines.map(l => (
    mono(str(l.position)),
    l.description,
    mono(l.quantity),
    mono(l.unit_price),
    mono(l.amount),
  )).flatten(),
)

#v(8pt)
#grid(columns: (1fr, auto), [], [
  #set text(size: 9.2pt)
  #grid(columns: (auto, auto), column-gutter: 18pt, row-gutter: 5pt, align: (right, right),
    muted[Subtotal / Ukupno], mono[#doc.subtotal #doc.currency],
    muted[#doc.vat_label], mono[#doc.vat #doc.currency],
    text(weight: "semibold", size: 10.5pt)[Total due / Za platiti], text(weight: "semibold", size: 10.5pt, mono[#doc.total #doc.currency]),
  )
])

#v(14pt)
#if doc.vat_note != "" [
  #block(inset: (left: 8pt), stroke: (left: 1.5pt + rgb("#e3e3df")), text(size: 8pt, fill: rgb("#3a3a36"), doc.vat_note))
]
#if doc.note != "" [
  #v(6pt)
  #text(size: 8.4pt, doc.note)
]

#v(1fr)
// ---- legal footer (what a Croatian invoice must carry) -------------------------
#set text(size: 7.4pt, fill: rgb("#6a6a64"))
#grid(columns: (1fr, 1fr), gutter: 10pt, row-gutter: 3pt,
  [Operator ID / Oznaka operatera: #doc.issuer.operator_id],
  [Issued by / Odgovorna osoba: #doc.issuer.issued_by],
  [Competent court / Nadležni sud: #doc.issuer.court],
  [Company reg. no. / MBS: #doc.issuer.registration_no],
  [Share capital / Temeljni kapital: #doc.issuer.share_capital],
  [Management board / Član uprave: #doc.issuer.board_member],
)
