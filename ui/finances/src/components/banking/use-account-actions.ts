"use client";

// The two things a person does to an account -- fetch now, switch scheduled
// fetches -- with every outcome the bank can answer said in a toast. Shared
// by the banks page and the accounts page so the words never drift.
import { useState } from "react";
import { toast } from "sonner";

import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Account, RefreshAccountResponse } from "@/lib/api/schema";
import { useLang, useT } from "@/lib/i18n";

export function useAccountActions(onChanged: () => void) {
  const t = useT();
  const { lang } = useLang();
  const [busy, setBusy] = useState<string>("");
  const hhmm = (d: Date) =>
    d.toLocaleTimeString(lang === "hr" ? "hr-HR" : "en-GB", { hour: "2-digit", minute: "2-digit" });

  /** One sentence for a fetch's outcome; the same words a single fetch toasts. */
  const outcomeText = (
    a: Account,
    r: RefreshAccountResponse,
  ): { level: "ok" | "warn" | "bad"; text: string } => {
    const backoffUntil = a.sync_backoff_until ? new Date(a.sync_backoff_until) : null;
    if (r.outcome === "ok")
      return {
        level: "ok",
        text: t("banking.fetched_toast", {
          inserted: r.inserted,
          booked: r.booked,
          duplicates: r.duplicates,
        }),
      };
    if (r.outcome === "skipped" && r.skipped === "backing_off" && backoffUntil)
      return { level: "warn", text: t("banking.backing_off_toast", { time: hhmm(backoffUntil) }) };
    if (r.outcome === "skipped" && r.skipped === "budget_spent")
      return { level: "warn", text: t("banking.budget_spent_toast") };
    if (r.outcome === "skipped")
      return {
        level: "warn",
        text: t("banking.not_fetched_toast", { reason: r.skipped.replace(/_/g, " ") }),
      };
    if (r.outcome === "rate_limited") return { level: "bad", text: t("banking.rate_limited_toast") };
    return {
      level: "bad",
      text: t("banking.bank_answered_toast", { outcome: r.outcome.replace(/_/g, " ") }),
    };
  };

  const say = ({ level, text }: { level: "ok" | "warn" | "bad"; text: string }) => {
    if (level === "ok") toast.success(text);
    else if (level === "warn") toast.warning(text);
    else toast.error(text);
  };

  const refresh = async (a: Account) => {
    setBusy(`refresh:${a.id}`);
    try {
      say(outcomeText(a, await api.refresh(a.id)));
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };

  /** Every account in turn; one toast per account is noise, so one summary. */
  const refreshAll = async (accounts: Account[]) => {
    setBusy("refresh:all");
    let ok = 0;
    let inserted = 0;
    const problems: string[] = [];
    try {
      for (const a of accounts) {
        try {
          const r = await api.refresh(a.id);
          if (r.outcome === "ok") {
            ok++;
            inserted += r.inserted;
          } else problems.push(`${a.name || a.currency}: ${outcomeText(a, r).text}`);
        } catch (e) {
          problems.push(`${a.name || a.currency}: ${describe(e)}`);
        }
      }
      if (problems.length === 0) toast.success(t("banking.fetched_all_toast", { n: ok, inserted }));
      else
        toast.warning(t("banking.fetched_some_toast", { ok, n: accounts.length, inserted }), {
          description: problems.join("\n"),
        });
      onChanged();
    } finally {
      setBusy("");
    }
  };

  const setSync = async (a: Account, enabled: boolean) => {
    setBusy(`sync:${a.id}`);
    try {
      await api.setAccountSync(a.id, enabled);
      toast.success(enabled ? t("banking.sync_on_toast") : t("banking.sync_off_toast"));
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy("");
    }
  };

  return { busy, refresh, refreshAll, setSync, hhmm };
}
