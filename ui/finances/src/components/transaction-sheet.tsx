"use client";

import { ChevronDown, Filter, Wand2 } from "lucide-react";
import { useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { StatusBadge } from "@/components/status-badge";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Category, Rule, Transaction } from "@/lib/api/schema";
import { day, money, when } from "@/lib/format";
import { useT } from "@/lib/i18n";

/** Erste prefixes a card line with the masked PAN; nobody reads it twice. */
export function cleanRemittance(r: string): string {
  return r.replace(/^\d{6}X{6}\d{4},\s*/, "").replace(/^HR99\s*\|\s*/, "");
}

/** One transaction, everything the bank sent about it, and the two things a
 *  person does with it: set its category, or turn it into a rule. */
export function TransactionSheet({
  id,
  cats,
  rules,
  onClose,
  onChanged,
  onMakeRule,
  onFilter,
}: {
  id: string | null;
  cats: Category[];
  rules: Rule[];
  onClose: () => void;
  onChanged: () => void;
  onMakeRule: (t: Transaction) => void;
  onFilter: (counterparty: string) => void;
}) {
  const t = useT();
  const { multi, partyName } = useFinance();
  const one = useFetch(() => (id ? api.transaction(id) : Promise.resolve(null)), 0, [id]);
  const [busy, setBusy] = useState(false);
  const tx = one.data?.transaction ?? null;

  const declare = async (v: string) => {
    if (!tx || v === tx.category_id) return;
    setBusy(true);
    try {
      await api.declare(tx.id, v);
      toast.success(t("transactions.category_set"), {
        action: {
          label: t("transactions.make_it_a_rule"),
          onClick: () => onMakeRule({ ...tx, category_id: v }),
        },
      });
      one.reload();
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const rule = tx?.category_rule_id ? rules.find((r) => r.id === tx.category_rule_id) : undefined;
  const mine = tx
    ? cats.filter((c) => c.party_id === tx.party_id && (!c.archived || c.id === tx.category_id))
    : [];
  const negative = tx?.amount_minor.startsWith("-") ?? false;
  let record: string | null = null;
  if (tx?.raw) {
    try {
      record = JSON.stringify(JSON.parse(tx.raw), null, 2);
    } catch {
      record = tx.raw;
    }
  }

  return (
    <Sheet open={id !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-0 overflow-y-auto p-0 sm:max-w-lg">
        {one.error ? (
          <p className="text-destructive p-6 text-sm">{one.error}</p>
        ) : !tx ? (
          <div className="space-y-3 p-6">
            <Skeleton className="h-8 w-40" />
            <Skeleton className="h-4 w-64" />
            <Skeleton className="h-48 w-full" />
          </div>
        ) : (
          <>
            <SheetHeader className="border-b p-6">
              <SheetDescription className="flex items-center gap-2 text-xs">
                {day(tx.booking_date)}
                <StatusBadge status={tx.status.toLowerCase()} className="text-[10px]" />
                {tx.internal ? (
                  <Badge variant="outline" className="text-[10px]">
                    {t("transactions.own_transfer")}
                  </Badge>
                ) : null}
              </SheetDescription>
              <SheetTitle
                className={
                  negative
                    ? "font-mono text-3xl tabular-nums"
                    : "font-mono text-3xl text-emerald-700 tabular-nums dark:text-emerald-300"
                }
              >
                {money(tx.amount_minor, tx.currency, { sign: true })}
              </SheetTitle>
              <p className="text-base font-medium">
                {tx.counterparty_name || (
                  <span className="text-muted-foreground italic">{t("transactions.no_counterparty")}</span>
                )}
              </p>
            </SheetHeader>

            <div className="space-y-6 p-6">
              <section className="space-y-2">
                <h3 className="text-muted-foreground text-xs font-medium tracking-wide uppercase">
                  {t("transactions.category")}
                </h3>
                <div className="flex flex-wrap items-center gap-2">
                  <Select
                    value={tx.category_id || "none"}
                    disabled={busy}
                    onValueChange={(v) => void declare(v)}
                  >
                    <SelectTrigger className="w-56">
                      <SelectValue placeholder={t("transactions.uncategorised")} />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="none" disabled>
                        {t("transactions.uncategorised")}
                      </SelectItem>
                      {mine.map((c) => (
                        <SelectItem key={c.id} value={c.id}>
                          {c.name}
                          {c.archived ? ` (${t("transactions.archived")})` : ""}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {tx.category_source ? (
                    <StatusBadge status={tx.category_source} className="text-[10px]" />
                  ) : null}
                </div>
                <p className="text-muted-foreground text-xs">
                  {tx.category_source === "declared"
                    ? t("transactions.set_by_hand", {
                        at: tx.categorised_at ? ` ${when(tx.categorised_at)}` : "",
                      })
                    : tx.category_source === "inferred"
                      ? t("transactions.claimed_by_rule", {
                          rule: rule
                            ? `“${rule.name}” (${t("transactions.priority", { n: rule.priority })})`
                            : t("transactions.rule_gone"),
                          at: tx.categorised_at ? `, ${when(tx.categorised_at)}` : "",
                        })
                      : t("transactions.no_rule")}
                </p>
                <div className="flex flex-wrap gap-2 pt-1">
                  <Button size="sm" variant="outline" onClick={() => onMakeRule(tx)}>
                    <Wand2 /> {t("transactions.make_rule_from_this")}
                  </Button>
                  {tx.counterparty_name ? (
                    <Button size="sm" variant="ghost" onClick={() => onFilter(tx.counterparty_name)}>
                      <Filter /> {t("transactions.all_from_counterparty")}
                    </Button>
                  ) : null}
                </div>
              </section>

              <section className="space-y-2">
                <h3 className="text-muted-foreground text-xs font-medium tracking-wide uppercase">
                  {t("transactions.details")}
                </h3>
                <dl className="grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1.5 text-sm">
                  <Row k={t("transactions.remittance")} v={cleanRemittance(tx.remittance) || "—"} mono />
                  {tx.reference_number ? (
                    <Row k={t("transactions.reference")} v={tx.reference_number} mono />
                  ) : null}
                  <Row k={t("transactions.booked")} v={day(tx.booking_date)} />
                  {tx.value_date && tx.value_date !== tx.booking_date ? (
                    <Row k={t("transactions.value_date")} v={day(tx.value_date)} />
                  ) : null}
                  <Row k={t("transactions.account")} v={tx.account_name || tx.account_id} />
                  {multi ? <Row k={t("common.party")} v={partyName(tx.party_id)} /> : null}
                  {tx.counterparty_iban ? (
                    <Row k={t("transactions.their_iban")} v={tx.counterparty_iban} mono />
                  ) : null}
                  {tx.entry_reference ? (
                    <Row k={t("transactions.bank_ref")} v={tx.entry_reference} mono />
                  ) : null}
                  <Row k={t("transactions.id")} v={tx.id} mono muted />
                </dl>
              </section>

              {record ? (
                <Collapsible>
                  <CollapsibleTrigger className="text-muted-foreground hover:text-foreground flex items-center gap-1 text-xs font-medium tracking-wide uppercase">
                    <ChevronDown className="size-3.5" /> {t("transactions.bank_record")}
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <pre className="bg-muted mt-2 max-h-96 overflow-auto rounded-md p-3 font-mono text-[11px] leading-snug">
                      {record}
                    </pre>
                  </CollapsibleContent>
                </Collapsible>
              ) : null}
            </div>
          </>
        )}
      </SheetContent>
    </Sheet>
  );
}

function Row({ k, v, mono, muted }: { k: string; v: string; mono?: boolean; muted?: boolean }) {
  return (
    <>
      <dt className="text-muted-foreground">{k}</dt>
      <dd
        className={["break-all", mono ? "font-mono text-xs" : "", muted ? "text-muted-foreground" : ""].join(
          " ",
        )}
      >
        {v}
      </dd>
    </>
  );
}
