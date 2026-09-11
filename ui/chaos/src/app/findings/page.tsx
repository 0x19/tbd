"use client";

import { RefreshCw, Search } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";

import { useChaos } from "@/app/providers";
import { FilterRail, PageTitle } from "@/components/kit";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { api, type FindingQuery } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { FindingGroup, FindingSummary } from "@/lib/api/schema";
import { ago } from "@/lib/format";

export default function FindingsPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <FindingsFromUrl />
    </Suspense>
  );
}

/** `?run=`, `?campaign=` and `?invariant=` narrow the API query; the rail narrows further on the client. */
function FindingsFromUrl() {
  const params = useSearchParams();
  const query: FindingQuery = {
    limit: 500,
    ...(params.get("run") ? { run: params.get("run")! } : {}),
    ...(params.get("campaign") ? { campaign: params.get("campaign")! } : {}),
    ...(params.get("invariant") ? { invariant: params.get("invariant")! } : {}),
  };
  return <Findings key={params.toString()} query={query} />;
}

type View = "grouped" | "all";

/** The kit's Events & Logs page over findings: grouped by signature, or every finding, newest first. */
function Findings({ query }: { query: FindingQuery }) {
  const { lastEvent } = useChaos();
  const groups = useFetch(() => api.findingGroups(query), 10_000, [lastEvent]);
  const list = useFetch(() => api.findings(query), 10_000, [lastEvent]);
  const [view, setView] = useState<View>("grouped");
  const [q, setQ] = useState("");
  const [selected, setSelected] = useState<Record<string, string[]>>({});

  const all = list.data ?? [];
  const countBy = (f: (x: FindingSummary) => string | null) => {
    const m = new Map<string, number>();
    for (const x of all) {
      const k = f(x);
      if (k) m.set(k, (m.get(k) ?? 0) + 1);
    }
    return [...m.entries()].map(([value, count]) => ({ value, label: value, count }));
  };
  const pick = (g: string, v: string | null) =>
    !selected[g]?.length || (v !== null && selected[g].includes(v));
  const matches = (x: FindingSummary) =>
    pick("Invariant", x.invariant) &&
    pick("Campaign", x.campaign) &&
    pick("Worker", x.worker) &&
    (!q || `${x.message} ${x.invariant} ${x.subject} ${x.target}`.toLowerCase().includes(q.toLowerCase()));
  const shownGroups = (groups.data ?? []).filter((g) => matches(g.sample));
  const shownAll = all.filter(matches);
  const scope = [
    query.run ? `run ${query.run.slice(0, 13)}` : "",
    query.campaign ? `campaign ${query.campaign}` : "",
  ]
    .filter(Boolean)
    .join(", ");

  return (
    <>
      <PageTitle
        title="Findings"
        description={
          scope
            ? `Rules the ledger broke in ${scope}. A finding is a bug in the ledger or in the model; the trace says which.`
            : "Every rule the ledger broke across every campaign, grouped by signature: the same rule broken the same way is one row however often it recurred."
        }
      >
        {scope ? (
          <Button variant="outline" size="sm" asChild>
            <Link href="/findings/">All findings</Link>
          </Button>
        ) : null}
        <Button variant="outline" size="sm" onClick={() => (groups.reload(), list.reload())}>
          <RefreshCw />
        </Button>
      </PageTitle>
      <div className="grid gap-6 lg:grid-cols-[16rem_1fr]">
        <FilterRail
          groups={[
            { title: "Invariant", options: countBy((x) => x.invariant) },
            { title: "Campaign", options: countBy((x) => x.campaign) },
            { title: "Worker", options: countBy((x) => x.worker) },
          ]}
          selected={selected}
          onChange={(g, v) => setSelected({ ...selected, [g]: v })}
          onReset={() => setSelected({})}
        />
        <div className="grid content-start gap-3">
          <div className="flex flex-wrap items-center gap-2">
            <Tabs value={view} onValueChange={(v) => setView(v as View)}>
              <TabsList>
                <TabsTrigger value="grouped">By signature ({shownGroups.length})</TabsTrigger>
                <TabsTrigger value="all">Every finding ({shownAll.length})</TabsTrigger>
              </TabsList>
            </Tabs>
            <InputGroup className="ml-auto max-w-sm">
              <InputGroupAddon>
                <Search />
              </InputGroupAddon>
              <InputGroupInput
                placeholder="Search findings"
                value={q}
                onChange={(e) => setQ(e.target.value)}
              />
            </InputGroup>
          </div>
          {groups.error || list.error ? (
            <p className="text-destructive text-sm">{groups.error ?? list.error}</p>
          ) : null}
          {!groups.data || !list.data ? (
            <Skeleton className="h-40" />
          ) : view === "grouped" ? (
            <GroupsTable groups={shownGroups} />
          ) : (
            <FindingsTable findings={shownAll} />
          )}
        </div>
      </div>
    </>
  );
}

function Empty() {
  return (
    <div className="text-muted-foreground rounded-lg border border-dashed p-8 text-center text-sm">
      No findings. Every campaign run so far held every invariant it evaluated.
    </div>
  );
}

function GroupsTable({ groups }: { groups: FindingGroup[] }) {
  if (!groups.length) return <Empty />;
  return (
    <div className="overflow-x-auto rounded-xl border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Invariant</TableHead>
            <TableHead>Message</TableHead>
            <TableHead className="text-right">Findings</TableHead>
            <TableHead>Campaigns</TableHead>
            <TableHead>First</TableHead>
            <TableHead>Last</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {groups.map((g) => (
            <TableRow key={g.signature}>
              <TableCell>
                <Link
                  href={`/findings/view/?id=${g.sample.id}`}
                  className="font-mono text-xs font-medium hover:underline"
                >
                  {g.invariant}
                </Link>
                <div className="text-muted-foreground font-mono text-[11px]">{g.signature}</div>
              </TableCell>
              <TableCell className="max-w-xl">
                <Link
                  href={`/findings/view/?id=${g.sample.id}`}
                  className="line-clamp-2 text-xs hover:underline"
                >
                  {g.sample.message}
                </Link>
              </TableCell>
              <TableCell className="text-right tabular-nums">
                <Badge variant={g.count > 1 ? "destructive" : "outline"}>{g.count}</Badge>
                <div className="text-muted-foreground text-[11px]">
                  {g.runs.length} run{g.runs.length === 1 ? "" : "s"}
                </div>
              </TableCell>
              <TableCell className="text-muted-foreground text-xs">{g.campaigns.join(", ")}</TableCell>
              <TableCell className="text-muted-foreground text-xs" title={g.first}>
                {ago(g.first)}
              </TableCell>
              <TableCell className="text-muted-foreground text-xs" title={g.last}>
                {ago(g.last)}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}

export function FindingsTable({ findings }: { findings: FindingSummary[] }) {
  if (!findings.length) return <Empty />;
  return (
    <div className="overflow-x-auto rounded-xl border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Invariant</TableHead>
            <TableHead>Message</TableHead>
            <TableHead>Campaign</TableHead>
            <TableHead>Target</TableHead>
            <TableHead className="text-right">Trace</TableHead>
            <TableHead>Run</TableHead>
            <TableHead>Found</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {findings.map((f) => (
            <TableRow key={f.id}>
              <TableCell>
                <Link
                  href={`/findings/view/?id=${f.id}`}
                  className="font-mono text-xs font-medium hover:underline"
                >
                  {f.invariant}
                </Link>
                <div className="text-muted-foreground font-mono text-[11px]">{f.id.slice(0, 13)}</div>
              </TableCell>
              <TableCell className="max-w-xl">
                <Link href={`/findings/view/?id=${f.id}`} className="line-clamp-2 text-xs hover:underline">
                  {f.message}
                </Link>
              </TableCell>
              <TableCell className="text-muted-foreground text-xs">
                {f.campaign}
                <Badge variant="outline" className="ml-2 text-[10px]">
                  {f.worker}
                </Badge>
              </TableCell>
              <TableCell className="text-muted-foreground font-mono text-xs">{f.target}</TableCell>
              <TableCell className="text-right text-xs tabular-nums">
                {f.trace_len} step{f.trace_len === 1 ? "" : "s"}
                {f.shrunk ? <span className="text-muted-foreground"> · shrunk</span> : null}
              </TableCell>
              <TableCell>
                {f.run_id ? (
                  <Link href={`/runs/view/?id=${f.run_id}`} className="font-mono text-[11px] hover:underline">
                    {f.run_id.slice(0, 13)}
                  </Link>
                ) : (
                  <span className="text-muted-foreground text-xs">CLI</span>
                )}
              </TableCell>
              <TableCell className="text-muted-foreground text-xs" title={f.found_at}>
                {ago(f.found_at)}
              </TableCell>
            </TableRow>
          ))}
        </TableBody>
      </Table>
    </div>
  );
}
