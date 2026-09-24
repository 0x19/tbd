"use client";

// Every account in view: what the bank says it holds, when that was fetched,
// and whether the sync is healthy. Sync health lives here rather than in a
// card of its own because the balance is only worth what its timestamp is.
import { Wallet } from "lucide-react";

import type { Account } from "@/lib/api/schema";
import { ago, money } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

import { Empty, Rows, WidgetCard } from "./cards";

/** The balance to show: the booked closing one when the bank gives it. */
function shown(a: Account): { amount: string; currency: string } | null {
  const order = ["closingBooked", "expected", "interimBooked", "interimAvailable", "openingBooked"];
  const sorted = [...a.balances].sort((x, y) => {
    const ix = order.indexOf(x.balance_type);
    const iy = order.indexOf(y.balance_type);
    return (ix < 0 ? 99 : ix) - (iy < 0 ? 99 : iy);
  });
  const b = sorted[0];
  return b ? { amount: String(b.amount_minor), currency: b.currency || a.currency } : null;
}

type Health = "ok" | "failing" | "paused";
function health(a: Account): Health {
  if (!a.sync_enabled) return "paused";
  if (a.last_sync_status && a.last_sync_status !== "ok") return "failing";
  if (a.sync_backoff_until && new Date(a.sync_backoff_until).getTime() > Date.now()) return "failing";
  return "ok";
}

export function AccountsCard({ accounts, loading }: { accounts: Account[]; loading: boolean }) {
  const t = useT();
  // One total per currency, from the balances actually shown.
  const totals = new Map<string, bigint>();
  for (const a of accounts) {
    const b = shown(a);
    if (!b) continue;
    totals.set(b.currency, (totals.get(b.currency) ?? 0n) + BigInt(b.amount));
  }
  const newest = accounts
    .map((a) => a.last_synced_at)
    .filter(Boolean)
    .sort()
    .at(-1);

  return (
    <WidgetCard
      icon={Wallet}
      title={t("overview.accounts.title")}
      description={newest ? t("overview.accounts.last_sync", { ago: ago(newest) }) : undefined}
      href="/accounts/"
      hrefLabel={t("overview.accounts.open")}
    >
      {loading ? (
        <Rows />
      ) : accounts.length === 0 ? (
        <Empty>{t("overview.accounts.empty")}</Empty>
      ) : (
        <>
          <ul className="divide-y">
            {accounts.map((a) => {
              const b = shown(a);
              const h = health(a);
              return (
                <li key={a.id} className="flex items-center gap-3 py-2 text-sm">
                  <span
                    className={cn(
                      "size-2 shrink-0 rounded-full",
                      h === "ok"
                        ? "bg-emerald-500"
                        : h === "failing"
                          ? "bg-red-500"
                          : "bg-muted-foreground/40",
                    )}
                    aria-hidden
                  />
                  <span className="min-w-0 flex-1 truncate">
                    {a.name || a.iban}
                    {h !== "ok" ? (
                      <span
                        className={cn(
                          "ml-2 text-xs",
                          h === "failing" ? "text-red-600 dark:text-red-400" : "text-muted-foreground",
                        )}
                      >
                        {t(h === "failing" ? "overview.accounts.failing" : "overview.accounts.paused")}
                      </span>
                    ) : null}
                  </span>
                  <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
                    {ago(a.last_synced_at)}
                  </span>
                  <span className="w-32 shrink-0 text-right font-mono text-sm tabular-nums">
                    {b ? money(b.amount, b.currency) : "—"}
                  </span>
                </li>
              );
            })}
          </ul>
          <div className="text-muted-foreground flex flex-wrap items-baseline justify-between gap-2 border-t pt-3 text-xs">
            <span>{t("overview.accounts.total")}</span>
            <span className="text-foreground font-mono text-sm font-medium tabular-nums">
              {[...totals.entries()].map(([c, v]) => money(v.toString(), c)).join(" · ") || "—"}
            </span>
          </div>
        </>
      )}
    </WidgetCard>
  );
}
