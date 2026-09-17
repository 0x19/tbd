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
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Account } from "@/lib/api/schema";
import { ago, money, when } from "@/lib/format";

export default function AccountsPage() {
  const { partyIds, partyName, multi } = useFinance();
  const accounts = useFetch(() => api.accounts(partyIds), 30_000, [partyIds.join(",")]);
  const list = accounts.data?.accounts ?? [];
  const byParty = new Map<string, Account[]>();
  for (const a of list) byParty.set(a.party_id, [...(byParty.get(a.party_id) ?? []), a]);

  return (
    <>
      <PageTitle
        title="Accounts"
        description="Every linked account: what the bank says it holds, and when we last asked."
      >
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
  const [busy, setBusy] = useState(false);
  const closing = a.balances.find((b) => b.balance_type === "CLBD") ?? a.balances[0];
  const stale = a.last_synced_at ? Date.now() - new Date(a.last_synced_at).getTime() > 2 * 3600_000 : true;
  const refresh = async () => {
    setBusy(true);
    try {
      const r = await api.refresh(a.id);
      if (r.outcome === "ok")
        toast.success(`Fetched: ${r.inserted} new, ${r.booked} booked, ${r.duplicates} already known.`);
      else if (r.outcome === "skipped") toast.warning(`Not fetched: ${r.skipped.replace(/_/g, " ")}.`);
      else toast.error(`Bank answered: ${r.outcome.replace(/_/g, " ")}.`);
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
        <StatusBadge status={a.sync_backoff_until ? "backing_off" : a.last_sync_status} />
      </CardHeader>
      <CardContent className="space-y-3">
        <div>
          <div className="text-2xl font-semibold tabular-nums">
            {closing ? money(closing.amount_minor, closing.currency) : "—"}
          </div>
          <div className="text-muted-foreground text-xs">
            {closing ? `${closing.balance_type} · ${when(closing.observed_at)}` : "no balance yet"}
          </div>
        </div>
        <dl className="text-muted-foreground grid grid-cols-2 gap-x-3 gap-y-1 text-xs">
          <dt>Last fetched</dt>
          <dd className={stale ? "text-amber-600" : undefined}>{ago(a.last_synced_at)}</dd>
          <dt>Booked through</dt>
          <dd>{a.last_booked_through || "—"}</dd>
          <dt>Fetches today</dt>
          <dd>{a.sync_budget_used}</dd>
          {a.last_sync_error ? (
            <>
              <dt>Last error</dt>
              <dd className="text-destructive col-span-2 truncate">{a.last_sync_error}</dd>
            </>
          ) : null}
        </dl>
        <Button
          size="sm"
          variant="outline"
          onClick={() => void refresh()}
          disabled={busy || !a.connection_id}
        >
          <RefreshCw className={busy ? "animate-spin" : undefined} /> {busy ? "Fetching…" : "Fetch now"}
        </Button>
      </CardContent>
    </Card>
  );
}
