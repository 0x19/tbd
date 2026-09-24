// Money and time, formatted once. Amounts are minor units as strings (int64
// on the wire); they are turned into a display string without ever becoming
// a float, so 14500.82 is 14500.82.

/** Minor units → "14.500,82" in the currency's own style, sign kept. */
export function money(
  minor: string | number,
  currency = "EUR",
  opts?: { sign?: boolean; compact?: boolean },
): string {
  const s = String(minor);
  const negative = s.startsWith("-");
  const digits = negative ? s.slice(1) : s;
  const scale = currency === "JPY" ? 0 : 2;
  const whole = digits.length > scale ? digits.slice(0, digits.length - scale) : "0";
  const frac = digits.slice(-scale).padStart(scale, "0");
  if (opts?.compact) {
    const n = Number(whole);
    const body =
      n >= 1_000_000
        ? `${(n / 1_000_000).toFixed(1)}M`
        : n >= 10_000
          ? `${(n / 1000).toFixed(1)}k`
          : group(whole);
    return `${negative ? "−" : opts.sign ? "+" : ""}${body} ${currency}`;
  }
  const value = scale ? `${group(whole)},${frac}` : group(whole);
  return `${negative ? "−" : opts?.sign ? "+" : ""}${value} ${currency}`;
}

/** Minor units as a Number, for charts only (never for arithmetic on money). */
export function minorToNumber(minor: string | number): number {
  return Number(minor) / 100;
}

function group(whole: string): string {
  return whole.replace(/\B(?=(\d{3})+(?!\d))/g, ".");
}

// The language the page is in, set by the LangProvider: dates and counts
// follow it. Money never does: minor units are formatted the same way in
// both, the European way, with a code after the figure.
let lang: "en" | "hr" = "en";
export function setFormatLang(l: "en" | "hr") {
  lang = l;
}
function locale(): string {
  return lang === "hr" ? "hr-HR" : "en-GB";
}

const MONTHS: Record<"en" | "hr", string[]> = {
  en: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
  hr: ["sij", "velj", "ožu", "tra", "svi", "lip", "srp", "kol", "ruj", "lis", "stu", "pro"],
};
const MONTHS_LONG: Record<"en" | "hr", string[]> = {
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

/** "2026-09" → "Sep 2026". */
export function monthLabel(ym: string): string {
  const [y, m] = ym.split("-");
  const i = Number(m) - 1;
  const names = MONTHS[lang];
  return names[i] ? `${names[i]} ${y}` : ym;
}

/** "2026-09" → "September 2026". */
export function monthLong(ym: string): string {
  const [y, m] = ym.split("-");
  const i = Number(m) - 1;
  const names = MONTHS_LONG[lang];
  return names[i] ? `${names[i]} ${y}` : ym;
}

/** "2026-09" → "Sep". */
export function monthShort(ym: string): string {
  const i = Number(ym.split("-")[1]) - 1;
  return MONTHS[lang][i] ?? ym;
}

/** "2026-07" → "Q3 2026" (the quarter the month is in). */
export function quarterLabel(ym: string): string {
  const [y, m] = ym.split("-");
  const q = Math.floor((Number(m) - 1) / 3) + 1;
  return `Q${q} ${y}`;
}

/** "2026-07" → "2026". */
export function yearLabel(ym: string): string {
  return ym.split("-")[0] ?? ym;
}

/** The label of a period by its kind and first month. */
export function periodLabel(kind: "month" | "quarter" | "year", start: string): string {
  return kind === "month" ? monthLabel(start) : kind === "quarter" ? quarterLabel(start) : yearLabel(start);
}

/** This month as YYYY-MM, local time. */
export function thisMonth(d = new Date()): string {
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

/** n months before a YYYY-MM. */
export function monthsBefore(ym: string, n: number): string {
  const [y, m] = ym.split("-").map(Number);
  const d = new Date(y!, m! - 1 - n, 1);
  return thisMonth(d);
}

export function num(v: number | null | undefined): string {
  return v == null ? "—" : v.toLocaleString(locale());
}

const AGO = {
  en: {
    never: "never",
    now: "just now",
    min: (n: number) => `${n} min ago`,
    h: (n: number) => `${n} h ago`,
    d: (n: number) => `${n} d ago`,
  },
  hr: {
    never: "nikad",
    now: "upravo sada",
    min: (n: number) => `prije ${n} min`,
    h: (n: number) => `prije ${n} h`,
    d: (n: number) => `prije ${n} d`,
  },
};

export function ago(iso: string | null | undefined): string {
  const w = AGO[lang];
  if (!iso) return w.never;
  const s = Math.max(0, (Date.now() - new Date(iso).getTime()) / 1000);
  if (s < 60) return w.now;
  if (s < 3600) return w.min(Math.floor(s / 60));
  if (s < 86400) return w.h(Math.floor(s / 3600));
  return w.d(Math.floor(s / 86400));
}

export function when(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleString(locale(), { dateStyle: "medium", timeStyle: "short" });
}

export function day(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(`${iso}T00:00:00`).toLocaleDateString(locale(), { day: "2-digit", month: "short" });
}

/** The date of a timestamp or a bare YYYY-MM-DD, as the page's language writes it
 *  with the year: 15 Mar 2027 or 15. ožu 2027. */
export function dateOf(iso: string | null | undefined): string {
  if (!iso) return "—";
  const d = /^\d{4}-\d{2}-\d{2}$/.test(iso) ? new Date(`${iso}T00:00:00`) : new Date(iso);
  if (Number.isNaN(d.getTime())) return "—";
  return d.toLocaleDateString(locale(), { day: "numeric", month: "short", year: "numeric" });
}

/** A YYYY-MM-DD as the page's language writes a date: 11.08.2026. or 11 Aug 2026. */
export function dateOnly(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(`${iso}T00:00:00`).toLocaleDateString(locale(), { dateStyle: "medium" });
}
