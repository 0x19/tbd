"use client";

// Banks: every consent the company or the person gave a bank, with the
// accounts it reaches under it. The numbers at the top answer the first
// question -- is the money coming in -- and each card answers the next
// ones: how long the consent lasts, when each account was last fetched and
// when the next one is, what the bank last said, and the four things a
// person can do about it: link, renew, fetch, remove. Consents nobody
// finished are swept by the service after an hour.
import { AlertTriangle, Building2, CalendarClock, Plus, RefreshCw } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { ConsentCard } from "@/components/banking/consent-card";
import { LinkBankDialog, type Psu } from "@/components/banking/link-dialog";
import { KpiStrip, PageTitle } from "@/components/kit";
import { ScopeToggle } from "@/components/scope-toggle";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Account, Connection } from "@/lib/api/schema";
import { consentLive, daysUntil, fetchesLeft } from "@/lib/banking";
import { day } from "@/lib/format";
import { useT } from "@/lib/i18n";

export default function ConnectionsPage() {
  const t = useT();
  const { parties, partyIds, partyName, multi } = useFinance();
  const key = partyIds.join(",");
  const connections = useFetch(() => api.connections(partyIds), 30_000, [key]);
  const accounts = useFetch(() => api.accounts(partyIds), 30_000, [key]);
  const [linking, setLinking] = useState<{ party: string; psu: Psu } | null>(null);
  const [renewing, setRenewing] = useState<string | null>(null);

  const list = useMemo(() => connections.data?.connections ?? [], [connections.data]);
  const byConnection = useMemo(() => {
    const m = new Map<string, Account[]>();
    for (const a of accounts.data?.accounts ?? []) {
      m.set(a.connection_id, [...(m.get(a.connection_id) ?? []), a]);
    }
    return m;
  }, [accounts.data]);

  // Newest first, but a live consent before a dead one, and grouped by party
  // when the view is combined.
  const sorted = useMemo(
    () =>
      [...list].sort(
        (a, b) =>
          (multi ? partyName(a.party_id).localeCompare(partyName(b.party_id)) : 0) ||
          Number(consentLive(b)) - Number(consentLive(a)) ||
          b.created_at.localeCompare(a.created_at),
      ),
    [list, multi, partyName],
  );

  const live = list.filter(consentLive);
  const allAccounts = accounts.data?.accounts ?? [];
  const syncing = allAccounts.filter((a) => a.sync_enabled && live.some((c) => c.id === a.connection_id));
  const soonest = live
    .filter((c) => c.valid_until)
    .map((c) => ({ c, days: daysUntil(c.valid_until) }))
    .sort((a, b) => a.days - b.days)[0];
  const left = syncing.reduce((s, a) => s + fetchesLeft(a), 0);
  const failing = syncing.filter((a) => a.last_sync_status && a.last_sync_status !== "ok").length;

  const renew = async (c: Connection) => {
    setRenewing(c.id);
    try {
      const s = await api.startConnection(c.party_id, c.psu_type as Psu);
      window.location.href = s.url;
    } catch (e) {
      toast.error(describe(e));
      setRenewing(null);
    }
  };
  const remove = async (c: Connection) => {
    try {
      await api.deleteConnection(c.id);
      toast.success(consentLive(c) ? t("banking.removed_live_toast") : t("banking.removed_toast"));
      connections.reload();
      accounts.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };
  const reload = () => {
    connections.reload();
    accounts.reload();
  };

  const loading = (connections.loading && !connections.data) || (accounts.loading && !accounts.data);

  return (
    <>
      <PageTitle title={t("banking.connections.title")} description={t("banking.connections.description")}>
        <ScopeToggle className="md:hidden" />
        <Button size="sm" onClick={() => setLinking({ party: partyIds[0] ?? "", psu: "business" })}>
          <Plus /> {t("banking.link_bank")}
        </Button>
      </PageTitle>

      {connections.error ? <p className="text-destructive text-sm">{connections.error}</p> : null}
      {accounts.error ? <p className="text-destructive text-sm">{accounts.error}</p> : null}

      {loading ? (
        <Skeleton className="h-28 w-full" />
      ) : (
        <KpiStrip
          items={[
            {
              icon: Building2,
              label: t("banking.kpi.banks"),
              value: live.length,
              hint: t("banking.kpi.banks_hint", { n: list.length - live.length }),
            },
            {
              icon: RefreshCw,
              label: t("banking.kpi.syncing"),
              value: `${syncing.length} / ${allAccounts.length}`,
              hint: failing ? t("banking.kpi.failing", { n: failing }) : t("banking.kpi.syncing_hint"),
            },
            {
              icon: CalendarClock,
              label: t("banking.kpi.consent"),
              value: soonest ? t("banking.kpi.days", { n: Math.max(0, soonest.days) }) : "—",
              hint: soonest
                ? t(soonest.days < 30 ? "banking.kpi.consent_renew" : "banking.kpi.consent_hint", {
                    bank: soonest.c.aspsp_name,
                    until: day(soonest.c.valid_until),
                  })
                : t("banking.kpi.consent_none"),
            },
            {
              icon: AlertTriangle,
              label: t("banking.kpi.fetches"),
              value: left,
              hint: t("banking.kpi.fetches_hint"),
            },
          ]}
        />
      )}

      {loading ? <Skeleton className="h-48 w-full" /> : null}
      {!loading && sorted.length === 0 ? (
        <Card>
          <CardContent className="space-y-3 py-10 text-center">
            <p className="text-sm">{t("banking.empty")}</p>
            <Button size="sm" onClick={() => setLinking({ party: partyIds[0] ?? "", psu: "business" })}>
              <Plus /> {t("banking.link_bank")}
            </Button>
          </CardContent>
        </Card>
      ) : null}
      <div className="space-y-4">
        {sorted.map((c) => (
          <ConsentCard
            key={c.id}
            c={c}
            accounts={byConnection.get(c.id) ?? []}
            partyName={partyName(c.party_id)}
            multi={multi}
            onRenew={() => void renew(c)}
            onRemove={() => remove(c)}
            onChanged={reload}
            renewing={renewing === c.id}
          />
        ))}
      </div>

      <LinkBankDialog
        open={linking !== null}
        onOpenChange={(o) => !o && setLinking(null)}
        parties={parties}
        defaultParty={linking?.party ?? ""}
        defaultPsu={linking?.psu ?? "business"}
      />
    </>
  );
}
