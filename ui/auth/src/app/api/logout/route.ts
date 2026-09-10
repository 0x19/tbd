import { NextResponse } from "next/server";

import { hydraAdmin, sessionFromCookie } from "@/lib/server";

/**
 * Revokes every OAuth2 login session and consent for the signed-in person at
 * Hydra. Their refresh tokens stop working, so the UI hosts' cookies fail to
 * refresh and Envoy sends the browser back to sign in. The Kratos session is
 * ended by the page afterwards.
 */
export async function POST(request: Request) {
  const { logout_challenge } = (await request.json().catch(() => ({}))) as { logout_challenge?: string };
  const session = await sessionFromCookie(request.headers.get("cookie"));
  let redirect_to: string | undefined;
  if (session?.identity?.id) {
    const subject = session.identity.id;
    await Promise.all([
      hydraAdmin.revokeOAuth2ConsentSessions({ subject, all: true }),
      hydraAdmin.revokeOAuth2LoginSessions({ subject }),
    ]);
  }
  // Hydra's RP-initiated logout (a client called /oauth2/sessions/logout) hands us
  // a challenge; accepting it tells Hydra where to send the browser afterwards.
  if (logout_challenge) {
    try {
      ({ redirect_to } = await hydraAdmin.acceptOAuth2LogoutRequest({ logoutChallenge: logout_challenge }));
    } catch {
      redirect_to = undefined;
    }
  }
  return NextResponse.json({ revoked: Boolean(session), redirect_to });
}
