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

const MONTHS = ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"];

/** "2026-09" → "Sep 2026". */
export function monthLabel(ym: string): string {
  const [y, m] = ym.split("-");
  const i = Number(m) - 1;
  return MONTHS[i] ? `${MONTHS[i]} ${y}` : ym;
}

/** "2026-09" → "Sep". */
export function monthShort(ym: string): string {
  const i = Number(ym.split("-")[1]) - 1;
  return MONTHS[i] ?? ym;
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
  return v == null ? "—" : v.toLocaleString("en-US");
}

export function ago(iso: string | null | undefined): string {
  if (!iso) return "never";
  const s = Math.max(0, (Date.now() - new Date(iso).getTime()) / 1000);
  if (s < 60) return "just now";
  if (s < 3600) return `${Math.floor(s / 60)} min ago`;
  if (s < 86400) return `${Math.floor(s / 3600)} h ago`;
  return `${Math.floor(s / 86400)} d ago`;
}

export function when(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(iso).toLocaleString("en-GB", { dateStyle: "medium", timeStyle: "short" });
}

export function day(iso: string | null | undefined): string {
  if (!iso) return "—";
  return new Date(`${iso}T00:00:00`).toLocaleDateString("en-GB", { day: "2-digit", month: "short" });
}
