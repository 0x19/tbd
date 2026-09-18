"use client";

import { ChevronDown, ExternalLink, Pencil, Plus, Search } from "lucide-react";
import Link from "next/link";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { CategoryDialog } from "@/components/category-dialog";
import { PageTitle } from "@/components/kit";
import { pluralForm, RuleDialog } from "@/components/rule-dialog";
import { ScopeToggle } from "@/components/scope-toggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent } from "@/components/ui/card";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsContent, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Category, Rule, SummaryRow } from "@/lib/api/schema";
import { money, monthsBefore, thisMonth } from "@/lib/format";
import { useT } from "@/lib/i18n";

const MONTHS = 12;

type Usage = { count: number; out: bigint; in: bigint };

/** Twelve months of use per category id, in one currency. */
function usage(rows: SummaryRow[], currency: string): Map<string, Usage> {
  const m = new Map<string, Usage>();
  for (const r of rows) {
    if (r.currency !== currency) continue;
    const key = r.category_id || (r.internal ? "internal" : "none");
    const u = m.get(key) ?? { count: 0, out: 0n, in: 0n };
    const v = BigInt(r.total_minor);
    u.count += r.count;
    if (v < 0n) u.out -= v;
    else u.in += v;
    m.set(key, u);
  }
  return m;
}

type T = ReturnType<typeof useT>;

function matches(r: Rule, t: T): string {
  return [
    r.match_counterparty_like && t("categories.match.name", { v: r.match_counterparty_like }),
    r.match_remittance_like && t("categories.match.text", { v: r.match_remittance_like }),
    r.match_counterparty_iban && t("categories.match.iban", { v: r.match_counterparty_iban }),
    r.match_currency,
    r.match_credit_debit,
  ]
    .filter(Boolean)
    .join(" · ");
}

/** "{n} rows", bent to the count in the current language. */
function rows(n: number, t: T): string {
  return t(`categories.rows.${pluralForm(n)}`, { n });
}

function ruleCount(n: number, t: T): string {
  return t(`categories.rules.${pluralForm(n)}`, { n });
}

export default function CategoriesPage() {
  const t = useT();
  const { partyIds, partyName, multi } = useFinance();
  const key = partyIds.join(",");
  const categories = useFetch(() => api.categories(partyIds), 0, [key]);
  const rules = useFetch(() => api.rules(partyIds), 0, [key]);
  const from = useMemo(() => monthsBefore(thisMonth(), MONTHS - 1), []);
  const summary = useFetch(() => api.summary(partyIds, from, true), 0, [key, from]);
  const [editingRule, setEditingRule] = useState<Rule | "new" | null>(null);
  const [editingCat, setEditingCat] = useState<Category | "new" | null>(null);
  const [query, setQuery] = useState("");
  const [only, setOnly] = useState<"all" | "idle" | "off">("all");

  const cats = useMemo(() => categories.data?.categories ?? [], [categories.data]);
  const list = useMemo(() => rules.data?.rules ?? [], [rules.data]);
  const used = useMemo(() => usage(summary.data?.rows ?? [], "EUR"), [summary.data]);
  const catName = (id: string) => cats.find((c) => c.id === id)?.name ?? "?";
  const reloadAll = () => {
    categories.reload();
    rules.reload();
    summary.reload();
  };

  const toggle = async (r: Rule, enabled: boolean) => {
    try {
      const done = await api.upsertRule({ ...r, enabled });
      toast.success(
        t(enabled ? "categories.toggled.enabled" : "categories.toggled.disabled", {
          n: done.categorised,
          m: done.unmatched,
        }),
      );
      reloadAll();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  // Rules, filtered, then grouped under their category, busiest group first.
  const groups = useMemo(() => {
    const q = query.trim().toUpperCase();
    const kept = list.filter((r) => {
      if (only === "idle" && !(r.enabled && r.hits === "0")) return false;
      if (only === "off" && r.enabled) return false;
      if (!q) return true;
      return `${r.name} ${matches(r, t)} ${catName(r.category_id)}`.toUpperCase().includes(q);
    });
    const byCat = new Map<string, Rule[]>();
    for (const r of kept) byCat.set(r.category_id, [...(byCat.get(r.category_id) ?? []), r]);
    return [...byCat.entries()]
      .map(([id, rs]) => ({
        id,
        name: catName(id),
        party: cats.find((c) => c.id === id)?.party_id ?? "",
        rules: rs.sort((a, b) => a.priority - b.priority || a.name.localeCompare(b.name)),
        hits: rs.reduce((n, r) => n + Number(r.hits), 0),
      }))
      .sort((a, b) => b.hits - a.hits || a.name.localeCompare(b.name));
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [list, query, only, cats, t]);

  const idle = list.filter((r) => r.enabled && r.hits === "0").length;
  const off = list.filter((r) => !r.enabled).length;
  const none = used.get("none");
  const live = cats.filter((c) => !c.archived);
  const archived = cats.filter((c) => c.archived);
  const byParty = multi
    ? [...new Set(live.map((c) => c.party_id))].map((p) => ({
        party: p,
        cats: live.filter((c) => c.party_id === p),
      }))
    : [{ party: "", cats: live }];
  const rulesOf = (id: string) => list.filter((r) => r.category_id === id);

  return (
    <>
      <PageTitle title={t("categories.title")} description={t("categories.description")}>
        <ScopeToggle className="md:hidden" />
      </PageTitle>

      <Tabs defaultValue="categories">
        <div className="flex flex-wrap items-center justify-between gap-2">
          <TabsList>
            <TabsTrigger value="categories">{t("categories.tab.categories", { n: live.length })}</TabsTrigger>
            <TabsTrigger value="rules">{t("categories.tab.rules", { n: list.length })}</TabsTrigger>
          </TabsList>
          <div className="flex gap-2">
            <Button size="sm" variant="outline" onClick={() => setEditingCat("new")}>
              <Plus /> {t("categories.new_category")}
            </Button>
            <Button size="sm" onClick={() => setEditingRule("new")} disabled={!live.length}>
              <Plus /> {t("categories.new_rule")}
            </Button>
          </div>
        </div>

        <TabsContent value="categories" className="space-y-6">
          {categories.error ? (
            <p className="text-destructive text-sm">{categories.error}</p>
          ) : categories.loading && !categories.data ? (
            <Skeleton className="h-64 w-full" />
          ) : (
            <>
              <Card
                className={
                  none && none.count
                    ? "border-amber-500/40 bg-amber-500/5"
                    : "border-emerald-500/40 bg-emerald-500/5"
                }
              >
                <CardContent className="flex flex-wrap items-center justify-between gap-3 py-4">
                  <div>
                    <p className="font-medium">
                      {none && none.count
                        ? t("categories.none.some", { n: none.count, months: MONTHS })
                        : t("categories.none.all", { months: MONTHS })}
                    </p>
                    <p className="text-muted-foreground text-sm">
                      {none && none.count
                        ? t("categories.none.some_hint", {
                            out: money(none.out.toString()),
                            in: money(none.in.toString()),
                          })
                        : t("categories.none.all_hint")}
                    </p>
                  </div>
                  {none && none.count ? (
                    <Button asChild size="sm">
                      <Link href="/transactions/?category=none">{t("categories.sort_them_out")}</Link>
                    </Button>
                  ) : null}
                </CardContent>
              </Card>

              {byParty.map(({ party, cats: mine }) => (
                <section key={party} className="space-y-3">
                  {multi ? <h2 className="text-sm font-medium">{partyName(party)}</h2> : null}
                  <div className="grid gap-3 sm:grid-cols-2 xl:grid-cols-3">
                    {[...mine]
                      .sort((a, b) => {
                        const ua = used.get(a.id);
                        const ub = used.get(b.id);
                        const va = (ua?.out ?? 0n) + (ua?.in ?? 0n);
                        const vb = (ub?.out ?? 0n) + (ub?.in ?? 0n);
                        return va < vb ? 1 : va > vb ? -1 : a.name.localeCompare(b.name);
                      })
                      .map((c) => (
                        <CategoryCard
                          key={c.id}
                          c={c}
                          u={used.get(c.id)}
                          rules={rulesOf(c.id)}
                          onEdit={() => setEditingCat(c)}
                        />
                      ))}
                  </div>
                </section>
              ))}

              {archived.length ? (
                <Collapsible>
                  <CollapsibleTrigger className="text-muted-foreground hover:text-foreground flex items-center gap-1 text-sm">
                    <ChevronDown className="size-4" /> {t("categories.archived_n", { n: archived.length })}
                  </CollapsibleTrigger>
                  <CollapsibleContent className="mt-2 flex flex-wrap gap-2">
                    {archived.map((c) => (
                      <Button key={c.id} variant="outline" size="sm" onClick={() => setEditingCat(c)}>
                        {c.name}
                        {multi ? (
                          <span className="text-muted-foreground">· {partyName(c.party_id)}</span>
                        ) : null}
                      </Button>
                    ))}
                  </CollapsibleContent>
                </Collapsible>
              ) : null}
            </>
          )}
        </TabsContent>

        <TabsContent value="rules" className="space-y-4">
          <div className="flex flex-wrap items-center gap-2">
            <div className="relative">
              <Search className="text-muted-foreground absolute top-1/2 left-2.5 size-4 -translate-y-1/2" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder={t("categories.search_placeholder")}
                className="w-64 pl-8"
              />
            </div>
            <Button size="sm" variant={only === "all" ? "default" : "outline"} onClick={() => setOnly("all")}>
              {t("common.all")}
            </Button>
            <Button
              size="sm"
              variant={only === "idle" ? "default" : "outline"}
              onClick={() => setOnly(only === "idle" ? "all" : "idle")}
              className={idle && only !== "idle" ? "text-amber-700 dark:text-amber-300" : undefined}
            >
              {t("categories.filter.idle", { n: idle })}
            </Button>
            <Button
              size="sm"
              variant={only === "off" ? "default" : "outline"}
              onClick={() => setOnly(only === "off" ? "all" : "off")}
            >
              {t("categories.filter.off", { n: off })}
            </Button>
            <span className="text-muted-foreground ml-auto text-xs">{t("categories.priority_hint")}</span>
          </div>

          {rules.error ? (
            <p className="text-destructive text-sm">{rules.error}</p>
          ) : rules.loading && !rules.data ? (
            <Skeleton className="h-64 w-full" />
          ) : groups.length === 0 ? (
            <p className="text-muted-foreground py-10 text-center text-sm">
              {t("categories.no_rule_matches")}
            </p>
          ) : (
            groups.map((g) => (
              <Collapsible key={g.id} defaultOpen={groups.length <= 8 || Boolean(query) || only !== "all"}>
                <Card>
                  <CollapsibleTrigger className="hover:bg-muted/50 flex w-full items-center justify-between gap-3 px-4 py-3 text-left">
                    <span className="flex items-center gap-2">
                      <ChevronDown className="text-muted-foreground size-4" />
                      <span className="font-medium">{g.name}</span>
                      {multi && g.party ? (
                        <span className="text-muted-foreground text-xs">{partyName(g.party)}</span>
                      ) : null}
                    </span>
                    <span className="text-muted-foreground font-mono text-xs tabular-nums">
                      {ruleCount(g.rules.length, t)} · {rows(g.hits, t)}
                    </span>
                  </CollapsibleTrigger>
                  <CollapsibleContent>
                    <CardContent className="p-0">
                      <Table>
                        <TableHeader>
                          <TableRow>
                            <TableHead className="w-16">{t("categories.th.prio")}</TableHead>
                            <TableHead>{t("common.name")}</TableHead>
                            <TableHead>{t("categories.th.matches")}</TableHead>
                            <TableHead className="w-20 text-right">{t("categories.th.rows")}</TableHead>
                            <TableHead className="w-16">{t("categories.th.on")}</TableHead>
                          </TableRow>
                        </TableHeader>
                        <TableBody>
                          {g.rules.map((r) => (
                            <TableRow key={r.id} className="cursor-pointer" onClick={() => setEditingRule(r)}>
                              <TableCell className="font-mono text-xs">{r.priority}</TableCell>
                              <TableCell className="font-medium">{r.name}</TableCell>
                              <TableCell className="text-muted-foreground font-mono text-xs">
                                {matches(r, t)}
                              </TableCell>
                              <TableCell
                                className={
                                  r.enabled && r.hits === "0"
                                    ? "text-right font-mono text-amber-600 tabular-nums"
                                    : "text-right font-mono tabular-nums"
                                }
                              >
                                {r.hits}
                              </TableCell>
                              <TableCell onClick={(e) => e.stopPropagation()}>
                                <Switch checked={r.enabled} onCheckedChange={(v) => void toggle(r, v)} />
                              </TableCell>
                            </TableRow>
                          ))}
                        </TableBody>
                      </Table>
                    </CardContent>
                  </CollapsibleContent>
                </Card>
              </Collapsible>
            ))
          )}
        </TabsContent>
      </Tabs>

      {editingRule ? (
        <RuleDialog
          rule={editingRule === "new" ? null : editingRule}
          categories={live}
          onClose={() => setEditingRule(null)}
          onSaved={() => {
            setEditingRule(null);
            reloadAll();
          }}
        />
      ) : null}
      {editingCat ? (
        <CategoryDialog
          category={editingCat === "new" ? null : editingCat}
          onClose={() => setEditingCat(null)}
          onSaved={() => {
            setEditingCat(null);
            reloadAll();
          }}
        />
      ) : null}
    </>
  );
}

function CategoryCard({
  c,
  u,
  rules,
  onEdit,
}: {
  c: Category;
  u: Usage | undefined;
  rules: Rule[];
  onEdit: () => void;
}) {
  const t = useT();
  const idle = rules.filter((r) => r.enabled && r.hits === "0").length;
  const flow = c.kind === "income" ? (u?.in ?? 0n) : (u?.out ?? 0n);
  return (
    <Card className="gap-2 py-4">
      <CardContent className="space-y-2 px-4">
        <div className="flex items-start justify-between gap-2">
          <div className="min-w-0">
            <p className="truncate font-medium">{c.name}</p>
            <p className="text-muted-foreground text-xs">
              <Badge variant="outline" className="mr-1 text-[10px]">
                {t(`categories.kind.${c.kind}`)}
              </Badge>
              {c.deductible ? t("categories.card.deductible") : ""}
            </p>
          </div>
          <Button
            size="icon-sm"
            variant="ghost"
            onClick={onEdit}
            aria-label={t("categories.card.edit", { name: c.name })}
          >
            <Pencil />
          </Button>
        </div>
        <div className="flex items-baseline justify-between">
          <span className="font-mono text-lg tabular-nums">{money(flow.toString())}</span>
          <span className="text-muted-foreground text-xs">
            {rows(u?.count ?? 0, t)} · {t("categories.months", { n: MONTHS })}
          </span>
        </div>
        <div className="flex items-center justify-between text-xs">
          <span className={idle ? "text-amber-700 dark:text-amber-300" : "text-muted-foreground"}>
            {rules.length === 0
              ? t("categories.card.no_rules")
              : `${ruleCount(rules.length, t)}${idle ? t("categories.card.idle", { n: idle }) : ""}`}
          </span>
          <Link
            href={`/transactions/?category=${c.id}`}
            className="text-muted-foreground hover:text-foreground inline-flex items-center gap-1"
          >
            {t("categories.card.rows")} <ExternalLink className="size-3" />
          </Link>
        </div>
      </CardContent>
    </Card>
  );
}
