"use client";

import ReactMarkdown from "react-markdown";
import remarkGfm from "remark-gfm";

import { highlightToml } from "@/lib/toml-highlight";
import { cn } from "@/lib/utils";

/** Heading text to an anchor id, the way GitHub does it. */
export function slug(text: string): string {
  return text
    .toLowerCase()
    .replace(/[`[\]]/g, "")
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");
}

function textOf(node: React.ReactNode): string {
  if (typeof node === "string" || typeof node === "number") return String(node);
  if (Array.isArray(node)) return node.map(textOf).join("");
  if (node && typeof node === "object" && "props" in node) {
    return textOf((node as { props: { children?: React.ReactNode } }).props.children);
  }
  return "";
}

/** A code block: TOML gets the editor's highlighter, everything else stays plain. */
function CodeBlock({ className, children }: { className?: string; children?: React.ReactNode }) {
  const code = textOf(children).replace(/\n$/, "");
  const lang = /language-(\w+)/.exec(className ?? "")?.[1];
  if (lang !== "toml") return <code className={className}>{children}</code>;
  return (
    <code className={className}>
      {highlightToml(code).map((s, i) => (
        <span key={i} className={s.cls ?? undefined}>
          {s.text}
        </span>
      ))}
    </code>
  );
}

/** Render markdown with the repo's conventions: GFM tables, anchored headings, highlighted TOML. */
export function Markdown({ text, className }: { text: string; className?: string }) {
  return (
    <div className={cn("doc", className)}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        components={{
          h1: ({ children }) => <h1 id={slug(textOf(children))}>{children}</h1>,
          h2: ({ children }) => <h2 id={slug(textOf(children))}>{children}</h2>,
          h3: ({ children }) => <h3 id={slug(textOf(children))}>{children}</h3>,
          code: ({ className, children }) => <CodeBlock className={className}>{children}</CodeBlock>,
          table: ({ children }) => (
            <div className="overflow-x-auto">
              <table>{children}</table>
            </div>
          ),
          a: ({ href, children }) => (
            <a href={href} target={href?.startsWith("http") ? "_blank" : undefined} rel="noreferrer">
              {children}
            </a>
          ),
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}

/** The h2/h3 outline of a markdown text, for a table of contents. */
export function outline(text: string): { level: 2 | 3; title: string; id: string }[] {
  const out: { level: 2 | 3; title: string; id: string }[] = [];
  let inFence = false;
  for (const line of text.split("\n")) {
    if (line.startsWith("```")) inFence = !inFence;
    if (inFence) continue;
    const m = /^(##|###)\s+(.*)$/.exec(line);
    if (!m) continue;
    const title = m[2]!.replace(/`/g, "");
    out.push({ level: m[1] === "##" ? 2 : 3, title, id: slug(m[2]!) });
  }
  return out;
}
