"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";
import { MoreHorizontal, Play, Plus, Search } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { PageTitle } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { useChaos } from "@/components/shell/providers";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { RunSummary, ScenarioEntry } from "@/lib/api/schema";
import { ago } from "@/lib/format";

type Tab = "all" | "ok" | "broken" | "skipped";

/** The kit's product list: title + primary action, tabs, toolbar, table with row menus. */
export default function ScenariosPage() {
  const router = useRouter();
  const { lastEvent } = useChaos();
  const list = useFetch(() => api.scenarios(), 5000);
  const runs = useFetch(() => api.runs(300), 10_000, [lastEvent]);
  const [tab, setTab] = useState<Tab>("all");
  const [q, setQ] = useState("");
  const [busy, setBusy] = useState<string | null>(null);

  const lastRun = useMemo(() => {
    const m = new Map<string, RunSummary>();
    for (const r of runs.data ?? []) if (r.scenario_id && !m.has(r.scenario_id)) m.set(r.scenario_id, r);
    return m;
  }, [runs.data]);

  const shown = (list.data ?? []).filter((s) => {
    if (tab === "ok" && !(s.ok && !s.skip)) return false;
    if (tab === "broken" && s.ok) return false;
    if (tab === "skipped" && !s.skip) return false;
    if (q && !`${s.id} ${s.name ?? ""} ${s.description}`.toLowerCase().includes(q.toLowerCase()))
      return false;
    return true;
  });

  const run = async (id: string) => {
    setBusy(id);
    try {
      const s = await api.runScenario(id);
      toast.success(`started ${s.name}`);
      router.push(`/runs/view/?id=${s.id}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(null);
    }
  };

  const remove = async (s: ScenarioEntry) => {
    if (!confirm(`Delete ${s.file}?`)) return;
    try {
      await api.scenarioDelete(s.id);
      list.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  const count = (f: (s: ScenarioEntry) => boolean) => (list.data ?? []).filter(f).length;

  return (
    <>
      <PageTitle
        title="Scenarios"
        description="Browse, edit and run the scenario files: a stack, load, a fault timeline and assertions."
      >
        <Button render={<Link href="/scenarios/view/?id=new" />}>
          <Plus /> New scenario
        </Button>
      </PageTitle>

      <Tabs value={tab} onValueChange={(v) => setTab(v as Tab)}>
        <TabsList variant="line">
          <TabsTrigger value="all">All scenarios ({count(() => true)})</TabsTrigger>
          <TabsTrigger value="ok">Ready ({count((s) => s.ok && !s.skip)})</TabsTrigger>
          <TabsTrigger value="skipped">Skipped ({count((s) => s.skip)})</TabsTrigger>
          <TabsTrigger value="broken">Not checking ({count((s) => !s.ok)})</TabsTrigger>
        </TabsList>
      </Tabs>

      <div className="flex flex-wrap items-center gap-2">
        <InputGroup className="max-w-xs">
          <InputGroupAddon>
            <Search />
          </InputGroupAddon>
          <InputGroupInput placeholder="Search scenarios" value={q} onChange={(e) => setQ(e.target.value)} />
        </InputGroup>
        <span className="ml-auto text-xs text-muted-foreground">{shown.length} shown</span>
      </div>

      {!list.data ? (
        <Skeleton className="h-40" />
      ) : (
        <div className="overflow-x-auto rounded-xl border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Scenario</TableHead>
                <TableHead>Description</TableHead>
                <TableHead>Check</TableHead>
                <TableHead>Last run</TableHead>
                <TableHead className="text-right">Requests</TableHead>
                <TableHead className="text-right">p99</TableHead>
                <TableHead className="w-24 text-right" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {shown.map((s) => {
                const last = lastRun.get(s.id);
                return (
                  <TableRow key={s.id}>
                    <TableCell>
                      <Link
                        href={`/scenarios/view/?id=${encodeURIComponent(s.id)}`}
                        className="font-medium hover:underline"
                      >
                        {s.name ?? s.id}
                      </Link>
                      <div className="font-mono text-[11px] text-muted-foreground">{s.file}</div>
                    </TableCell>
                    <TableCell className="max-w-md truncate text-muted-foreground">{s.description}</TableCell>
                    <TableCell>
                      {s.skip ? (
                        <Badge variant="outline">skip</Badge>
                      ) : s.ok ? (
                        <Badge variant="outline" className="text-emerald-600 dark:text-emerald-400">
                          checks
                        </Badge>
                      ) : (
                        <span className="text-xs text-destructive" title={s.error ?? ""}>
                          {s.error}
                        </span>
                      )}
                    </TableCell>
                    <TableCell>
                      {last ? (
                        <Link href={`/runs/view/?id=${last.id}`} className="flex items-center gap-2">
                          <StatusBadge status={last.status} />
                          <span className="text-xs text-muted-foreground">{ago(last.started_at)}</span>
                        </Link>
                      ) : (
                        <span className="text-xs text-muted-foreground">never</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right tabular-nums">{last?.requests_total ?? "–"}</TableCell>
                    <TableCell className="text-right tabular-nums">
                      {last?.p99_ms !== null && last?.p99_ms !== undefined
                        ? `${last.p99_ms.toFixed(1)} ms`
                        : "–"}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button
                          size="sm"
                          variant="outline"
                          disabled={!s.ok || busy === s.id}
                          onClick={() => run(s.id)}
                        >
                          <Play /> Run
                        </Button>
                        <DropdownMenu>
                          <DropdownMenuTrigger
                            render={<Button size="icon-sm" variant="ghost" aria-label="More" />}
                          >
                            <MoreHorizontal />
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem
                              render={<Link href={`/scenarios/view/?id=${encodeURIComponent(s.id)}`} />}
                            >
                              Edit
                            </DropdownMenuItem>
                            <DropdownMenuItem
                              render={<Link href={`/runs/?scenario=${encodeURIComponent(s.id)}`} />}
                            >
                              Runs
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem variant="destructive" onClick={() => remove(s)}>
                              Delete
                            </DropdownMenuItem>
                          </DropdownMenuContent>
                        </DropdownMenu>
                      </div>
                    </TableCell>
                  </TableRow>
                );
              })}
              {!shown.length ? (
                <TableRow>
                  <TableCell colSpan={7} className="py-8 text-center text-muted-foreground">
                    No scenarios here.
                  </TableCell>
                </TableRow>
              ) : null}
            </TableBody>
          </Table>
        </div>
      )}
    </>
  );
}
