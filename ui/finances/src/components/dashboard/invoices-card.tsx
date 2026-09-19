"use client";

// The invoices that touch the period: how many are open and how much they
// come to, which are overdue, which got paid, and the handful worth seeing.
import { FileText } from "lucide-react";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import type { Invoice } from "@/lib/api/schema";
import { dateOnly, money } from "@/lib/format";
import { useT } from "@/lib/i18n";
import type { Period } from "@/lib/summary";
import { cn } from "@/lib/utils";

import { Empty, Rows, WidgetCard } from "./cards";

const OPEN = new Set(["approved", "sent", "issued"]);

function inPeriod(iso: string, period: Period): boolean {
  return iso.length >= 7 && period.months.includes(iso.slice(0, 7));
}

export function InvoicesCard({
  invoices,
  period,
  currency,
  clientName,
  loading,
}: {
  invoices: Invoice[];
  period: Period;
  currency: string;
  clientName: (id: string) => string | undefined;
  loading: boolean;
}) {
  const t = useT();
  const today = new Date().toISOString().slice(0, 10);
  // Open ones are shown whatever their date; they are the money still to
  // come. Paid and cancelled ones belong to the period they were issued in.
  const rows = invoices
    .filter((i) => OPEN.has(i.status) || inPeriod(i.issued_at, period))
    .filter((i) => i.currency === currency || !i.currency);
  const open = rows.filter((i) => OPEN.has(i.status));
  const overdue = open.filter((i) => i.due_date && i.due_date.slice(0, 10) < today);
  const paid = rows.filter((i) => i.status === "paid");
  const openTotal = open.reduce((s, i) => s + BigInt(String(i.total_minor)), 0n);
  const shown = [...open]
    .sort((a, b) => (a.due_date || "9").localeCompare(b.due_date || "9"))
    .concat(paid.sort((a, b) => b.issued_at.localeCompare(a.issued_at)))
    .slice(0, 5);

  return (
    <WidgetCard
      icon={FileText}
      title={t("overview.invoices.title")}
      description={
        rows.length
          ? [
              `${t("overview.invoices.open_n", { n: open.length })} · ${money(openTotal.toString(), currency)}`,
              overdue.length ? t("overview.invoices.overdue_n", { n: overdue.length }) : null,
              paid.length ? t("overview.invoices.paid_n", { n: paid.length }) : null,
            ]
              .filter(Boolean)
              .join(" · ")
          : undefined
      }
      href="/invoices/"
      hrefLabel={t("overview.invoices.open")}
    >
      {loading ? (
        <Rows />
      ) : shown.length === 0 ? (
        <Empty>{t("overview.invoices.empty")}</Empty>
      ) : (
        <ul className="divide-y">
          {shown.map((i) => {
            const late = OPEN.has(i.status) && i.due_date && i.due_date.slice(0, 10) < today;
            return (
              <li key={i.id} className="flex items-center gap-3 py-2 text-sm">
                <Link href={`/invoices/view/?id=${i.id}`} className="min-w-0 flex-1 truncate hover:underline">
                  <span className="font-mono text-xs">{i.number || "—"}</span>
                  <span className="text-muted-foreground"> · {clientName(i.client_id) ?? "—"}</span>
                </Link>
                <Badge
                  variant={late ? "destructive" : i.status === "paid" ? "secondary" : "outline"}
                  className="shrink-0 text-[10px]"
                >
                  {late
                    ? t("overview.invoices.overdue")
                    : i.status === "paid"
                      ? i.status
                      : i.due_date
                        ? t("overview.invoices.due", { date: dateOnly(i.due_date.slice(0, 10)) })
                        : i.status}
                </Badge>
                <span
                  className={cn(
                    "w-28 shrink-0 text-right font-mono tabular-nums",
                    late && "text-red-600 dark:text-red-400",
                  )}
                >
                  {money(String(i.total_minor), i.currency || currency)}
                </span>
              </li>
            );
          })}
        </ul>
      )}
    </WidgetCard>
  );
}
