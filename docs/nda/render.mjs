// Renders docs/nda/nda.{hr,en}.md to docs/nda/nda.{hr,en}.pdf (A4, numbered pages), which
// are committed next to their sources; NDA_OUT=<dir> writes elsewhere.
// Run through `mise run nda:pdf`; it uses the Playwright Chromium already installed
// for the finances UI's e2e tests (ui/finances).
// Markdown becomes HTML through `npx marked` (fetched once, then cached by npm).
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { createRequire } from "node:module";

const here = dirname(fileURLToPath(import.meta.url));
const root = resolve(here, "../..");
const out = process.env.NDA_OUT ?? here;
// Playwright is a dependency of ui/finances, so resolve it from there, whatever the cwd.
const { chromium } = createRequire(resolve(root, "ui/finances/package.json"))("playwright");
mkdirSync(out, { recursive: true });

const css = `
  @page { size: A4; margin: 22mm 20mm 24mm 20mm; }
  body { font: 11pt/1.45 "Liberation Serif", "DejaVu Serif", serif; color: #111; }
  h1 { font-size: 15pt; text-align: center; letter-spacing: .04em; margin: 0 0 1.4em; }
  h2 { font-size: 11.5pt; margin: 1.6em 0 .6em; page-break-after: avoid; }
  p { margin: 0 0 .55em; text-align: justify; }
  table { width: 100%; border-collapse: collapse; margin-top: 2em; page-break-inside: avoid; font-size: 10pt; }
  th, td { padding: .35em .5em; text-align: left; vertical-align: top; border-bottom: 1px solid #bbb; }
  th { font-weight: bold; }
`;

const browser = await chromium.launch();
try {
  for (const lang of ["hr", "en"]) {
    const md = resolve(here, `nda.${lang}.md`);
    // The blanks are runs of underscores, which Markdown would pair up as emphasis
    // and swallow; escape them so they print as lines to write on.
    const text = readFileSync(md, "utf8").replace(/_{3,}/g, (m) => "\\_".repeat(m.length));
    const body = execFileSync("npx", ["-y", "marked@18"], { input: text, encoding: "utf8" });
    const html = `<!doctype html><html lang="${lang}"><head><meta charset="utf-8"><style>${css}</style></head><body>${body}</body></html>`;
    const page = await browser.newPage();
    await page.setContent(html, { waitUntil: "load" });
    const pdf = resolve(out, `nda.${lang}.pdf`);
    await page.pdf({
      path: pdf,
      format: "A4",
      printBackground: true,
      preferCSSPageSize: true,
      displayHeaderFooter: true,
      headerTemplate: "<span></span>",
      footerTemplate:
        '<div style="width:100%;text-align:center;font:9pt \'Liberation Serif\',serif;color:#666;">' +
        '<span class="pageNumber"></span> / <span class="totalPages"></span></div>',
    });
    await page.close();
    process.stdout.write(`${pdf}\n`);
  }
} finally {
  await browser.close();
}
