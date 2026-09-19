// Turning the server's summary rows into what the pages draw. Sums are done
// on integers (minor units as BigInt) and only converted for display or for
// a chart axis, so 0.1 + 0.2 never happens to money.
import type { SummaryRow } from "@/lib/api/schema";

export type MonthTotals = {
  month: string;
  /** Money in, minor units, positive. */
  income: bigint;
  /** Money spent (kind expense/tax), minor units, positive. */
  spent: bigint;
  /** Everything out of any kind, minor units, positive. */
  out: bigint;
  /** Everything in of any kind, minor units, positive. */
  in: bigint;
  count: number;
};

const ZERO = 0n;

function abs(v: bigint): bigint {
  return v < ZERO ? -v : v;
}

/** Per-month totals for one currency, oldest first. */
export function byMonth(rows: SummaryRow[], currency: string): MonthTotals[] {
  const m = new Map<string, MonthTotals>();
  for (const r of rows) {
    if (r.currency !== currency) continue;
    const t = m.get(r.month) ?? { month: r.month, income: ZERO, spent: ZERO, out: ZERO, in: ZERO, count: 0 };
    const v = BigInt(r.total_minor);
    if (v < ZERO) {
      t.out += abs(v);
      // Money moved to another of the caller's own accounts is out, but it is
      // not spent -- however the row was (or was not) categorised.
      if (!r.internal && (r.kind === "expense" || r.kind === "tax" || r.kind === "")) t.spent += abs(v);
    } else {
      t.in += v;
      if (r.kind === "income") t.income += v;
    }
    t.count += r.count;
    m.set(r.month, t);
  }
  return [...m.values()].sort((a, b) => a.month.localeCompare(b.month));
}

export type CategoryTotal = {
  category_id: string;
  category: string;
  kind: string;
  /** Signed minor units across the selection. */
  total: bigint;
  count: number;
};

/** Per-category totals for one currency, biggest outflow first. */
export function byCategory(rows: SummaryRow[], currency: string, months?: Set<string>): CategoryTotal[] {
  const m = new Map<string, CategoryTotal>();
  for (const r of rows) {
    if (r.currency !== currency) continue;
    if (months && !months.has(r.month)) continue;
    // An own transfer with no category of its own is shown as one line,
    // "Own accounts", apart from the genuinely uncategorised.
    const key = r.category_id || (r.internal ? "internal" : "none");
    const t = m.get(key) ?? {
      category_id: key,
      category: r.category || (r.internal ? "Own accounts" : "Uncategorised"),
      kind: r.category_id ? r.kind : r.internal ? "transfer" : r.kind,
      total: ZERO,
      count: 0,
    };
    t.total += BigInt(r.total_minor);
    t.count += r.count;
    m.set(key, t);
  }
  return [...m.values()].sort((a, b) => (a.total < b.total ? -1 : a.total > b.total ? 1 : 0));
}

/** The currencies present, most rows first. */
export function currencies(rows: SummaryRow[]): string[] {
  const n = new Map<string, number>();
  for (const r of rows) n.set(r.currency, (n.get(r.currency) ?? 0) + r.count);
  return [...n.entries()].sort((a, b) => b[1] - a[1]).map(([c]) => c);
}

/** BigInt minor units → a Number for a chart axis (display only). */
export function chartValue(v: bigint): number {
  return Number(v) / 100;
}

// ---- periods ---------------------------------------------------------------
// A period is a run of whole months: one, a quarter's three, or a year's
// twelve. Everything the overview shows is a slice of monthly totals, so a
// period is just the set of months to sum -- switching from month to year
// needs no new fetch, only a bigger set.

export type PeriodKind = "month" | "quarter" | "year";

export type Period = {
  kind: PeriodKind;
  /** The first month of the period, YYYY-MM. */
  start: string;
  /** Every month in it, oldest first. */
  months: string[];
};

function ym(y: number, m0: number): string {
  const d = new Date(y, m0, 1);
  return `${d.getFullYear()}-${String(d.getMonth() + 1).padStart(2, "0")}`;
}

/** The period of `kind` that contains the month `anchor`. */
export function periodOf(kind: PeriodKind, anchor: string): Period {
  const [y, m] = anchor.split("-").map(Number) as [number, number];
  const m0 = m - 1;
  if (kind === "month") return { kind, start: anchor, months: [anchor] };
  if (kind === "quarter") {
    const q0 = Math.floor(m0 / 3) * 3;
    return { kind, start: ym(y, q0), months: [0, 1, 2].map((i) => ym(y, q0 + i)) };
  }
  return { kind, start: ym(y, 0), months: Array.from({ length: 12 }, (_, i) => ym(y, i)) };
}

/** The same kind of period, `delta` periods later (negative for earlier). */
export function shiftPeriod(period: Period, delta: number): Period {
  const [y, m] = period.start.split("-").map(Number) as [number, number];
  const step = period.kind === "month" ? 1 : period.kind === "quarter" ? 3 : 12;
  return periodOf(period.kind, ym(y, m - 1 + delta * step));
}

/** The previous period of the same size. */
export function previousPeriod(period: Period): Period {
  return shiftPeriod(period, -1);
}

export type Totals = {
  income: bigint;
  spent: bigint;
  out: bigint;
  in: bigint;
  count: number;
  /** How many of the period's months had any rows at all. */
  monthsWithData: number;
};

/** Sum the monthly totals that fall inside `period`. Months with no rows
 *  count as zero, and `monthsWithData` says how many were real. */
export function periodTotals(months: MonthTotals[], period: Period): Totals {
  const want = new Set(period.months);
  const t: Totals = { income: ZERO, spent: ZERO, out: ZERO, in: ZERO, count: 0, monthsWithData: 0 };
  for (const m of months) {
    if (!want.has(m.month)) continue;
    t.income += m.income;
    t.spent += m.spent;
    t.out += m.out;
    t.in += m.in;
    t.count += m.count;
    t.monthsWithData += 1;
  }
  return t;
}

/** Percent change from `before` to `after`, one decimal; undefined when there
 *  is nothing to compare against. */
export function percentChange(after: bigint, before: bigint): number | undefined {
  if (before === ZERO) return undefined;
  return Number(((after - before) * 1000n) / (before < ZERO ? -before : before)) / 10;
}

/** What stayed: (in − out) ÷ in, as a percentage; null with nothing in. */
export function keptRate(t: { in: bigint; out: bigint }): number | null {
  if (t.in === ZERO) return null;
  return Number(((t.in - t.out) * 1000n) / t.in) / 10;
}

/** A year against the one before it, month by month (index 0 = January).
 *  Missing months are null so a chart can leave a gap rather than draw zero. */
export function yearOverYear(
  months: MonthTotals[],
  year: string,
  pick: (m: MonthTotals) => bigint,
): { month: string; thisYear: number | null; lastYear: number | null }[] {
  const by = new Map(months.map((m) => [m.month, m]));
  const y = Number(year);
  return Array.from({ length: 12 }, (_, i) => {
    const cur = by.get(ym(y, i));
    const prev = by.get(ym(y - 1, i));
    return {
      month: ym(y, i),
      thisYear: cur ? chartValue(pick(cur)) : null,
      lastYear: prev ? chartValue(pick(prev)) : null,
    };
  });
}

/** The period's first and last day, YYYY-MM-DD, for date-bounded API calls. */
export function periodBounds(period: Period): { from: string; to: string } {
  const first = period.months[0] ?? period.start;
  const last = period.months.at(-1) ?? period.start;
  const [y, m] = last.split("-").map(Number) as [number, number];
  const end = new Date(y, m, 0).getDate();
  return { from: `${first}-01`, to: `${last}-${String(end).padStart(2, "0")}` };
}

/** The `n` months ending with `last`, oldest first, as YYYY-MM. */
export function monthsEnding(last: string, n: number): string[] {
  const [y, m] = last.split("-").map(Number) as [number, number];
  return Array.from({ length: n }, (_, i) => ym(y, m - 1 - (n - 1 - i)));
}

/** Money out per month in categories of one `kind` (say `tax`), minor units,
 *  positive, for one currency; months with none are absent. */
export function outflowOfKind(rows: SummaryRow[], currency: string, kind: string): Map<string, bigint> {
  const m = new Map<string, bigint>();
  for (const r of rows) {
    if (r.currency !== currency || r.kind !== kind) continue;
    const v = BigInt(r.total_minor);
    if (v >= ZERO) continue;
    m.set(r.month, (m.get(r.month) ?? ZERO) - v);
  }
  return m;
}
