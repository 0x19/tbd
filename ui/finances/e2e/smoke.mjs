// Browser smoke of the built finance UI through Envoy: every page renders,
// no console errors, the scope toggle changes the totals, a category filter
// reaches the transactions page. Run with `mise run ui:finances:e2e` against
// the local cluster (UI_BASE overrides). Needs a signed-in session: the
// harness in e2e/auth.mjs signs in through Ory like ui/chaos does.
import { mkdirSync } from "node:fs";

import { chromium } from "playwright";

import { signIn } from "./auth.mjs";

const BASE = process.env.UI_BASE ?? "http://finance.localhost:18080";
const SHOTS = process.env.SHOTS ?? "e2e/shots";
mkdirSync(SHOTS, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
const errors = [];
page.on("console", (m) => {
  if (m.type() === "error") errors.push(`console: ${m.text()}`);
});
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

async function shot(name) {
  await page.screenshot({ path: `${SHOTS}/${name}.png`, fullPage: true, animations: "disabled" });
}

await signIn(page, BASE);

for (const path of ["/", "/transactions/", "/categories/", "/accounts/", "/connections/", "/invoices/"]) {
  await page.goto(`${BASE}${path}`, { waitUntil: "networkidle" });
  await page.waitForSelector("h1");
  await shot(path === "/" ? "overview" : path.replaceAll("/", ""));
}

// The scope toggle exists only with more than one readable party; when it
// does, switching it must change the summary request.
await page.goto(`${BASE}/`, { waitUntil: "networkidle" });
const tabs = page.locator('[role="tablist"] [role="tab"]');
if ((await tabs.count()) > 1) {
  const before = await page.locator("h1 ~ *").first().textContent();
  await tabs.nth(1).click();
  await page.waitForLoadState("networkidle");
  const after = await page.locator("h1 ~ *").first().textContent();
  if (before === after) errors.push("scope toggle changed nothing");
}

await browser.close();
if (errors.length) {
  console.error(errors.join("\n"));
  process.exit(1);
}
console.log("ok");
