"use client";

// Money in against money spent over twelve months with the chosen period lit;
// in a year view, the same year against the one before it as a second tab.
import { BarChart3 } from "lucide-react";
import { useState } from "react";

import { Legend, MoneyFlowChart, YearOverYearChart } from "@/components/charts";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { useT } from "@/lib/i18n";
import { type MonthTotals, type Period, yearOverYear } from "@/lib/summary";

import { Empty, WidgetCard } from "./cards";

export function FlowCard({
  window,
  all,
  period,
  currency,
  onPickMonth,
  loading,
}: {
  /** The months the bar chart draws, oldest first. */
  window: MonthTotals[];
  /** Every month fetched, for the year-over-year view. */
  all: MonthTotals[];
  period: Period;
  currency: string;
  onPickMonth: (ym: string) => void;
  loading: boolean;
}) {
  const t = useT();
  const [view, setView] = useState<"flow" | "yoy">("flow");
  const year = period.start.slice(0, 4);
  const showYoy = period.kind === "year" && view === "yoy";

  return (
    <WidgetCard
      icon={BarChart3}
      title={t("overview.flow_title")}
      description={showYoy ? `${t("overview.yoy")} · ${t("overview.yoy.spent")}` : t("overview.flow_desc_p")}
      actions={
        <div className="flex flex-wrap items-center gap-3">
          {showYoy ? (
            <Legend
              items={[
                { label: t("overview.yoy.this"), color: "var(--chart-1)" },
                { label: t("overview.yoy.last"), color: "var(--chart-4)" },
              ]}
            />
          ) : (
            <Legend
              items={[
                { label: t("nav.chart.in"), color: "var(--chart-2)" },
                { label: t("nav.chart.spent"), color: "var(--chart-1)" },
                { label: t("nav.chart.other"), color: "var(--chart-4)" },
              ]}
            />
          )}
          {period.kind === "year" ? (
            <Tabs value={view} onValueChange={(v) => setView(v as "flow" | "yoy")}>
              <TabsList className="h-8">
                <TabsTrigger value="flow" className="text-xs">
                  {t("overview.flow_title")}
                </TabsTrigger>
                <TabsTrigger value="yoy" className="text-xs">
                  {t("overview.yoy")}
                </TabsTrigger>
              </TabsList>
            </Tabs>
          ) : null}
        </div>
      }
    >
      {loading ? (
        <Skeleton className="h-[280px] w-full" />
      ) : window.length === 0 ? (
        <Empty>{t("overview.no_booked")}</Empty>
      ) : showYoy ? (
        <YearOverYearChart data={yearOverYear(all, year, (m) => m.spent)} currency={currency} />
      ) : (
        <MoneyFlowChart months={window} currency={currency} range={period.months} onSelect={onPickMonth} />
      )}
    </WidgetCard>
  );
}
