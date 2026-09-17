// One fetch layer for the finance API, which is the protocol's REST surface
// for tbd.finance.v1.FinanceService (docs/protocol/README.md). Every response
// is parsed through its Zod schema so pages never touch untyped JSON.
import type { z } from "zod";

import {
  CompleteConnectionResponse,
  DeclareCategoryResponse,
  ListAccountsResponse,
  ListCategoriesResponse,
  ListConnectionsResponse,
  ListPartiesResponse,
  ListRulesResponse,
  ListTransactionsResponse,
  Me,
  MonthlySummaryResponse,
  RefreshAccountResponse,
  StartConnectionResponse,
  type UpsertRule,
  UpsertRuleResponse,
} from "./schema";

/**
 * Where the API is. Same origin in every deployment: Envoy serves the UI at
 * the root of the finance host and routes /v1/ to the protocol. `next dev`
 * has no API of its own, so it talks to the local cluster's edge unless
 * NEXT_PUBLIC_FINANCE_API says otherwise. Under `next dev` the browser has no
 * Envoy session for that origin, so requests carry the token in
 * NEXT_PUBLIC_FINANCE_TOKEN (`mise run auth:token`).
 */
export function apiBase(): string {
  const configured = process.env.NEXT_PUBLIC_FINANCE_API;
  if (configured) return configured.replace(/\/$/, "");
  if (process.env.NODE_ENV === "development") return "http://127.0.0.1:18080";
  if (typeof window !== "undefined") return window.location.origin;
  return "";
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

const RELOGIN_KEY = "finance-relogin-at";

/**
 * The browser session behind Envoy is gone (401 on an API call). A fetch
 * cannot follow the sign-in redirect, a page load can: reload, and Envoy sends
 * the browser through auth and back. At most once a minute, so a broken
 * deployment shows its error instead of a reload loop.
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

function query(params: Record<string, string | number | string[] | boolean | undefined>): string {
  const q = new URLSearchParams();
  for (const [k, v] of Object.entries(params)) {
    if (v === undefined || v === "" || v === false) continue;
    if (Array.isArray(v)) v.forEach((x) => q.append(k, x));
    else q.set(k, String(v));
  }
  const s = q.toString();
  return s ? `?${s}` : "";
}

async function call<T>(
  schema: z.ZodType<T>,
  path: string,
  init?: RequestInit & { json?: unknown },
): Promise<T> {
  const { json, ...rest } = init ?? {};
  const token = process.env.NEXT_PUBLIC_FINANCE_TOKEN;
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
    try {
      const body = (await res.json()) as { error?: string; message?: string };
      if (body.message) message = body.message;
      else if (body.error) message = body.error;
    } catch {
      // not JSON
    }
    throw new ApiError(res.status, message);
  }
  return schema.parse(await res.json());
}

export type TransactionQuery = {
  party_ids?: string[];
  limit?: number;
  offset?: number;
  month?: string;
  category_id?: string;
  account_id?: string;
  search?: string;
};

export const api = {
  me: () => call(Me, "/v1/me"),
  parties: () => call(ListPartiesResponse, "/v1/finance/parties"),
  summary: (party_ids: string[], from_month?: string, include_internal = false) =>
    call(MonthlySummaryResponse, `/v1/finance/summary${query({ party_ids, from_month, include_internal })}`),
  transactions: (q: TransactionQuery) =>
    call(ListTransactionsResponse, `/v1/finance/transactions${query(q)}`),
  accounts: (party_ids: string[]) =>
    call(ListAccountsResponse, `/v1/finance/accounts${query({ party_ids })}`),
  refresh: (account_id: string) =>
    call(RefreshAccountResponse, "/v1/finance/accounts/refresh", { method: "POST", json: { account_id } }),
  categories: (party_ids: string[]) =>
    call(ListCategoriesResponse, `/v1/finance/categories${query({ party_ids })}`),
  declare: (transaction_id: string, category_id: string) =>
    call(DeclareCategoryResponse, "/v1/finance/transactions/declare", {
      method: "POST",
      json: { transaction_id, category_id },
    }),
  rules: (party_ids: string[]) => call(ListRulesResponse, `/v1/finance/rules${query({ party_ids })}`),
  upsertRule: (rule: UpsertRule) =>
    call(UpsertRuleResponse, "/v1/finance/rules", { method: "POST", json: rule }),
  connections: (party_ids: string[]) =>
    call(ListConnectionsResponse, `/v1/finance/connections${query({ party_ids })}`),
  startConnection: (party_id: string, psu_type: "business" | "personal") =>
    call(StartConnectionResponse, "/v1/finance/connections", {
      method: "POST",
      json: { party_id, psu_type },
    }),
  completeConnection: (state: string, code: string) =>
    call(CompleteConnectionResponse, "/v1/finance/connections/complete", {
      method: "POST",
      json: { state, code },
    }),
};
