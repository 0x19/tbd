// One fetch layer for the finance API, which is the protocol's REST surface
// for tbd.finance.v1.FinanceService (docs/protocol/README.md). Every response
// is parsed through its Zod schema so pages never touch untyped JSON.
import { z } from "zod";

import {
  type ClientProfile,
  CompleteConnectionResponse,
  ConnectorResponse,
  CounterpartyPolicyResponse,
  DeclareCategoryResponse,
  DeleteLineTemplateResponse,
  DocumentResponse,
  GetDocumentResponse,
  GetIssuerResponse,
  GetTransactionResponse,
  InvoiceDocumentResponse,
  type InvoiceLine,
  InvoiceResponse,
  type IssuerProfile,
  type LineTemplate,
  ListAccountsResponse,
  ListCategoriesResponse,
  ListClientsResponse,
  ListConnectionsResponse,
  ListConnectorKindsResponse,
  ListConnectorRunsResponse,
  ListConnectorsResponse,
  ListDocumentsResponse,
  ListInvoicesResponse,
  ListLineTemplatesResponse,
  ListPartiesResponse,
  ListRulesResponse,
  ListTransactionsResponse,
  Me,
  MonthlyReconciliationResponse,
  MonthlySummaryResponse,
  PreviewInvoiceResponse,
  ReconciliationRowResponse,
  RefreshAccountResponse,
  SetAccountSyncResponse,
  StartConnectionResponse,
  StartConnectorResponse,
  SyncConnectorResponse,
  TestConnectorResponse,
  type UpsertCategory,
  UpsertCategoryResponse,
  UpsertClientResponse,
  UpsertIssuerResponse,
  UpsertLineTemplateResponse,
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
  setAccountSync: (account_id: string, enabled: boolean) =>
    call(SetAccountSyncResponse, "/v1/finance/accounts/sync", {
      method: "POST",
      json: { account_id, enabled },
    }),
  categories: (party_ids: string[]) =>
    call(ListCategoriesResponse, `/v1/finance/categories${query({ party_ids })}`),
  transaction: (id: string) => call(GetTransactionResponse, `/v1/finance/transactions/${id}`),
  upsertCategory: (category: UpsertCategory) =>
    call(UpsertCategoryResponse, "/v1/finance/categories", { method: "POST", json: category }),
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

  // ---- invoicing ----
  issuer: (party_id: string) => call(GetIssuerResponse, `/v1/finance/issuer${query({ party_id })}`),
  upsertIssuer: (issuer: IssuerProfile) =>
    call(UpsertIssuerResponse, "/v1/finance/issuer", { method: "POST", json: { issuer } }),
  clients: (party_ids: string[]) => call(ListClientsResponse, `/v1/finance/clients${query({ party_ids })}`),
  upsertClient: (client: Omit<ClientProfile, "archived">) =>
    call(UpsertClientResponse, "/v1/finance/clients", { method: "POST", json: { client } }),
  invoices: (party_ids: string[]) =>
    call(ListInvoicesResponse, `/v1/finance/invoices${query({ party_ids })}`),
  invoice: (id: string) => call(InvoiceResponse, `/v1/finance/invoices/${id}`),
  createInvoice: (client_id: string) =>
    call(InvoiceResponse, "/v1/finance/invoices", { method: "POST", json: { client_id } }),
  updateInvoice: (
    id: string,
    draft: {
      delivery_date: string;
      due_date: string;
      place_of_issue: string;
      note: string;
      lines: InvoiceLine[];
    },
  ) => call(InvoiceResponse, `/v1/finance/invoices/${id}/update`, { method: "POST", json: draft }),
  previewInvoice: (id: string) =>
    call(PreviewInvoiceResponse, `/v1/finance/invoices/${id}/preview`, { method: "POST", json: {} }),
  approveInvoice: (id: string, content_hash: string) =>
    call(InvoiceResponse, `/v1/finance/invoices/${id}/approve`, { method: "POST", json: { content_hash } }),
  cancelInvoice: (id: string, reason: string) =>
    call(InvoiceResponse, `/v1/finance/invoices/${id}/cancel`, { method: "POST", json: { reason } }),
  invoiceDocument: (id: string) => call(InvoiceDocumentResponse, `/v1/finance/invoices/${id}/document`),
  lineTemplates: (client_id: string) =>
    call(ListLineTemplatesResponse, `/v1/finance/clients/${client_id}/lines`),
  upsertLineTemplate: (client_id: string, template: LineTemplate) =>
    call(UpsertLineTemplateResponse, `/v1/finance/clients/${client_id}/lines`, {
      method: "POST",
      json: { template },
    }),
  deleteLineTemplate: (client_id: string, id: string) =>
    call(DeleteLineTemplateResponse, `/v1/finance/clients/${client_id}/lines/delete`, {
      method: "POST",
      json: { id },
    }),

  // ---- connectors ----
  connectorKinds: () => call(ListConnectorKindsResponse, "/v1/finance/connectors/kinds"),
  connectors: (party_ids: string[]) =>
    call(ListConnectorsResponse, `/v1/finance/connectors${query({ party_ids })}`),
  startConnector: (party_id: string, kind: string) =>
    call(StartConnectorResponse, "/v1/finance/connectors", { method: "POST", json: { party_id, kind } }),
  completeConnector: (state: string, code: string) =>
    call(ConnectorResponse, "/v1/finance/connectors/complete", { method: "POST", json: { state, code } }),
  testConnector: (id: string) =>
    call(TestConnectorResponse, `/v1/finance/connectors/${id}/test`, { method: "POST", json: {} }),
  syncConnector: (id: string) =>
    call(SyncConnectorResponse, `/v1/finance/connectors/${id}/sync`, { method: "POST", json: {} }),
  configureConnector: (id: string, change: { config?: string; party_id?: string; label?: string }) =>
    call(ConnectorResponse, `/v1/finance/connectors/${id}/configure`, { method: "POST", json: change }),
  deleteConnector: (id: string) =>
    call(z.object({}), `/v1/finance/connectors/${id}/delete`, { method: "POST", json: {} }),
  connectorRuns: (id: string) => call(ListConnectorRunsResponse, `/v1/finance/connectors/${id}/runs`),
  /** The connectors feed: an SSE URL for `useEvents`. */
  connectorEventsUrl: (party_ids: string[]) =>
    `${apiBase()}/v1/finance/connectors/events${query({ party_ids })}`,
  documents: (p: {
    party_ids: string[];
    kind?: string;
    q?: string;
    from?: string;
    to?: string;
    vendor?: string;
    limit?: number;
    offset?: number;
  }) => call(ListDocumentsResponse, `/v1/finance/documents${query({ kind: "receipt", limit: 100, ...p })}`),
  document: (id: string) => call(GetDocumentResponse, `/v1/finance/documents/${id}`),
  uploadDocument: (party_id: string, filename: string, content_type: string, bytes: string) =>
    call(DocumentResponse, "/v1/finance/documents/upload", {
      method: "POST",
      json: { party_id, filename, content_type, bytes },
    }),
  // ---- reconciliation ----
  reconciliation: (party_id: string, month: string) =>
    call(MonthlyReconciliationResponse, `/v1/finance/reconciliation/${party_id}/${month}`),
  linkDocument: (transaction_id: string, document_id: string) =>
    call(ReconciliationRowResponse, "/v1/finance/reconciliation/link", {
      method: "POST",
      json: { transaction_id, document_id },
    }),
  unlinkDocument: (transaction_id: string, document_id: string) =>
    call(ReconciliationRowResponse, "/v1/finance/reconciliation/unlink", {
      method: "POST",
      json: { transaction_id, document_id },
    }),
  setPolicy: (p: { party_id: string; match: string; exact: boolean; policy: string; note: string }) =>
    call(CounterpartyPolicyResponse, "/v1/finance/reconciliation/policies", { method: "POST", json: p }),
  deletePolicy: (id: string) =>
    call(z.object({}), `/v1/finance/reconciliation/policies/${id}/delete`, { method: "POST", json: {} }),
  updateDocument: (
    id: string,
    fields: {
      vendor: string;
      doc_date: string;
      total_minor: string;
      currency: string;
      invoice_no: string;
      party_id: string;
    },
  ) => call(DocumentResponse, `/v1/finance/documents/${id}/update`, { method: "POST", json: fields }),
  extractDocument: (id: string) =>
    call(DocumentResponse, `/v1/finance/documents/${id}/extract`, { method: "POST", json: {} }),
};

/** A base64 PDF from the API as an object URL for an <iframe> or a download. */
export function pdfUrl(base64: string): string {
  const bytes = Uint8Array.from(atob(base64), (c) => c.charCodeAt(0));
  return URL.createObjectURL(new Blob([bytes], { type: "application/pdf" }));
}
