// One fetch layer for the cv API, which is the protocol's REST surface for
// tbd.cv.v1.CvService (docs/protocol/README.md). Every response is parsed
// through its Zod schema so pages never touch untyped JSON.
import type { z } from "zod";

import { AccessResponse, DecideResponse, DownloadResponse, ListRequestsResponse, Me } from "./schema";

/**
 * Where the API is. Same origin in every deployment: Envoy serves the UI at
 * the root of the cv host and routes /v1/ to the protocol. `next dev` has no
 * API of its own, so it talks to the local cluster's edge unless
 * NEXT_PUBLIC_CV_API says otherwise, with the token in NEXT_PUBLIC_CV_TOKEN
 * (`mise run auth:token`).
 */
export function apiBase(): string {
  const configured = process.env.NEXT_PUBLIC_CV_API;
  if (configured) return configured.replace(/\/$/, "");
  if (process.env.NODE_ENV === "development") return "http://127.0.0.1:18080";
  if (typeof window !== "undefined") return window.location.origin;
  return "";
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
    /** The gateway's code word: `failed_precondition`, `permission_denied`, … */
    public code = "",
  ) {
    super(message);
  }
}

const RELOGIN_KEY = "cv-relogin-at";

/**
 * The browser session behind Envoy is gone (401 on an API call). A fetch
 * cannot follow the sign-in redirect, a page load can: reload, and Envoy
 * sends the browser through auth and back. At most once a minute.
 */
function relogin(): boolean {
  if (typeof window === "undefined" || process.env.NODE_ENV === "development") return false;
  const now = Date.now();
  let last = 0;
  try {
    last = Number(window.sessionStorage.getItem(RELOGIN_KEY) ?? 0);
  } catch {
    // storage unavailable: still reload once per page life
  }
  if (now - last < 60_000) return false;
  try {
    window.sessionStorage.setItem(RELOGIN_KEY, String(now));
  } catch {
    // ignore
  }
  window.location.reload();
  return true;
}

async function call<T>(
  schema: z.ZodType<T>,
  path: string,
  init?: RequestInit & { json?: unknown },
): Promise<T> {
  const { json, ...rest } = init ?? {};
  const token = process.env.NEXT_PUBLIC_CV_TOKEN;
  const res = await fetch(`${apiBase()}${path}`, {
    ...rest,
    credentials: "include",
    headers: {
      accept: "application/json",
      ...(json !== undefined ? { "content-type": "application/json" } : {}),
      ...(token ? { authorization: `Bearer ${token}` } : {}),
      ...(rest.headers ?? {}),
    },
    body: json !== undefined ? JSON.stringify(json) : rest.body,
  });
  if (res.status === 401 && relogin()) {
    await new Promise<never>(() => {});
  }
  if (!res.ok) {
    let message = `${res.status} ${res.statusText}`;
    let code = "";
    try {
      const body = (await res.json()) as { error?: string; message?: string };
      if (body.message) message = body.message;
      else if (body.error) message = body.error;
      code = body.error ?? "";
    } catch {
      // not JSON
    }
    throw new ApiError(res.status, message, code);
  }
  return schema.parse(await res.json());
}

export const api = {
  me: () => call(Me, "/v1/me"),
  access: () => call(AccessResponse, "/v1/cv/access"),
  request: (note: string) => call(AccessResponse, "/v1/cv/access", { method: "POST", json: { note } }),
  download: () => call(DownloadResponse, "/v1/cv/document"),
  requests: (status = "") =>
    call(ListRequestsResponse, `/v1/cv/requests${status ? `?status=${encodeURIComponent(status)}` : ""}`),
  decide: (id: string, decision: "approve" | "refuse" | "revoke") =>
    call(DecideResponse, `/v1/cv/requests/${encodeURIComponent(id)}/decide`, {
      method: "POST",
      json: { decision },
    }),
};

/** Base64 bytes from the API as an object URL, for a download. */
export function pdfUrl(base64: string): string {
  const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
  return URL.createObjectURL(new Blob([bytes], { type: "application/pdf" }));
}
