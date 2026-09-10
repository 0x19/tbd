// Browser check of the identity stack (docs/auth/README.md): register a person on
// auth.<domain>, run an authorization-code + PKCE flow for the public client tbd-app,
// exchange the code, and read /userinfo. Run with `mise run auth:e2e`
// (AUTH_URL overrides the host; screenshots under e2e/shots/).
import { execFileSync } from "node:child_process";
import { createHash, randomBytes } from "node:crypto";
import { createServer } from "node:http";

import { chromium } from "playwright";
const BASE = process.env.AUTH_URL ?? "https://auth.proximity.is";
const email = `e2e-${Date.now()}@example.com`;
const password = "correct-horse-battery-staple-9";
const shots = new URL("./shots/", import.meta.url).pathname;
// IPv4 only: hosts without an IPv6 route get Cloudflare's AAAA answers first and fail.
const browser = await chromium.launch({ args: ["--disable-ipv6"] });
const ctx = await browser.newContext();
const page = await ctx.newPage();
const errors = [];
page.on("pageerror", (e) => errors.push(String(e)));
try {
  // 1. registration (two-step: traits first, then the method)
  await page.goto(`${BASE}/registration`, { waitUntil: "networkidle" });
  await page.screenshot({ path: `${shots}/1-registration.png` });
  await page.fill('input[name="traits.email"]', email);
  await page.fill('input[name="traits.name"]', "E2E Person");
  const pw = page.locator('input[name="password"]');
  if (!(await pw.count())) {
    // two-step: submit the traits, then choose password
    await page.click(
      'button[name="method"][value="profile"], button[name="screen"][value="credential-selection"]',
    );
    // Ory Elements swaps screens in place: wait for the method choice, pick password.
    await page.waitForSelector('button:has-text("Password"), input[name="password"]', {
      timeout: 15000,
    });
    const choose = page.locator('button:has-text("Password")').first();
    if (await choose.count()) await choose.click();
    await page.waitForSelector('input[name="password"]', { timeout: 15000 });
    await page.screenshot({ path: `${shots}/2-method.png` });
  }
  await page.fill('input[name="password"]', password);
  await page.click('button[name="method"][value="password"], button:has-text("Sign up")');
  // Elements submits in place and then follows Kratos' redirect to "/".
  await page.waitForURL((u) => !u.pathname.startsWith("/registration"), { timeout: 20000 });
  await page.waitForLoadState("networkidle");
  await page.screenshot({ path: `${shots}/3-after-signup.png` });
  console.log("after sign-up:", page.url());
  const who = await page.request.get(`${BASE}/sessions/whoami`);
  const session = await who.json();
  console.log("whoami:", who.status(), session.active, session.identity?.traits);

  // 2. authorization code + PKCE for the public client
  const verifier = randomBytes(32).toString("base64url");
  const challenge = createHash("sha256").update(verifier).digest("base64url");
  const redirect = "http://localhost:3001/callback";
  // A real listener for the client's redirect_uri: the browser follows Hydra's
  // redirect there and the code arrives in the query string.
  let code = null;
  let callbackError = null;
  const server = createServer((req, res) => {
    const u = new URL(req.url, "http://localhost:3001");
    code = u.searchParams.get("code");
    callbackError = u.searchParams.get("error_description");
    res.end("callback received");
  });
  await new Promise((r) => server.listen(3001, "127.0.0.1", r));
  const state = randomBytes(12).toString("base64url");
  const authUrl = `${BASE}/oauth2/auth?client_id=tbd-app&response_type=code&scope=${encodeURIComponent("openid offline_access email profile tbd.api")}&redirect_uri=${encodeURIComponent(redirect)}&state=${state}&code_challenge=${challenge}&code_challenge_method=S256&audience=tbd-api`;
  await page.goto(authUrl, { waitUntil: "networkidle" });
  // Kratos asks an already signed-in person to confirm their password before it
  // hands the identity to an OAuth2 client for the first time. Elements renders
  // the form after fetching the flow, so wait for the field rather than the page.
  if (page.url().includes("/login")) {
    const pwField = await page
      .waitForSelector('input[name="password"]', { timeout: 15000 })
      .catch(() => null);
    if (pwField) {
      await page.fill('input[name="password"]', password);
      await page.click('button[name="method"][value="password"], button:has-text("Sign in")');
      await page.waitForURL((u) => !u.pathname.startsWith("/login"), { timeout: 20000 });
      await page.waitForLoadState("networkidle");
    }
  }
  await page.screenshot({ path: `${shots}/4-after-authorize.png` });
  console.log("after authorize:", page.url(), "code:", code ? "yes" : "no");
  if (!code)
    throw new Error(
      `no authorization code: ${callbackError ?? (await page.textContent("body")).slice(0, 300)}`,
    );

  // 3. exchange
  const tok = await page.request.post(`${BASE}/oauth2/token`, {
    form: {
      grant_type: "authorization_code",
      client_id: "tbd-app",
      code,
      code_verifier: verifier,
      redirect_uri: redirect,
    },
  });
  const tokens = await tok.json();
  console.log("token exchange:", tok.status(), Object.keys(tokens));
  const claims = JSON.parse(Buffer.from(tokens.access_token.split(".")[1], "base64url").toString());
  console.log("access token claims:", { iss: claims.iss, sub: claims.sub, aud: claims.aud, scp: claims.scp });
  const id = JSON.parse(Buffer.from(tokens.id_token.split(".")[1], "base64url").toString());
  console.log("id token:", {
    sub: id.sub,
    email: id.email,
    aud: id.aud,
    role: id.role,
    grafana_role: id.grafana_role,
  });
  console.log("access token ext:", claims.ext);
  const info = await page.request.get(`${BASE}/userinfo`, {
    headers: { authorization: `Bearer ${tokens.access_token}` },
  });
  console.log("userinfo:", info.status(), await info.json());

  // 4. the gated UI hosts and the role model. A new person is a viewer: the
  // observability hosts let them in (Grafana as Viewer), the chaos admin UI does
  // not. After an operator makes them admin and they sign in again, it does.
  const base = new URL(BASE);
  const domain = base.hostname.replace(/^auth\./, "");
  if (domain !== base.hostname) {
    const host = (h, p = "/") => `${base.protocol}//${h}.${domain}${p}`;
    // Opens a gated host as this person. Envoy sends the browser through Hydra; Kratos
    // may ask for the password again for a new client. Returns the status the host
    // itself answers with once the browser is back on it.
    const status = async (url) => {
      let resp = await page.goto(url, { waitUntil: "networkidle" });
      if (page.url().startsWith(`${BASE}/login`)) {
        await page.waitForSelector('input[name="password"]', { timeout: 15000 });
        await page.fill('input[name="password"]', password);
        await page.click('button[name="method"][value="password"], button:has-text("Sign in")');
        await page.waitForURL((u) => !u.href.startsWith(`${BASE}/`), { timeout: 20000 }).catch(() => null);
        await page.waitForLoadState("networkidle");
        resp = await page.goto(url, { waitUntil: "networkidle" });
      }
      // The OAuth2 callback ends in a redirect chain; the body is the reliable verdict.
      const body = (await page.textContent("body")) ?? "";
      return body.includes("RBAC: access denied") ? 403 : resp?.status();
    };
    const grafanaRole = async () => {
      await status(host("grafana", "/api/user/orgs")); // completes any sign-in prompt
      return JSON.parse((await page.textContent("body")) ?? "[]")[0]?.role;
    };
    console.log(
      "grafana as viewer:",
      await status(host("grafana", "/api/user")),
      "role",
      await grafanaRole(),
    );
    const viewer = await status(host("chaosadmin"));
    console.log("chaosadmin as viewer:", viewer, "(403 expected)");
    if (viewer !== 403) throw new Error("a viewer reached the chaos admin UI");

    execFileSync("mise", ["run", "auth:role", email, "admin"], {
      cwd: `${import.meta.dirname}/../../..`,
      stdio: "ignore",
    });
    // Global sign-out revokes the OAuth2 sessions; each UI host's cookie lives on
    // until its five-minute token expires, or until that host's /oauth2/signout.
    await page.goto(`${BASE}/logout`, { waitUntil: "networkidle" });
    await page.waitForURL(/\/login/, { timeout: 15000 });
    for (const h of ["grafana", "chaosadmin"]) {
      await page.goto(host(h, "/oauth2/signout"), { waitUntil: "networkidle" }).catch(() => null);
    }
    const cookiesLeft = (await ctx.cookies(host("grafana"))).filter((c) => c.name.startsWith("tbd_")).length;
    console.log("signed out everywhere; grafana cookies left:", cookiesLeft, "(0 expected)");
    if (cookiesLeft) throw new Error("per-host sign-out left cookies behind");
    await page.goto(`${BASE}/login`, { waitUntil: "networkidle" });
    await page.waitForSelector('input[name="identifier"]', { timeout: 15000 });
    await page.fill('input[name="identifier"]', email);
    await page.fill('input[name="password"]', password);
    await page.click('button[name="method"][value="password"]');
    await page.waitForURL((u) => !u.pathname.startsWith("/login"), { timeout: 20000 });
    await page.waitForLoadState("networkidle");
    const admin = await status(host("chaosadmin"));
    const adminBody = (await page.textContent("body")) ?? "";
    const adminOk = admin === 200 && page.url().startsWith(host("chaosadmin")) && !adminBody.includes("RBAC");
    console.log("chaosadmin as admin:", admin, adminOk ? "ok" : `FAILED at ${page.url().slice(0, 60)}`);
    if (!adminOk) throw new Error("an admin could not reach the chaos admin UI");
    console.log("grafana as admin: role", await grafanaRole());
  }
  console.log(errors.length ? `page errors: ${errors}` : "no page errors");
} finally {
  await browser.close();
}
process.exit(0);
