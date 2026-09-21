"use client";

// Receivables by age, from the service's report: one bar per currency cut
// into the five buckets, and the clients who owe, most overdue first. A
// client row narrows the invoices list to them.

import { Hourglass } from "lucide-react";

import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { AgingBucket } from "@/lib/api/schema";
import { day, money } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

const TONE: Record<string, string> = {
  current: "bg-muted-foreground/30",
  d1_30: "bg-amber-300",
  d31_60: "bg-amber-500",
  d61_90: "bg-orange-600",
  d90_plus: "bg-destructive",
};

export function AgingCard({
  partyIds,
  version,
  onClient,
}: {
  partyIds: string[];
  /** Bumped by the page when invoices change, so the report follows. */
  version: number;
  onClient: (clientId: string) => void;
}) {
  const t = useT();
  const key = partyIds.join(",");
  const report = useFetch(
    () => (partyIds.length ? api.agingReport(partyIds) : Promise.resolve(null)),
    30_000,
    [key, version],
  );
  if (report.loading && !report.data) return <Skeleton className="h-40 w-full" />;
  if (!report.data || report.data.buckets.length === 0) return null;
  const byCurrency = new Map<string, AgingBucket[]>();
  for (const b of report.data.buckets) {
    byCurrency.set(b.currency, [...(byCurrency.get(b.currency) ?? []), b]);
  }
  return (
    <Card>
      <CardHeader>
        <CardTitle className="flex items-center gap-2">
          <Hourglass className="size-4" /> {t("invoices.aging.title")}
        </CardTitle>
        <CardDescription>{t("invoices.aging.hint", { date: day(report.data.as_of) })}</CardDescription>
      </CardHeader>
      <CardContent className="space-y-5">
        {[...byCurrency.entries()].map(([currency, buckets]) => {
          const total = buckets.reduce((s, b) => s + BigInt(b.amount_minor), 0n);
          return (
            <div key={currency} className="space-y-2">
              <div className="bg-muted flex h-3 w-full overflow-hidden rounded-full">
                {buckets.map((b) =>
                  BigInt(b.amount_minor) > 0n ? (
                    <div
                      key={b.bucket}
                      className={cn("h-full", TONE[b.bucket])}
                      style={{ width: `${Number((BigInt(b.amount_minor) * 1000n) / (total || 1n)) / 10}%` }}
                      title={`${t(`invoices.aging.${b.bucket}`)} · ${money(b.amount_minor, currency)}`}
                    />
                  ) : null,
                )}
              </div>
              <div className="grid grid-cols-2 gap-x-4 gap-y-1 text-xs sm:grid-cols-5">
                {buckets.map((b) => (
                  <div key={b.bucket} className="flex items-center gap-2">
                    <span className={cn("size-2.5 shrink-0 rounded-full", TONE[b.bucket])} />
                    <span className="text-muted-foreground truncate">{t(`invoices.aging.${b.bucket}`)}</span>
                    <span className="ml-auto font-mono tabular-nums">
                      {money(b.amount_minor, currency, { compact: true })}
                      {b.count ? <span className="text-muted-foreground"> ·{b.count}</span> : null}
                    </span>
                  </div>
                ))}
              </div>
            </div>
          );
        })}
        {report.data.clients.length ? (
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("invoices.aging.col.client")}</TableHead>
                <TableHead className="text-right">{t("invoices.aging.col.open")}</TableHead>
                <TableHead className="text-right">{t("invoices.aging.col.outstanding")}</TableHead>
                <TableHead className="text-right">{t("invoices.aging.col.overdue")}</TableHead>
                <TableHead className="text-right">{t("invoices.aging.col.oldest")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {report.data.clients.map((c) => (
                <TableRow
                  key={`${c.client_id}-${c.currency}`}
                  className="cursor-pointer"
                  onClick={() => onClient(c.client_id)}
                >
                  <TableCell className="font-medium">{c.client_name || "—"}</TableCell>
                  <TableCell className="text-right tabular-nums">{c.count}</TableCell>
                  <TableCell className="text-right font-mono tabular-nums">
                    {money(c.outstanding_minor, c.currency)}
                  </TableCell>
                  <TableCell
                    className={cn(
                      "text-right font-mono tabular-nums",
                      BigInt(c.overdue_minor) > 0n ? "text-destructive font-medium" : "text-muted-foreground",
                    )}
                  >
                    {money(c.overdue_minor, c.currency)}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">
                    {c.oldest_days > 0 ? t("invoices.aging.days", { n: c.oldest_days }) : "—"}
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : null}
      </CardContent>
    </Card>
  );
}
