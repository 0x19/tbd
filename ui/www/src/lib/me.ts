"use client";

import { useEffect, useState } from "react";

/**
 * `GET /v1/me` on the same origin: the visitor Envoy verified, or nothing.
 *
 * The site sets no cookie and asks for none. The one that may exist is the sign-in
 * cookie Envoy's oauth2 filter set when an admin opened `/lab/`; this fetch sends it
 * if it is there and gets 401 otherwise, which is the whole point of the route
 * (`devops/envoy/envoy.yaml`, `www.*` host): a fetch cannot follow a sign-in
 * redirect, so `/v1/me` never redirects. Anything but a 200 is "nobody", quietly.
 */
export type Me = {
  subject: string;
  role?: string | null;
  email?: string | null;
  name?: string | null;
};

let cached: Promise<Me | null> | null = null;

function fetchMe(): Promise<Me | null> {
  cached ??= fetch("/v1/me", { credentials: "include", headers: { accept: "application/json" } })
    .then(async (res) => {
      if (!res.ok) return null;
      const body: unknown = await res.json();
      if (
        typeof body !== "object" ||
        body === null ||
        typeof (body as { subject?: unknown }).subject !== "string"
      ) {
        return null;
      }
      return body as Me;
    })
    .catch(() => null);
  return cached;
}

/** The signed-in visitor, once per page life; `null` until known and for everyone who is not. */
export function useMe(): Me | null {
  const [me, setMe] = useState<Me | null>(null);
  useEffect(() => {
    let live = true;
    void fetchMe().then((m) => {
      if (live) setMe(m);
    });
    return () => {
      live = false;
    };
  }, []);
  return me;
}

/** True only for a signed-in visitor whose token carries the `admin` role. */
export function useIsAdmin(): boolean {
  return useMe()?.role === "admin";
}
