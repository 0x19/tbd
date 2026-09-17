// Zod schemas at the API boundary. They mirror proto/tbd/finance/v1/finance.proto
// as the protocol's transcoder renders it: proto field names, int64 as a JSON
// string, enums by name, every field present. A field added to the proto is
// added here.
import { z } from "zod";

/** int64 arrives as a decimal string; money never goes through a float. */
export const Minor = z.string().regex(/^-?\d+$/);

export const Transaction = z.object({
  id: z.string(),
  account_id: z.string(),
  party_id: z.string(),
  status: z.string(),
  amount_minor: Minor,
  currency: z.string(),
  scale: z.number(),
  booking_date: z.string(),
  counterparty_name: z.string(),
  remittance: z.string(),
  value_date: z.string(),
  counterparty_iban: z.string(),
  category_id: z.string(),
  category: z.string(),
  category_source: z.string(),
  internal: z.boolean(),
});
export type Transaction = z.infer<typeof Transaction>;

export const ListTransactionsResponse = z.object({
  transactions: z.array(Transaction),
  party_ids: z.array(z.string()),
});

export const SummaryRow = z.object({
  month: z.string(),
  party_id: z.string(),
  category_id: z.string(),
  category: z.string(),
  kind: z.string(),
  currency: z.string(),
  total_minor: Minor,
  count: z.number(),
});
export type SummaryRow = z.infer<typeof SummaryRow>;

export const MonthlySummaryResponse = z.object({
  rows: z.array(SummaryRow),
  party_ids: z.array(z.string()),
});

export const Party = z.object({
  id: z.string(),
  kind: z.string(),
  display_name: z.string(),
  capability: z.string(),
});
export type Party = z.infer<typeof Party>;
export const ListPartiesResponse = z.object({ parties: z.array(Party) });

export const Balance = z.object({
  balance_type: z.string(),
  amount_minor: Minor,
  currency: z.string(),
  observed_at: z.string(),
});
export type Balance = z.infer<typeof Balance>;

export const Account = z.object({
  id: z.string(),
  party_id: z.string(),
  connection_id: z.string(),
  provider: z.string(),
  iban: z.string(),
  currency: z.string(),
  name: z.string(),
  sync_enabled: z.boolean(),
  last_synced_at: z.string(),
  last_sync_status: z.string(),
  last_sync_error: z.string(),
  last_booked_through: z.string(),
  sync_backoff_until: z.string(),
  sync_budget_used: z.number(),
  balances: z.array(Balance),
});
export type Account = z.infer<typeof Account>;
export const ListAccountsResponse = z.object({ accounts: z.array(Account) });

export const RefreshAccountResponse = z.object({
  outcome: z.string(),
  skipped: z.string(),
  inserted: z.number(),
  booked: z.number(),
  duplicates: z.number(),
});
export type RefreshAccountResponse = z.infer<typeof RefreshAccountResponse>;

export const Category = z.object({
  id: z.string(),
  party_id: z.string(),
  slug: z.string(),
  name: z.string(),
  kind: z.string(),
  deductible: z.boolean(),
  archived: z.boolean(),
});
export type Category = z.infer<typeof Category>;
export const ListCategoriesResponse = z.object({ categories: z.array(Category) });

export const DeclareCategoryResponse = z.object({ transaction: Transaction.nullable().optional() });

export const Rule = z.object({
  id: z.string(),
  party_id: z.string(),
  priority: z.number(),
  name: z.string(),
  category_id: z.string(),
  match_counterparty_like: z.string(),
  match_counterparty_iban: z.string(),
  match_remittance_like: z.string(),
  match_currency: z.string(),
  match_credit_debit: z.string(),
  enabled: z.boolean(),
  hits: Minor,
});
export type Rule = z.infer<typeof Rule>;
export const ListRulesResponse = z.object({ rules: z.array(Rule) });

export type UpsertRule = {
  id?: string;
  party_id: string;
  priority: number;
  name: string;
  category_id: string;
  match_counterparty_like?: string;
  match_counterparty_iban?: string;
  match_remittance_like?: string;
  match_currency?: string;
  match_credit_debit?: string;
  enabled: boolean;
};
export const UpsertRuleResponse = z.object({
  rule: Rule.nullable().optional(),
  categorised: z.number(),
  unmatched: z.number(),
});
export type UpsertRuleResponse = z.infer<typeof UpsertRuleResponse>;

export const Connection = z.object({
  id: z.string(),
  party_id: z.string(),
  provider: z.string(),
  psu_type: z.string(),
  aspsp_name: z.string(),
  status: z.string(),
  valid_until: z.string(),
  authorized_at: z.string(),
  accounts: z.number(),
});
export type Connection = z.infer<typeof Connection>;
export const ListConnectionsResponse = z.object({ connections: z.array(Connection) });

export const StartConnectionResponse = z.object({ connection_id: z.string(), url: z.string() });
export const CompleteConnectionResponse = z.object({
  connection_id: z.string(),
  account_ids: z.array(z.string()),
});

/** `GET /v1/me` on the protocol: the principal Envoy verified. */
export const Me = z.object({
  subject: z.string(),
  kind: z.string(),
  client_id: z.string().nullable().optional(),
  org: z.string().nullable().optional(),
  scopes: z.array(z.string()),
  role: z.string().nullable().optional(),
});
export type Me = z.infer<typeof Me>;
