// Bundles the repository's documentation into the UI as data, so the knowledge
// base is the same text the repo documents itself with: every page under docs/,
// the root README and ARCHITECTURE, the devops READMEs and the crate notes. Also
// keeps the three named exports the scenario and campaign editors insert from.
// Runs before dev, build, lint and typecheck (package.json); the output is
// gitignored. Edit the markdown, never src/generated/docs.ts.
import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, relative, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const out = resolve(here, "../src/generated/docs.ts");

/** Category per path, first match wins; this order is the knowledge base's order. */
const CATEGORIES = [
  {
    id: "start",
    title: "Start here",
    description: "What the repository is, how it fits together, and the everyday commands.",
    match: (p) =>
      ["README.md", "ARCHITECTURE.md", "docs/README.md", "docs/ci.md", "docs/local-cluster.md"].includes(p),
  },
  {
    id: "chaos",
    title: "Chaos",
    description: "The tool behind these pages: commands, scenarios, campaigns, the API and the runbook.",
    match: (p) => p.startsWith("docs/chaos/"),
  },
  {
    id: "observability",
    title: "Observability",
    description:
      "Metrics, traces, logs and profiles: where each signal comes from and how to follow one request.",
    match: (p) => p.startsWith("docs/observability/"),
  },
  {
    id: "services",
    title: "Services",
    description: "The contract of every service and the identity stack in front of them.",
    match: (p) => /^docs\/(protocol|ledger|auth|tbd|humans|engine)\//.test(p),
  },
  {
    id: "deployment",
    title: "Deployment",
    description: "Envoy, Kubernetes, the public edge and the compose stack.",
    match: (p) => p.startsWith("devops/"),
  },
  {
    id: "crates",
    title: "Crate notes",
    description: "Per-crate gotchas and boundaries, the notes an engineer reads before touching the code.",
    match: (p) => /^crates\/[^/]+\/CLAUDE\.md$/.test(p),
  },
  {
    id: "design",
    title: "Design notes",
    description: "Earlier idea material for the product direction. Kept for the reasoning trail; not a spec.",
    match: (p) => p.startsWith("docs/design/"),
    advisory: true,
  },
];

/** Which files are bundled: everything the categories claim, walked from these roots. */
function collect() {
  const files = ["README.md", "ARCHITECTURE.md"];
  const walk = (dir, keep) => {
    for (const entry of readdirSync(join(root, dir), { withFileTypes: true })) {
      const rel = join(dir, entry.name);
      if (entry.isDirectory()) {
        if (entry.name === "node_modules" || entry.name.startsWith(".")) continue;
        walk(rel, keep);
      } else if (entry.isFile() && keep(rel)) files.push(rel);
    }
  };
  walk("docs", (p) => p.endsWith(".md"));
  walk("devops", (p) => p.endsWith("/README.md"));
  walk("crates", (p) => /^crates\/[^/]+\/CLAUDE\.md$/.test(p));
  return files.filter((p) => CATEGORIES.some((c) => c.match(p)));
}

/** GitHub-style anchor, the same as `slug()` in markdown.tsx. */
const slug = (text) =>
  text
    .toLowerCase()
    .replace(/[`[\]]/g, "")
    .replace(/[^a-z0-9\s-]/g, "")
    .trim()
    .replace(/\s+/g, "-");

/** Headings outside fences, ids deduplicated in document order the way the renderer does it. */
function headings(text) {
  const out = [];
  const seen = new Map();
  let inFence = false;
  for (const line of text.split("\n")) {
    if (line.startsWith("```")) inFence = !inFence;
    if (inFence) continue;
    const m = /^(#{1,4})\s+(.*?)\s*$/.exec(line);
    if (!m) continue;
    const title = m[2].replace(/`/g, "").replace(/\*\*/g, "");
    let id = slug(m[2]) || "section";
    const n = seen.get(id) ?? 0;
    seen.set(id, n + 1);
    if (n) id = `${id}-${n}`;
    out.push({ level: m[1].length, title, id });
  }
  return out;
}

/** The first paragraph of prose after the title: no fences, tables, lists, quotes or HTML. */
function summary(text) {
  let inFence = false;
  let para = [];
  let seenTitle = false;
  for (const line of text.split("\n")) {
    if (line.startsWith("```")) {
      inFence = !inFence;
      continue;
    }
    if (inFence) continue;
    if (/^#\s/.test(line)) {
      seenTitle = true;
      continue;
    }
    if (!seenTitle) continue;
    if (/^(#|\||[-*] |\d+\. |>|<)/.test(line.trim()) || line.trim() === "") {
      if (para.length) break;
      continue;
    }
    para.push(line.trim());
  }
  const joined = para
    .join(" ")
    .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
    .replace(/[`*_]/g, "")
    .replace(/\s+/g, " ");
  if (joined.length <= 240) return joined;
  const cut = joined.slice(0, 240);
  const end = Math.max(cut.lastIndexOf(". "), cut.lastIndexOf("; "));
  return end > 80 ? cut.slice(0, end + 1) : `${cut.trimEnd()}…`;
}

/** Relative markdown links out of a doc, as repo paths (anchors dropped, directories to their README). */
function links(text, from) {
  const out = new Set();
  const dir = dirname(from);
  for (const m of text.matchAll(/\]\(([^)\s]+)\)/g)) {
    const href = m[1];
    if (/^(https?:|mailto:|#)/.test(href)) continue;
    let target = href.split("#")[0];
    if (!target) continue;
    target = relative(root, resolve(root, dir, target)).replaceAll("\\", "/");
    if (!target.endsWith(".md")) target = `${target.replace(/\/$/, "")}/README.md`;
    out.add(target);
  }
  return [...out];
}

/** The last commit that touched the file; null outside a checkout. */
function updatedAt(path) {
  try {
    return (
      execFileSync("git", ["log", "-1", "--format=%cI", "--", path], {
        cwd: root,
        encoding: "utf8",
      }).trim() || null
    );
  } catch {
    return null;
  }
}

const docs = collect()
  .map((path) => {
    const text = readFileSync(join(root, path), "utf8");
    const category = CATEGORIES.find((c) => c.match(path));
    const hs = headings(text);
    const h1 = hs.find((h) => h.level === 1);
    return {
      id: path.replace(/\.md$/, ""),
      path,
      title: h1?.title ?? path,
      summary: summary(text),
      category: category.id,
      generated: /^Generated by /m.test(text) || text.includes("Generated by `mise run"),
      words: text.split(/\s+/).filter(Boolean).length,
      updated: updatedAt(path),
      headings: hs.filter((h) => h.level >= 2),
      links: links(text, path),
      text,
    };
  })
  .sort((a, b) => a.path.localeCompare(b.path));

const ids = new Set(docs.map((d) => d.id));
for (const d of docs)
  d.links = d.links.map((p) => p.replace(/\.md$/, "")).filter((id) => ids.has(id) && id !== d.id);

const named = (p) => {
  const d = docs.find((x) => x.path === p);
  if (!d) throw new Error(`gen-docs: ${p} not found`);
  return d.text;
};
const categories = CATEGORIES.map(({ id, title, description, advisory }) => ({
  id,
  title,
  description,
  advisory: !!advisory,
}));

mkdirSync(dirname(out), { recursive: true });
writeFileSync(
  out,
  `// Generated by scripts/gen-docs.mjs from the repository's markdown. Do not edit.\n` +
    `export const scenariosDoc = ${JSON.stringify(named("docs/chaos/scenarios.md"))};\n` +
    `export const kindsDoc = ${JSON.stringify(named("docs/chaos/kinds.md"))};\n` +
    `export const stressDoc = ${JSON.stringify(named("docs/chaos/stress.md"))};\n` +
    `export const kbCategories = ${JSON.stringify(categories)};\n` +
    `export const kbDocs = ${JSON.stringify(docs)};\n`,
);
console.log(`gen-docs: ${docs.length} documents, ${docs.reduce((n, d) => n + d.text.length, 0)} chars`);
