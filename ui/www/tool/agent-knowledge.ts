// Writes configs/llm/agents/site.knowledge.json: what the site guide (RFC 0011)
// knows. A `brief` the model service puts in front of every turn, and the full
// text of each page, of which it adds only the one the visitor is reading.
//
// Only what the public site already says: `src/data/site.ts` (the site's own
// facts) and the lab's public RFCs and studies. Drafts (`public: false`) are
// never read. A withheld fact stays withheld: `[REDACTED: reason]` becomes
// "[redacted: reason]", and the lab's own redaction scan runs over every
// document's text again here, so a leak fails this build as it fails the
// site's. The file is committed; `mise run ui:www:check` fails when it is
// stale (`mise run www:agent` rewrites it).
//
//   node --no-warnings tool/agent-knowledge.ts
import { existsSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

import { labs } from "../src/data/labs.ts";
import {
  about,
  chapters,
  company,
  languages,
  pipeline,
  playgrounds,
  principles,
  projects,
} from "../src/data/site.ts";
import { scan } from "./lab-redaction.ts";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../../..");
const out = join(root, "configs/llm/agents/site.knowledge.json");

/** The brief goes into every turn: keep it to a few thousand tokens. */
const MAX_BRIEF = 14_000;
/** One page's text, added when the visitor is on it. */
const MAX_PAGE = 16_000;

type Doc = {
  kind: "rfc" | "study";
  number: string;
  slug: string;
  path: string;
  title: string;
  status: string;
  summary: string;
  headline?: string;
  lab: string;
  body: string;
  lastLog?: string;
};

const errors: string[] = [];

function front(text: string): { meta: Record<string, string>; body: string } | null {
  const lines = text.split("\n");
  if (lines[0]?.trim() !== "---") return null;
  const meta: Record<string, string> = {};
  let i = 1;
  for (; i < lines.length && lines[i].trim() !== "---"; i++) {
    const m = /^([a-z_]+):\s*(.*)$/.exec(lines[i]);
    if (m) meta[m[1]] = m[2].trim().replace(/^"(.*)"$/, "$1");
  }
  return { meta, body: lines.slice(i + 1).join("\n") };
}

/** Markdown as plain prose for a model: withheld facts as their reasons, fences kept. */
function plain(body: string): string {
  return body
    .replace(/```redacted\n([\s\S]*?)```/g, (_, why: string) => `[redacted: ${why.trim()}]`)
    .replace(/\[REDACTED:\s*([^\]]*)\]/g, "[redacted: $1]")
    .replace(/\n{3,}/g, "\n\n")
    .trim();
}

function docs(): Doc[] {
  const denylist = (
    JSON.parse(readFileSync(join(root, "docs/lab/redaction.json"), "utf8")) as { words: string[] }
  ).words;
  const found: Doc[] = [];
  for (const [kind, dir, base] of [
    ["rfc", "docs/rfcs", "/lab/rfc/"],
    ["study", "docs/studies", "/lab/studies/"],
  ] as const) {
    const abs = join(root, dir);
    if (!existsSync(abs)) continue;
    for (const name of readdirSync(abs).sort()) {
      const m = /^(\d{4})-([a-z0-9-]+)\.md$/.exec(name);
      if (!m) continue;
      const parsed = front(readFileSync(join(abs, name), "utf8"));
      if (!parsed || parsed.meta.public !== "true") continue; // drafts are never read
      const body = plain(parsed.body);
      for (const h of scan(body.split("\n"), denylist)) {
        errors.push(`${dir}/${name}:${h.line} [${h.rule}] "${h.text}" (${h.why})`);
      }
      const log = /^##\s+Status log\s*$([\s\S]*)/m.exec(parsed.body)?.[1] ?? "";
      const bullets = log.split("\n").filter((l) => /^- \d{4}-\d{2}-\d{2}:/.test(l));
      found.push({
        kind,
        number: m[1],
        slug: `${m[1]}-${m[2]}`,
        path: `${base}${m[1]}-${m[2]}/`,
        title: parsed.meta.title ?? name,
        status: parsed.meta.status ?? "",
        summary: parsed.meta.summary ?? "",
        headline: parsed.meta.headline,
        lab: parsed.meta.lab ?? "",
        body,
        lastLog: bullets.at(-1)?.replace(/^- /, ""),
      });
    }
  }
  return found;
}

const lab = docs();
const person = company.person;

// The pages a visitor can be on, with the one line the brief gives each.
const pages: { path: string; title: string; line: string; text: string }[] = [
  {
    path: "/",
    title: "Home",
    line: `${person}: ${company.title}. What is being built, the playgrounds, the lab once public, and how to get in touch.`,
    text: [
      `${person}, ${company.title}.`,
      company.focus,
      company.availability,
      "Principles:",
      ...principles.map((p) => `- ${p.title}: ${p.body}`),
      "How the platform is built, layer by layer:",
      ...pipeline.map((p) => `- ${p.stage} (${p.name}): ${p.rows.map((r) => `${r.k} ${r.v}`).join("; ")}`),
    ].join("\n"),
  },
  {
    path: "/about/",
    title: "About",
    line: "Who the site is about: the story, the path in chapters, the languages; the CV is a PDF linked from there.",
    text: [
      ...about,
      "The path:",
      ...chapters.map((c) => `- ${c.when}, ${c.title}: ${c.body}${c.where ? ` (${c.where})` : ""}`),
      `Languages: ${languages.join(", ")}.`,
    ].join("\n"),
  },
  {
    path: "/open-source/",
    title: "Open source",
    line: "What shipped and is public: libraries and tools, with years and links.",
    text: projects.map((p) => `- ${p.name} (${p.year}, ${p.language}): ${p.what} ${p.href}`).join("\n"),
  },
  {
    path: "/playgrounds/",
    title: "Play",
    line: "Things to try in the browser: music tools that listen through the microphone and never upload, and Break it (paused).",
    text: playgrounds
      .map(
        (p) =>
          `- ${p.name}${p.paused ? " (paused, being rebuilt)" : ""}: ${p.what}${p.href && !p.paused ? ` ${p.href}` : ""}`,
      )
      .join("\n"),
  },
  {
    path: "/contact/",
    title: "Contact",
    line: "How to get in touch: the address, and what a message should say.",
    text: `Write to ${company.email}. ${company.availability}`,
  },
  {
    path: "/lab/",
    title: "Lab",
    line: "What is being built now, in the open: one card per lab, the latest status lines, every RFC and study.",
    text: labs.map((l) => `- ${l.name.en} (${l.href}): ${l.what.en}`).join("\n"),
  },
  ...labs.map((l) => ({
    path: l.href,
    title: l.name.en,
    line: `The ${l.id} lab: ${l.name.en}.`,
    text: [
      l.what.en,
      `Ways in: ${l.surfaces.join(", ")}.`,
      ...lab
        .filter((d) => d.lab === l.id)
        .map((d) => `- ${d.kind.toUpperCase()} ${d.number} ${d.title} (${d.status}): ${d.summary} ${d.path}`),
    ].join("\n"),
  })),
  ...lab.map((d) => ({
    path: d.path,
    title: `${d.kind === "rfc" ? "RFC" : "Study"} ${d.number}: ${d.title}`,
    line: d.summary,
    text: `${d.title} (${d.status}). ${d.summary}\n\n${d.body}`,
  })),
];

const brief = [
  `# The site: ${person}`,
  `${person}, ${company.title}. ${company.focus}`,
  company.availability,
  "",
  "## Pages",
  ...pages
    .filter((p) => !p.path.startsWith("/lab/rfc/") && !p.path.startsWith("/lab/studies/"))
    .map((p) => `- ${p.path} ${p.title}: ${p.line}`),
  "",
  "## The path",
  ...chapters.map((c) => `- ${c.when}: ${c.title}`),
  "",
  "## Open source",
  ...projects.map((p) => `- ${p.name} (${p.year}): ${p.what}`),
  "",
  "## The lab's documents",
  ...lab.map(
    (d) =>
      `- ${d.path} ${d.kind === "rfc" ? "RFC" : "Study"} ${d.number} "${d.title}" (${d.status}${d.headline ? `, ${d.headline}` : ""}): ${d.summary}${d.lastLog ? ` Latest: ${d.lastLog}` : ""}`,
  ),
].join("\n");

if (brief.length > MAX_BRIEF) {
  errors.push(`the brief is ${brief.length} bytes, over ${MAX_BRIEF}: shorten a summary or a line`);
}
if (errors.length) {
  console.error(
    `agent knowledge: ${errors.length} problem${errors.length === 1 ? "" : "s"}, nothing written`,
  );
  for (const e of errors) console.error(`  ${e}`);
  process.exit(1);
}

const knowledge = {
  note: "Generated by ui/www/tool/agent-knowledge.ts from src/data/site.ts and the public lab documents. Do not edit; run `mise run www:agent`.",
  brief,
  pages: Object.fromEntries(
    pages.map((p) => [
      p.path,
      {
        title: p.title,
        text:
          p.text.length > MAX_PAGE ? `${p.text.slice(0, MAX_PAGE)}\n[the rest of this page is cut]` : p.text,
      },
    ]),
  ),
};
writeFileSync(out, `${JSON.stringify(knowledge, null, 2)}\n`);
console.log(
  `agent knowledge: brief ${brief.length} bytes, ${pages.length} pages -> configs/llm/agents/site.knowledge.json`,
);
