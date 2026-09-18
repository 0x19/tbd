"use client";

// Where the money went in the period: the biggest categories as a donut,
// the long tail folded into "Other", every slice a link into the ledger.
import { PieChart } from "lucide-react";

import { CategoryDonut, type Slice } from "@/components/charts";
import { Skeleton } from "@/components/ui/skeleton";
import { periodLabel } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { type CategoryTotal, chartValue, type Period } from "@/lib/summary";

import { Empty, WidgetCard } from "./cards";

const COLORS = ["var(--chart-1)", "var(--chart-2)", "var(--chart-3)", "var(--chart-4)", "var(--chart-5)"];
const TOP = 5;

export function DonutCard({
  categories,
  period,
  currency,
  onPick,
  loading,
}: {
  /** Outflow categories, biggest first (negative totals). */
  categories: CategoryTotal[];
  period: Period;
  currency: string;
  onPick: (category_id: string) => void;
  loading: boolean;
}) {
  const t = useT();
  const out = categories.filter((c) => c.total < 0n);
  const total = out.reduce((s, c) => s - c.total, 0n);
  const head = out.slice(0, TOP);
  const tail = out.slice(TOP);
  const pct = (v: bigint) => (total > 0n ? Number((v * 10000n) / total) / 100 : 0);
  const slices: Slice[] = head.map((c, i) => ({
    id: c.category_id,
    label: c.category,
    value: chartValue(-c.total),
    pct: pct(-c.total),
    color: COLORS[i % COLORS.length]!,
  }));
  if (tail.length) {
    const rest = tail.reduce((s, c) => s - c.total, 0n);
    slices.push({
      id: "",
      label: t("overview.donut.other"),
      value: chartValue(rest),
      pct: pct(rest),
      color: "var(--muted-foreground)",
    });
  }

  return (
    <WidgetCard
      icon={PieChart}
      title={t("overview.donut.title")}
      description={t("overview.donut.desc", { period: periodLabel(period.kind, period.start) })}
      className="min-h-full"
    >
      {loading ? (
        <Skeleton className="h-48 w-full" />
      ) : slices.length === 0 ? (
        <Empty>{t("overview.donut.empty")}</Empty>
      ) : (
        <CategoryDonut slices={slices} currency={currency} onPick={(id) => (id ? onPick(id) : undefined)} />
      )}
    </WidgetCard>
  );
}
