"use client";

import { RefreshCw } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Account } from "@/lib/api/schema";
import { ago, money, when } from "@/lib/format";
import { useLang, useT } from "@/lib/i18n";

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
  const { lang } = useLang();
  const [busy, setBusy] = useState(false);
  const [toggling, setToggling] = useState(false);
  const setSync = async (enabled: boolean) => {
    setToggling(true);
    try {
      await api.setAccountSync(a.id, enabled);
      toast.success(enabled ? t("banking.sync_on_toast") : t("banking.sync_off_toast"));
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setToggling(false);
    }
  };
  const closing = a.balances.find((b) => b.balance_type === "CLBD") ?? a.balances[0];
  const stale = a.last_synced_at ? Date.now() - new Date(a.last_synced_at).getTime() > 9 * 3600_000 : true;
  const backoffUntil = a.sync_backoff_until ? new Date(a.sync_backoff_until) : null;
  const backingOff = backoffUntil !== null && backoffUntil.getTime() > Date.now();
  const hhmm = (d: Date) =>
    d.toLocaleTimeString(lang === "hr" ? "hr-HR" : "en-GB", { hour: "2-digit", minute: "2-digit" });
  const refresh = async () => {
    setBusy(true);
    try {
      const r = await api.refresh(a.id);
      if (r.outcome === "ok")
        toast.success(
          t("banking.fetched_toast", { inserted: r.inserted, booked: r.booked, duplicates: r.duplicates }),
        );
      else if (r.outcome === "skipped" && r.skipped === "backing_off" && backoffUntil)
        toast.warning(t("banking.backing_off_toast", { time: hhmm(backoffUntil) }));
      else if (r.outcome === "skipped" && r.skipped === "budget_spent")
        toast.warning(t("banking.budget_spent_toast"));
      else if (r.outcome === "skipped")
        toast.warning(t("banking.not_fetched_toast", { reason: r.skipped.replace(/_/g, " ") }));
      else if (r.outcome === "rate_limited") toast.error(t("banking.rate_limited_toast"));
      else toast.error(t("banking.bank_answered_toast", { outcome: r.outcome.replace(/_/g, " ") }));
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };
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
              <dd className="text-amber-600">{t("banking.until_time", { time: hhmm(backoffUntil) })}</dd>
            </>
          ) : null}
          {a.last_sync_error ? (
            <>
              <dt>{t("banking.last_error")}</dt>
              <dd className="text-destructive col-span-2 truncate">{a.last_sync_error}</dd>
            </>
          ) : null}
        </dl>
        <Button
          size="sm"
          variant="outline"
          onClick={() => void refresh()}
          disabled={busy || !a.connection_id || backingOff || a.sync_budget_used >= 4}
          title={
            backingOff && backoffUntil
              ? t("banking.wait_until_hint", { time: hhmm(backoffUntil) })
              : a.sync_budget_used >= 4
                ? t("banking.budget_spent_hint")
                : t("banking.one_kept_hint")
          }
        >
          <RefreshCw className={busy ? "animate-spin" : undefined} />{" "}
          {busy ? t("banking.fetching") : t("banking.fetch_now")}
        </Button>
        <label className="text-muted-foreground flex items-center gap-2 text-xs">
          <Switch
            checked={a.sync_enabled}
            disabled={toggling}
            onCheckedChange={(v) => void setSync(v)}
            aria-label={t("banking.fetch_on_schedule")}
          />
          {a.sync_enabled ? t("banking.scheduled_on") : t("banking.scheduled_off")}
        </label>
      </CardContent>
    </Card>
  );
}
