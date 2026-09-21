"use client";

// Books: the trial balance of one company for one fiscal year, class by
// class, from the opening the accountant handed over plus the postings, with
// the twelve months and their status. Every figure is the service's integer;
// the page adds nothing up that the service did not. Filters live in the URL.
import { CalendarCheck, ChevronDown, Lock, LockOpen, Scale, Sigma } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { ImportOpening } from "@/components/books/import-opening";
import { ConfirmDialog } from "@/components/confirm-dialog";
import { KpiStrip, PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { Badge } from "@/components/ui/badge";
import { Card, CardContent } from "@/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Period, TrialBalanceClass, TrialBalanceRow } from "@/lib/api/schema";
import { CLASSES, mm, rowsOfClass } from "@/lib/books";
import { dateOf, money } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

export default function BooksPage() {
  return (
    <Suspense>
      <Books />
    </Suspense>
  );
}

const MONEY_COLUMNS = [
  "opening_debit_minor",
  "opening_credit_minor",
  "period_debit_minor",
  "period_credit_minor",
  "total_debit_minor",
  "total_credit_minor",
  "balance_minor",
] as const;

const COLUMN_LABEL: Record<(typeof MONEY_COLUMNS)[number], string> = {
  opening_debit_minor: "books.opening_debit",
  opening_credit_minor: "books.opening_credit",
  period_debit_minor: "books.period_debit",
  period_credit_minor: "books.period_credit",
  total_debit_minor: "books.total_debit",
  total_credit_minor: "books.total_credit",
  balance_minor: "books.balance",
};

function Books() {
  const t = useT();
  const router = useRouter();
  const params = useSearchParams();
  const { parties, partyIds, partyName } = useFinance();
  // Books belong to companies: the companies in scope, the first one by default.
  const companies = useMemo(
    () => parties.filter((p) => p.kind === "org" && partyIds.includes(p.id)),
    [parties, partyIds],
  );
  const wanted = params.get("party") ?? "";
  const party = companies.find((p) => p.id === wanted)?.id ?? companies[0]?.id ?? "";
  const owner = companies.find((p) => p.id === party)?.capability === "own";
  const year = Number(params.get("year") ?? "0") || 0;
  const through = Number(params.get("through") ?? "0") || 0;
  const [lockAsk, setLockAsk] = useState<Period | null>(null);

  const set = (patch: Record<string, string>) => {
    const next = new URLSearchParams(params.toString());
    for (const [k, v] of Object.entries(patch)) {
      if (v) next.set(k, v);
      else next.delete(k);
    }
    const s = next.toString();
    router.replace(s ? `/books/?${s}` : "/books/");
  };

  const tb = useFetch(
    () =>
      party
        ? api.trialBalance({ party_id: party, fiscal_year: year, through_month: through })
        : Promise.resolve(null),
    30_000,
    [party, year, through],
  );
  const data = tb.data;
  const fiscalYear = data?.fiscal_year ?? (year || new Date().getFullYear());
  // The years with books, plus the last five so an opening can be imported
  // into a year that has nothing yet.
  const years = useMemo(() => {
    const now = new Date().getFullYear();
    const ys = new Set<number>(data?.years ?? []);
    ys.add(fiscalYear);
    for (let y = now; y >= now - 5; y--) ys.add(y);
    return [...ys].sort((a, b) => b - a);
  }, [data?.years, fiscalYear]);
  const rows = data?.rows ?? [];
  const difference = data ? BigInt(data.total_debit_minor) - BigInt(data.total_credit_minor) : 0n;

  const period = async (p: Period, action: "close" | "reopen" | "lock") => {
    const ym = `${p.fiscal_year}-${mm(p.month)}`;
    try {
      const r =
        action === "lock"
          ? await api.lockPeriod(party, p.fiscal_year, p.month)
          : await api.closePeriod(party, p.fiscal_year, p.month, action === "reopen");
      toast.success(
        t("books.period.changed", { status: t(`books.period.${r.period?.status ?? "open"}`), ym }),
      );
      tb.reload();
    } catch (e) {
      toast.error(t("books.period.failed", { ym, reason: describe(e) }));
    }
  };

  return (
    <>
      <PageTitle title={t("books.title")} description={t("books.description")}>
        <ScopeToggle className="md:hidden" />
        <div className="flex items-center gap-2">
          {companies.length > 1 ? (
            <Select value={party} onValueChange={(v) => set({ party: v })}>
              <SelectTrigger className="w-48">
                <SelectValue placeholder={t("books.for")} />
              </SelectTrigger>
              <SelectContent>
                {companies.map((p) => (
                  <SelectItem key={p.id} value={p.id}>
                    {partyName(p.id)}
                  </SelectItem>
                ))}
              </SelectContent>
            </Select>
          ) : null}
          <Select value={String(fiscalYear)} onValueChange={(v) => set({ year: v })}>
            <SelectTrigger className="w-28">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              {years.map((y) => (
                <SelectItem key={y} value={String(y)}>
                  {y}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          <Select
            value={through ? String(through) : "all"}
            onValueChange={(v) => set({ through: v === "all" ? "" : v })}
          >
            <SelectTrigger className="w-36">
              <SelectValue />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="all">{t("books.through_all")}</SelectItem>
              {Array.from({ length: 12 }, (_, i) => i + 1).map((m) => (
                <SelectItem key={m} value={String(m)}>
                  {t("books.through")} {mm(m)}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {owner ? (
            <ImportOpening
              partyId={party}
              year={fiscalYear}
              hasOpening={Boolean(data?.opening_as_of)}
              onImported={() => tb.reload()}
            />
          ) : null}
        </div>
      </PageTitle>

      {!party ? (
        <Card>
          <CardContent className="text-muted-foreground py-10 text-center text-sm">
            {t("books.person")}
          </CardContent>
        </Card>
      ) : tb.loading && !data ? (
        <Skeleton className="h-28 w-full" />
      ) : data ? (
        <>
          <KpiStrip
            items={[
              {
                icon: Sigma,
                label: t("books.kpi.debit"),
                value: money(data.total_debit_minor, data.currency),
                hint: t("books.kpi.accounts", { n: rows.length }),
              },
              {
                icon: Sigma,
                label: t("books.kpi.credit"),
                value: money(data.total_credit_minor, data.currency),
                hint: t("books.kpi.hint"),
              },
              {
                icon: Scale,
                label: data.balanced ? t("books.kpi.balanced") : t("books.kpi.difference"),
                value: data.balanced ? "✓" : money(difference.toString(), data.currency, { sign: true }),
                hint: `${data.fiscal_year} · ${t("books.through")} ${mm(data.through_month)}`,
              },
              {
                icon: CalendarCheck,
                label: t("books.kpi.opening"),
                value: data.opening_as_of ? dateOf(data.opening_as_of) : "—",
                hint: data.opening_as_of ? "" : t("books.kpi.no_opening"),
              },
            ]}
          />

          <Card>
            <CardContent className="flex flex-wrap items-center gap-2 py-3">
              <span className="text-muted-foreground mr-1 text-sm">{t("books.periods")}</span>
              {data.periods.map((p) => (
                <PeriodChip
                  key={p.month}
                  period={p}
                  owner={owner}
                  onClose={() => void period(p, "close")}
                  onReopen={() => void period(p, "reopen")}
                  onLock={() => setLockAsk(p)}
                />
              ))}
            </CardContent>
          </Card>

          {rows.length === 0 ? (
            <Card>
              <CardContent className="text-muted-foreground py-10 text-center text-sm">
                {t("books.nothing")}
              </CardContent>
            </Card>
          ) : (
            CLASSES.filter((c) => rowsOfClass(rows, c).length > 0).map((c) => (
              <ClassCard
                key={c}
                klass={c}
                rows={rowsOfClass(rows, c)}
                total={data.classes.find((k) => k.class === c)}
                currency={data.currency}
              />
            ))
          )}
        </>
      ) : tb.error ? (
        <Card>
          <CardContent className="py-10 text-center text-sm text-red-600 dark:text-red-400">
            {tb.error}
          </CardContent>
        </Card>
      ) : null}

      <ConfirmDialog
        open={lockAsk !== null}
        onOpenChange={(o) => {
          if (!o) setLockAsk(null);
        }}
        title={t("books.period.lock_title", {
          ym: lockAsk ? `${lockAsk.fiscal_year}-${mm(lockAsk.month)}` : "",
        })}
        desc={t("books.period.lock_desc")}
        confirmText={t("books.period.lock")}
        destructive
        handleConfirm={() => {
          const p = lockAsk;
          setLockAsk(null);
          if (p) void period(p, "lock");
        }}
      />
    </>
  );
}

function PeriodChip({
  period,
  owner,
  onClose,
  onReopen,
  onLock,
}: {
  period: Period;
  owner: boolean;
  onClose: () => void;
  onReopen: () => void;
  onLock: () => void;
}) {
  const t = useT();
  const cls =
    period.status === "locked"
      ? "border-red-300 text-red-700 dark:border-red-800 dark:text-red-300"
      : period.status === "closed"
        ? "border-amber-300 text-amber-700 dark:border-amber-800 dark:text-amber-300"
        : "";
  const chip = (
    <Badge
      variant="outline"
      className={cn("gap-1 font-mono", cls)}
      title={t(`books.period.${period.status}`)}
    >
      {period.status === "locked" ? (
        <Lock className="size-3" />
      ) : period.status === "closed" ? (
        <LockOpen className="size-3" />
      ) : null}
      {mm(period.month)}
    </Badge>
  );
  if (!owner || period.status === "locked") return chip;
  return (
    <DropdownMenu>
      <DropdownMenuTrigger className="cursor-pointer">{chip}</DropdownMenuTrigger>
      <DropdownMenuContent align="start">
        {period.status === "open" ? (
          <DropdownMenuItem onClick={onClose}>{t("books.period.close")}</DropdownMenuItem>
        ) : (
          <DropdownMenuItem onClick={onReopen}>{t("books.period.reopen")}</DropdownMenuItem>
        )}
        <DropdownMenuItem onClick={onLock} className="text-red-600 dark:text-red-400">
          {t("books.period.lock")}
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

function ClassCard({
  klass,
  rows,
  total,
  currency,
}: {
  klass: number;
  rows: TrialBalanceRow[];
  total: TrialBalanceClass | undefined;
  currency: string;
}) {
  const t = useT();
  return (
    <Collapsible defaultOpen>
      <Card>
        <CollapsibleTrigger className="hover:bg-muted/50 flex w-full items-center justify-between gap-3 px-4 py-3 text-left">
          <div className="flex items-center gap-2">
            <ChevronDown className="text-muted-foreground size-4" />
            <span className="font-medium">{t("books.class", { n: klass })}</span>
            <span className="text-muted-foreground text-sm">{t(`books.class.${klass}`)}</span>
          </div>
          {total ? (
            <span className="text-muted-foreground font-mono text-xs tabular-nums">
              {t("books.total_debit")} {money(total.total_debit_minor, currency)} · {t("books.total_credit")}{" "}
              {money(total.total_credit_minor, currency)} · {t("books.balance")}{" "}
              <span
                className={cn(
                  "font-medium",
                  BigInt(total.balance_minor) < 0n && "text-red-700 dark:text-red-300",
                )}
              >
                {money(total.balance_minor, currency)}
              </span>
            </span>
          ) : null}
        </CollapsibleTrigger>
        <CollapsibleContent>
          <CardContent className="overflow-x-auto p-0">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-24">{t("books.account")}</TableHead>
                  <TableHead></TableHead>
                  {MONEY_COLUMNS.map((c) => (
                    <TableHead key={c} className="text-right whitespace-nowrap">
                      {t(COLUMN_LABEL[c])}
                    </TableHead>
                  ))}
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.map((r) => (
                  <TableRow key={r.account_code}>
                    <TableCell className="font-mono text-xs">{r.account_code}</TableCell>
                    <TableCell className="text-muted-foreground max-w-[28rem] truncate" title={r.name}>
                      {r.name}
                    </TableCell>
                    {MONEY_COLUMNS.map((c) => (
                      <TableCell
                        key={c}
                        className={cn(
                          "text-right tabular-nums",
                          r[c] === "0" && "text-muted-foreground/60",
                          c === "balance_minor" && BigInt(r[c]) < 0n && "text-red-700 dark:text-red-300",
                        )}
                      >
                        {money(r[c], currency)}
                      </TableCell>
                    ))}
                  </TableRow>
                ))}
                {total ? (
                  <TableRow className="bg-muted/40 font-medium">
                    <TableCell></TableCell>
                    <TableCell>{t("books.class", { n: klass })}</TableCell>
                    {MONEY_COLUMNS.map((c) => (
                      <TableCell key={c} className="text-right tabular-nums">
                        {money(total[c], currency)}
                      </TableCell>
                    ))}
                  </TableRow>
                ) : null}
              </TableBody>
            </Table>
          </CardContent>
        </CollapsibleContent>
      </Card>
    </Collapsible>
  );
}
