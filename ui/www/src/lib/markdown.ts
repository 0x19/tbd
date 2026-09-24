import { Marked } from "marked";

/**
 * A model's answer as HTML, safely. The text is untrusted (anyone's prompt can
 * steer it), so nothing it writes may run or load anything:
 *
 * - raw HTML in the answer is shown as text, never parsed;
 * - an image is shown as its description and never fetched: the site loads
 *   nothing from elsewhere (`/legal/`), and an image URL is a way to report
 *   who read the answer;
 * - a link is kept only for `http(s)`, opens in a new tab, and carries
 *   `noopener noreferrer nofollow`; any other scheme (`javascript:`, `data:`)
 *   is its text alone;
 * - a code block is escaped text in a `<pre>`, with its language named and a
 *   copy button the page wires up (`data-copy`).
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
      if (!/^https?:\/\//i.test(href)) return inner;
      const t = title ? ` title="${esc(title)}"` : "";
      return `<a href="${esc(href)}"${t} target="_blank" rel="noopener noreferrer nofollow">${inner}</a>`;
    },
    code({ text, lang }) {
      const name = (lang ?? "").trim().split(/\s+/)[0] || "text";
      return `<div class="md-code"><div class="md-code-head"><span>${esc(name)}</span><button type="button" data-copy>copy</button></div><pre><code>${esc(text)}</code></pre></div>`;
    },
  },
});

/** The answer as HTML; see the module comment for what is and is not allowed. */
export function renderMarkdown(src: string): string {
  return md.parse(src, { async: false });
}
