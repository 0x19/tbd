"use client";

import { Radio, RefreshCw, Search, Trash2 } from "lucide-react";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { toast } from "sonner";

import { useChaos } from "@/app/providers";
import { FilterRail, PageTitle } from "@/components/kit";
import { RunsTable } from "@/components/runs-table";
import { Button } from "@/components/ui/button";
import { InputGroup, InputGroupAddon, InputGroupInput } from "@/components/ui/input-group";
import { Skeleton } from "@/components/ui/skeleton";
import { TableCell, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { RunSummary } from "@/lib/api/schema";
import { num } from "@/lib/format";

export default function RunsPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <RunsFromUrl />
    </Suspense>
  );
}

/** Deep links from the sidebar and the scenario menu pick the initial filters;
 *  keying on the query string resets them on navigation without an effect. */
function RunsFromUrl() {
  const params = useSearchParams();
  const kind = params.get("kind");
  const scenario = params.get("scenario");
  return (
    <Runs
      key={params.toString()}
      initial={{
        ...(kind ? { Kind: [kind] } : {}),
        ...(scenario ? { Scenario: [scenario] } : {}),
      }}
    />
  );
}

/** The kit's Events & Logs page: filter rail, toolbar with search and Live, table with a totals row. */
function Runs({ initial }: { initial: Record<string, string[]> }) {
  const { lastEvent, connected } = useChaos();
  const runs = useFetch(() => api.runs(500), 5000, [lastEvent]);
  const [q, setQ] = useState("");
  const [selected, setSelected] = useState<Record<string, string[]>>(initial);

  const all = runs.data ?? [];
  const countBy = (f: (r: RunSummary) => string | null) => {
    const m = new Map<string, number>();
    for (const r of all) {
      const k = f(r);
      if (k) m.set(k, (m.get(k) ?? 0) + 1);
    }
    return [...m.entries()].map(([value, count]) => ({
      value,
      label: value,
      count,
    }));
  };

  const shown = all.filter((r) => {
    const pick = (g: string, v: string | null) =>
      !selected[g]?.length || (v !== null && selected[g].includes(v));
    return (
      pick("Kind", r.kind) &&
      pick("Status", r.status) &&
      pick("Scenario", r.scenario_id) &&
      (!q || `${r.name} ${r.id} ${r.scenario_id ?? ""}`.toLowerCase().includes(q.toLowerCase()))
    );
  });
  const totalReq = shown.reduce((n, r) => n + (r.requests_total ?? 0), 0);
  const totalFailed = shown.filter((r) => r.status === "failed" || r.status === "error").length;

  const remove = async (id: string) => {
    try {
      await api.runDelete(id);
      runs.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  return (
    <>
      <PageTitle
        title="Runs"
        description="Every scenario, load and validate run this serve recorded. Records are JSON files; newest first."
      />
      <div className="grid gap-6 lg:grid-cols-[16rem_1fr]">
        <FilterRail
          groups={[
            { title: "Kind", options: countBy((r) => r.kind) },
            { title: "Status", options: countBy((r) => r.status) },
            {
              title: "Scenario",
              options: countBy((r) => r.scenario_id),
            },
          ]}
          selected={selected}
          onChange={(g, v) => setSelected({ ...selected, [g]: v })}
          onReset={() => setSelected({})}
        />
        <div className="grid content-start gap-3">
          <div className="flex flex-wrap items-center gap-2">
            <InputGroup className="max-w-sm">
              <InputGroupAddon>
                <Search />
              </InputGroupAddon>
              <InputGroupInput placeholder="Search runs" value={q} onChange={(e) => setQ(e.target.value)} />
            </InputGroup>
            <div className="ml-auto flex items-center gap-1">
              <Button variant="outline" size="sm" onClick={runs.reload}>
                <RefreshCw />
              </Button>
              <Button
                variant={connected ? "default" : "outline"}
                size="sm"
                title={connected ? "live feed connected" : "live feed off"}
              >
                <Radio /> Live
              </Button>
            </div>
          </div>
          {!runs.data ? (
            <Skeleton className="h-40" />
          ) : (
            <div className="rounded-xl border">
              <RunsTable
                runs={shown}
                actions={(r) =>
                  r.status === "running" ? null : (
                    <Button
                      size="icon-sm"
                      variant="ghost"
                      aria-label="delete run"
                      onClick={() => remove(r.id)}
                    >
                      <Trash2 />
                    </Button>
                  )
                }
                footer={
                  shown.length ? (
                    <TableRow className="bg-muted/40 font-medium">
                      <TableCell>Total</TableCell>
                      <TableCell colSpan={2} className="text-muted-foreground">
                        {shown.length} runs, {totalFailed} failed
                      </TableCell>
                      <TableCell className="text-right tabular-nums">{num(totalReq)}</TableCell>
                      <TableCell colSpan={7} />
                    </TableRow>
                  ) : null
                }
              />
            </div>
          )}
        </div>
      </div>
    </>
  );
}
