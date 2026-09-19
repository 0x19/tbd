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
};

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
