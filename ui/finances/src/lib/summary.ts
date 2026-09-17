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
      if (r.kind === "expense" || r.kind === "tax" || r.kind === "") t.spent += abs(v);
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
    const key = r.category_id || "none";
    const t = m.get(key) ?? {
      category_id: key,
      category: r.category || "Uncategorised",
      kind: r.kind,
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
