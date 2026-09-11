"use client";

import { Check, Copy, Link as LinkIcon } from "lucide-react";
import Link from "next/link";
import { useState } from "react";
import ReactMarkdown from "react-markdown";
import rehypeRaw from "rehype-raw";
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

/** The frame around a fenced block: the language in the corner and a copy button. */
function Pre({ children }: { children?: React.ReactNode }) {
  const [copied, setCopied] = useState(false);
  const child = Array.isArray(children) ? children[0] : children;
  const className =
    child && typeof child === "object" && "props" in child
      ? ((child as { props: { className?: string } }).props.className ?? "")
      : "";
  const lang = /language-(\w+)/.exec(className)?.[1];
  const copy = () => {
    void navigator.clipboard?.writeText(textOf(children).replace(/\n$/, "")).then(() => {
      setCopied(true);
      setTimeout(() => setCopied(false), 1200);
    });
  };
  return (
    <div className="doc-pre">
      <pre>{children}</pre>
      <div className="doc-pre-tools">
        {lang ? <span className="doc-pre-lang">{lang}</span> : null}
        <button type="button" className="doc-pre-copy" onClick={copy} aria-label="Copy code">
          {copied ? <Check className="size-3" /> : <Copy className="size-3" />}
        </button>
      </div>
    </div>
  );
}

type Props = {
  text: string;
  className?: string;
  /**
   * Where a link goes: the href to use, or `null` for a relative link that has no
   * page here (rendered as text). Default: relative links as written, external
   * ones in a new tab.
   */
  resolve?: (href: string) => string | null;
  /** Hide the document's own `# Title` (the page shows it in its header). */
  hideTitle?: boolean;
};

/**
 * Render markdown with the repo's conventions: GFM tables, anchored headings
 * with ids the way GitHub numbers duplicates, highlighted TOML, copyable fences,
 * `<details>` from the design notes, and links resolved by the caller.
 */
export function Markdown({ text, className, resolve, hideTitle }: Props) {
  // Ids are deduplicated in document order, the same way scripts/gen-docs.mjs
  // numbers them, so a table of contents built from the bundle lands on the
  // right heading. Rebuilt on every render; headings render in order.
  const seen = new Map<string, number>();
  const idFor = (children: React.ReactNode) => {
    let id = slug(textOf(children)) || "section";
    const n = seen.get(id) ?? 0;
    seen.set(id, n + 1);
    if (n) id = `${id}-${n}`;
    return id;
  };
  const heading = (Tag: "h1" | "h2" | "h3" | "h4") => {
    function Heading({ children }: { children?: React.ReactNode }) {
      const id = idFor(children);
      if (Tag === "h1" && hideTitle) return null;
      return (
        <Tag id={id}>
          {children}
          <a href={`#${id}`} className="doc-anchor" aria-label="Link to this section">
            <LinkIcon className="size-3.5" />
          </a>
        </Tag>
      );
    }
    return Heading;
  };
  return (
    <div className={cn("doc", className)}>
      <ReactMarkdown
        remarkPlugins={[remarkGfm]}
        rehypePlugins={[rehypeRaw]}
        components={{
          h1: heading("h1"),
          h2: heading("h2"),
          h3: heading("h3"),
          h4: heading("h4"),
          pre: ({ children }) => <Pre>{children}</Pre>,
          code: ({ className, children }) => <CodeBlock className={className}>{children}</CodeBlock>,
          table: ({ children }) => (
            <div className="overflow-x-auto">
              <table>{children}</table>
            </div>
          ),
          a: ({ href, children }) => {
            const target = href === undefined ? null : resolve ? resolve(href) : href;
            if (target === null) {
              return (
                <span className="doc-missing" title={`${href ?? ""} is not in the knowledge base`}>
                  {children}
                </span>
              );
            }
            if (/^https?:/.test(target)) {
              return (
                <a href={target} target="_blank" rel="noreferrer">
                  {children}
                </a>
              );
            }
            if (target.startsWith("/")) return <Link href={target}>{children}</Link>;
            return <a href={target}>{children}</a>;
          },
        }}
      >
        {text}
      </ReactMarkdown>
    </div>
  );
}

/** The h2/h3 outline of a markdown text, for a table of contents; ids as the renderer makes them. */
export function outline(text: string): { level: 2 | 3; title: string; id: string }[] {
  const out: { level: 2 | 3; title: string; id: string }[] = [];
  const seen = new Map<string, number>();
  let inFence = false;
  for (const line of text.split("\n")) {
    if (line.startsWith("```")) inFence = !inFence;
    if (inFence) continue;
    const m = /^(#{1,4})\s+(.*?)\s*$/.exec(line);
    if (!m) continue;
    let id = slug(m[2]!) || "section";
    const n = seen.get(id) ?? 0;
    seen.set(id, n + 1);
    if (n) id = `${id}-${n}`;
    if (m[1] !== "##" && m[1] !== "###") continue;
    out.push({ level: m[1] === "##" ? 2 : 3, title: m[2]!.replace(/`/g, ""), id });
  }
  return out;
}
