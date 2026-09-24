// A receipt that came as a mail with no attachment, printed. Data comes in
// through `sys.inputs.doc`: who sent it, the subject, when, and the text.
// See crates/finance/src/documents/mail.rs.
#import sys: inputs
#let doc = inputs.doc

#set page(paper: "a4", margin: (top: 18mm, bottom: 16mm, x: 18mm), footer: context [
  #set text(size: 7.2pt, fill: rgb("#6a6a64"))
  #grid(columns: (1fr, auto), align: (left, right),
    [Printed from the mailbox #doc.mailbox · message #doc.message_id],
    [#counter(page).display() / #counter(page).final().first()])
])
#set text(font: "Inter", size: 9.2pt, fill: rgb("#1a1a18"))
#set par(leading: 0.55em)

#let muted(body) = text(fill: rgb("#6a6a64"), body)

#text(size: 15pt, weight: "semibold", tracking: -0.3pt, doc.subject)
#v(4pt)
#grid(columns: (auto, 1fr), column-gutter: 10pt, row-gutter: 4pt,
  muted[From], [#doc.from],
  muted[Received], [#doc.received],
)
#v(8pt)
#line(length: 100%, stroke: 0.5pt + rgb("#d9d9d4"))
#v(8pt)

#for l in doc.lines [
  #if l == "" [ #v(5pt) ] else [ #l \ ]
]
