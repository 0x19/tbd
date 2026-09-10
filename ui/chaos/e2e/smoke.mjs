// Browser smoke of the built UI: every page renders real data, no console
// errors, a scenario run streams to the end, a fault applies, load cancels.
// Run with `mise run ui:e2e` against the local cluster (UI_BASE overrides).
import { chromium } from "playwright";
import { mkdirSync } from "node:fs";

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

const results = [];
async function step(name, fn) {
  try {
    await fn();
    results.push(`ok    ${name}`);
  } catch (e) {
    results.push(`FAIL  ${name}: ${String(e).split("\n")[0]}`);
  }
}

await step("overview renders the KPI strip, stack and charts", async () => {
  await page.goto(`${BASE}/`);
  await page.getByText("Stack health").waitFor({ timeout: 15000 });
  await page.getByText("engine-1").first().waitFor();
  await page.getByText("Throughput").first().waitFor();
  await page.screenshot({ path: `${SHOTS}/overview.png`, fullPage: true });
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
  await page.screenshot({ path: `${SHOTS}/stack-fault.png`, fullPage: true });
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
  await page.screenshot({ path: `${SHOTS}/stack-stopped.png`, fullPage: true });
  await page
    .getByRole("button", { name: /^start$/i })
    .first()
    .click();
  await page.waitForFunction(() => !document.body.innerText.includes("Stopped"), null, { timeout: 15000 });
});

await step("scenarios list and editor check", async () => {
  await page.goto(`${BASE}/scenarios/`);
  await page.getByText("error_injection").first().waitFor();
  await page.goto(`${BASE}/scenarios/view/?id=error_injection`);
  await page.getByText("checks", { exact: true }).waitFor({ timeout: 10000 });
  await page.screenshot({ path: `${SHOTS}/scenario.png`, fullPage: true });
});

await step("run a scenario live to the end", async () => {
  await page.goto(`${BASE}/scenarios/`);
  const row = page.getByRole("row", { name: /error_injection/ });
  await row.getByRole("button", { name: /^run$/i }).click();
  await page.waitForURL(/runs\/view\/\?id=/, { timeout: 10000 });
  await page.getByText("Lifecycle").waitFor({ timeout: 10000 });
  await page.screenshot({ path: `${SHOTS}/run-live.png`, fullPage: true });
  await page.getByText("passed", { exact: true }).first().waitFor({ timeout: 30000 });
  await page.getByText("max_error_rate").waitFor();
  await page
    .getByText(/set_behavior engine-1/)
    .first()
    .waitFor();
  await page.getByText("Latency grid").waitFor();
  await page.screenshot({ path: `${SHOTS}/run-done.png`, fullPage: true });
});

await step("runs list filters by kind from the sidebar link", async () => {
  await page.goto(`${BASE}/runs/?kind=scenario`);
  await page
    .getByRole("row", { name: /error_injection/ })
    .first()
    .waitFor({ timeout: 10000 });
  await page.getByText("Filters").waitFor();
  await page.screenshot({ path: `${SHOTS}/runs.png`, fullPage: true });
});

await step("load page starts and cancels a run", async () => {
  await page.goto(`${BASE}/load/`);
  await page.getByLabel(/Duration/).fill("30s");
  await page.getByRole("button", { name: /run load/i }).click();
  await page.waitForURL(/runs\/view\/\?id=/, { timeout: 10000 });
  await page.getByRole("button", { name: /cancel run/i }).waitFor({ timeout: 10000 });
  await page.waitForTimeout(2500);
  await page.screenshot({ path: `${SHOTS}/load-live.png`, fullPage: true });
  await page.getByRole("button", { name: /cancel run/i }).click();
  await page.getByText("cancelled", { exact: true }).first().waitFor({ timeout: 20000 });
});

await step("validate runs against the deployed stack", async () => {
  await page.goto(`${BASE}/validate/`);
  await page.getByRole("button", { name: /run validate/i }).click();
  await page.getByText("grpc_protocol_ping").waitFor({ timeout: 20000 });
  await page.screenshot({ path: `${SHOTS}/validate.png`, fullPage: true });
});

await step("runbook renders with links", async () => {
  await page.goto(`${BASE}/runbook/`);
  await page.getByText("http_readyz fails").waitFor();
  await page.getByText("Dashboards").first().waitFor({ timeout: 10000 });
});

await step("command palette opens with ctrl+k and lists scenarios", async () => {
  await page.keyboard.press("Control+k");
  await page.getByPlaceholder("Search pages or run commands…").waitFor({ timeout: 5000 });
  await page.getByText("Run a scenario").waitFor();
  await page.screenshot({ path: `${SHOTS}/command.png` });
  await page.keyboard.press("Escape");
});

await step("dark mode toggles", async () => {
  await page.getByRole("button", { name: "Toggle theme" }).click();
  await page.waitForFunction(() => document.documentElement.classList.contains("dark"));
  await page.screenshot({ path: `${SHOTS}/runbook-dark.png`, fullPage: true });
});

await browser.close();
console.log(results.join("\n"));
console.log(errors.length ? `\nBROWSER ERRORS:\n${errors.join("\n")}` : "\nno browser errors");
process.exit(results.some((r) => r.startsWith("FAIL")) || errors.length ? 1 : 0);
