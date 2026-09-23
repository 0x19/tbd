// The CV as a PDF, from the same facts as the site (cv.json is written by
// ui/www/tool/cv-data.ts from ui/www/src/data/site.ts). Two pages, plain,
// first person, nothing a recruiter's parser cannot read: no columns, no
// icons, no tables.
//
// Two optional inputs make it the full CV the cv service renders for one
// approved reader: `private` (a JSON string: phone, address, references)
// and `reader` (name, email, date), who is then named on every page.
// Without them this is the public document, byte for byte.
#let d = json("cv.json")
#let private = if "private" in sys.inputs { json(bytes(sys.inputs.private)) } else { none }
#let reader = if "reader" in sys.inputs { sys.inputs.reader } else { none }
#let muted = rgb("#5b5b5b")

#set page(
  paper: "a4",
  margin: (x: 15mm, y: 11mm),
  numbering: none,
  footer: if reader != none {
    align(center, text(size: 7.5pt, fill: muted)[
      Prepared for #reader.name (#reader.email) on #reader.date. Not for redistribution.
    ])
  } else { none },
)
#set text(font: "Inter", size: 8.8pt, fill: rgb("#111111"), hyphenate: false)
#set par(leading: 0.52em, justify: false)
#let rule = line(length: 100%, stroke: 0.5pt + rgb("#d9d9d9"))

#let section(title) = block(above: 10pt, below: 5pt, width: 100%)[
  #text(size: 7.8pt, weight: "semibold", tracking: 0.12em, fill: muted, upper(title))
  #v(-3pt)
  #rule
]

// Header
#text(size: 21pt, weight: "semibold", tracking: -0.02em, d.person)
#v(1pt)
#text(size: 10.5pt, d.title)
#v(3pt)
#text(size: 8.6pt, fill: muted)[
  #d.city · #link("mailto:" + d.email, d.email) · #link("https://" + d.site, d.site) · #link("https://" + d.github, d.github) · #link("https://" + d.linkedin, d.linkedin)
]
#if private != none and (private.phone != "" or private.address != "") [
  #v(2pt)
  #text(size: 8.6pt, fill: muted)[
    #if private.phone != "" [ #link("tel:" + private.phone.replace(" ", ""), private.phone) ]
    #if private.phone != "" and private.address != "" [ · ]
    #if private.address != "" [ #private.address ]
  ]
]
#v(6pt)
#block(inset: (x: 8pt, y: 6pt), fill: rgb("#f3f3f3"), radius: 2pt, width: 100%)[
  #text(size: 9pt, weight: "medium", d.availability)
]

#section("Summary")
#if private != none and private.summary != "" [ #private.summary ] else [ #d.summary ]

#section("Selected work")
#for a in ((if private != none { private.achievements } else { () }) + d.achievements) [
  - #a
]

#section("Experience")
#for e in d.experience [
  #block(breakable: false)[
    #grid(columns: (1fr, auto), align: (left, right))[
      #text(weight: "semibold", e.company) #text(fill: muted)[ · #e.role]
    ][
      #text(size: 8.4pt, fill: muted)[#e.when · #e.where]
    ]
    #let more = if private != none { private.experience.find(x => x.company == e.company) } else { none }
    #v(2pt)
    #if more != none and more.body != "" [ #more.body ] else [ #e.body ]
    #if e.highlights.len() > 0 [
      #v(1pt)
      #for h in e.highlights [
        - #h
      ]
    ]
    #if more != none [
      #if more.highlights.len() > 0 [
        #v(1pt)
        #for h in more.highlights [
          - #h
        ]
      ]
    ]
    #v(3pt)
  ]
]

#section("Earlier")
#d.earlier.map(e => e.when + " " + e.company + ", " + lower(e.role.slice(0, 1)) + e.role.slice(1)).join("; ").
#v(2pt)

#section("Open source")
#for p in d.projects [
  #grid(columns: (9.5em, 1fr), gutter: 6pt)[#text(weight: "medium", p.name) #text(size: 8.4pt, fill: muted, p.year)][#p.what #text(size: 8.4pt, fill: muted, link("https://" + p.href, p.href))]
  #v(2pt)
]

#if private != none and private.references.len() > 0 [
  #section("References")
  #for r in private.references [
    #grid(columns: (9.5em, 1fr), gutter: 6pt)[#text(weight: "medium", r.name)][#r.role #text(size: 8.4pt, fill: muted, r.contact)]
    #v(2pt)
  ]
]

#section("Languages and education")
#d.languages.join(", "). #d.education
