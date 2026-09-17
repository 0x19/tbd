"use client";

import { Plus } from "lucide-react";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { useFinance } from "@/app/providers";
import { PageTitle, SectionTitle } from "@/components/kit";
import { RuleDialog } from "@/components/rule-dialog";
import { ScopeToggle } from "@/components/scope-toggle";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Rule } from "@/lib/api/schema";

export default function CategoriesPage() {
  const { partyIds, partyName, multi } = useFinance();
  const key = partyIds.join(",");
  const categories = useFetch(() => api.categories(partyIds), 0, [key]);
  const rules = useFetch(() => api.rules(partyIds), 0, [key]);
  const [editing, setEditing] = useState<Rule | "new" | null>(null);
  const cats = useMemo(() => categories.data?.categories ?? [], [categories.data]);
  const catName = (id: string) => cats.find((c) => c.id === id)?.name ?? "?";
  const list = rules.data?.rules ?? [];
  const zero = list.filter((r) => r.enabled && r.hits === "0").length;

  const toggle = async (r: Rule, enabled: boolean) => {
    try {
      const done = await api.upsertRule({ ...r, enabled });
      toast.success(
        `${enabled ? "Enabled" : "Disabled"}; ${done.categorised} rows categorised, ${done.unmatched} unmatched.`,
      );
      rules.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  return (
    <>
      <PageTitle
        title="Categories & rules"
        description="Rules apply in priority order, lowest number first; a category you set by hand always wins."
      >
        <ScopeToggle className="md:hidden" />
        <Button size="sm" onClick={() => setEditing("new")} disabled={!cats.length}>
          <Plus /> New rule
        </Button>
      </PageTitle>

      <div className="grid gap-6 xl:grid-cols-3">
        <Card className="xl:col-span-2">
          <CardHeader>
            <SectionTitle
              title="Rules"
              description={
                zero
                  ? `${zero} enabled ${zero === 1 ? "rule claims" : "rules claim"} nothing: wrong, or obsolete.`
                  : "Every enabled rule claims at least one row."
              }
            />
          </CardHeader>
          <CardContent className="p-0">
            {rules.error ? (
              <p className="text-destructive p-4 text-sm">{rules.error}</p>
            ) : rules.loading && !rules.data ? (
              <Skeleton className="m-4 h-64" />
            ) : (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-16">Prio</TableHead>
                    <TableHead>Name</TableHead>
                    <TableHead>Matches</TableHead>
                    <TableHead>Category</TableHead>
                    {multi ? <TableHead className="hidden md:table-cell">Party</TableHead> : null}
                    <TableHead className="text-right">Hits</TableHead>
                    <TableHead className="w-16">On</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {list.map((r) => (
                    <TableRow key={r.id} className="cursor-pointer" onClick={() => setEditing(r)}>
                      <TableCell className="font-mono text-xs">{r.priority}</TableCell>
                      <TableCell className="font-medium">{r.name}</TableCell>
                      <TableCell className="text-muted-foreground font-mono text-xs">
                        {[
                          r.match_counterparty_like && `name ~ ${r.match_counterparty_like}`,
                          r.match_remittance_like && `text ~ ${r.match_remittance_like}`,
                          r.match_counterparty_iban && `iban ${r.match_counterparty_iban}`,
                          r.match_currency && r.match_currency,
                          r.match_credit_debit && r.match_credit_debit,
                        ]
                          .filter(Boolean)
                          .join(" · ")}
                      </TableCell>
                      <TableCell>{catName(r.category_id)}</TableCell>
                      {multi ? (
                        <TableCell className="text-muted-foreground hidden text-xs md:table-cell">
                          {partyName(r.party_id)}
                        </TableCell>
                      ) : null}
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
            )}
          </CardContent>
        </Card>

        <Card>
          <CardHeader>
            <CardTitle>Categories</CardTitle>
            <CardDescription>What a category does to the books, not just what it is called.</CardDescription>
          </CardHeader>
          <CardContent className="space-y-1.5">
            {cats
              .filter((c) => !c.archived)
              .map((c) => (
                <div key={c.id} className="flex items-center justify-between gap-2 text-sm">
                  <span className="truncate">
                    {c.name}
                    {multi ? (
                      <span className="text-muted-foreground ml-2 text-xs">{partyName(c.party_id)}</span>
                    ) : null}
                  </span>
                  <Badge variant="outline" className="text-[10px]">
                    {c.kind}
                  </Badge>
                </div>
              ))}
          </CardContent>
        </Card>
      </div>

      {editing ? (
        <RuleDialog
          rule={editing === "new" ? null : editing}
          categories={cats.filter((c) => !c.archived)}
          onClose={() => setEditing(null)}
          onSaved={() => {
            setEditing(null);
            rules.reload();
          }}
        />
      ) : null}
    </>
  );
}
