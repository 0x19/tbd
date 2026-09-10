"use client";

import { useState } from "react";
import { Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { ErrorNote, PageHeader } from "@/components/page-header";
import { RunsTable } from "@/components/runs-table";
import { useChaos } from "@/components/shell/providers";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";

const FILTERS = ["all", "scenario", "load", "validate"] as const;

export default function RunsPage() {
  const { lastEvent } = useChaos();
  const runs = useFetch(() => api.runs(200), 5000, [lastEvent]);
  const [filter, setFilter] = useState<(typeof FILTERS)[number]>("all");

  const remove = async (id: string) => {
    try {
      await api.runDelete(id);
      runs.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  const shown = (runs.data ?? []).filter((r) => filter === "all" || r.kind === filter);
  return (
    <>
      <PageHeader
        title="Runs"
        description="Every scenario, load and validate run this serve recorded. Records are JSON files; the newest is first."
      >
        <div className="flex gap-1">
          {FILTERS.map((f) => (
            <Button
              key={f}
              size="sm"
              variant={filter === f ? "default" : "outline"}
              onClick={() => setFilter(f)}
            >
              {f}
            </Button>
          ))}
        </div>
      </PageHeader>
      <ErrorNote message={runs.error} />
      {!runs.data ? (
        <Skeleton className="h-40" />
      ) : (
        <RunsTable
          runs={shown}
          actions={(r) =>
            r.status === "running" ? null : (
              <Button size="icon-sm" variant="ghost" aria-label="delete run" onClick={() => remove(r.id)}>
                <Trash2 />
              </Button>
            )
          }
        />
      )}
    </>
  );
}
