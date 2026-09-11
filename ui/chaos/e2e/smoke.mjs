// Browser smoke of the built UI: every page renders real data, no console
// errors, a scenario run streams to the end, a fault applies, load cancels,
// run-all queues, a schedule is created, fired by hand, paused and deleted.
// Run with `mise run ui:e2e` against the local cluster (UI_BASE overrides).
import { mkdirSync } from "node:fs";

import { chromium } from "playwright";

const BASE = process.env.UI_BASE ?? "http://chaos.localhost:18080";
const SHOTS = process.env.SHOTS ?? "e2e/shots";
mkdirSync(SHOTS, { recursive: true });

const browser = await chromium.launch();
const page = await browser.newPage({ viewport: { width: 1440, height: 1000 } });
const errors = [];
page.on("console", (m) => {
  if (m.type() === "error") errors.push(`console: ${m.text()}`);
});
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
page.on("requestfailed", (r) => {
  const t = r.failure()?.errorText;
  if (t !== "net::ERR_ABORTED") errors.push(`requestfailed: ${r.url()} ${t}`);
});

/** Full-page capture fails while a CSS animation runs, so capture with animations
 *  paused; a live re-render can still interrupt it, so try again. */
async function shot(options) {
  for (let attempt = 0; ; attempt++) {
    try {
      return await page.screenshot({ animations: "disabled", ...options });
    } catch (e) {
      if (attempt >= 2) throw e;
      await page.waitForTimeout(500);
    }
  }
}

const results = [];
async function step(name, fn) {
  const started = Date.now();
  console.error(`▶ ${name}`);
  try {
    await fn();
    results.push(`ok    ${name}`);
  } catch (e) {
    results.push(`FAIL  ${name}: ${String(e).split("\n")[0]}`);
  }
  console.error(`  ${((Date.now() - started) / 1000).toFixed(1)}s`);
}

await step("overview renders the KPI strip, stack and charts", async () => {
  await page.goto(`${BASE}/`);
  await page.getByText("Stack health").waitFor({ timeout: 15000 });
  await page.getByText("engine-1").first().waitFor();
  await page.getByText("Throughput").first().waitFor();
  await shot({ path: `${SHOTS}/overview.png`, fullPage: true });
});

await step("stack: fault dialog applies an error behaviour", async () => {
  await page.goto(`${BASE}/stack/`);
  await page.getByText("Stack integrity").waitFor({ timeout: 15000 });
  await page
    .getByRole("button", { name: /^fault$/i })
    .first()
    .click();
  await page.getByRole("dialog").waitFor();
  await page.getByRole("dialog").locator("select").first().selectOption("error");
  await page.getByRole("button", { name: "Apply" }).click();
  await page
    .getByText(/error unavailable 50%/)
    .first()
    .waitFor({ timeout: 10000 });
  await shot({ path: `${SHOTS}/stack-fault.png`, fullPage: true });
  await page
    .getByRole("button", { name: /^fault$/i })
    .first()
    .click();
  await page.getByRole("dialog").locator("select").first().selectOption("healthy");
  await page.getByRole("button", { name: "Apply" }).click();
  await page.waitForFunction(() => !document.body.innerText.includes("error unavailable 50%"), null, {
    timeout: 10000,
  });
});

await step("stack: stop and start the engine", async () => {
  await page
    .getByRole("button", { name: /^stop$/i })
    .first()
    .click();
  await page.getByText("Stopped").first().waitFor({ timeout: 10000 });
  await shot({
    path: `${SHOTS}/stack-stopped.png`,
    fullPage: true,
  });
  await page
    .getByRole("button", { name: /^start$/i })
    .first()
    .click();
  await page.waitForFunction(() => !document.body.innerText.includes("Stopped"), null, { timeout: 15000 });
});

/** The overview as the page sees it: the registered kinds and the check catalogue drive the assertions below. */
const overview = await page.evaluate(async () => {
  const r = await fetch("/api/chaos/v1/overview", { headers: { accept: "application/json" } });
  return r.json();
});
const addableKinds = overview.kinds.filter((k) => k.addable);
const targetKinds = overview.kinds.filter((k) => k.target);
const lastCheck = overview.validate.checks[overview.validate.checks.length - 1].name;

await step("stack: a protocol replica comes up on a fresh port and can be removed", async () => {
  await page.goto(`${BASE}/stack/`);
  const p1 = page.getByTestId("instance-protocol-1");
  await p1.waitFor({ timeout: 15000 });
  await p1.getByRole("button", { name: /replica/i }).click();
  const p2 = page.getByTestId("instance-protocol-2");
  await p2.waitFor({ timeout: 15000 });
  await p2.getByText("added").waitFor();
  await p2.getByText("Healthy").waitFor({ timeout: 15000 });
  await shot({ path: `${SHOTS}/stack-replica.png`, fullPage: true });
  await page.getByRole("button", { name: "Add instance" }).click();
  await page.getByRole("dialog").waitFor();
  // The kind picker lists every addable kind the API registers.
  await page.getByRole("combobox", { name: "Kind" }).click();
  for (const k of addableKinds) await page.getByRole("option", { name: k.label }).waitFor();
  await page.keyboard.press("Escape");
  await page.getByRole("button", { name: "Add and start" }).click();
  await page.getByTestId("instance-protocol-3").waitFor({ timeout: 15000 });
  await page.getByRole("button", { name: "Remove protocol-3" }).click();
  await page.getByTestId("instance-protocol-3").waitFor({ state: "hidden", timeout: 15000 });
  // The success toasts sit over the last rows on a tall stack; let them go.
  await page.getByText(/removed protocol-3/).waitFor({ state: "hidden", timeout: 10000 });
  await p2.getByRole("button", { name: "Remove protocol-2" }).click();
  await p2.waitFor({ state: "hidden", timeout: 15000 });
});

await step("scenarios list and editor check", async () => {
  await page.goto(`${BASE}/scenarios/`);
  await page.getByText("error_injection").first().waitFor();
  await page.goto(`${BASE}/scenarios/view/?id=error_injection`);
  await page.getByText("checks", { exact: true }).waitFor({ timeout: 10000 });
  await page.locator(".cm-editor .tok-string").first().waitFor({ timeout: 10000 });
  await shot({ path: `${SHOTS}/scenario.png`, fullPage: true });
  // The reference sheet inserts a block at the cursor and the file still checks.
  await page.getByRole("button", { name: "Reference", exact: true }).click();
  await page.getByRole("heading", { name: "Writing scenarios" }).waitFor({ timeout: 10000 });
  await shot({ path: `${SHOTS}/reference-sheet.png` });
  await page.getByRole("button", { name: "Log marker" }).click();
  await page.locator(".cm-editor").getByText("halfway").waitFor({ timeout: 5000 });
  await page.getByText("checks", { exact: true }).waitFor({ timeout: 10000 });
});

await step("knowledge base: categories, search, and an article with its contents", async () => {
  await page.goto(`${BASE}/kb/`);
  await page.getByRole("heading", { name: "Knowledge base" }).waitFor({ timeout: 10000 });
  await page.getByText("Start here").first().waitFor();
  await page.getByText("Observability").first().waitFor();
  await shot({ path: `${SHOTS}/kb.png`, fullPage: true });
  // A symptom from the runbook is found by its section.
  await page.getByLabel("Search the knowledge base").fill("TCP_NODELAY");
  const results = page.getByTestId("kb-results");
  await results.waitFor({ timeout: 5000 });
  await results.getByText("Runbook").first().waitFor();
  await shot({ path: `${SHOTS}/kb-search.png` });
  await results
    .getByRole("link", { name: /Runbook/ })
    .first()
    .click();
  await page.waitForURL(/kb\/view\/\?doc=docs%2Fchaos%2Frunbook/, { timeout: 10000 });
  await page.getByRole("heading", { name: "Runbook", level: 1 }).waitFor({ timeout: 10000 });
  await page.getByText("http_readyz").first().waitFor();
  await page.getByText("On this page").first().waitFor();
  await page.getByRole("link", { name: "Validate", exact: true }).first().waitFor();
  await shot({ path: `${SHOTS}/kb-article.png`, fullPage: true });
});

await step("knowledge base: the stress reference renders and in-app links resolve", async () => {
  await page.goto(`${BASE}/kb/view/?doc=docs%2Fchaos%2Fstress`);
  await page.getByRole("heading", { name: "The invariants" }).waitFor({ timeout: 10000 });
  await page.getByText("cascade_tombstones_counterparty").first().waitFor();
  // A relative markdown link to another page is an in-app route, with its anchor.
  await page.getByRole("link", { name: "commands.md" }).first().click();
  await page.waitForURL(/doc=docs%2Fchaos%2Fcommands#chaos-stress/, { timeout: 10000 });
  await page.getByRole("heading", { name: "chaos commands", level: 1 }).waitFor({ timeout: 10000 });
  // The old routes land in the knowledge base.
  await page.goto(`${BASE}/runbook/`);
  await page.waitForURL(/doc=docs%2Fchaos%2Frunbook/, { timeout: 10000 });
});

await step("campaigns list and editor check", async () => {
  await page.goto(`${BASE}/stress/`);
  await page.getByText("smoke", { exact: true }).first().waitFor({ timeout: 10000 });
  await page.getByText("its own stack").first().waitFor();
  await shot({ path: `${SHOTS}/campaigns.png`, fullPage: true });
  await page.goto(`${BASE}/stress/view/?id=smoke`);
  await page.getByText("checks", { exact: true }).first().waitFor({ timeout: 10000 });
  await page.locator(".cm-editor .tok-string").first().waitFor({ timeout: 10000 });
  await page.getByText("Owner workers").first().waitFor();
  await shot({ path: `${SHOTS}/campaign.png`, fullPage: true });
  await page.getByRole("button", { name: "Reference", exact: true }).click();
  await page.getByRole("heading", { name: "The invariants" }).waitFor({ timeout: 10000 });
  await page.getByRole("button", { name: "Operation mix" }).click();
  await page.locator(".cm-editor").getByText("idempotent_replay").waitFor({ timeout: 5000 });
  await page.getByText("checks", { exact: true }).first().waitFor({ timeout: 10000 });
});

await step("run the smoke campaign live to the end", async () => {
  await page.goto(`${BASE}/stress/`);
  const row = page.getByRole("row", { name: /smoke/ });
  await row.getByRole("button", { name: /^run$/i }).click();
  await page.waitForURL(/runs\/view\/\?id=/, { timeout: 10000 });
  await page.getByText("Lifecycle").waitFor({ timeout: 10000 });
  await page.getByText("Invariant evaluations").waitFor({ timeout: 10000 });
  await shot({ path: `${SHOTS}/stress-live.png`, fullPage: true });
  // The run ends either way; a finding is reported below, not hidden.
  await page
    .getByText(/^(passed|failed)$/, { exact: true })
    .first()
    .waitFor({ timeout: 90000 });
  await page.getByText("append_echo").waitFor();
  await page.getByText("held").first().waitFor();
  const findings = await page.locator('a[href^="/findings/view/"]').count();
  console.log("  findings on this run:", findings / 2);
  if (findings) throw new Error("the smoke campaign found something; open Findings");
  await shot({ path: `${SHOTS}/stress-done.png`, fullPage: true });
  // The stress entry under Campaigns is the one current Runs link for ?kind=stress.
  await page.goto(`${BASE}/runs/?kind=stress`);
  await page.getByRole("row", { name: /smoke/ }).first().waitFor({ timeout: 10000 });
  const current = page.locator('[data-sidebar="menu-sub-button"][data-active="true"]');
  await current.waitFor();
  if ((await current.count()) !== 1) throw new Error("more than one sidebar entry marked current");
});

await step("findings page renders, grouped and flat", async () => {
  await page.goto(`${BASE}/findings/`);
  await page.getByRole("heading", { name: "Findings" }).waitFor({ timeout: 10000 });
  await page.getByRole("tab", { name: /By signature/ }).waitFor();
  await page.getByRole("tab", { name: /Every finding/ }).click();
  await page.getByText("Filters").waitFor();
  await shot({ path: `${SHOTS}/findings.png`, fullPage: true });
});

await step("run a scenario live to the end", async () => {
  await page.goto(`${BASE}/scenarios/`);
  const row = page.getByRole("row", { name: /error_injection/ });
  await row.getByRole("button", { name: /^run$/i }).click();
  await page.waitForURL(/runs\/view\/\?id=/, { timeout: 10000 });
  await page.getByText("Lifecycle").waitFor({ timeout: 10000 });
  await shot({ path: `${SHOTS}/run-live.png`, fullPage: true });
  await page.getByText("passed", { exact: true }).first().waitFor({ timeout: 30000 });
  await page.getByText("max_error_rate").waitFor();
  await page
    .getByText(/set_behavior engine-1/)
    .first()
    .waitFor();
  await page.getByText("Latency grid").waitFor();
  await shot({ path: `${SHOTS}/run-done.png`, fullPage: true });
});

await step("run all queues every ready scenario and drains one at a time", async () => {
  await page.goto(`${BASE}/scenarios/`);
  await page.getByRole("button", { name: /run all \(\d+\)/i }).click();
  await page.waitForURL(/\/runs\//, { timeout: 10000 });
  await page.getByTestId("queue-panel").waitFor({ timeout: 10000 });
  await page
    .getByText(/\+\d+ queued/)
    .first()
    .waitFor();
  await shot({ path: `${SHOTS}/queue.png`, fullPage: true });
  await page.getByRole("button", { name: "Clear all" }).click();
  await page.getByTestId("queue-panel").waitFor({ state: "hidden", timeout: 10000 });
  // The run that already started finishes on its own.
  await page.getByText("running", { exact: true }).first().waitFor({ state: "hidden", timeout: 60000 });
});

await step("schedules: create from a scenario, run now, pause, delete", async () => {
  await page.goto(`${BASE}/schedules/?new=scenario:error_injection`);
  await page.getByRole("dialog").waitFor({ timeout: 10000 });
  await page.getByRole("button", { name: "Create schedule" }).click();
  const row = page.getByRole("row", { name: /error_injection on a schedule/ });
  await row.waitFor({ timeout: 10000 });
  await row.getByText("Every 15 minutes").waitFor();
  await row.getByRole("button", { name: /run now/i }).click();
  await page
    .getByText(/queued|started/)
    .first()
    .waitFor({ timeout: 10000 });
  await row.getByRole("switch").click();
  await row.getByText("paused").waitFor({ timeout: 10000 });
  await shot({ path: `${SHOTS}/schedules.png`, fullPage: true });
  page.once("dialog", (d) => d.accept());
  await row.getByRole("button", { name: "More" }).click();
  await page.getByRole("menuitem", { name: "Delete" }).click();
  await row.waitFor({ state: "hidden", timeout: 10000 });
  // A validate schedule round-trips through the API's null fields.
  await page.getByRole("button", { name: "New schedule" }).click();
  await page.getByRole("combobox", { name: "Runs" }).click();
  await page.getByRole("option", { name: "Validate" }).click();
  await page.getByRole("button", { name: "Create schedule" }).click();
  const vrow = page.getByRole("row", { name: /scheduled validate/ });
  await vrow.waitFor({ timeout: 10000 });
  page.once("dialog", (d) => d.accept());
  await vrow.getByRole("button", { name: "More" }).click();
  await page.getByRole("menuitem", { name: "Delete" }).click();
  await vrow.waitFor({ state: "hidden", timeout: 10000 });
  // A load schedule takes the load page's form: the mix by kind and a target per kind.
  await page.getByRole("button", { name: "New schedule" }).click();
  await page.getByRole("combobox", { name: "Runs" }).click();
  await page.getByRole("option", { name: "Load" }).click();
  await page.getByLabel("ledger_append").fill("3");
  await page.getByRole("combobox", { name: "Ledger target" }).waitFor({ timeout: 5000 });
  await page.getByRole("combobox", { name: "Protocol target" }).waitFor();
  await page.getByRole("button", { name: "Cancel" }).click();
  await page.getByText("running", { exact: true }).first().waitFor({ state: "hidden", timeout: 60000 });
});

await step("no login locally: no user menu; the Slack card reports its state", async () => {
  await page.goto(`${BASE}/schedules/`);
  const card = page.getByTestId("notify-card");
  await card.waitFor({ timeout: 10000 });
  // Either configured (channel shown) or off (how to turn it on).
  await card
    .getByText(/CHAOS_SLACK_WEBHOOK|post to/)
    .first()
    .waitFor();
  const me = await page.evaluate(async () => (await fetch("/api/chaos/v1/me")).json());
  if (me.user !== null) throw new Error(`expected no user locally, got ${JSON.stringify(me.user)}`);
  if ((await page.getByRole("button", { name: /signed in as/i }).count()) !== 0)
    throw new Error("user menu shown without a login");
});

await step("runs list filters by kind from the sidebar link", async () => {
  await page.goto(`${BASE}/runs/?kind=scenario`);
  await page
    .getByRole("row", { name: /error_injection/ })
    .first()
    .waitFor({ timeout: 10000 });
  await page.getByText("Filters").waitFor();
  // Exactly one Runs entry is current, the one whose query matches.
  const current = page.locator('[data-sidebar="menu-sub-button"][data-active="true"]');
  await current.waitFor();
  console.log("  current sidebar entry:", (await current.allInnerTexts()).join(" | "));
  if ((await current.count()) !== 1) throw new Error("more than one Runs entry marked current");
  await shot({ path: `${SHOTS}/runs.png`, fullPage: true });
  // One "<Kind> runs" entry per registered kind; the ledger one lists the ledger scenarios.
  for (const k of overview.kinds) await page.getByRole("link", { name: `${k.label} runs` }).waitFor();
  await page.getByRole("link", { name: "Ledger runs" }).click();
  await page.waitForURL(/service=ledger/);
  await page
    .getByRole("row", { name: /ledger_/ })
    .first()
    .waitFor({ timeout: 10000 });
});

await step("load page starts and cancels a run", async () => {
  await page.goto(`${BASE}/load/`);
  await page.getByLabel(/Duration/).fill("30s");
  await page.getByRole("button", { name: /run load/i }).click();
  await page.waitForURL(/runs\/view\/\?id=/, { timeout: 10000 });
  await page.getByRole("button", { name: /cancel run/i }).waitFor({ timeout: 10000 });
  await page.waitForTimeout(2500);
  await shot({ path: `${SHOTS}/load-live.png`, fullPage: true });
  await page.getByRole("button", { name: /cancel run/i }).click();
  await page.getByText("cancelled", { exact: true }).first().waitFor({ timeout: 20000 });
});

await step("validate runs against the deployed stack", async () => {
  await page.goto(`${BASE}/validate/`);
  // One input per kind with a validate target, and the check count in the copy.
  for (const k of targetKinds) await page.getByTestId(`target-${k.name}`).waitFor();
  await page.getByText(`${overview.validate.checks.length} checks`).waitFor();
  await page.getByRole("button", { name: /run validate/i }).click();
  await page.getByText(lastCheck).waitFor({ timeout: 20000 });
  await shot({ path: `${SHOTS}/validate.png`, fullPage: true });
});

await step("command palette opens with ctrl+k and lists scenarios", async () => {
  await page.keyboard.press("Control+k");
  await page.getByPlaceholder("Type a command or search...").waitFor({ timeout: 5000 });
  await page.getByText("Run a scenario").waitFor();
  await shot({ path: `${SHOTS}/command.png` });
  // Knowledge base pages are in the palette too.
  await page.getByPlaceholder("Type a command or search...").fill("runbook");
  await page
    .getByRole("option", { name: /Runbook/ })
    .first()
    .waitFor({ timeout: 5000 });
  await page.keyboard.press("Escape");
});

await step("dark mode toggles", async () => {
  await page.getByRole("button", { name: "Toggle theme" }).click();
  await page.getByRole("menuitem", { name: /^dark/i }).click();
  await page.waitForFunction(() => document.documentElement.classList.contains("dark"));
  await shot({
    path: `${SHOTS}/runbook-dark.png`,
    fullPage: true,
  });
});

await browser.close();
console.log(results.join("\n"));
console.log(errors.length ? `\nBROWSER ERRORS:\n${errors.join("\n")}` : "\nno browser errors");
process.exit(results.some((r) => r.startsWith("FAIL")) || errors.length ? 1 : 0);
