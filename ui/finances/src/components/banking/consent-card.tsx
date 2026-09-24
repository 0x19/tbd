"use client";

// One consent at a bank, and the accounts it reaches: the consent's life in
// a sentence, and per account what an accountant asks first -- is the money
// coming in, when was it last fetched, when is the next one, what went wrong.
import { AlertTriangle, Building2, KeyRound, RefreshCw, Trash2 } from "lucide-react";
import { useState } from "react";

import { useAccountActions } from "@/components/banking/use-account-actions";
import { StatusBadge } from "@/components/status-badge";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from "@/components/ui/alert-dialog";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Switch } from "@/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import type { Account, Connection } from "@/lib/api/schema";
import {
  consentLive,
  type ConsentState,
  consentState,
  daysUntil,
  FETCHES_PER_DAY,
  isStale,
  nextFetch,
} from "@/lib/banking";
import { ago, dateOf, day, money, when } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

const TONE: Record<ConsentState, string> = {
  active: "text-emerald-700 dark:text-emerald-300",
  ending: "text-amber-700 dark:text-amber-300",
  ended: "text-destructive",
  pending: "text-muted-foreground",
  removed: "text-muted-foreground",
  replaced: "text-muted-foreground",
  failed: "text-destructive",
};

export function ConsentCard({
  c,
  accounts,
  partyName,
  multi,
  onRenew,
  onRemove,
  onChanged,
  renewing,
}: {
  c: Connection;
  accounts: Account[];
  partyName: string;
  multi: boolean;
  onRenew: () => void;
  onRemove: () => Promise<void>;
  onChanged: () => void;
  renewing: boolean;
}) {
  const t = useT();
  const state = consentState(c);
  const live = consentLive(c);
  const { busy, refresh, refreshAll, setSync, hhmm } = useAccountActions(onChanged);
  const [confirming, setConfirming] = useState(false);
  const [removing, setRemoving] = useState(false);
  const days = c.valid_until ? daysUntil(c.valid_until) : null;
  const syncing = accounts.filter((a) => a.sync_enabled);
  const gone = state === "removed" || state === "replaced" || state === "failed";

  const line = (() => {
    switch (state) {
      case "active":
        return t("banking.consent.active", { until: dateOf(c.valid_until), days: days ?? 0 });
      case "ending":
        return t("banking.consent.ending", { days: days ?? 0, until: dateOf(c.valid_until) });
      case "ended":
        return t("banking.consent.ended", { on: dateOf(c.valid_until) });
      case "pending":
        return t("banking.consent.pending", { ago: ago(c.created_at) });
      case "removed":
        return t("banking.consent.removed");
      case "replaced":
        return t("banking.consent.replaced");
      case "failed":
        return t("banking.consent.failed", { reason: c.failure });
    }
  })();

  return (
    <Card id={c.id} className={cn(gone && "opacity-70")}>
      <CardContent className="space-y-4 pt-4">
        <div className="flex flex-wrap items-start justify-between gap-3">
          <div className="min-w-0 space-y-1">
            <div className="flex flex-wrap items-center gap-2">
              <Building2 className="text-muted-foreground size-4" />
              <span className="font-medium">{c.aspsp_name}</span>
              <Badge variant="outline" className="text-[10px]">
                {t(`banking.login.${c.psu_type}`)}
              </Badge>
              {multi ? (
                <Badge variant="secondary" className="text-[10px]">
                  {partyName}
                </Badge>
              ) : null}
              <StatusBadge status={c.status} className="text-[10px]" />
            </div>
            <p className={cn("text-sm", TONE[state])}>
              {state === "ending" || state === "ended" ? (
                <AlertTriangle className="mr-1 inline size-3.5" />
              ) : null}
              {line}
              {c.authorized_at ? (
                <span className="text-muted-foreground">
                  {" "}
                  · {t("banking.consent.since", { when: dateOf(c.authorized_at) })}
                </span>
              ) : null}
            </p>
          </div>
          <div className="flex flex-wrap items-center gap-1">
            {live && syncing.length ? (
              <Button
                variant="outline"
                size="sm"
                disabled={busy !== ""}
                onClick={() => void refreshAll(syncing)}
              >
                <RefreshCw className={busy === "refresh:all" ? "animate-spin" : undefined} />{" "}
                {t("banking.fetch_all", { n: syncing.length })}
              </Button>
            ) : null}
            {state !== "pending" && state !== "replaced" ? (
              <Button
                variant={state === "ending" || state === "ended" ? "default" : "outline"}
                size="sm"
                disabled={renewing}
                onClick={onRenew}
                title={t("banking.renew_title")}
              >
                <KeyRound /> {renewing ? t("banking.opening_bank") : t("banking.renew")}
              </Button>
            ) : null}
            {state !== "removed" && state !== "replaced" ? (
              <Button
                variant="ghost"
                size="sm"
                onClick={() => setConfirming(true)}
                title={t("common.remove")}
              >
                <Trash2 />
              </Button>
            ) : null}
          </div>
        </div>

        {accounts.length ? (
          <div className="-mx-2 overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>{t("banking.col.account")}</TableHead>
                  <TableHead className="text-right">{t("banking.col.balance")}</TableHead>
                  <TableHead className="hidden md:table-cell">{t("banking.last_fetched")}</TableHead>
                  <TableHead className="hidden lg:table-cell">{t("banking.col.next_fetch")}</TableHead>
                  <TableHead className="hidden sm:table-cell">{t("banking.col.today")}</TableHead>
                  <TableHead>{t("banking.col.outcome")}</TableHead>
                  <TableHead className="text-right">{t("banking.col.scheduled")}</TableHead>
                  <TableHead />
                </TableRow>
              </TableHeader>
              <TableBody>
                {accounts.map((a) => {
                  const closing = a.balances.find((b) => b.balance_type === "CLBD") ?? a.balances[0];
                  const next = nextFetch(a, live);
                  const stale = live && a.sync_enabled && isStale(a);
                  const nextText = (() => {
                    switch (next.kind) {
                      case "consent_ended":
                        return t("banking.next.consent_ended");
                      case "off":
                        return t("banking.next.off");
                      case "backing_off":
                        return t("banking.next.backing_off", { time: hhmm(next.until) });
                      case "budget_spent":
                        return t("banking.next.budget_spent");
                      case "due":
                        return t("banking.next.due");
                      case "at":
                        return t("banking.next.at", { when: when(next.at.toISOString()) });
                    }
                  })();
                  return (
                    <TableRow key={a.id} className={cn(!a.sync_enabled && "text-muted-foreground")}>
                      <TableCell>
                        <div className="font-medium">{a.name || a.currency}</div>
                        <div className="text-muted-foreground font-mono text-[11px]">
                          {a.iban || a.provider} · {a.currency}
                        </div>
                      </TableCell>
                      <TableCell className="text-right font-mono tabular-nums">
                        {closing ? (
                          <>
                            <div>{money(closing.amount_minor, closing.currency)}</div>
                            <div className="text-muted-foreground text-[11px]">
                              {dateOf(closing.observed_at)}
                            </div>
                          </>
                        ) : (
                          <span className="text-muted-foreground text-xs">{t("banking.no_balance_yet")}</span>
                        )}
                      </TableCell>
                      <TableCell
                        className={cn(
                          "hidden text-xs md:table-cell",
                          stale && "text-amber-700 dark:text-amber-300",
                        )}
                      >
                        {a.last_synced_at ? ago(a.last_synced_at) : t("common.never")}
                        {a.last_booked_through ? (
                          <div className="text-muted-foreground text-[11px]">
                            {t("banking.booked_through")} {day(a.last_booked_through)}
                          </div>
                        ) : null}
                      </TableCell>
                      <TableCell className="hidden text-xs lg:table-cell">{nextText}</TableCell>
                      <TableCell className="hidden font-mono text-xs tabular-nums sm:table-cell">
                        {a.sync_budget_used} / {FETCHES_PER_DAY}
                      </TableCell>
                      <TableCell>
                        {a.last_sync_status ? (
                          <span title={a.last_sync_error || undefined}>
                            <StatusBadge status={a.last_sync_status} className="text-[10px]" />
                          </span>
                        ) : (
                          <span className="text-muted-foreground text-xs">—</span>
                        )}
                        {a.last_sync_error && a.last_sync_status !== "ok" ? (
                          <div
                            className="text-muted-foreground mt-0.5 max-w-56 truncate text-[11px]"
                            title={a.last_sync_error}
                          >
                            {a.last_sync_error}
                          </div>
                        ) : null}
                      </TableCell>
                      <TableCell className="text-right">
                        <Switch
                          checked={a.sync_enabled}
                          disabled={!live || busy !== ""}
                          onCheckedChange={(v) => void setSync(a, v)}
                          aria-label={t("banking.fetch_on_schedule")}
                        />
                      </TableCell>
                      <TableCell className="text-right">
                        <Button
                          variant="ghost"
                          size="sm"
                          disabled={!live || busy !== ""}
                          onClick={() => void refresh(a)}
                          title={t("banking.fetch_now")}
                        >
                          <RefreshCw className={busy === `refresh:${a.id}` ? "animate-spin" : undefined} />
                        </Button>
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        ) : state === "pending" ? null : (
          <p className="text-muted-foreground text-sm">{t("banking.no_accounts_under")}</p>
        )}

        <AlertDialog open={confirming} onOpenChange={setConfirming}>
          <AlertDialogContent>
            <AlertDialogHeader>
              <AlertDialogTitle>{t("banking.remove.title", { bank: c.aspsp_name })}</AlertDialogTitle>
              <AlertDialogDescription>
                {live ? t("banking.remove.live_hint", { n: accounts.length }) : t("banking.remove.dead_hint")}
              </AlertDialogDescription>
            </AlertDialogHeader>
            <AlertDialogFooter>
              <AlertDialogCancel disabled={removing}>{t("common.cancel")}</AlertDialogCancel>
              <AlertDialogAction
                disabled={removing}
                onClick={(e) => {
                  e.preventDefault();
                  setRemoving(true);
                  void onRemove().finally(() => {
                    setRemoving(false);
                    setConfirming(false);
                  });
                }}
              >
                {removing ? t("common.saving") : t("common.remove")}
              </AlertDialogAction>
            </AlertDialogFooter>
          </AlertDialogContent>
        </AlertDialog>
      </CardContent>
    </Card>
  );
}
