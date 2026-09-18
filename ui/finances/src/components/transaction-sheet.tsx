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
  const { multi, partyName } = useFinance();
  const one = useFetch(() => (id ? api.transaction(id) : Promise.resolve(null)), 0, [id]);
  const [busy, setBusy] = useState(false);
  const t = one.data?.transaction ?? null;

  const declare = async (v: string) => {
    if (!t || v === t.category_id) return;
    setBusy(true);
    try {
      await api.declare(t.id, v);
      toast.success("Category set; rules will not change it again.", {
        action: { label: "Make it a rule", onClick: () => onMakeRule({ ...t, category_id: v }) },
      });
      one.reload();
      onChanged();
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const rule = t?.category_rule_id ? rules.find((r) => r.id === t.category_rule_id) : undefined;
  const mine = t
    ? cats.filter((c) => c.party_id === t.party_id && (!c.archived || c.id === t.category_id))
    : [];
  const negative = t?.amount_minor.startsWith("-") ?? false;
  let record: string | null = null;
  if (t?.raw) {
    try {
      record = JSON.stringify(JSON.parse(t.raw), null, 2);
    } catch {
      record = t.raw;
    }
  }

  return (
    <Sheet open={id !== null} onOpenChange={(o) => !o && onClose()}>
      <SheetContent className="flex w-full flex-col gap-0 overflow-y-auto p-0 sm:max-w-lg">
        {one.error ? (
          <p className="text-destructive p-6 text-sm">{one.error}</p>
        ) : !t ? (
          <div className="space-y-3 p-6">
            <Skeleton className="h-8 w-40" />
            <Skeleton className="h-4 w-64" />
            <Skeleton className="h-48 w-full" />
          </div>
        ) : (
          <>
            <SheetHeader className="border-b p-6">
              <SheetDescription className="flex items-center gap-2 text-xs">
                {day(t.booking_date)}
                <StatusBadge status={t.status.toLowerCase()} className="text-[10px]" />
                {t.internal ? (
                  <Badge variant="outline" className="text-[10px]">
                    own transfer
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
                {money(t.amount_minor, t.currency, { sign: true })}
              </SheetTitle>
              <p className="text-base font-medium">
                {t.counterparty_name || <span className="text-muted-foreground italic">No counterparty</span>}
              </p>
            </SheetHeader>

            <div className="space-y-6 p-6">
              <section className="space-y-2">
                <h3 className="text-muted-foreground text-xs font-medium tracking-wide uppercase">
                  Category
                </h3>
                <div className="flex flex-wrap items-center gap-2">
                  <Select
                    value={t.category_id || "none"}
                    disabled={busy}
                    onValueChange={(v) => void declare(v)}
                  >
                    <SelectTrigger className="w-56">
                      <SelectValue placeholder="Uncategorised" />
                    </SelectTrigger>
                    <SelectContent>
                      <SelectItem value="none" disabled>
                        Uncategorised
                      </SelectItem>
                      {mine.map((c) => (
                        <SelectItem key={c.id} value={c.id}>
                          {c.name}
                          {c.archived ? " (archived)" : ""}
                        </SelectItem>
                      ))}
                    </SelectContent>
                  </Select>
                  {t.category_source ? (
                    <StatusBadge status={t.category_source} className="text-[10px]" />
                  ) : null}
                </div>
                <p className="text-muted-foreground text-xs">
                  {t.category_source === "declared"
                    ? `Set by hand${t.categorised_at ? ` ${when(t.categorised_at)}` : ""}; no rule will change it.`
                    : t.category_source === "inferred"
                      ? `Claimed by the rule ${rule ? `“${rule.name}” (priority ${rule.priority})` : "that has since gone"}${t.categorised_at ? `, ${when(t.categorised_at)}` : ""}.`
                      : "No rule claims it. Pick a category here, or make a rule so the next one is claimed too."}
                </p>
                <div className="flex flex-wrap gap-2 pt-1">
                  <Button size="sm" variant="outline" onClick={() => onMakeRule(t)}>
                    <Wand2 /> Make a rule from this
                  </Button>
                  {t.counterparty_name ? (
                    <Button size="sm" variant="ghost" onClick={() => onFilter(t.counterparty_name)}>
                      <Filter /> All from this counterparty
                    </Button>
                  ) : null}
                </div>
              </section>

              <section className="space-y-2">
                <h3 className="text-muted-foreground text-xs font-medium tracking-wide uppercase">Details</h3>
                <dl className="grid grid-cols-[8rem_1fr] gap-x-3 gap-y-1.5 text-sm">
                  <Row k="Remittance" v={cleanRemittance(t.remittance) || "—"} mono />
                  {t.reference_number ? <Row k="Reference" v={t.reference_number} mono /> : null}
                  <Row k="Booked" v={day(t.booking_date)} />
                  {t.value_date && t.value_date !== t.booking_date ? (
                    <Row k="Value date" v={day(t.value_date)} />
                  ) : null}
                  <Row k="Account" v={t.account_name || t.account_id} />
                  {multi ? <Row k="Party" v={partyName(t.party_id)} /> : null}
                  {t.counterparty_iban ? <Row k="Their IBAN" v={t.counterparty_iban} mono /> : null}
                  {t.entry_reference ? <Row k="Bank's ref" v={t.entry_reference} mono /> : null}
                  <Row k="Id" v={t.id} mono muted />
                </dl>
              </section>

              {record ? (
                <Collapsible>
                  <CollapsibleTrigger className="text-muted-foreground hover:text-foreground flex items-center gap-1 text-xs font-medium tracking-wide uppercase">
                    <ChevronDown className="size-3.5" /> The bank&apos;s record
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
