// Browser check of the identity stack (docs/auth/README.md): register a person on
// auth.<domain>, run an authorization-code + PKCE flow for the public client tbd-app,
// exchange the code, and read /userinfo. Run with `mise run auth:e2e`
// (AUTH_URL overrides the host; screenshots under e2e/shots/).
import { createHash, randomBytes } from "node:crypto";
import { createServer } from "node:http";

import { chromium } from "playwright";
const BASE = process.env.AUTH_URL ?? "https://auth.proximity.is";
const email = `e2e-${Date.now()}@example.com`;
const password = "correct-horse-battery-staple-9";
const shots = new URL("./shots/", import.meta.url).pathname;
// IPv4 only: hosts without an IPv6 route get Cloudflare's AAAA answers first and fail.
const browser = await chromium.launch({ args: ["--disable-ipv6"] });
const page = await browser.newPage();
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
    await page.waitForLoadState("networkidle");
    await page.screenshot({ path: `${shots}/2-method.png` });
  }
  await page.fill('input[name="password"]', password);
  await page.click('button[name="method"][value="password"]');
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
  // hands the identity to an OAuth2 client for the first time.
  if (page.url().includes("/login") && (await page.locator('input[name="password"]').count())) {
    await page.fill('input[name="password"]', password);
    await page.click('button[name="method"][value="password"]');
    await page.waitForLoadState("networkidle");
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
  console.log("id token:", { sub: id.sub, email: id.email, aud: id.aud });
  const info = await page.request.get(`${BASE}/userinfo`, {
    headers: { authorization: `Bearer ${tokens.access_token}` },
  });
  console.log("userinfo:", info.status(), await info.json());

  // 4. the gated UI hosts: Envoy's OAuth2 filter sends the browser to sign in
  // (the Kratos session is still there, so no password prompt) and back.
  const base = new URL(BASE);
  const domain = base.hostname.replace(/^auth\./, "");
  if (domain !== base.hostname) {
    for (const [host, path, marker] of [
      ["chaosadmin", "/", "chaos"],
      ["grafana", "/api/user", '"email"'],
    ]) {
      const url = `${base.protocol}//${host}.${domain}${path}`;
      const resp = await page.goto(url, { waitUntil: "networkidle" });
      const body = await page.textContent("body");
      const ok =
        resp?.ok() && page.url().startsWith(`${base.protocol}//${host}.${domain}`) && body.includes(marker);
      console.log(`${host}: ${resp?.status()} at ${page.url().slice(0, 60)} ${ok ? "ok" : "FAILED"}`);
      if (host === "grafana" && ok) console.log("grafana user:", JSON.parse(body).email);
      if (!ok) throw new Error(`${host} did not let the signed-in person through`);
    }
  }
  console.log(errors.length ? `page errors: ${errors}` : "no page errors");
} finally {
  await browser.close();
}
process.exit(0);
