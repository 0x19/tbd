"use client";

// The last booked rows across the accounts in view: date, who, category,
// amount. Enough to notice something odd without opening the ledger.
import { ListOrdered } from "lucide-react";
import Link from "next/link";

import { Badge } from "@/components/ui/badge";
import type { Transaction } from "@/lib/api/schema";
import { day, money } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

import { Empty, Rows, WidgetCard } from "./cards";

export function ActivityCard({ transactions, loading }: { transactions: Transaction[]; loading: boolean }) {
  const t = useT();
  return (
    <WidgetCard
      icon={ListOrdered}
      title={t("overview.activity.title")}
      description={t("overview.activity.desc")}
      href="/transactions/"
      hrefLabel={t("overview.activity.open")}
    >
      {loading ? (
        <Rows n={6} />
      ) : transactions.length === 0 ? (
        <Empty>{t("overview.activity.empty")}</Empty>
      ) : (
        <ul className="divide-y">
          {transactions.map((x) => {
            const negative = String(x.amount_minor).startsWith("-");
            return (
              <li
                key={x.id}
                className="grid grid-cols-[3.5rem_minmax(0,1fr)_auto_7rem] items-center gap-3 py-2 text-sm"
              >
                <span className="text-muted-foreground text-xs tabular-nums">
                  {day(x.booking_date.slice(0, 10))}
                </span>
                <Link
                  href={`/transactions/?search=${encodeURIComponent(x.counterparty_name)}`}
                  className="min-w-0 truncate hover:underline"
                >
                  {x.counterparty_name || x.remittance || "—"}
                  {x.account_name ? <span className="text-muted-foreground"> · {x.account_name}</span> : null}
                </Link>
                <Badge
                  variant={x.internal ? "secondary" : x.category_id ? "outline" : "destructive"}
                  className="hidden max-w-36 truncate text-[10px] sm:inline-flex"
                >
                  {x.internal ? t("overview.donut.other") : x.category || t("overview.uncategorised")}
                </Badge>
                <span
                  className={cn(
                    "text-right font-mono tabular-nums",
                    !negative && "text-emerald-700 dark:text-emerald-400",
                  )}
                >
                  {money(String(x.amount_minor), x.currency, { sign: true })}
                </span>
              </li>
            );
          })}
        </ul>
      )}
    </WidgetCard>
  );
}
