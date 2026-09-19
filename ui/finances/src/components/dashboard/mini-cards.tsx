"use client";

// Three small cards that put the period in context rather than restating it:
// the twelve-month average of spending and of what comes in, each with how
// this period sits against it, and taxes and contributions -- the one outflow
// a company owner watches on its own -- for the period and the year so far.
import { ArrowDownRight, ArrowUpRight, Landmark } from "lucide-react";

import { SparkLine } from "@/components/charts";
import { money, periodLabel } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { chartValue, type MonthTotals, percentChange, type Period, type Totals } from "@/lib/summary";

import { MiniChartCard } from "./cards";

function avg(values: bigint[]): bigint {
  return values.length ? values.reduce((s, v) => s + v, 0n) / BigInt(values.length) : 0n;
}

export function MiniCards({
  window,
  before,
  period,
  cur,
  taxes,
  currency,
  loading,
}: {
  /** The twelve months in view (or the year's), oldest first. */
  window: MonthTotals[];
  /** The same run of months before them: last year, in a year view. */
  before: MonthTotals[];
  period: Period;
  /** The period's own totals. */
  cur: Totals;
  taxes: { window: bigint[]; before: bigint; period: bigint; ytd: bigint; any: boolean };
  currency: string;
  loading: boolean;
}) {
  const t = useT();
  const label = periodLabel(period.kind, period.start);
  const n = BigInt(period.months.length);
  // In a year view the window *is* the period, so the baseline is the year
  // before; otherwise it is the trailing twelve months the sparkline draws.
  const yearly = period.kind === "year";
  const base = yearly ? before : window;
  const vs = t(yearly ? "overview.mini.vs_last_year" : "overview.mini.vs_avg");
  const has = (rows: MonthTotals[]) => rows.some((m) => m.count > 0);

  const spentAvg = avg(base.map((m) => m.spent));
  const inAvg = avg(base.map((m) => m.in));
  const taxBase = yearly ? taxes.before / BigInt(before.length || 1) : avg(taxes.window);

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MiniChartCard
        icon={ArrowDownRight}
        title={t("overview.mini.spent_avg")}
        value={money(spentAvg.toString(), currency)}
        delta={has(base) ? percentChange(cur.spent / n, spentAvg) : undefined}
        deltaLabel={vs}
        goodWhen="down"
        hint={t("overview.mini.this_period", { period: label, value: money(cur.spent.toString(), currency) })}
        loading={loading}
      >
        <SparkLine values={window.map((m) => chartValue(m.spent))} />
      </MiniChartCard>
      <MiniChartCard
        icon={ArrowUpRight}
        title={t("overview.mini.in_avg")}
        value={money(inAvg.toString(), currency)}
        delta={has(base) ? percentChange(cur.in / n, inAvg) : undefined}
        deltaLabel={vs}
        goodWhen="up"
        hint={t("overview.mini.this_period", { period: label, value: money(cur.in.toString(), currency) })}
        loading={loading}
      >
        <SparkLine values={window.map((m) => chartValue(m.in))} />
      </MiniChartCard>
      <MiniChartCard
        icon={Landmark}
        title={t("overview.mini.tax")}
        value={taxes.any ? money(taxes.period.toString(), currency) : "—"}
        delta={taxes.any && taxBase > 0n ? percentChange(taxes.period / n, taxBase) : undefined}
        deltaLabel={taxes.any ? vs : ""}
        goodWhen="down"
        hint={
          taxes.any
            ? t("overview.mini.tax_ytd", { value: money(taxes.ytd.toString(), currency) })
            : t("overview.mini.tax_none")
        }
        loading={loading}
      >
        {taxes.any ? <SparkLine values={taxes.window.map(chartValue)} /> : null}
      </MiniChartCard>
    </div>
  );
}
