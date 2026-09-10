"use client";

import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";

import { AuthShell } from "@/components/auth-shell";
import { frontend } from "@/lib/client";

/**
 * Global sign-out. First the server revokes every OAuth2 session and consent
 * for this person at Hydra (so the UI hosts' refresh tokens die and Envoy sends
 * them back to sign in), then Kratos ends the browser session, then /login.
 * Hydra's own logout (a client called /oauth2/sessions/logout) arrives with a
 * logout_challenge, which the server answers with where to go next.
 */
export default function LogoutPage() {
  const [message, setMessage] = useState("Signing you out…");
  const params = useSearchParams();
  const challenge = params.get("logout_challenge");

  useEffect(() => {
    (async () => {
      let next = "/login";
      try {
        const res = await fetch("/api/logout", {
          method: "POST",
          credentials: "include",
          headers: { "content-type": "application/json" },
          body: JSON.stringify({ logout_challenge: challenge }),
        });
        const body = (await res.json()) as { redirect_to?: string };
        if (body.redirect_to) next = body.redirect_to;
      } catch {
        // Hydra unreachable: still end the browser session below.
      }
      try {
        const { logout_token } = await frontend.createBrowserLogoutFlow();
        await frontend.updateLogoutFlow({ token: logout_token });
      } catch {
        // Not signed in, or already signed out: the destination is the same.
      }
      window.location.replace(next);
    })().catch(() => setMessage("Could not sign out; try again."));
  }, [challenge]);

  return (
    <AuthShell>
      <p className="text-muted-foreground text-sm">{message}</p>
    </AuthShell>
  );
}
