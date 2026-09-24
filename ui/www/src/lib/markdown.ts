import { Marked } from "marked";

import { runnable } from "@/lib/runner";

/**
 * A model's answer as HTML, safely. The text is untrusted (anyone's prompt can
 * steer it), so nothing it writes may run or load anything:
 *
 * - raw HTML in the answer is shown as text, never parsed;
 * - an image is shown as its description and never fetched: the site loads
 *   nothing from elsewhere (`/legal/`), and an image URL is a way to report
 *   who read the answer;
 * - a link is kept only for `http(s)`, opens in a new tab, and carries
 *   `noopener noreferrer nofollow`; a path on this site (`/lab/`, one leading
 *   slash, plain path characters, never `//host`) is kept as a link in the
 *   same tab, which is how an agent points at a page (RFC 0011); any other
 *   scheme (`javascript:`, `data:`) is its text alone;
 * - a code block is escaped text in a `<pre>`, with its language named and a
 *   copy button the page wires up (`data-copy`); a Go or Rust block also gets a
 *   run button (`data-run`, the language only), which the page sends to the
 *   runner (RFC 0010).
 *
 * Everything else is marked's own escaping of text. Re-rendered on every chunk
 * while an answer streams, so an unclosed fence reads as code until it closes.
 */

const esc = (s: string) =>
  s
    .replace(/&/g, "&amp;")
    .replace(/</g, "&lt;")
    .replace(/>/g, "&gt;")
    .replace(/"/g, "&quot;")
    .replace(/'/g, "&#39;");

/** A path on this site: one leading slash and plain path characters, so never `//host` or a scheme. */
const SITE_PATH = /^\/(?![/\\])[A-Za-z0-9\-._~/%#?=&]*$/;

const md = new Marked({
  gfm: true,
  breaks: false,
  renderer: {
    html({ text }) {
      return esc(text);
    },
    image({ text }) {
      return `<span class="md-image">${esc(text ? `[image: ${text}]` : "[image]")}</span>`;
    },
    link({ href, title, tokens }) {
      const inner = this.parser.parseInline(tokens);
      const t = title ? ` title="${esc(title)}"` : "";
      if (SITE_PATH.test(href)) return `<a href="${esc(href)}"${t}>${inner}</a>`;
      if (!/^https?:\/\//i.test(href)) return inner;
      return `<a href="${esc(href)}"${t} target="_blank" rel="noopener noreferrer nofollow">${inner}</a>`;
    },
    code({ text, lang }) {
      const name = (lang ?? "").trim().split(/\s+/)[0] || "text";
      const run = runnable(name);
      const runButton = run ? `<button type="button" data-run="${run}">run</button>` : "";
      return `<div class="md-code"><div class="md-code-head"><span>${esc(name)}</span><span class="md-code-actions">${runButton}<button type="button" data-copy>copy</button></span></div><pre><code>${esc(text)}</code></pre></div>`;
    },
  },
});

/** The answer as HTML; see the module comment for what is and is not allowed. */
export function renderMarkdown(src: string): string {
  return md.parse(src, { async: false });
}
