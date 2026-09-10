import { randomBytes } from "node:crypto";

import { Consent } from "@ory/elements-react/theme";
import { headers } from "next/headers";
import { redirect } from "next/navigation";

import { AuthShell } from "@/components/auth-shell";
import { oryComponents } from "@/components/ory/components";
import { oryConfig } from "@/lib/ory-config";
import { claimsFor, hydraAdmin, sessionFromCookie, trustedClients } from "@/lib/server";

export const dynamic = "force-dynamic";

/**
 * Hydra's consent step. First-party clients and remembered consents are
 * accepted here without a screen, with the person's claims (email, name, role)
 * stamped into the tokens. Anything else gets the consent card.
 */
export default async function ConsentPage(props: {
  searchParams: Promise<Record<string, string | undefined>>;
}) {
  const { consent_challenge } = await props.searchParams;
  if (!consent_challenge)
    redirect("/error?error=missing_challenge&error_description=No consent challenge in the request.");

  const request = await hydraAdmin.getOAuth2ConsentRequest({ consentChallenge: consent_challenge });
  const cookie = (await headers()).get("cookie");
  const session = await sessionFromCookie(cookie);
  if (!session) {
    redirect(`/login?return_to=${encodeURIComponent(`/consent?consent_challenge=${consent_challenge}`)}`);
  }

  const clientId = request.client?.client_id ?? "";
  if (request.skip || trustedClients.has(clientId)) {
    const { redirect_to } = await hydraAdmin.acceptOAuth2ConsentRequest({
      consentChallenge: consent_challenge,
      acceptOAuth2ConsentRequest: {
        grant_scope: request.requested_scope,
        grant_access_token_audience: request.requested_access_token_audience,
        remember: true,
        remember_for: 0,
        session: await claimsFor(session),
      },
    });
    redirect(redirect_to);
  }

  const csrf = randomBytes(16).toString("hex");
  return (
    <AuthShell>
      <Consent
        consentChallenge={request}
        session={session}
        config={oryConfig()}
        csrfToken={csrf}
        formActionUrl="/api/consent"
        components={oryComponents}
      />
    </AuthShell>
  );
}
