"use client";

import { Bug, MoreHorizontal, Play, Plus, Search } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useMemo, useState } from "react";
import { toast } from "sonner";

import { useChaos } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { QueuePanel } from "@/components/queue-panel";
import { StatusBadge } from "@/components/status-badge";
import { StressTargetDialog } from "@/components/stress-target";
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
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { CampaignEntry, RunSummary } from "@/lib/api/schema";
import { ago } from "@/lib/format";

type Tab = "all" | "ok" | "broken" | "skipped";

/** The kit's product list over the campaign files: title + primary action, tabs, toolbar, table with row menus. */
export default function CampaignsPage() {
  const router = useRouter();
  const { lastEvent, overview } = useChaos();
  const list = useFetch(() => api.campaigns(), 5000);
  const runs = useFetch(() => api.runs(300), 10_000, [lastEvent]);
  const [tab, setTab] = useState<Tab>("all");
  const [q, setQ] = useState("");
  const [busy, setBusy] = useState<string | null>(null);
  const [against, setAgainst] = useState<CampaignEntry | null>(null);

  const lastRun = useMemo(() => {
    const m = new Map<string, RunSummary>();
    for (const r of runs.data ?? []) if (r.campaign_id && !m.has(r.campaign_id)) m.set(r.campaign_id, r);
    return m;
  }, [runs.data]);

  const shown = (list.data ?? []).filter((c) => {
    if (tab === "ok" && !(c.ok && !c.skip)) return false;
    if (tab === "broken" && c.ok) return false;
    if (tab === "skipped" && !c.skip) return false;
    if (q && !`${c.id} ${c.name ?? ""} ${c.description}`.toLowerCase().includes(q.toLowerCase()))
      return false;
    return true;
  });

  const run = async (id: string) => {
    setBusy(id);
    try {
      const s = await api.runStress({ campaign: id });
      toast.success(`started ${s.name}`);
      router.push(`/runs/view/?id=${s.id}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(null);
    }
  };

  const remove = async (c: CampaignEntry) => {
    if (!confirm(`Delete ${c.file}?`)) return;
    try {
      await api.campaignDelete(c.id);
      list.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  const count = (f: (c: CampaignEntry) => boolean) => (list.data ?? []).filter(f).length;
  const findings = overview?.findings ?? 0;

  return (
    <>
      <PageTitle
        title="Stress campaigns"
        description="Model-checking workers on the ledger: every answer judged against its contract, every broken rule a finding with the trace that got there."
      >
        <Button variant="outline" asChild>
          <Link href="/findings/">
            <Bug /> Findings{findings ? ` (${findings})` : ""}
          </Link>
        </Button>
        <Button asChild>
          <Link href="/stress/view/?id=new">
            <Plus /> New campaign
          </Link>
        </Button>
      </PageTitle>

      <QueuePanel />

      <Tabs value={tab} onValueChange={(v) => setTab(v as Tab)}>
        <TabsList>
          <TabsTrigger value="all">All campaigns ({count(() => true)})</TabsTrigger>
          <TabsTrigger value="ok">Ready ({count((c) => c.ok && !c.skip)})</TabsTrigger>
          <TabsTrigger value="skipped">Skipped ({count((c) => c.skip)})</TabsTrigger>
          <TabsTrigger value="broken">Not checking ({count((c) => !c.ok)})</TabsTrigger>
        </TabsList>
      </Tabs>

      <div className="flex flex-wrap items-center gap-2">
        <InputGroup className="max-w-xs">
          <InputGroupAddon>
            <Search />
          </InputGroupAddon>
          <InputGroupInput placeholder="Search campaigns" value={q} onChange={(e) => setQ(e.target.value)} />
        </InputGroup>
        <span className="text-muted-foreground ml-auto text-xs">{shown.length} shown</span>
      </div>

      {!list.data ? (
        <Skeleton className="h-40" />
      ) : (
        <div className="overflow-x-auto rounded-xl border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Campaign</TableHead>
                <TableHead>Proves</TableHead>
                <TableHead>Runs on</TableHead>
                <TableHead>Check</TableHead>
                <TableHead>Last run</TableHead>
                <TableHead className="text-right">Invariants</TableHead>
                <TableHead className="text-right">Findings</TableHead>
                <TableHead className="w-24 text-right" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {shown.map((c) => {
                const last = lastRun.get(c.id);
                return (
                  <TableRow key={c.id}>
                    <TableCell>
                      <Link
                        href={`/stress/view/?id=${encodeURIComponent(c.id)}`}
                        className="font-medium hover:underline"
                      >
                        {c.name ?? c.id}
                      </Link>
                      <div className="text-muted-foreground font-mono text-[11px]">{c.file}</div>
                    </TableCell>
                    <TableCell className="text-muted-foreground max-w-md truncate">{c.description}</TableCell>
                    <TableCell className="text-muted-foreground text-xs">
                      {c.has_stack ? "its own stack" : "the serve stack's ledgers"}
                    </TableCell>
                    <TableCell>
                      {c.skip ? (
                        <Badge variant="outline">skip</Badge>
                      ) : c.ok ? (
                        <Badge variant="outline" className="text-emerald-600 dark:text-emerald-400">
                          checks
                        </Badge>
                      ) : (
                        <span className="text-destructive text-xs" title={c.error ?? ""}>
                          {c.error}
                        </span>
                      )}
                    </TableCell>
                    <TableCell>
                      {last ? (
                        <Link href={`/runs/view/?id=${last.id}`} className="flex items-center gap-2">
                          <StatusBadge status={last.status} />
                          <span className="text-muted-foreground text-xs">{ago(last.started_at)}</span>
                        </Link>
                      ) : (
                        <span className="text-muted-foreground text-xs">never</span>
                      )}
                    </TableCell>
                    <TableCell className="text-right tabular-nums">
                      {last?.passed ? `${last.passed[0]}/${last.passed[1]}` : "–"}
                    </TableCell>
                    <TableCell
                      className={`text-right tabular-nums ${last?.findings ? "text-destructive font-medium" : ""}`}
                    >
                      {last ? (last.findings ?? 0) : "–"}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button
                          size="sm"
                          variant="outline"
                          disabled={!c.ok || busy === c.id}
                          onClick={() => run(c.id)}
                        >
                          <Play /> Run
                        </Button>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button size="icon-sm" variant="ghost" aria-label="More">
                              <MoreHorizontal />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem disabled={!c.ok} onClick={() => setAgainst(c)}>
                              Run against…
                            </DropdownMenuItem>
                            <DropdownMenuItem asChild>
                              <Link href={`/stress/view/?id=${encodeURIComponent(c.id)}`}>Edit</Link>
                            </DropdownMenuItem>
                            <DropdownMenuItem asChild>
                              <Link href={`/runs/?kind=stress&campaign=${encodeURIComponent(c.id)}`}>
                                Runs
                              </Link>
                            </DropdownMenuItem>
                            <DropdownMenuItem asChild>
                              <Link href={`/findings/?campaign=${encodeURIComponent(c.name ?? c.id)}`}>
                                Findings
                              </Link>
                            </DropdownMenuItem>
                            <DropdownMenuSeparator />
                            <DropdownMenuItem variant="destructive" onClick={() => remove(c)}>
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
                  <TableCell colSpan={8} className="text-muted-foreground py-8 text-center">
                    No campaigns here.
                  </TableCell>
                </TableRow>
              ) : null}
            </TableBody>
          </Table>
        </div>
      )}

      {against ? (
        <StressTargetDialog
          open
          onOpenChange={(open) => {
            if (!open) setAgainst(null);
          }}
          title={`Run ${against.name ?? against.id} against…`}
          description="Which ledger the workers hit. The campaign's own stack is booted for the run; anything else is an existing ledger."
          hasStack={against.has_stack}
          onRun={async (targets) => {
            try {
              const s = await api.runStress({ campaign: against.id, targets });
              toast.success(`started ${s.name}`);
              router.push(`/runs/view/?id=${s.id}`);
            } catch (e) {
              toast.error(describe(e));
            }
          }}
        />
      ) : null}
    </>
  );
}
