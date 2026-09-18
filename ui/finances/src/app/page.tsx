"use client";

import { ArrowDownRight, ArrowUpRight, PiggyBank, Receipt, Wallet } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { useFinance } from "@/app/providers";
import { CategoryBars, Legend, MoneyFlowChart, SparkLine } from "@/components/charts";
import { KpiStrip, PageTitle } from "@/components/kit";
import { MonthStepper } from "@/components/month-stepper";
import { ScopeToggle } from "@/components/scope-toggle";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import { money, monthLabel, monthsBefore, thisMonth } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { byCategory, byMonth, chartValue, currencies } from "@/lib/summary";

const WINDOWS = [3, 6, 12, 24] as const;

export default function OverviewPage() {
  const t = useT();
  const router = useRouter();
  const { partyIds, scope, loading: partiesLoading, error: partiesError } = useFinance();
  const [window, setWindow] = useState<(typeof WINDOWS)[number]>(12);
  const from = monthsBefore(thisMonth(), window - 1);
  // Transfers between your own accounts count twice in a combined view, so
  // there they are left out. In a single party's view they are real: an owner
  // draw is the company's money out and the person's money in.
  const includeInternal = scope !== "all";
  const summary = useFetch(() => api.summary(partyIds, from, includeInternal), 60_000, [
    partyIds.join(","),
    from,
    includeInternal,
  ]);
  const rows = useMemo(() => summary.data?.rows ?? [], [summary.data]);
  const ccys = useMemo(() => currencies(rows), [rows]);
  const [currency, setCurrency] = useState<string | null>(null);
  const ccy = currency && ccys.includes(currency) ? currency : (ccys[0] ?? "EUR");
  const months = useMemo(() => byMonth(rows, ccy), [rows, ccy]);
  const [month, setMonth] = useState<string | null>(null);
  const current =
    month && months.some((m) => m.month === month) ? month : (months.at(-1)?.month ?? thisMonth());
  const cur = months.find((m) => m.month === current);
  const prev = months[months.findIndex((m) => m.month === current) - 1];
  const cats = useMemo(
    () => byCategory(rows, ccy, new Set([current])).filter((c) => c.total < 0n),
    [rows, ccy, current],
  );
  const spentTotal = cats
    .filter((c) => c.kind === "expense" || c.kind === "tax" || c.kind === "")
    .reduce((s, c) => s - c.total, 0n);
  const allOut = cats.reduce((s, c) => s - c.total, 0n);
  const uncategorised = cats.find((c) => c.category_id === "none");

  const delta = (a?: bigint, b?: bigint) =>
    a == null || b == null || b === 0n ? undefined : Number(((a - b) * 1000n) / b) / 10;
  const loading = partiesLoading || (summary.loading && !summary.data);
  const error = partiesError ?? summary.error;

  return (
    <>
      <PageTitle
        title={t("overview.title")}
        description={includeInternal ? t("overview.desc_internal") : t("overview.desc_no_internal")}
      >
        <ScopeToggle className="md:hidden" />
        <Select
          value={String(window)}
          onValueChange={(v) => setWindow(Number(v) as (typeof WINDOWS)[number])}
        >
          <SelectTrigger className="w-36">
            <SelectValue />
          </SelectTrigger>
          <SelectContent>
            {WINDOWS.map((w) => (
              <SelectItem key={w} value={String(w)}>
                {t("overview.last_n_months", { n: w })}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        {ccys.length > 1 ? (
          <Tabs value={ccy} onValueChange={setCurrency}>
            <TabsList>
              {ccys.map((c) => (
                <TabsTrigger key={c} value={c}>
                  {c}
                </TabsTrigger>
              ))}
            </TabsList>
          </Tabs>
        ) : null}
      </PageTitle>

      {error ? (
        <Card className="border-destructive/40">
          <CardHeader>
            <CardTitle>{t("overview.nothing_to_show")}</CardTitle>
            <CardDescription>{error}</CardDescription>
          </CardHeader>
        </Card>
      ) : null}

      {loading ? (
        <Skeleton className="h-28 w-full" />
      ) : (
        <KpiStrip
          items={[
            {
              icon: ArrowDownRight,
              label: t("overview.kpi.spent", { month: monthLabel(current) }),
              value: cur ? money(cur.spent.toString(), ccy) : "—",
              previous: prev ? `${monthLabel(prev.month)}: ${money(prev.spent.toString(), ccy)}` : undefined,
              delta:
                cur && prev && delta(cur.spent, prev.spent) != null
                  ? {
                      value: delta(cur.spent, prev.spent)!,
                      label: t("overview.vs_previous_month"),
                      goodWhen: "down",
                    }
                  : undefined,
            },
            {
              icon: ArrowUpRight,
              label: t("overview.kpi.in", { month: monthLabel(current) }),
              value: cur ? money(cur.in.toString(), ccy) : "—",
              previous: prev ? `${monthLabel(prev.month)}: ${money(prev.in.toString(), ccy)}` : undefined,
              delta:
                cur && prev && delta(cur.in, prev.in) != null
                  ? { value: delta(cur.in, prev.in)!, label: t("overview.vs_previous_month"), goodWhen: "up" }
                  : undefined,
            },
            {
              icon: PiggyBank,
              label: t("overview.kpi.net", { month: monthLabel(current) }),
              value: cur ? money((cur.in - cur.out).toString(), ccy, { sign: true }) : "—",
              hint: t("overview.net_hint"),
            },
            {
              icon: Receipt,
              label: t("overview.uncategorised"),
              value: uncategorised ? money((-uncategorised.total).toString(), ccy) : money(0, ccy),
              hint: uncategorised
                ? t("overview.uncategorised_hint", { n: uncategorised.count })
                : t("overview.all_categorised_hint"),
            },
          ]}
        />
      )}

      <div className="grid gap-6 xl:grid-cols-5">
        <Card className="xl:col-span-3">
          <CardHeader className="flex flex-row flex-wrap items-start justify-between gap-3">
            <div>
              <CardTitle>{t("overview.flow_title")}</CardTitle>
              <CardDescription>{t("overview.flow_desc")}</CardDescription>
            </div>
            <div className="flex flex-wrap items-center gap-4">
              <Legend
                items={[
                  { label: t("nav.chart.in"), color: "var(--chart-2)" },
                  { label: t("nav.chart.spent"), color: "var(--chart-1)" },
                  { label: t("nav.chart.other"), color: "var(--chart-4)" },
                ]}
              />
              {months.length ? (
                <MonthStepper months={months.map((m) => m.month)} value={current} onChange={setMonth} />
              ) : null}
            </div>
          </CardHeader>
          <CardContent>
            {loading ? (
              <Skeleton className="h-[280px] w-full" />
            ) : months.length ? (
              <MoneyFlowChart months={months} currency={ccy} selected={current} onSelect={setMonth} />
            ) : (
              <p className="text-muted-foreground text-sm">{t("overview.no_booked")}</p>
            )}
          </CardContent>
        </Card>

        <Card className="xl:col-span-2">
          <CardHeader>
            <CardTitle>{t("overview.by_category", { month: monthLabel(current) })}</CardTitle>
            <CardDescription>
              {cur
                ? t("overview.out_across", {
                    amount: money(allOut.toString(), ccy),
                    n: cats.reduce((n, c) => n + c.count, 0),
                  })
                : "—"}
            </CardDescription>
          </CardHeader>
          <CardContent>
            {loading ? (
              <Skeleton className="h-64 w-full" />
            ) : cats.length ? (
              <CategoryBars
                items={cats}
                currency={ccy}
                total={spentTotal > 0n ? spentTotal : allOut}
                onPick={(id) =>
                  id === "internal"
                    ? undefined
                    : router.push(`/transactions/?month=${current}&category=${id}`)
                }
              />
            ) : (
              <p className="text-muted-foreground text-sm">{t("overview.nothing_spent")}</p>
            )}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-6 md:grid-cols-3">
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>{t("overview.spent_monthly")}</CardDescription>
            <CardTitle className="text-2xl tabular-nums">
              {months.length
                ? money((months.reduce((s, m) => s + m.spent, 0n) / BigInt(months.length)).toString(), ccy)
                : "—"}
              <span className="text-muted-foreground ml-2 text-sm font-normal">{t("overview.avg")}</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            {months.length ? <SparkLine values={months.map((m) => chartValue(m.spent))} /> : null}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>{t("overview.in_monthly")}</CardDescription>
            <CardTitle className="text-2xl tabular-nums">
              {months.length
                ? money((months.reduce((s, m) => s + m.in, 0n) / BigInt(months.length)).toString(), ccy)
                : "—"}
              <span className="text-muted-foreground ml-2 text-sm font-normal">{t("overview.avg")}</span>
            </CardTitle>
          </CardHeader>
          <CardContent>
            {months.length ? <SparkLine values={months.map((m) => chartValue(m.in))} /> : null}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="pb-2">
            <CardDescription>{t("overview.accounts")}</CardDescription>
            <CardTitle className="flex items-center gap-2 text-2xl">
              <Wallet className="text-muted-foreground size-5" /> {t("overview.balances_sync")}
            </CardTitle>
          </CardHeader>
          <CardContent className="flex items-end justify-between gap-2">
            <p className="text-muted-foreground text-sm">{t("overview.accounts_desc")}</p>
            <Button asChild variant="outline" size="sm">
              <Link href="/accounts/">{t("overview.open")}</Link>
            </Button>
          </CardContent>
        </Card>
      </div>
    </>
  );
}
