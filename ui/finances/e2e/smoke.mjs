// Browser smoke of the built finance UI through Envoy: every page renders,
// no console errors, the scope toggle changes the totals. Run with
// `mise run ui:finances:e2e` against the local cluster (UI_BASE overrides).
//
// The host has no open variant -- it is one person's money -- so the smoke
// needs a signed-in admin session. Until the register-promote-purge harness
// from `auth:e2e` is wired in here, pass the `tbd_id` cookie of a signed-in
// browser in E2E_COOKIE; without it the smoke says so and exits non-zero
// rather than passing on the sign-in redirect.
import { mkdirSync } from "node:fs";

import { chromium } from "playwright";

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

const cookie = process.env.E2E_COOKIE;
if (!cookie) {
  console.error("E2E_COOKIE is not set: sign in to the finance host in a browser and pass its tbd_id cookie");
  process.exit(2);
}
await page.context().addCookies([{ name: "tbd_id", value: cookie, url: BASE }]);

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
