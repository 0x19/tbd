---
name: design-doc
description: Create a new numbered design decision doc in docs/design/ or docs/design/id/, following the repo's format, and register it in the README index.
disable-model-invocation: true
argument-hint: <plane: root|id> <short title>
allowed-tools: Read, Write, Edit, Bash(ls *), Bash(git log *)
---
Create a design decision doc for: $ARGUMENTS

The first word is the plane: `root` means `docs/design/`, `id` means `docs/design/id/`.
The rest is the title.

1. List the target directory and take the next free three-digit number.
2. Read `docs/design/README.md` and one recent doc in the target directory to
   match the voice and format exactly.
3. Write `NNN-<kebab-title>.md` with:
   - `# NNN — Title`
   - `Status: open`
   - one-line summary of the single question this doc decides
   - `---`
   - sections as the question demands, typically: the question, the decision,
     why, consequences, what stays open. Keep it under 150 lines.
   - links to every existing doc it depends on or contradicts.
4. Add a row to the index table in the target directory's README.
5. In the reply, list any existing doc whose statements this new doc would
   supersede or contradict. Do not edit those docs.
