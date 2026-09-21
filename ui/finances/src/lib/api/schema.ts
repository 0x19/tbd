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
  reference_number: z.string(),
  entry_reference: z.string(),
  category_rule_id: z.string(),
  categorised_at: z.string(),
  account_name: z.string(),
  // The bank's record as JSON; only GetTransaction fills it.
  raw: z.string(),
});
export type Transaction = z.infer<typeof Transaction>;
export const GetTransactionResponse = z.object({ transaction: Transaction.nullable().optional() });

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
  internal: z.boolean(),
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
  // The call carried your address, so the bank did not count it.
  attended: z.boolean(),
});
export const SetAccountSyncResponse = z.object({ account: Account.nullable().optional() });
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

export type UpsertCategory = {
  id?: string;
  party_id: string;
  name: string;
  kind: string;
  deductible: boolean;
  archived: boolean;
};
export const UpsertCategoryResponse = z.object({
  category: Category.nullable().optional(),
  categorised: z.number(),
  unmatched: z.number(),
});
export type UpsertCategoryResponse = z.infer<typeof UpsertCategoryResponse>;

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
  // Why it is failed or revoked: "code exchange refused", "removed by you", "replaced by a newer consent".
  failure: z.string(),
  created_at: z.string(),
  // The consent that took this one's accounts over, when one did.
  replaced_by: z.string(),
});
export const DeleteConnectionResponse = z.object({});
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

// ---- invoicing --------------------------------------------------------------

export const IssuerProfile = z.object({
  party_id: z.string(),
  legal_name: z.string(),
  address_lines: z.array(z.string()),
  oib: z.string(),
  vat_id: z.string(),
  iban: z.string(),
  swift: z.string(),
  bank_name: z.string(),
  court: z.string(),
  registration_no: z.string(),
  share_capital: z.string(),
  board_member: z.string(),
  issued_by: z.string(),
  place_of_issue: z.string(),
  operator_id: z.string(),
  premises: z.string(),
  device: z.string(),
  due_days: z.number(),
});
export type IssuerProfile = z.infer<typeof IssuerProfile>;
export const GetIssuerResponse = z.object({ issuer: IssuerProfile.nullable().optional() });
export const UpsertIssuerResponse = z.object({ issuer: IssuerProfile.nullable().optional() });

export const ClientProfile = z.object({
  id: z.string(),
  party_id: z.string(),
  name: z.string(),
  address_lines: z.array(z.string()),
  country_code: z.string(),
  tax_id: z.string(),
  vat_treatment: z.string(),
  recipients: z.array(z.string()),
  currency: z.string(),
  archived: z.boolean(),
  // The company's default client, one per company; set through SetDefaultClient.
  is_default: z.boolean(),
});
export type ClientProfile = z.infer<typeof ClientProfile>;
export const ListClientsResponse = z.object({ clients: z.array(ClientProfile) });
export const ListIssuersResponse = z.object({ issuers: z.array(IssuerProfile) });
export const CreateIssuerResponse = z.object({
  party: Party.nullable().optional(),
  issuer: IssuerProfile.nullable().optional(),
});
export const UpsertClientResponse = z.object({ client: ClientProfile.nullable().optional() });

export const InvoiceLine = z.object({
  position: z.number(),
  description: z.string(),
  quantity_milli: Minor,
  unit_price_minor: Minor,
  amount_minor: Minor,
  template_id: z.string(),
});
export type InvoiceLine = z.infer<typeof InvoiceLine>;

export const InvoiceDelivery = z.object({
  id: z.string(),
  mail_id: z.string(),
  to: z.array(z.string()),
  sent_at: z.string(),
  // invoice (it went out) or reminder (it is still owed).
  kind: z.string(),
});
export type InvoiceDelivery = z.infer<typeof InvoiceDelivery>;
export const AgingBucket = z.object({
  currency: z.string(),
  // current, d1_30, d31_60, d61_90, d90_plus.
  bucket: z.string(),
  count: z.number(),
  amount_minor: Minor,
});
export type AgingBucket = z.infer<typeof AgingBucket>;
export const ClientAging = z.object({
  client_id: z.string(),
  client_name: z.string(),
  currency: z.string(),
  count: z.number(),
  outstanding_minor: Minor,
  overdue_minor: Minor,
  oldest_days: z.number(),
});
export type ClientAging = z.infer<typeof ClientAging>;
export const AgingReportResponse = z.object({
  as_of: z.string(),
  buckets: z.array(AgingBucket),
  clients: z.array(ClientAging),
});
export type AgingReportResponse = z.infer<typeof AgingReportResponse>;
export const InvoicePayment = z.object({
  id: z.string(),
  invoice_id: z.string(),
  transaction_id: z.string(),
  amount_minor: Minor,
  currency: z.string(),
  paid_on: z.string(),
  // inferred (the matcher) or declared (a person).
  source: z.string(),
  reason: z.string(),
  note: z.string(),
  counterparty: z.string(),
});
export type InvoicePayment = z.infer<typeof InvoicePayment>;
export const Invoice = z.object({
  id: z.string(),
  party_id: z.string(),
  client_id: z.string(),
  status: z.string(),
  number: z.string(),
  year: z.number(),
  issued_at: z.string(),
  delivery_date: z.string(),
  due_date: z.string(),
  place_of_issue: z.string(),
  currency: z.string(),
  subtotal_minor: Minor,
  vat_minor: Minor,
  total_minor: Minor,
  vat_treatment: z.string(),
  vat_note: z.string(),
  note: z.string(),
  content_hash: z.string(),
  approved_at: z.string(),
  document_id: z.string(),
  prefilled_from: z.string(),
  cancelled_at: z.string(),
  created_at: z.string(),
  updated_at: z.string(),
  lines: z.array(InvoiceLine),
  // The series the number is (or will be) in.
  premises: z.string(),
  device: z.string(),
  // What the payments cover so far; "paid" once they cover the total.
  paid_minor: Minor,
  paid_at: z.string(),
  payments: z.array(InvoicePayment),
  // The first time it went out, and every time.
  sent_at: z.string(),
  deliveries: z.array(InvoiceDelivery),
  // What is still owed, days past due (zero until then), the last reminder.
  outstanding_minor: Minor,
  days_overdue: z.number(),
  reminded_at: z.string(),
});
export type Invoice = z.infer<typeof Invoice>;
export const ListInvoicesResponse = z.object({ invoices: z.array(Invoice) });
export const InvoiceResponse = z.object({ invoice: Invoice.nullable().optional() });
export const PreviewInvoiceResponse = z.object({
  content_hash: z.string(),
  number: z.string(),
  pdf: z.string(),
});
export type PreviewInvoiceResponse = z.infer<typeof PreviewInvoiceResponse>;
export const InvoiceDocumentResponse = z.object({
  content_type: z.string(),
  pdf: z.string(),
  number: z.string(),
});

export const LineTemplate = z.object({
  id: z.string(),
  client_id: z.string(),
  position: z.number(),
  description: z.string(),
  // fixed | variable | optional
  mode: z.string(),
  quantity_milli: Minor,
  unit_price_minor: Minor,
  enabled: z.boolean(),
});
export type LineTemplate = z.infer<typeof LineTemplate>;
export const ListLineTemplatesResponse = z.object({ templates: z.array(LineTemplate) });
export const UpsertLineTemplateResponse = z.object({ template: LineTemplate.nullable().optional() });
export const DeleteLineTemplateResponse = z.object({});

// ---- connectors -------------------------------------------------------------

export const ConnectorKind = z.object({
  name: z.string(),
  label: z.string(),
  description: z.string(),
  auth: z.string(),
  consent_note: z.string(),
  configured: z.boolean(),
  // read, send, both: what the kind can be linked for.
  purposes: z.array(z.string()),
});
export type ConnectorKind = z.infer<typeof ConnectorKind>;
export const ListConnectorKindsResponse = z.object({ kinds: z.array(ConnectorKind) });

export const Connector = z.object({
  id: z.string(),
  party_id: z.string(),
  kind: z.string(),
  label: z.string(),
  status: z.string(),
  config: z.string(),
  external_id: z.string(),
  linked_at: z.string(),
  last_sync_at: z.string(),
  last_sync_status: z.string(),
  last_sync_error: z.string(),
  failure: z.string(),
  created_at: z.string(),
  can_send: z.boolean(),
  // The consent included reading; a mailbox linked for sending only is never pulled.
  can_read: z.boolean(),
});
export type Connector = z.infer<typeof Connector>;
export const ListConnectorsResponse = z.object({ connectors: z.array(Connector) });

// Mail from a linked mailbox.
export const MailTemplate = z.object({
  id: z.string(),
  party_id: z.string(),
  name: z.string(),
  subject: z.string(),
  body: z.string(),
  to: z.array(z.string()),
  cc: z.array(z.string()),
  bcc: z.array(z.string()),
  updated_at: z.string(),
});
export type MailTemplate = z.infer<typeof MailTemplate>;
export const ListMailTemplatesResponse = z.object({ templates: z.array(MailTemplate) });
export const UpsertMailTemplateResponse = z.object({ template: MailTemplate.nullable().optional() });
export const DeleteMailTemplateResponse = z.object({});
export const MailDocument = z.object({
  document_id: z.string(),
  filename: z.string(),
  content_type: z.string(),
  size_bytes: Minor,
});
export type MailDocument = z.infer<typeof MailDocument>;
export const Mail = z.object({
  id: z.string(),
  party_id: z.string(),
  connector_id: z.string(),
  direction: z.string(),
  from: z.string(),
  to: z.array(z.string()),
  cc: z.array(z.string()),
  bcc: z.array(z.string()),
  subject: z.string(),
  body: z.string(),
  status: z.string(),
  error: z.string(),
  sent_at: z.string(),
  received_at: z.string(),
  template_id: z.string(),
  parent_id: z.string(),
  replies: z.number(),
  documents: z.array(MailDocument),
  thread_key: z.string(),
  // The bundle zip's file name, when one went with the mail.
  bundle: z.string(),
});
export type Mail = z.infer<typeof Mail>;
export const SendMailResponse = z.object({ mail: Mail.nullable().optional() });
export const ListMailResponse = z.object({ mails: z.array(Mail), total: z.number() });
export const GetMailResponse = z.object({ mail: Mail.nullable().optional(), thread: z.array(Mail) });
// `url` is empty for a kind linked by pasting credentials; `state` then goes to CompleteConnector.
export const StartConnectorResponse = z.object({
  connector_id: z.string(),
  url: z.string(),
  state: z.string(),
});
export const ConnectorResponse = z.object({ connector: Connector.nullable().optional() });
export const TestConnectorResponse = z.object({ status: z.string() });

export const ConnectorRun = z.object({
  id: z.string(),
  started_at: z.string(),
  finished_at: z.string(),
  trigger: z.string(),
  outcome: z.string(),
  found: z.number(),
  stored: z.number(),
  skipped: z.number(),
  error: z.string(),
});
export type ConnectorRun = z.infer<typeof ConnectorRun>;
// The run just opened; the pull is detached. Its progress and outcome arrive on the
// connectors feed (WatchConnectors).
export const SyncConnectorResponse = z.object({ run: ConnectorRun });
export const WatchConnectorsResponse = z.object({
  connector: Connector,
  run: ConnectorRun.nullable().optional(),
  deleted: z.boolean(),
});
export type WatchConnectorsResponse = z.infer<typeof WatchConnectorsResponse>;
export const ListConnectorRunsResponse = z.object({ runs: z.array(ConnectorRun) });

export const DocumentSource = z.object({
  connector_id: z.string(),
  external_ref: z.string(),
  subject: z.string(),
  sender: z.string(),
  received_at: z.string(),
});
export const Document = z.object({
  id: z.string(),
  party_id: z.string(),
  kind: z.string(),
  filename: z.string(),
  content_type: z.string(),
  size_bytes: Minor,
  sha256: z.string(),
  vendor: z.string(),
  doc_date: z.string(),
  total_minor: z.string(),
  currency: z.string(),
  created_at: z.string(),
  sources: z.array(DocumentSource),
  invoice_no: z.string(),
  extracted_at: z.string(),
  // A person set the fields; a re-read leaves them.
  declared: z.boolean(),
  // vendor / date / amount / invoice_no → label | sender | first | received | declared | "".
  found_by: z.record(z.string(), z.string()),
});
export type Document = z.infer<typeof Document>;
export const VendorCount = z.object({ vendor: z.string(), count: z.number() });
export const ListDocumentsResponse = z.object({
  documents: z.array(Document),
  total: z.number(),
  vendors: z.array(VendorCount),
});
export const DocumentResponse = z.object({ document: Document.nullable().optional() });
export const GetDocumentResponse = z.object({ document: Document.nullable().optional(), bytes: z.string() });

// An ePorezna form read from its XML. Amounts are the decimal strings as filed,
// never rounded; `decimalToMinor` converts for a sum or `money()`.
export const Filing = z.object({
  id: z.string(),
  party_id: z.string(),
  form: z.string(),
  schema: z.string(),
  period_from: z.string(),
  period_to: z.string(),
  oib: z.string(),
  obveznik: z.string(),
  prepared_at: z.string(),
  author: z.string(),
  filename: z.string(),
  report_mark: z.string(),
  headline: z.record(z.string(), z.string()),
  // Only from GetFiling; empty in a listing.
  values: z.record(z.string(), z.string()),
  rows_json: z.string(),
  error: z.string(),
  parsed_at: z.string(),
  parser_version: z.string(),
});
export type Filing = z.infer<typeof Filing>;
export const ListFilingsResponse = z.object({
  filings: z.array(Filing),
  total: z.number(),
  years: z.array(z.number()),
});
export const GetFilingResponse = z.object({ filing: Filing.nullable().optional() });

// ---- books (the general ledger; docs/finance/books.md) ----
export const LedgerAccount = z.object({
  code: z.string(),
  name: z.string(),
  parent_code: z.string(),
  class: z.number(),
  kind: z.string(),
  synthetic: z.boolean(),
  archived: z.boolean(),
});
export type LedgerAccount = z.infer<typeof LedgerAccount>;
export const ListLedgerAccountsResponse = z.object({ accounts: z.array(LedgerAccount) });
export const UpsertLedgerAccountResponse = z.object({ account: LedgerAccount.nullable().optional() });
export const Period = z.object({
  fiscal_year: z.number(),
  month: z.number(),
  // open, closed or locked.
  status: z.string(),
  changed_at: z.string(),
});
export type Period = z.infer<typeof Period>;
export const ListPeriodsResponse = z.object({ fiscal_year: z.number(), periods: z.array(Period) });
export const PeriodResponse = z.object({ period: Period.nullable().optional() });
export const ImportOpeningBalancesResponse = z.object({
  accounts: z.number(),
  total_debit_minor: Minor,
  total_credit_minor: Minor,
});
// Every figure is minor units as a string; the opening is signed as filed.
export const TrialBalanceRow = z.object({
  account_code: z.string(),
  name: z.string(),
  class: z.number(),
  kind: z.string(),
  opening_debit_minor: Minor,
  opening_credit_minor: Minor,
  period_debit_minor: Minor,
  period_credit_minor: Minor,
  total_debit_minor: Minor,
  total_credit_minor: Minor,
  balance_minor: Minor,
});
export type TrialBalanceRow = z.infer<typeof TrialBalanceRow>;
export const TrialBalanceClass = z.object({
  class: z.number(),
  opening_debit_minor: Minor,
  opening_credit_minor: Minor,
  period_debit_minor: Minor,
  period_credit_minor: Minor,
  total_debit_minor: Minor,
  total_credit_minor: Minor,
  balance_minor: Minor,
});
export type TrialBalanceClass = z.infer<typeof TrialBalanceClass>;
export const TrialBalanceResponse = z.object({
  party_id: z.string(),
  fiscal_year: z.number(),
  through_month: z.number(),
  currency: z.string(),
  opening_as_of: z.string(),
  rows: z.array(TrialBalanceRow),
  classes: z.array(TrialBalanceClass),
  total_debit_minor: Minor,
  total_credit_minor: Minor,
  balanced: z.boolean(),
  periods: z.array(Period),
  years: z.array(z.number()),
});
export type TrialBalanceResponse = z.infer<typeof TrialBalanceResponse>;

// ---- reconciliation (the accountant's month) ----
export const Reason = z.object({ code: z.string(), args: z.record(z.string(), z.string()) });
export type Reason = z.infer<typeof Reason>;
export const LinkedDocument = z.object({
  document_id: z.string(),
  vendor: z.string(),
  doc_date: z.string(),
  total_minor: z.string(),
  currency: z.string(),
  filename: z.string(),
  invoice_no: z.string(),
  // declared | inferred; empty for a suggestion.
  source: z.string(),
  confidence: z.number(),
  reason: z.string(),
  why: z.array(Reason),
});
export type LinkedDocument = z.infer<typeof LinkedDocument>;
export const ReconciliationRow = z.object({
  transaction: Transaction,
  // eracun | receipt | none | personal | income | internal.
  need: z.string(),
  need_reason: z.string(),
  // covered | missing | "".
  status: z.string(),
  documents: z.array(LinkedDocument),
  suggestions: z.array(LinkedDocument),
  policy_id: z.string(),
  original_amount_minor: z.string(),
  original_currency: z.string(),
  need_why: Reason.nullable().optional(),
  // A person's note for the accountant; "" when none.
  note: z.string(),
});
export type ReconciliationRow = z.infer<typeof ReconciliationRow>;
export const CounterpartyPolicy = z.object({
  id: z.string(),
  party_id: z.string(),
  match: z.string(),
  exact: z.boolean(),
  policy: z.string(),
  note: z.string(),
});
export type CounterpartyPolicy = z.infer<typeof CounterpartyPolicy>;
export const MonthlyReconciliationResponse = z.object({
  rows: z.array(ReconciliationRow),
  summary: z.object({
    transactions: z.number(),
    eracun: z.number(),
    receipt_covered: z.number(),
    receipt_missing: z.number(),
    none: z.number(),
    personal: z.number(),
    income: z.number(),
    internal: z.number(),
    missing_minor: z.record(z.string(), z.string()),
  }),
  policies: z.array(CounterpartyPolicy),
});
export type MonthlyReconciliationResponse = z.infer<typeof MonthlyReconciliationResponse>;
export const ReconciliationRowResponse = z.object({ row: ReconciliationRow });
export const CounterpartyPolicyResponse = z.object({ policy: CounterpartyPolicy });
