import "server-only";

import { Configuration, FrontendApi, IdentityApi, OAuth2Api, type Session } from "@ory/client-fetch";

// Server-side clients. All three URLs are in-cluster (compose: service names).
const env = (name: string, fallback: string) => process.env[name] ?? fallback;

export const kratosPublic = new FrontendApi(
  new Configuration({ basePath: env("KRATOS_PUBLIC_URL", "http://kratos:4433") }),
);
export const kratosAdmin = new IdentityApi(
  new Configuration({ basePath: env("KRATOS_ADMIN_URL", "http://kratos:4434") }),
);
export const hydraAdmin = new OAuth2Api(
  new Configuration({ basePath: env("HYDRA_ADMIN_URL", "http://hydra:4445") }),
);

/** First-party clients: consent is granted without a screen. */
export const trustedClients = new Set(
  env("TRUSTED_CLIENT_IDS", "")
    .split(",")
    .map((s) => s.trim())
    .filter(Boolean),
);

/** The Kratos session behind the browser's cookie, or null. */
export async function sessionFromCookie(cookie: string | null): Promise<Session | null> {
  if (!cookie) return null;
  try {
    return await kratosPublic.toSession({ cookie });
  } catch {
    return null;
  }
}

/** Roles live in `metadata_admin.role`, set by operators (never by the person). */
export type Role = "admin" | "editor" | "viewer";

export async function roleOf(identityId: string): Promise<Role> {
  try {
    const identity = await kratosAdmin.getIdentity({ id: identityId });
    const meta = identity.metadata_admin as { role?: string } | null | undefined;
    const role = meta?.role;
    return role === "admin" || role === "editor" ? role : "viewer";
  } catch {
    return "viewer";
  }
}

/** Grafana's role names, from ours. */
export function grafanaRole(role: Role): "Admin" | "Editor" | "Viewer" {
  return role === "admin" ? "Admin" : role === "editor" ? "Editor" : "Viewer";
}

/** The claims every token for a person carries (docs/auth/README.md). */
export async function claimsFor(session: Session) {
  const traits = (session.identity?.traits ?? {}) as { email?: string; name?: string };
  const role = await roleOf(session.identity?.id ?? "");
  const verified = session.identity?.verifiable_addresses?.some((a) => a.verified) ?? false;
  return {
    id_token: {
      email: traits.email,
      email_verified: verified,
      name: traits.name,
      role,
      grafana_role: grafanaRole(role),
    },
    access_token: { role },
  };
}
