// Template helpers: `{{Month}}` and friends, filled from a chosen month and
// the party. Rendered here, on the page; the service stores what was sent.

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
};

/** What the reconciliation page hands the composer: the month, its summary
 *  and the receipts that cover it. Carried through `sessionStorage`, so a
 *  reload of the composer does not repeat it and nothing reaches the URL. */
export type Prefill = {
  party_id: string;
  month: string;
  summary: string;
  attachments: { id: string; filename: string; vendor: string }[];
};

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
    return JSON.parse(raw) as Prefill;
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

/** Every helper and what it becomes, for the legend and for `render`. */
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

/** Comma- or newline-separated addresses, trimmed, empties dropped. */
export function splitAddresses(s: string): string[] {
  return s
    .split(/[,;\n]/)
    .map((a) => a.trim())
    .filter(Boolean);
}
