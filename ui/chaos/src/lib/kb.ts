// The knowledge base: the repository's markdown, bundled by scripts/gen-docs.mjs
// into src/generated/docs.ts, with the little structure the pages need on top:
// category order, link resolution, related documents and a search over sections.
import { kbCategories, kbDocs } from "@/generated/docs";

export type KbDoc = (typeof kbDocs)[number];
export type KbHeading = KbDoc["headings"][number];
export type KbCategory = (typeof kbCategories)[number] & { docs: KbDoc[] };

/** Hand-picked order inside a category; anything else follows, READMEs first, then by path. */
const ORDER: string[] = [
  "README",
  "ARCHITECTURE",
  "docs/README",
  "docs/local-cluster",
  "docs/ci",
  "docs/chaos/README",
  "docs/chaos/runbook",
  "docs/chaos/scenarios",
  "docs/chaos/stress",
  "docs/chaos/commands",
  "docs/chaos/api",
  "docs/chaos/config",
  "docs/chaos/kinds",
  "docs/chaos/extending",
  "docs/chaos/architecture",
  "docs/chaos/ui",
  "docs/observability/README",
  "docs/protocol/README",
  "docs/ledger/README",
  "docs/auth/README",
  "docs/tbd/README",
  "devops/README",
  "devops/envoy/README",
  "devops/k8s/README",
  "devops/edge/README",
  "docs/design/README",
];

function rank(d: KbDoc): number {
  const i = ORDER.indexOf(d.id);
  if (i >= 0) return i;
  return 1000 + (d.id.endsWith("/README") ? 0 : 1);
}

export const categories: KbCategory[] = kbCategories.map((c) => ({
  ...c,
  docs: kbDocs
    .filter((d) => d.category === c.id)
    .sort((a, b) => rank(a) - rank(b) || a.path.localeCompare(b.path)),
}));

/** Every document in knowledge-base order (category, then rank). */
export const ordered: KbDoc[] = categories.flatMap((c) => c.docs);

const byIdMap = new Map(kbDocs.map((d) => [d.id, d]));
export const byId = (id: string | null | undefined): KbDoc | undefined => (id ? byIdMap.get(id) : undefined);
export const categoryOf = (d: KbDoc): KbCategory => categories.find((c) => c.id === d.category)!;

/** The in-app route of a document, with an optional heading. */
export const docHref = (id: string, hash = "") => `/kb/view/?doc=${encodeURIComponent(id)}${hash}`;

function dirname(p: string): string {
  const i = p.lastIndexOf("/");
  return i < 0 ? "" : p.slice(0, i);
}

/** Resolve a `..`/`.` path the way a file system would. */
function normalise(path: string): string {
  const out: string[] = [];
  for (const part of path.split("/")) {
    if (part === "" || part === ".") continue;
    if (part === "..") out.pop();
    else out.push(part);
  }
  return out.join("/");
}

/**
 * Where a markdown link in `from` goes: an external URL as is, `#anchor` as is,
 * another bundled document as its in-app route, and `null` for a relative link
 * to something the knowledge base does not carry (shown as plain text).
 */
export function resolveLink(from: KbDoc, href: string): string | null {
  if (/^(https?:|mailto:)/.test(href) || href.startsWith("#")) return href;
  const [target = "", hash] = href.split("#");
  let path = normalise(`${dirname(from.path)}/${target}`);
  if (!path.endsWith(".md")) path = `${path.replace(/\/$/, "")}/README.md`;
  const id = path.replace(/\.md$/, "");
  if (!byIdMap.has(id)) return null;
  return docHref(id, hash ? `#${hash}` : "");
}

/** Documents this one links to, and documents that link to it. */
export function related(d: KbDoc): { to: KbDoc[]; from: KbDoc[] } {
  const to = d.links.map((id) => byIdMap.get(id)).filter((x): x is KbDoc => !!x);
  const from = ordered.filter((x) => x.id !== d.id && x.links.includes(d.id));
  return { to, from };
}

/** Previous and next document in the category. */
export function neighbours(d: KbDoc): { prev: KbDoc | null; next: KbDoc | null } {
  const docs = categoryOf(d).docs;
  const i = docs.findIndex((x) => x.id === d.id);
  return { prev: docs[i - 1] ?? null, next: docs[i + 1] ?? null };
}

export const readMinutes = (words: number) => Math.max(1, Math.round(words / 220));

// ------------------------------------------------------------------ search --

export type Hit = {
  doc: KbDoc;
  heading: KbHeading | null;
  /** Plain text around the first match, for the result row. */
  snippet: string;
  score: number;
};

type Section = { doc: KbDoc; heading: KbHeading | null; body: string; lower: string };

/** Markdown to plain text, enough to search and to show a snippet. */
function plain(md: string): string {
  return md
    .replace(/```[\s\S]*?```/g, " ")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/^\s*\|?[-:| ]+\|?\s*$/gm, " ")
    .replace(/[|`*>#]/g, " ")
    .replace(/\s+/g, " ")
    .trim();
}

/** On equal hits the pages an operator reads first come first; agent notes and idea material last. */
const CATEGORY_BOOST: Record<string, number> = {
  start: 2,
  chaos: 3,
  observability: 2,
  services: 2,
  deployment: 1,
  crates: 0,
  design: -1,
};

let sections: Section[] | null = null;

/** Split every document into its headed sections, once. */
function index(): Section[] {
  if (sections) return sections;
  sections = [];
  for (const doc of ordered) {
    let inFence = false;
    let heading: KbHeading | null = null;
    let buf: string[] = [];
    let hi = 0;
    const flush = () => {
      const body = plain(buf.join("\n"));
      if (body || heading) sections!.push({ doc, heading, body, lower: body.toLowerCase() });
      buf = [];
    };
    for (const line of doc.text.split("\n")) {
      if (line.startsWith("```")) inFence = !inFence;
      const m = !inFence && /^(#{2,4})\s+(.*?)\s*$/.exec(line);
      if (m) {
        flush();
        heading = doc.headings[hi++] ?? null;
        continue;
      }
      buf.push(line);
    }
    flush();
  }
  return sections;
}

function terms(query: string): string[] {
  return [
    ...new Set(
      query
        .toLowerCase()
        .split(/[^a-z0-9_./:-]+/)
        .filter((t) => t.length >= 2),
    ),
  ];
}

function snippetAround(body: string, lower: string, term: string): string {
  const at = lower.indexOf(term);
  if (at < 0) return body.slice(0, 160);
  const start = Math.max(0, at - 70);
  const end = Math.min(body.length, at + term.length + 110);
  return `${start ? "…" : ""}${body.slice(start, end)}${end < body.length ? "…" : ""}`;
}

/**
 * Rank sections by the query: every term must appear somewhere in the document
 * title, the section heading or the section body; title and heading matches
 * weigh more than body occurrences.
 */
export function search(query: string, limit = 20): Hit[] {
  const ts = terms(query);
  if (!ts.length) return [];
  const hits: Hit[] = [];
  for (const s of index()) {
    const title = s.doc.title.toLowerCase();
    const heading = s.heading?.title.toLowerCase() ?? "";
    let score = 0;
    let first: string | null = null;
    for (const t of ts) {
      let hit = 0;
      if (title.includes(t)) hit += 8;
      if (heading.includes(t)) hit += 5;
      let n = 0;
      let i = s.lower.indexOf(t);
      while (i >= 0 && n < 6) {
        n++;
        i = s.lower.indexOf(t, i + t.length);
      }
      if (n) {
        hit += n;
        first ??= t;
      }
      if (!hit) {
        score = 0;
        break;
      }
      score += hit;
    }
    if (!score) continue;
    score += CATEGORY_BOOST[s.doc.category] ?? 0;
    hits.push({
      doc: s.doc,
      heading: s.heading,
      snippet: snippetAround(s.body, s.lower, first ?? ts[0]!),
      score,
    });
  }
  hits.sort((a, b) => b.score - a.score || a.doc.title.localeCompare(b.doc.title));
  return hits.slice(0, limit);
}

/** Split `text` around the query terms so the result row can highlight them. */
export function highlight(text: string, query: string): { text: string; hit: boolean }[] {
  const ts = terms(query);
  if (!ts.length) return [{ text, hit: false }];
  const re = new RegExp(`(${ts.map((t) => t.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})`, "ig");
  return text
    .split(re)
    .filter((s) => s !== "")
    .map((s) => ({ text: s, hit: ts.includes(s.toLowerCase()) }));
}
