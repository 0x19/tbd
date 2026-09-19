"use client";

// The year as a table: one row per month, in / spent / net, and how spending
// moved against the month before. The row is the month picker.
import { CalendarRange } from "lucide-react";

import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { money, monthLabel } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { type MonthTotals, percentChange } from "@/lib/summary";
import { cn } from "@/lib/utils";

import { Delta, Rows, WidgetCard } from "./cards";

export function YearTable({
  months,
  currency,
  selected,
  onPick,
  loading,
}: {
  /** The year's months that have data, oldest first. */
  months: MonthTotals[];
  currency: string;
  selected?: string;
  onPick: (ym: string) => void;
  loading: boolean;
}) {
  const t = useT();
  const total = months.reduce((s, m) => ({ in: s.in + m.in, spent: s.spent + m.spent, out: s.out + m.out }), {
    in: 0n,
    spent: 0n,
    out: 0n,
  });
  return (
    <WidgetCard icon={CalendarRange} title={t("overview.year.title")} description={t("overview.year.desc")}>
      {loading ? (
        <Rows n={6} />
      ) : (
        <div className="overflow-x-auto">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>{t("overview.year.month")}</TableHead>
                <TableHead className="text-right">{t("overview.year.in")}</TableHead>
                <TableHead className="text-right">{t("overview.year.spent")}</TableHead>
                <TableHead className="text-right">{t("overview.year.net")}</TableHead>
                <TableHead className="text-right">{t("overview.year.vs_prev")}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {months.map((m, i) => {
                const prev = months[i - 1];
                const change = prev ? percentChange(m.spent, prev.spent) : undefined;
                return (
                  <TableRow
                    key={m.month}
                    onClick={() => onPick(m.month)}
                    className={cn("cursor-pointer", m.month === selected && "bg-muted/50")}
                  >
                    <TableCell className={cn("font-medium", m.month === selected && "text-foreground")}>
                      {monthLabel(m.month)}
                    </TableCell>
                    <TableCell className="text-right font-mono tabular-nums">
                      {money(m.in.toString(), currency)}
                    </TableCell>
                    <TableCell className="text-right font-mono tabular-nums">
                      {money(m.spent.toString(), currency)}
                    </TableCell>
                    <TableCell className="text-right font-mono tabular-nums">
                      {money((m.in - m.out).toString(), currency, { sign: true })}
                    </TableCell>
                    <TableCell className="text-right">
                      <Delta value={change} label="" goodWhen="down" className="justify-end" />
                    </TableCell>
                  </TableRow>
                );
              })}
              <TableRow className="font-medium">
                <TableCell>{t("overview.year.total")}</TableCell>
                <TableCell className="text-right font-mono tabular-nums">
                  {money(total.in.toString(), currency)}
                </TableCell>
                <TableCell className="text-right font-mono tabular-nums">
                  {money(total.spent.toString(), currency)}
                </TableCell>
                <TableCell className="text-right font-mono tabular-nums">
                  {money((total.in - total.out).toString(), currency, { sign: true })}
                </TableCell>
                <TableCell />
              </TableRow>
            </TableBody>
          </Table>
        </div>
      )}
    </WidgetCard>
  );
}
