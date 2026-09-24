"use client";

import { RefreshCw } from "lucide-react";
import Link from "next/link";

import { useFinance } from "@/app/providers";
import { useAccountActions } from "@/components/banking/use-account-actions";
import { PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { Account } from "@/lib/api/schema";
import { isStale } from "@/lib/banking";
import { ago, money, when } from "@/lib/format";
import { useT } from "@/lib/i18n";

export default function AccountsPage() {
  const t = useT();
  const { partyIds, partyName, multi } = useFinance();
  const accounts = useFetch(() => api.accounts(partyIds), 30_000, [partyIds.join(",")]);
  const list = accounts.data?.accounts ?? [];
  const byParty = new Map<string, Account[]>();
  for (const a of list) byParty.set(a.party_id, [...(byParty.get(a.party_id) ?? []), a]);

  return (
    <>
      <PageTitle title={t("banking.accounts.title")} description={t("banking.accounts.description")}>
        <ScopeToggle className="md:hidden" />
      </PageTitle>
      {accounts.error ? <p className="text-destructive text-sm">{accounts.error}</p> : null}
      {accounts.loading && !accounts.data ? <Skeleton className="h-48 w-full" /> : null}
      {[...byParty.entries()].map(([party, rows]) => (
        <section key={party} className="space-y-3">
          {multi ? <h2 className="text-base font-semibold">{partyName(party)}</h2> : null}
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-3">
            {rows.map((a) => (
              <AccountCard key={a.id} a={a} onChanged={accounts.reload} />
            ))}
          </div>
        </section>
      ))}
    </>
  );
}

function AccountCard({ a, onChanged }: { a: Account; onChanged: () => void }) {
  const t = useT();
  const { busy, refresh, setSync, hhmm } = useAccountActions(onChanged);
  const closing = a.balances.find((b) => b.balance_type === "CLBD") ?? a.balances[0];
  const stale = isStale(a);
  const backoffUntil = a.sync_backoff_until ? new Date(a.sync_backoff_until) : null;
  const backingOff = backoffUntil !== null && backoffUntil.getTime() > Date.now();
  return (
    <Card>
      <CardHeader className="flex flex-row items-start justify-between gap-2">
        <div className="min-w-0">
          <CardTitle className="truncate">{a.name || a.currency}</CardTitle>
          <CardDescription className="font-mono text-xs">
            {a.iban || a.provider} · {a.currency}
          </CardDescription>
        </div>
        <StatusBadge status={backingOff ? "backing_off" : a.last_sync_status} />
      </CardHeader>
      <CardContent className="space-y-3">
        <div>
          <div className="text-2xl font-semibold tabular-nums">
            {closing ? money(closing.amount_minor, closing.currency) : "—"}
          </div>
          <div className="text-muted-foreground text-xs">
            {closing ? `${closing.balance_type} · ${when(closing.observed_at)}` : t("banking.no_balance_yet")}
          </div>
        </div>
        <dl className="text-muted-foreground grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
          <dt>{t("banking.last_fetched")}</dt>
          <dd className={stale ? "text-amber-600" : undefined}>{ago(a.last_synced_at)}</dd>
          <dt>{t("banking.booked_through")}</dt>
          <dd>{a.last_booked_through || "—"}</dd>
          <dt>{t("banking.fetches_today")}</dt>
          <dd>{t("banking.n_of_4", { n: a.sync_budget_used })}</dd>
          {backingOff && backoffUntil ? (
            <>
              <dt>{t("banking.bank_asked_to_wait")}</dt>
              <dd>{t("banking.until_time", { time: hhmm(backoffUntil) })}</dd>
            </>
          ) : null}
          {a.last_sync_error ? (
            <>
              <dt>{t("banking.last_error")}</dt>
              <dd className="text-destructive truncate" title={a.last_sync_error}>
                {a.last_sync_error}
              </dd>
            </>
          ) : null}
        </dl>
        <div className="flex flex-wrap items-center justify-between gap-2">
          <label className="flex items-center gap-2 text-xs">
            <Switch
              checked={a.sync_enabled}
              disabled={busy !== ""}
              onCheckedChange={(v) => void setSync(a, v)}
              aria-label={t("banking.fetch_on_schedule")}
            />
            {a.sync_enabled ? t("banking.scheduled_on") : t("banking.scheduled_off")}
          </label>
          <div className="flex items-center gap-1">
            {a.connection_id ? (
              <Button asChild variant="ghost" size="sm">
                <Link href={`/connections/#${a.connection_id}`}>{t("banking.open_bank")}</Link>
              </Button>
            ) : null}
            <Button variant="outline" size="sm" disabled={busy !== ""} onClick={() => void refresh(a)}>
              <RefreshCw className={busy === `refresh:${a.id}` ? "animate-spin" : undefined} />{" "}
              {busy === `refresh:${a.id}` ? t("banking.fetching") : t("banking.fetch_now")}
            </Button>
          </div>
        </div>
      </CardContent>
    </Card>
  );
}
