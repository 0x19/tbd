// Renders `docs/rfcs/` and `docs/studies/` into `src/generated/lab/` for the
// site's lab, and refuses to write anything when a public page leaks
// (tool/lab-redaction.ts). Runs as `pnpm gen` before dev, build, lint and
// typecheck, and as `mise run www:lab`; Node's own type stripping, no bundler.
//
//   node --no-warnings tool/lab-data.ts
//
// Markdown becomes static HTML here, at build time, with `marked`: a page is a
// document and ships no client JavaScript for its prose. Only the repository's
// own markdown is ever rendered, which is why raw HTML passing through is not
// a concern beyond the `embed` rule.
import { existsSync, mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { Marked } from "marked";

import type { LabDoc, LabEntry, LabKind, LabStatus } from "../src/lib/lab.ts";
import { scan } from "./lab-redaction.ts";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const out = resolve(here, "../src/generated/lab");

const SOURCES: { kind: LabKind; dir: string; base: string; statuses: LabStatus[] }[] = [
  { kind: "rfc", dir: "docs/rfcs", base: "/lab/rfc/", statuses: ["open", "decided", "superseded"] },
  {
    kind: "study",
    dir: "docs/studies",
    base: "/lab/studies/",
    statuses: ["running", "measured", "published"],
  },
];

const REQUIRED = ["title", "status", "date", "public", "summary"] as const;
const OPTIONAL = ["supersedes", "rfc", "headline", "headline_note"] as const;

type Front = Record<string, string | boolean>;

const errors: string[] = [];
const fail = (path: string, line: number | null, message: string) => {
  errors.push(`${path}${line ? `:${line}` : ""} ${message}`);
};

/** `---` … `---` at the top of the file: `key: value` lines, scalars only. */
function frontMatter(path: string, text: string): { front: Front; body: string; bodyStart: number } | null {
  const lines = text.split("\n");
  if (lines[0]?.trim() !== "---") {
    fail(path, 1, "the file must start with a `---` front matter block");
    return null;
  }
  const front: Front = {};
  let i = 1;
  for (; i < lines.length; i++) {
    const line = lines[i];
    if (line.trim() === "---") break;
    const m = /^([a-z_]+):\s*(.*)$/.exec(line);
    if (!m) {
      fail(path, i + 1, `not a \`key: value\` line: ${line}`);
      continue;
    }
    const [, key, rawValue] = m;
    let value: string | boolean = rawValue.trim();
    if (/^".*"$/.test(value)) value = value.slice(1, -1);
    if (value === "true") value = true;
    else if (value === "false") value = false;
    if (![...REQUIRED, ...OPTIONAL].includes(key as never)) fail(path, i + 1, `unknown key \`${key}\``);
    front[key] = value;
  }
  if (i >= lines.length) {
    fail(path, null, "the front matter never closes");
    return null;
  }
  return { front, body: lines.slice(i + 1).join("\n"), bodyStart: i + 2 };
}

/** GitHub-style anchor, the same as the chaos knowledge base uses. */
const slugify = (text: string) =>
  text
    .toLowerCase()
    .replace(/[`[\]]/g, "")
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");

const esc = (s: string) =>
  s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

/** Per-document state the renderer writes into. */
let current: { toc: LabDoc["toc"]; seen: Map<string, number> } = { toc: [], seen: new Map() };

const marked = new Marked({
  gfm: true,
  renderer: {
    heading({ tokens, depth }) {
      const title = this.parser.parseInline(tokens);
      let id = slugify(title.replace(/<[^>]+>/g, "")) || "section";
      const n = current.seen.get(id) ?? 0;
      current.seen.set(id, n + 1);
      if (n) id = `${id}-${n}`;
      if (depth <= 4) current.toc.push({ level: depth, id, title: title.replace(/<[^>]+>/g, "") });
      return `<h${depth} id="${id}">${title}</h${depth}>\n`;
    },
    code({ text, lang, escaped }) {
      if (lang === "redacted") {
        const reason = text.trim();
        return `<div class="redacted-block" role="img" title="${esc(reason)}" aria-label="redacted: ${esc(reason)}"><span>redacted</span></div>\n`;
      }
      const cls = lang ? ` class="language-${esc(lang)}"` : "";
      return `<pre><code${cls}>${escaped ? text : esc(text)}</code></pre>\n`;
    },
  },
  extensions: [
    {
      name: "redacted",
      level: "inline",
      start(src: string) {
        const i = src.indexOf("[REDACTED:");
        return i < 0 ? undefined : i;
      },
      tokenizer(src: string) {
        const m = /^\[REDACTED:\s*([^\]]*)\]/.exec(src);
        return m ? { type: "redacted", raw: m[0], reason: m[1].trim() } : undefined;
      },
      renderer(token) {
        const reason = String((token as unknown as { reason: string }).reason);
        return `<span class="redacted" role="img" title="${esc(reason)}" aria-label="redacted: ${esc(reason)}"></span>`;
      },
    },
  ],
});

const denylist = (() => {
  const file = join(root, "docs/lab/redaction.json");
  const parsed = JSON.parse(readFileSync(file, "utf8")) as { words?: unknown };
  if (!Array.isArray(parsed.words) || !parsed.words.every((w) => typeof w === "string")) {
    throw new Error(`${relative(root, file)}: "words" must be an array of strings`);
  }
  return parsed.words as string[];
})();

const entries: LabEntry[] = [];
const docs: Record<string, LabDoc> = {};
const publicSlugs = new Set<string>();
const privateSlugs = new Set<string>();

for (const source of SOURCES) {
  const dir = join(root, source.dir);
  if (!existsSync(dir)) continue;
  const numbers = new Map<string, string>();
  for (const name of readdirSync(dir).sort()) {
    if (!name.endsWith(".md") || name === "README.md") continue;
    const path = relative(root, join(dir, name));
    const m = /^(\d{4})-([a-z0-9-]+)\.md$/.exec(name);
    if (!m) {
      fail(path, null, "the file name must be NNNN-slug.md");
      continue;
    }
    const [, number, rest] = m;
    const slug = `${number}-${rest}`;
    if (numbers.has(number)) fail(path, null, `number ${number} is already ${numbers.get(number)}`);
    numbers.set(number, name);

    const text = readFileSync(join(dir, name), "utf8");
    const parsed = frontMatter(path, text);
    if (!parsed) continue;
    const { front, body, bodyStart } = parsed;
    for (const key of REQUIRED) if (!(key in front)) fail(path, null, `missing \`${key}\``);
    if (typeof front.public !== "boolean") fail(path, null, "`public` must be true or false");
    if (!source.statuses.includes(front.status as LabStatus)) {
      fail(path, null, `\`status\` must be one of ${source.statuses.join(", ")}`);
    }
    if (typeof front.date !== "string" || !/^\d{4}-\d{2}-\d{2}$/.test(front.date)) {
      fail(path, null, "`date` must be YYYY-MM-DD");
    }
    if (front.public !== true) {
      privateSlugs.add(slug);
      console.log(`skipped (private): ${path}`);
      continue;
    }
    publicSlugs.add(slug);

    // The check: front matter values and the body, line by line.
    const frontLines = text.split("\n").slice(0, bodyStart - 1);
    const bodyLines = body.split("\n");
    const hits = [
      ...scan(frontLines, denylist),
      ...scan(bodyLines, denylist).map((h) => ({ ...h, line: h.line + bodyStart - 1 })),
    ];
    for (const h of hits) fail(path, h.line, `[${h.rule}] "${h.text}" (${h.why})`);

    current = { toc: [], seen: new Map() };
    const html = marked.parse(body, { async: false });
    const entry: LabEntry = {
      kind: source.kind,
      number,
      slug,
      href: `${source.base}${slug}/`,
      title: String(front.title),
      status: front.status as LabStatus,
      date: String(front.date),
      summary: String(front.summary),
    };
    if (typeof front.supersedes === "string") entry.supersedes = front.supersedes;
    if (typeof front.rfc === "string") entry.rfc = front.rfc;
    if (typeof front.headline === "string") entry.headline = front.headline;
    if (typeof front.headline_note === "string") entry.headlineNote = front.headline_note;
    entries.push(entry);
    docs[`${source.kind}/${slug}`] = { ...entry, html, toc: current.toc };
    console.log(`rendered: ${path} -> ${entry.href}`);
  }
}

// Cross references: a public page may point only at public pages.
for (const e of entries) {
  for (const [key, target] of [
    ["supersedes", e.supersedes],
    ["rfc", e.rfc],
  ] as const) {
    if (!target) continue;
    if (privateSlugs.has(target))
      fail(`${e.kind} ${e.slug}`, null, `\`${key}\` names a private page (${target})`);
    else if (!publicSlugs.has(target))
      fail(`${e.kind} ${e.slug}`, null, `\`${key}\` names no page (${target})`);
  }
  if (e.supersedes) {
    const older = entries.find((o) => o.slug === e.supersedes);
    if (older) {
      older.supersededBy = e.slug;
      const doc = docs[`${older.kind}/${older.slug}`];
      if (doc) doc.supersededBy = e.slug;
    }
  }
}

if (errors.length) {
  console.error(`lab: ${errors.length} problem${errors.length === 1 ? "" : "s"}, nothing written`);
  for (const e of errors) console.error(`  ${e}`);
  process.exit(1);
}

const rfcs = entries.filter((e) => e.kind === "rfc").sort((a, b) => b.number.localeCompare(a.number));
const studies = entries
  .filter((e) => e.kind === "study")
  .sort((a, b) => b.date.localeCompare(a.date) || b.number.localeCompare(a.number));

const banner = `// Generated by tool/lab-data.ts from docs/rfcs and docs/studies. Do not edit.
// The source is the markdown; run \`pnpm gen\` (or \`mise run www:lab\`) after changing it.
`;
mkdirSync(out, { recursive: true });
writeFileSync(
  join(out, "index.ts"),
  `${banner}import type { LabEntry } from "@/lib/lab";

export const rfcs: LabEntry[] = ${JSON.stringify(rfcs, null, 2)};

export const studies: LabEntry[] = ${JSON.stringify(studies, null, 2)};
`,
);
writeFileSync(
  join(out, "docs.ts"),
  `${banner}import type { LabDoc } from "@/lib/lab";

export const docs: Record<string, LabDoc> = ${JSON.stringify(docs, null, 2)};
`,
);
console.log(
  `lab: ${rfcs.length} RFC${rfcs.length === 1 ? "" : "s"}, ${studies.length} stud${studies.length === 1 ? "y" : "ies"} -> ${relative(root, out)}`,
);
