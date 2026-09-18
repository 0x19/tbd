"use client";

// The overview: one period -- a month, a quarter or a year -- and everything
// on the page follows it. The numbers come from one 24-month summary fetch
// sliced client-side (a period is just a set of months), plus what the
// accounts, invoices, receipts and the ledger say right now. Each widget
// loads and empties on its own, so one slow endpoint dims one card.
import { ArrowDownRight, ArrowUpRight, PiggyBank, Receipt } from "lucide-react";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";

import { useFinance } from "@/app/providers";
import { AccountsCard } from "@/components/dashboard/accounts-card";
import { ActivityCard } from "@/components/dashboard/activity-card";
import { DonutCard } from "@/components/dashboard/donut-card";
import { FlowCard } from "@/components/dashboard/flow-card";
import { InvoicesCard } from "@/components/dashboard/invoices-card";
import { MiniCards } from "@/components/dashboard/mini-cards";
import { PeriodControl } from "@/components/dashboard/period-control";
import { ReceiptsCard } from "@/components/dashboard/receipts-card";
import { YearTable } from "@/components/dashboard/year-table";
import { KpiStrip, PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { Card, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import { money, monthsBefore, periodLabel, thisMonth } from "@/lib/format";
import { useT } from "@/lib/i18n";
import {
  byCategory,
  byMonth,
  currencies,
  monthsEnding,
  type MonthTotals,
  percentChange,
  type Period,
  periodBounds,
  periodOf,
  periodTotals,
  previousPeriod,
} from "@/lib/summary";

/** Enough history for a year and the year before it. */
const HISTORY = 24;

/** The months a chart draws: every month in the run, zero where nothing was booked. */
function fill(months: MonthTotals[], run: string[]): MonthTotals[] {
  const by = new Map(months.map((m) => [m.month, m]));
  return run.map((ym) => by.get(ym) ?? { month: ym, income: 0n, spent: 0n, out: 0n, in: 0n, count: 0 });
}

export default function OverviewPage() {
  const t = useT();
  const router = useRouter();
  const { partyIds, scope, loading: partiesLoading, error: partiesError } = useFinance();
  const key = partyIds.join(",");
  const from = monthsBefore(thisMonth(), HISTORY - 1);
  // Transfers between your own accounts count twice in a combined view, so
  // there they are left out. In a single party's view they are real.
  const includeInternal = scope !== "all";

  const summary = useFetch(() => api.summary(partyIds, from, includeInternal), 60_000, [
    key,
    from,
    includeInternal,
  ]);
  const rows = useMemo(() => summary.data?.rows ?? [], [summary.data]);
  const ccys = useMemo(() => currencies(rows), [rows]);
  const [currency, setCurrency] = useState<string | null>(null);
  const ccy = currency && ccys.includes(currency) ? currency : (ccys[0] ?? "EUR");
  const months = useMemo(() => byMonth(rows, ccy), [rows, ccy]);
  const monthList = useMemo(() => months.map((m) => m.month), [months]);

  // The period: the newest month with data until the person picks one.
  const [chosen, setChosen] = useState<Period | null>(null);
  const period = chosen ?? periodOf("month", monthList.at(-1) ?? thisMonth());
  const last = period.months.at(-1) ?? period.start;
  const prevPeriod = previousPeriod(period);

  const cur = useMemo(() => periodTotals(months, period), [months, period]);
  const prev = useMemo(() => periodTotals(months, prevPeriod), [months, prevPeriod]);
  const cats = useMemo(() => byCategory(rows, ccy, new Set(period.months)), [rows, ccy, period]);
  const uncategorised = cats.find((c) => c.category_id === "none" && c.total < 0n);

  // Twelve months ending with the period for the bar chart; the mini cards
  // compare the period's own run of months with the same run before it.
  const flowWindow = useMemo(() => fill(months, monthsEnding(last, 12)), [months, last]);
  const miniRun = period.kind === "year" ? period.months : monthsEnding(last, 12);
  const miniWindow = useMemo(() => fill(months, miniRun), [months, miniRun]);
  const miniBefore = useMemo(
    () => fill(months, monthsEnding(monthsBefore(miniRun[0] ?? last, 1), miniRun.length)),
    [months, miniRun, last],
  );
  const yearMonths = useMemo(() => months.filter((m) => period.months.includes(m.month)), [months, period]);

  const bounds = periodBounds(period);
  const accounts = useFetch(() => api.accounts(partyIds), 60_000, [key]);
  const invoices = useFetch(() => api.invoices(partyIds), 60_000, [key]);
  const clients = useFetch(() => api.clients(partyIds), 5 * 60_000, [key]);
  const documents = useFetch(
    () => api.documents({ party_ids: partyIds, from: bounds.from, to: bounds.to, limit: 50 }),
    60_000,
    [key, bounds.from, bounds.to],
  );
  const recent = useFetch(() => api.transactions({ party_ids: partyIds, limit: 8 }), 60_000, [key]);
  const clientName = (id: string) => clients.data?.clients.find((c) => c.id === id)?.name;

  const loading = partiesLoading || (summary.loading && !summary.data);
  const error = partiesError ?? summary.error;
  const label = periodLabel(period.kind, period.start);
  const vsPrev = t(`overview.vs_prev.${period.kind}`);
  const delta = (a: bigint, b: bigint) => (prev.monthsWithData ? percentChange(a, b) : undefined);
  const pickMonth = (ym: string) => setChosen(periodOf("month", ym));
  const pickCategory = (id: string) =>
    router.push(`/transactions/?category=${id}${period.kind === "month" ? `&month=${period.start}` : ""}`);

  return (
    <>
      <PageTitle
        title={t("overview.title")}
        description={includeInternal ? t("overview.desc_internal") : t("overview.desc_no_internal")}
      >
        <ScopeToggle className="md:hidden" />
        <PeriodControl period={period} months={monthList} onChange={setChosen} />
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
              label: t("overview.kpi.spent_p", { period: label }),
              value: money(cur.spent.toString(), ccy),
              previous: prev.monthsWithData
                ? t("overview.prev_value", { value: money(prev.spent.toString(), ccy) })
                : undefined,
              delta:
                delta(cur.spent, prev.spent) != null
                  ? { value: delta(cur.spent, prev.spent)!, label: vsPrev, goodWhen: "down" }
                  : undefined,
              hint: t("overview.kpi.rows", { n: cur.count }),
            },
            {
              icon: ArrowUpRight,
              label: t("overview.kpi.in_p", { period: label }),
              value: money(cur.in.toString(), ccy),
              previous: prev.monthsWithData
                ? t("overview.prev_value", { value: money(prev.in.toString(), ccy) })
                : undefined,
              delta:
                delta(cur.in, prev.in) != null
                  ? { value: delta(cur.in, prev.in)!, label: vsPrev, goodWhen: "up" }
                  : undefined,
            },
            {
              icon: PiggyBank,
              label: t("overview.kpi.net_p", { period: label }),
              value: money((cur.in - cur.out).toString(), ccy, { sign: true }),
              previous: prev.monthsWithData
                ? t("overview.prev_value", {
                    value: money((prev.in - prev.out).toString(), ccy, { sign: true }),
                  })
                : undefined,
              hint: t("overview.net_hint"),
            },
            {
              icon: Receipt,
              label: t("overview.uncategorised"),
              value: uncategorised ? money((-uncategorised.total).toString(), ccy) : money(0, ccy),
              hint: uncategorised
                ? t("overview.uncategorised_rows", { n: uncategorised.count })
                : t("overview.all_categorised_hint"),
            },
          ]}
        />
      )}

      <div className="grid gap-4 xl:grid-cols-5">
        <div className="xl:col-span-3">
          <FlowCard
            window={flowWindow}
            all={months}
            period={period}
            currency={ccy}
            onPickMonth={pickMonth}
            loading={loading}
          />
        </div>
        <div className="xl:col-span-2">
          <DonutCard
            categories={cats}
            period={period}
            currency={ccy}
            onPick={pickCategory}
            loading={loading}
          />
        </div>
      </div>

      <MiniCards window={miniWindow} before={miniBefore} currency={ccy} loading={loading} />

      {period.kind === "year" ? (
        <YearTable months={yearMonths} currency={ccy} onPick={pickMonth} loading={loading} />
      ) : null}

      <div className="grid gap-4 lg:grid-cols-3">
        <AccountsCard accounts={accounts.data?.accounts ?? []} loading={accounts.loading && !accounts.data} />
        <InvoicesCard
          invoices={invoices.data?.invoices ?? []}
          period={period}
          currency={ccy}
          clientName={clientName}
          loading={invoices.loading && !invoices.data}
        />
        <ReceiptsCard
          documents={documents.data?.documents ?? []}
          total={documents.data?.total ?? 0}
          loading={documents.loading && !documents.data}
        />
      </div>

      <ActivityCard transactions={recent.data?.transactions ?? []} loading={recent.loading && !recent.data} />
    </>
  );
}
