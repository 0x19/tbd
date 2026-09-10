---
paths:
  - "docs/design/**"
---
# Design doc rules

- Title `# NNN — Title`. Second line `Status: open | decided | superseded by NNN`.
- One decision per doc. If it does not close a question, it is notes and belongs
  in `001-open-questions.md`.
- Numbers are assigned in writing order per directory and never reused or renumbered.
  Superseded docs stay; add the forward link in the old doc and the backward link
  in the new one.
- Keep it short. apex had a 1,178-line spec and no code.
- Cite reality: file and line in apex or Proximity, the primary spec, or the paper.
- When you add or change a doc, update the index table in that directory's README.
- When a change contradicts another doc, say so in the reply with file and section.
  Do not quietly edit the other doc to match unless asked.
