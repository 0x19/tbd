import { NextResponse } from "next/server";

import { claimsFor, hydraAdmin, sessionFromCookie } from "@/lib/server";

/** The consent card posts here: accept with the chosen scopes, or reject. */
export async function POST(request: Request) {
  const form = await request.formData();
  const challenge = String(form.get("consent_challenge") ?? "");
  const action = String(form.get("action") ?? "");
  if (!challenge) return NextResponse.json({ error: "missing consent_challenge" }, { status: 400 });

  if (!["accept", "allow", "grant"].includes(action)) {
    const { redirect_to } = await hydraAdmin.rejectOAuth2ConsentRequest({
      consentChallenge: challenge,
      rejectOAuth2Request: { error: "access_denied", error_description: "The person declined." },
    });
    return NextResponse.redirect(redirect_to, 303);
  }

  const session = await sessionFromCookie(request.headers.get("cookie"));
  if (!session) return NextResponse.redirect(new URL("/login", request.url), 303);
  const consent = await hydraAdmin.getOAuth2ConsentRequest({ consentChallenge: challenge });
  const granted = form
    .getAll("grant_scope")
    .map(String)
    .filter((s) => consent.requested_scope?.includes(s));
  const { redirect_to } = await hydraAdmin.acceptOAuth2ConsentRequest({
    consentChallenge: challenge,
    acceptOAuth2ConsentRequest: {
      grant_scope: granted,
      grant_access_token_audience: consent.requested_access_token_audience,
      remember: form.get("remember") === "1" || form.get("remember") === "true",
      remember_for: 3600 * 24 * 30,
      session: await claimsFor(session),
    },
  });
  return NextResponse.redirect(redirect_to, 303);
}
