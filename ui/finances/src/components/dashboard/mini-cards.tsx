"use client";

// Three small cards over a trailing window of months: what goes out per
// month, what comes in, and what stays -- each with a sparkline and a
// change against the window before it, so a drift shows before a month does.
import { ArrowDownRight, ArrowUpRight, PiggyBank } from "lucide-react";

import { SparkLine } from "@/components/charts";
import { money } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { chartValue, keptRate, type MonthTotals, percentChange } from "@/lib/summary";

import { MiniChartCard } from "./cards";

function avg(values: bigint[]): bigint {
  return values.length ? values.reduce((s, v) => s + v, 0n) / BigInt(values.length) : 0n;
}

export function MiniCards({
  window,
  before,
  currency,
  loading,
}: {
  /** The months in view, oldest first. */
  window: MonthTotals[];
  /** The same number of months before them, for the change. */
  before: MonthTotals[];
  currency: string;
  loading: boolean;
}) {
  const t = useT();
  const n = window.length;
  const spentAvg = avg(window.map((m) => m.spent));
  const inAvg = avg(window.map((m) => m.in));
  const kept = keptRate({
    in: window.reduce((s, m) => s + m.in, 0n),
    out: window.reduce((s, m) => s + m.out, 0n),
  });
  const keptBefore = keptRate({
    in: before.reduce((s, m) => s + m.in, 0n),
    out: before.reduce((s, m) => s + m.out, 0n),
  });
  const vs = t("overview.mini.vs_prev_avg", { n: before.length || n });
  const over = t("overview.mini.avg_over", { n });

  return (
    <div className="grid gap-4 md:grid-cols-3">
      <MiniChartCard
        icon={ArrowDownRight}
        title={t("overview.mini.spent")}
        value={money(spentAvg.toString(), currency)}
        delta={before.length ? percentChange(spentAvg, avg(before.map((m) => m.spent))) : undefined}
        deltaLabel={vs}
        goodWhen="down"
        hint={over}
        loading={loading}
      >
        <SparkLine values={window.map((m) => chartValue(m.spent))} />
      </MiniChartCard>
      <MiniChartCard
        icon={ArrowUpRight}
        title={t("overview.mini.in")}
        value={money(inAvg.toString(), currency)}
        delta={before.length ? percentChange(inAvg, avg(before.map((m) => m.in))) : undefined}
        deltaLabel={vs}
        goodWhen="up"
        hint={over}
        loading={loading}
      >
        <SparkLine values={window.map((m) => chartValue(m.in))} />
      </MiniChartCard>
      <MiniChartCard
        icon={PiggyBank}
        title={t("overview.mini.kept")}
        value={kept == null ? "—" : `${kept.toFixed(0)}%`}
        delta={kept != null && keptBefore != null ? Math.round((kept - keptBefore) * 10) / 10 : undefined}
        deltaLabel={vs}
        goodWhen="up"
        hint={t("overview.mini.kept_hint")}
        loading={loading}
      >
        <SparkLine values={window.map((m) => keptRate(m) ?? 0)} />
      </MiniChartCard>
    </div>
  );
}
