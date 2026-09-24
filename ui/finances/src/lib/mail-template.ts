// Template helpers: `{{Month}}` and friends, filled from a chosen month and
// the party. Rendered here, on the page; the service stores what was sent.

import type { ReconciliationRow } from "@/lib/api/schema";
import type { Lang } from "@/lib/i18n";

export type TemplateContext = {
  /** YYYY-MM the mail is about, normally the month being closed. */
  month: string;
  /** The party's display name. */
  company: string;
  /** For month names and dates. */
  lang: Lang;
  /** Today, for `{{Today}}`; defaults to now. */
  today?: Date;
  /** The reconciliation page's summary of the month, for `{{Summary}}`. */
  summary?: string;
  /** The invoice being sent, for `{{InvoiceNumber}}` and friends. */
  invoice?: InvoiceFacts;
};

/** What an invoice mail says about itself: rendered on the page, never
 *  re-derived from the PDF. */
export type InvoiceFacts = {
  number: string;
  /** Formatted with its currency, the way the page shows money. */
  total: string;
  issued_at: string;
  due_date: string;
  client: string;
  /** Formatted with its currency; what a reminder asks for. */
  outstanding: string;
  days_overdue: number;
};

/** What the reconciliation page hands the composer: the month, its summary
 *  and the receipts that cover it. */
export type MonthPrefill = {
  kind: "month";
  party_id: string;
  /** The company's display name. */
  company: string;
  month: string;
  /** The month's rows as the service returned them: the composer builds the
   *  summary and the bundle from these in whatever language it is in. */
  rows: ReconciliationRow[];
};

/** What the invoices page hands the composer: the approved invoice, its
 *  stored PDF as the attachment, and the client's addresses as recipients. */
export type InvoicePrefill = {
  kind: "invoice";
  party_id: string;
  company: string;
  invoice: {
    id: string;
    number: string;
    issued_at: string;
    due_date: string;
    /** Already formatted with the currency. */
    total: string;
    outstanding: string;
    days_overdue: number;
    document_id: string;
    filename: string;
  };
  client: { id: string; name: string; recipients: string[] };
  /** The mail reminds the client the invoice is still owed. */
  reminder?: boolean;
};

/** A hand-off from another page, carried through `sessionStorage`, so a
 *  reload of the composer does not repeat it and nothing reaches the URL. */
export type Prefill = MonthPrefill | InvoicePrefill;

/** The accountant's bundle before any bytes: the text files with their
 *  content and the receipts by document with the name each takes inside the
 *  zip. The download zips it in the browser; a mail hands it to the service,
 *  which fetches the receipts and zips them there. */
export type BundlePlan = {
  filename: string;
  files: { name: string; text: string }[];
  receipts: { document_id: string; name: string }[];
};

/** UTF-8 text as base64, the way the gateway wants `bytes`. */
export function base64Utf8(s: string): string {
  return btoa(Array.from(new TextEncoder().encode(s), (b) => String.fromCharCode(b)).join(""));
}

const PREFILL_KEY = "finance.mail.prefill";

export function stashPrefill(p: Prefill): void {
  try {
    sessionStorage.setItem(PREFILL_KEY, JSON.stringify(p));
  } catch {
    // Storage blocked: the composer opens empty, and the page says nothing.
  }
}

/** Reads the prefill once; a second read finds nothing. */
export function takePrefill(): Prefill | null {
  try {
    const raw = sessionStorage.getItem(PREFILL_KEY);
    if (!raw) return null;
    sessionStorage.removeItem(PREFILL_KEY);
    const p = JSON.parse(raw) as Prefill | (Omit<MonthPrefill, "kind"> & { kind?: undefined });
    // A stash from before invoices could be sent carries no kind.
    return p.kind === undefined ? { ...p, kind: "month" } : p;
  } catch {
    return null;
  }
}

const MONTHS: Record<Lang, string[]> = {
  en: [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
  ],
  hr: [
    "siječanj",
    "veljača",
    "ožujak",
    "travanj",
    "svibanj",
    "lipanj",
    "srpanj",
    "kolovoz",
    "rujan",
    "listopad",
    "studeni",
    "prosinac",
  ],
};

/** The helpers grouped as the composer lists them: the month's, the
 *  invoice's (present only when an invoice is being sent), and `Summary`. */
export const HELPER_GROUPS: { id: "month" | "invoice" | "summary"; names: string[] }[] = [
  { id: "month", names: ["Month", "MonthPadded", "MonthName", "Year", "MonthYear", "Company", "Today"] },
  {
    id: "invoice",
    names: ["Client", "InvoiceNumber", "InvoiceTotal", "IssueDate", "DueDate", "Outstanding", "DaysOverdue"],
  },
  { id: "summary", names: ["Summary"] },
];

function localDate(iso: string, lang: Lang): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})/.exec(iso);
  if (!m) return iso;
  return lang === "hr" ? `${m[3]}.${m[2]}.${m[1]}.` : `${m[1]}-${m[2]}-${m[3]}`;
}

/** Every helper and what it becomes, for the legend and for `render`. The
 *  invoice helpers are present only with an invoice, so a template that
 *  names them shows them unfilled in any other mail. */
export function helpers(ctx: TemplateContext): Record<string, string> {
  const [y, m] = ctx.month.split("-");
  const mi = Math.max(0, Math.min(11, Number(m) - 1));
  const today = ctx.today ?? new Date();
  const dd = String(today.getDate()).padStart(2, "0");
  const mm = String(today.getMonth() + 1).padStart(2, "0");
  return {
    Month: String(mi + 1),
    MonthPadded: String(mi + 1).padStart(2, "0"),
    MonthName: MONTHS[ctx.lang][mi] ?? "",
    Year: y ?? "",
    MonthYear: `${mi + 1}/${y ?? ""}`,
    Company: ctx.company,
    Today: ctx.lang === "hr" ? `${dd}.${mm}.${today.getFullYear()}.` : `${today.getFullYear()}-${mm}-${dd}`,
    Summary: ctx.summary ?? "",
    ...(ctx.invoice
      ? {
          Client: ctx.invoice.client,
          Invoice: ctx.invoice.number,
          InvoiceNumber: ctx.invoice.number,
          InvoiceTotal: ctx.invoice.total,
          IssueDate: localDate(ctx.invoice.issued_at, ctx.lang),
          DueDate: localDate(ctx.invoice.due_date, ctx.lang),
          Outstanding: ctx.invoice.outstanding,
          DaysOverdue: String(ctx.invoice.days_overdue),
        }
      : {}),
  };
}

/** `{{Name}}` filled from the helpers; an unknown name stays as written, so
 *  a typo is visible in the preview rather than silently blank. */
export function render(template: string, ctx: TemplateContext): string {
  const h = helpers(ctx);
  return template.replace(/\{\{\s*([A-Za-z]+)\s*\}\}/g, (whole, name: string) =>
    name in h ? h[name]! : whole,
  );
}

/** Good enough to catch a missing @ or a stray word before the service
 *  refuses it: the service has the last word. */
export function looksLikeAddress(a: string): boolean {
  return /^[^\s@]+@[^\s@]+\.[^\s@]+$/.test(a);
}

/** Comma- or newline-separated addresses, trimmed, empties dropped. */
export function splitAddresses(s: string): string[] {
  return s
    .split(/[,;\n]/)
    .map((a) => a.trim())
    .filter(Boolean);
}
