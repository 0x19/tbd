"use client";

import { defaultKeymap, history, historyKeymap, indentWithTab } from "@codemirror/commands";
import { bracketMatching, syntaxHighlighting } from "@codemirror/language";
import { EditorState } from "@codemirror/state";
import {
  drawSelection,
  EditorView,
  highlightActiveLine,
  highlightActiveLineGutter,
  keymap,
  lineNumbers,
} from "@codemirror/view";
import { classHighlighter } from "@lezer/highlight";
import { useEffect, useImperativeHandle, useRef } from "react";

import { tomlLanguage } from "@/lib/toml-highlight";
import { cn } from "@/lib/utils";

export type TomlEditorHandle = {
  /** Insert at the cursor (or replace the selection) and focus. */
  insert: (text: string) => void;
  focus: () => void;
};

type Props = {
  value: string;
  onChange: (value: string) => void;
  className?: string;
  ariaLabel?: string;
  ref?: React.Ref<TomlEditorHandle>;
};

const theme = EditorView.theme({
  "&": { fontSize: "12px", backgroundColor: "transparent", height: "100%" },
  ".cm-scroller": { fontFamily: "var(--font-mono), ui-monospace, monospace", lineHeight: "1.65" },
  ".cm-content": { caretColor: "var(--foreground)", padding: "12px 0" },
  ".cm-line": { padding: "0 12px" },
  ".cm-gutters": {
    backgroundColor: "transparent",
    borderRight: "1px solid var(--border)",
    color: "var(--muted-foreground)",
  },
  ".cm-lineNumbers .cm-gutterElement": { padding: "0 10px 0 14px", minWidth: "2.5rem" },
  ".cm-activeLineGutter": { backgroundColor: "color-mix(in oklab, var(--muted) 70%, transparent)" },
  ".cm-activeLine": { backgroundColor: "color-mix(in oklab, var(--muted) 45%, transparent)" },
  "&.cm-focused": { outline: "none" },
  "&.cm-focused .cm-cursor": { borderLeftColor: "var(--foreground)" },
  "&.cm-focused .cm-selectionBackground, .cm-selectionBackground, ::selection": {
    backgroundColor: "color-mix(in oklab, var(--primary) 18%, transparent)",
  },
  ".cm-matchingBracket": { outline: "1px solid var(--border)", borderRadius: "2px" },
});

/** A CodeMirror 6 TOML editor styled with the kit's tokens. Controlled: `value` in, `onChange` out. */
export function TomlEditor({ value, onChange, className, ariaLabel, ref }: Props) {
  const host = useRef<HTMLDivElement>(null);
  const view = useRef<EditorView | null>(null);
  const onChangeRef = useRef(onChange);
  useEffect(() => {
    onChangeRef.current = onChange;
  });

  useEffect(() => {
    if (!host.current) return;
    const state = EditorState.create({
      doc: value,
      extensions: [
        lineNumbers(),
        highlightActiveLineGutter(),
        highlightActiveLine(),
        history(),
        drawSelection(),
        bracketMatching(),
        keymap.of([...defaultKeymap, ...historyKeymap, indentWithTab]),
        tomlLanguage,
        syntaxHighlighting(classHighlighter),
        theme,
        EditorView.lineWrapping,
        EditorView.contentAttributes.of({ "aria-label": ariaLabel ?? "TOML", spellcheck: "false" }),
        EditorView.updateListener.of((u) => {
          if (u.docChanged) onChangeRef.current(u.state.doc.toString());
        }),
      ],
    });
    const v = new EditorView({ state, parent: host.current });
    view.current = v;
    return () => {
      v.destroy();
      view.current = null;
    };
    // The editor is created once; `value` afterwards flows through the effect below.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // External changes (a loaded file, a snippet) replace the document; typing does not loop
  // because the listener already reported the same text.
  useEffect(() => {
    const v = view.current;
    if (!v) return;
    const current = v.state.doc.toString();
    if (current === value) return;
    v.dispatch({ changes: { from: 0, to: current.length, insert: value } });
  }, [value]);

  useImperativeHandle(ref, () => ({
    insert: (text) => {
      const v = view.current;
      if (!v) return;
      const { from, to } = v.state.selection.main;
      const before = v.state.doc.sliceString(Math.max(0, from - 1), from);
      const needsGap = from > 0 && before !== "\n";
      const insert = `${needsGap ? "\n" : ""}${text}`;
      v.dispatch({
        changes: { from, to, insert },
        selection: { anchor: from + insert.length },
        scrollIntoView: true,
      });
      v.focus();
    },
    focus: () => view.current?.focus(),
  }));

  return <div ref={host} className={cn("bg-background overflow-hidden rounded-xl border", className)} />;
}
