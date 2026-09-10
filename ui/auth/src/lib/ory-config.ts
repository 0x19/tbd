import type { OryClientConfiguration } from "@ory/elements-react";

// Ory Elements configuration. Every Kratos and Hydra endpoint is served on this
// same origin (auth.<domain>) by Envoy, so the SDK URL is the page's own origin
// in the browser and AUTH_PUBLIC_URL on the server (the consent page).
export function oryConfig(): OryClientConfiguration {
  const url =
    typeof window !== "undefined"
      ? window.location.origin
      : (process.env.AUTH_PUBLIC_URL ?? "http://localhost:3002");
  return {
    sdk: { url },
    project: {
      name: "tbd",
      default_redirect_url: "/",
      error_ui_url: "/error",
      login_ui_url: "/login",
      registration_ui_url: "/registration",
      recovery_ui_url: "/recovery",
      verification_ui_url: "/verification",
      settings_ui_url: "/settings",
      registration_enabled: true,
      recovery_enabled: process.env.NEXT_PUBLIC_RECOVERY_ENABLED === "true",
      verification_enabled: process.env.NEXT_PUBLIC_VERIFICATION_ENABLED === "true",
      hide_ory_branding: true,
    },
  };
}
