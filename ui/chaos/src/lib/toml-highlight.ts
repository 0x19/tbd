// One TOML highlighter for the editor and for code blocks in the reference:
// CodeMirror's legacy TOML mode, emitted as `tok-*` classes styled in
// src/app/editor.css with the kit's colours.
import { StreamLanguage } from "@codemirror/language";
import { toml } from "@codemirror/legacy-modes/mode/toml";
import { classHighlighter, highlightTree } from "@lezer/highlight";

export const tomlLanguage = StreamLanguage.define(toml);

export type Span = { text: string; cls: string | null };

/** Split `code` into styled spans. */
export function highlightToml(code: string): Span[] {
  const tree = tomlLanguage.parser.parse(code);
  const out: Span[] = [];
  let pos = 0;
  highlightTree(tree, classHighlighter, (from, to, classes) => {
    if (from > pos) out.push({ text: code.slice(pos, from), cls: null });
    out.push({ text: code.slice(from, to), cls: classes });
    pos = to;
  });
  if (pos < code.length) out.push({ text: code.slice(pos), cls: null });
  return out;
}
