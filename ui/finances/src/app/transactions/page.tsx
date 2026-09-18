"use client";

import { Search, X } from "lucide-react";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { RuleDialog, suggestName } from "@/components/rule-dialog";
import { ScopeToggle } from "@/components/scope-toggle";
import { StatusBadge } from "@/components/status-badge";
import { cleanRemittance, TransactionSheet } from "@/components/transaction-sheet";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Category, Transaction, UpsertRule } from "@/lib/api/schema";
import { day, money, monthLabel, monthsBefore, thisMonth } from "@/lib/format";
import { useT } from "@/lib/i18n";

const PAGE = 100;

export default function TransactionsPage() {
  return (
    <Suspense fallback={<Skeleton className="h-64 w-full" />}>
      <Transactions />
    </Suspense>
  );
}

/** A rule prefilled from a row: the counterparty as the condition, its
 *  category if it has one, a name from the first word. */
function ruleFrom(t: Transaction): Partial<UpsertRule> {
  const byName = t.counterparty_name.trim();
  return {
    party_id: t.party_id,
    name: suggestName(byName || cleanRemittance(t.remittance)),
    category_id: t.category_id || undefined,
    match_counterparty_like: byName,
    match_remittance_like: byName ? "" : (cleanRemittance(t.remittance).split(",")[0]?.trim() ?? ""),
    priority: 50,
  };
}

function Transactions() {
  const t = useT();
  const params = useSearchParams();
  const router = useRouter();
  const { partyIds, partyName, multi } = useFinance();
  const month = params.get("month") ?? "";
  const category = params.get("category") ?? "";
  const account = params.get("account") ?? "";
  const [search, setSearch] = useState(params.get("q") ?? "");
  const [applied, setApplied] = useState(search);
  const [offset, setOffset] = useState(0);
  const [selected, setSelected] = useState<string | null>(null);
  const [ruleSeed, setRuleSeed] = useState<Partial<UpsertRule> | null>(null);

  const set = (k: string, v: string) => {
    const q = new URLSearchParams(params.toString());
    if (v) q.set(k, v);
    else q.delete(k);
    setOffset(0);
    router.replace(`/transactions/?${q.toString()}`);
  };
  const applySearch = (text: string) => {
    setSearch(text);
    setApplied(text);
    setOffset(0);
    set("q", text);
  };

  const key = [partyIds.join(","), month, category, account, applied, offset].join("|");
  const list = useFetch(
    () =>
      api.transactions({
        party_ids: partyIds,
        limit: PAGE,
        offset,
        month,
        category_id: category,
        account_id: account,
        search: applied,
      }),
    0,
    [key],
  );
  const scope = partyIds.join(",");
  const categories = useFetch(() => api.categories(partyIds), 0, [scope]);
  const rules = useFetch(() => api.rules(partyIds), 0, [scope]);
  const cats = useMemo(() => categories.data?.categories ?? [], [categories.data]);
  const live = useMemo(() => cats.filter((c) => !c.archived), [cats]);
  const rows = list.data?.transactions ?? [];

  const months = useMemo(() => {
    const now = thisMonth();
    return Array.from({ length: 24 }, (_, i) => monthsBefore(now, i));
  }, []);
  const filtered = Boolean(month || category || account || applied);

  return (
    <>
      <PageTitle title={t("transactions.title")} description={t("transactions.description")}>
        <ScopeToggle className="md:hidden" />
      </PageTitle>

      <div className="flex flex-wrap items-center gap-2">
        <form
          className="flex items-center gap-2"
          onSubmit={(e) => {
            e.preventDefault();
            applySearch(search.trim());
          }}
        >
          <div className="relative">
            <Search className="text-muted-foreground absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
            <Input
              value={search}
              onChange={(e) => setSearch(e.target.value)}
              placeholder={t("transactions.search_placeholder")}
              className="w-64 pl-8"
            />
          </div>
          <Button type="submit" variant="outline" size="sm">
            {t("common.search")}
          </Button>
        </form>
        <Select value={month || "any"} onValueChange={(v) => set("month", v === "any" ? "" : v)}>
          <SelectTrigger className="w-40">
            <SelectValue placeholder={t("common.any_month")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="any">{t("common.any_month")}</SelectItem>
            {months.map((m) => (
              <SelectItem key={m} value={m}>
                {monthLabel(m)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Select value={category || "any"} onValueChange={(v) => set("category", v === "any" ? "" : v)}>
          <SelectTrigger className="w-52">
            <SelectValue placeholder={t("transactions.any_category")} />
          </SelectTrigger>
          <SelectContent>
            <SelectItem value="any">{t("transactions.any_category")}</SelectItem>
            <SelectItem value="none">{t("transactions.uncategorised")}</SelectItem>
            {live.map((c) => (
              <SelectItem key={c.id} value={c.id}>
                {c.name}
                {multi ? ` · ${partyName(c.party_id)}` : ""}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          variant={category === "none" ? "default" : "outline"}
          size="sm"
          onClick={() => set("category", category === "none" ? "" : "none")}
        >
          {t("transactions.uncategorised_only")}
        </Button>
        {filtered ? (
          <Button
            variant="ghost"
            size="sm"
            onClick={() => {
              setSearch("");
              setApplied("");
              setOffset(0);
              router.replace("/transactions/");
            }}
          >
            <X /> {t("transactions.clear")}
          </Button>
        ) : null}
      </div>

      <Card>
        <CardContent className="p-0">
          {list.error ? (
            <p className="text-destructive p-4 text-sm">{list.error}</p>
          ) : list.loading && !list.data ? (
            <Skeleton className="m-4 h-64" />
          ) : (
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead className="w-24">{t("common.date")}</TableHead>
                  <TableHead>{t("transactions.counterparty")}</TableHead>
                  <TableHead className="hidden lg:table-cell">{t("transactions.remittance")}</TableHead>
                  {multi ? <TableHead className="hidden md:table-cell">{t("common.party")}</TableHead> : null}
                  <TableHead>{t("transactions.category")}</TableHead>
                  <TableHead className="text-right">{t("common.amount")}</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {rows.length === 0 ? (
                  <TableRow>
                    <TableCell colSpan={6} className="text-muted-foreground py-10 text-center text-sm">
                      {t("common.nothing_matches")}
                    </TableCell>
                  </TableRow>
                ) : null}
                {rows.map((row) => (
                  <Row
                    key={row.id}
                    tx={row}
                    cats={live}
                    multi={multi}
                    partyName={partyName}
                    onOpen={() => setSelected(row.id)}
                    onChanged={list.reload}
                    onMakeRule={(seed) => setRuleSeed(ruleFrom(seed))}
                  />
                ))}
              </TableBody>
            </Table>
          )}
        </CardContent>
      </Card>
      <div className="flex items-center justify-between text-sm">
        <span className="text-muted-foreground">
          {rows.length ? `${offset + 1}–${offset + rows.length}` : "0"}
          {rows.length === PAGE ? ` · ${t("transactions.more_below")}` : ""}
        </span>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            disabled={offset === 0}
            onClick={() => setOffset(Math.max(0, offset - PAGE))}
          >
            {t("transactions.previous")}
          </Button>
          <Button
            variant="outline"
            size="sm"
            disabled={rows.length < PAGE}
            onClick={() => setOffset(offset + PAGE)}
          >
            {t("transactions.next")}
          </Button>
        </div>
      </div>

      <TransactionSheet
        id={selected}
        cats={cats}
        rules={rules.data?.rules ?? []}
        onClose={() => setSelected(null)}
        onChanged={list.reload}
        onMakeRule={(row) => setRuleSeed(ruleFrom(row))}
        onFilter={(name) => {
          setSelected(null);
          applySearch(name);
        }}
      />
      {ruleSeed ? (
        <RuleDialog
          rule={null}
          initial={ruleSeed}
          categories={live}
          onClose={() => setRuleSeed(null)}
          onSaved={() => {
            setRuleSeed(null);
            list.reload();
            rules.reload();
          }}
        />
      ) : null}
    </>
  );
}

function Row({
  tx,
  cats,
  multi,
  partyName,
  onOpen,
  onChanged,
  onMakeRule,
}: {
  tx: Transaction;
  cats: Category[];
  multi: boolean;
  partyName: (id: string) => string;
  onOpen: () => void;
  onChanged: () => void;
  onMakeRule: (t: Transaction) => void;
}) {
  const t = useT();
  const [busy, setBusy] = useState(false);
  const mine = cats.filter((c) => c.party_id === tx.party_id);
  const negative = tx.amount_minor.startsWith("-");
  const remittance = cleanRemittance(tx.remittance);
  return (
    <TableRow className={["cursor-pointer", tx.internal ? "opacity-60" : ""].join(" ")} onClick={onOpen}>
      <TableCell className="text-muted-foreground font-mono text-xs whitespace-nowrap">
        {day(tx.booking_date)}
        {tx.status !== "BOOKED" ? (
          <Badge variant="outline" className="ml-1 text-[10px]">
            {t("status.pending")}
          </Badge>
        ) : null}
      </TableCell>
      <TableCell className="max-w-64 truncate font-medium">
        {tx.counterparty_name || (
          <span className="text-muted-foreground italic">{remittance.split(",")[0] || "—"}</span>
        )}
        {tx.internal ? (
          <Badge variant="outline" className="ml-2 text-[10px]">
            {t("transactions.own_transfer")}
          </Badge>
        ) : null}
      </TableCell>
      <TableCell className="text-muted-foreground hidden max-w-80 truncate text-xs lg:table-cell">
        {remittance}
      </TableCell>
      {multi ? (
        <TableCell className="text-muted-foreground hidden text-xs md:table-cell">
          {partyName(tx.party_id)}
        </TableCell>
      ) : null}
      <TableCell onClick={(e) => e.stopPropagation()}>
        <div className="flex items-center gap-2">
          <Select
            value={tx.category_id || "none"}
            disabled={busy}
            onValueChange={async (v) => {
              if (v === "none" || v === tx.category_id) return;
              setBusy(true);
              try {
                await api.declare(tx.id, v);
                toast.success(t("transactions.category_set"), {
                  action: {
                    label: t("transactions.make_it_a_rule"),
                    onClick: () => onMakeRule({ ...tx, category_id: v }),
                  },
                });
                onChanged();
              } catch (e) {
                toast.error(describe(e));
              } finally {
                setBusy(false);
              }
            }}
          >
            <SelectTrigger
              className={
                tx.category_id
                  ? "h-7 w-44 text-xs"
                  : "h-7 w-44 border-dashed text-xs text-amber-700 dark:text-amber-300"
              }
            >
              <SelectValue placeholder={t("transactions.uncategorised")} />
            </SelectTrigger>
            <SelectContent>
              <SelectItem value="none" disabled>
                {t("transactions.uncategorised")}
              </SelectItem>
              {mine.map((c) => (
                <SelectItem key={c.id} value={c.id}>
                  {c.name}
                </SelectItem>
              ))}
            </SelectContent>
          </Select>
          {tx.category_source ? (
            <StatusBadge status={tx.category_source} className="hidden text-[10px] xl:inline-flex" />
          ) : null}
        </div>
      </TableCell>
      <TableCell
        className={
          negative
            ? "text-right font-mono tabular-nums"
            : "text-right font-mono text-emerald-700 tabular-nums dark:text-emerald-300"
        }
      >
        {money(tx.amount_minor, tx.currency, { sign: true })}
      </TableCell>
    </TableRow>
  );
}
