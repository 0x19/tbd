"use client";

import { ResponseError } from "@ory/client-fetch";
import { useSearchParams } from "next/navigation";
import { useEffect, useState } from "react";

import { frontend } from "@/lib/client";

type Kind = "login" | "registration" | "recovery" | "verification" | "settings";

const fetchers = {
  login: (id: string) => frontend.getLoginFlow({ id }),
  registration: (id: string) => frontend.getRegistrationFlow({ id }),
  recovery: (id: string) => frontend.getRecoveryFlow({ id }),
  verification: (id: string) => frontend.getVerificationFlow({ id }),
  settings: (id: string) => frontend.getSettingsFlow({ id }),
} as const;

type FlowOf<K extends Kind> = Awaited<ReturnType<(typeof fetchers)[K]>>;

/**
 * Loads a self-service flow for the page. Without `?flow=` it starts one at
 * Kratos' browser endpoint (carrying return_to, login_challenge, refresh, aal,
 * organization), which redirects back here with an id. Expired or foreign
 * flows (410, 403) start over; a missing session (401) goes to login.
 */
export function useFlow<K extends Kind>(kind: K): FlowOf<K> | null {
  const params = useSearchParams();
  const [flow, setFlow] = useState<FlowOf<K> | null>(null);
  const id = params.get("flow");

  useEffect(() => {
    if (!id) {
      const init = new URL(`/self-service/${kind}/browser`, window.location.origin);
      for (const key of ["return_to", "login_challenge", "refresh", "aal", "organization", "via"]) {
        const v = params.get(key);
        if (v) init.searchParams.set(key, v);
      }
      window.location.replace(init.toString());
      return;
    }
    let cancelled = false;
    (fetchers[kind](id) as Promise<FlowOf<K>>)
      .then((f) => {
        if (!cancelled) setFlow(f);
      })
      .catch(async (e: unknown) => {
        const status = e instanceof ResponseError ? e.response.status : 0;
        if (status === 401 && kind === "settings") {
          window.location.replace(`/login?return_to=${encodeURIComponent(window.location.href)}`);
        } else if (status === 410 || status === 403 || status === 404) {
          window.location.replace(`/${kind}`);
        } else {
          console.error(`${kind} flow ${id}`, e);
          window.location.replace(`/${kind}`);
        }
      });
    return () => {
      cancelled = true;
    };
  }, [id, kind, params]);

  return flow;
}
