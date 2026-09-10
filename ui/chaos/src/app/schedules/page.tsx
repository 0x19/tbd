"use client";

import { CalendarClock, MoreHorizontal, Play, Plus } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { toast } from "sonner";

import { useChaos } from "@/app/providers";
import { PageTitle } from "@/components/kit";
import { QueuePanel } from "@/components/queue-panel";
import { ScheduleDialog, type SchedulePreset } from "@/components/schedule-dialog";
import { StatusBadge } from "@/components/status-badge";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Skeleton } from "@/components/ui/skeleton";
import { Switch } from "@/components/ui/switch";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Schedule } from "@/lib/api/schema";
import { ago, when } from "@/lib/format";
import { describeJob, type JobKind, presetLabel } from "@/lib/jobs";

export default function SchedulesPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <SchedulesFromUrl />
    </Suspense>
  );
}

/** `?new=scenario:<id>` (from a scenario's row menu), `?new=all`, `?new=load`, `?new=validate` open the dialog. */
function SchedulesFromUrl() {
  const params = useSearchParams();
  const raw = params.get("new");
  let preset: SchedulePreset | null = null;
  if (raw) {
    const [kind, scenario] = raw.split(":", 2) as [string, string | undefined];
    const k: JobKind =
      kind === "scenario"
        ? "scenario"
        : kind === "load"
          ? "load"
          : kind === "validate"
            ? "validate"
            : "all_scenarios";
    preset = { kind: k, ...(scenario ? { scenario } : {}) };
  }
  return <Schedules key={params.toString()} preset={preset} />;
}

/** The kit's list page: title with primary action, a table with a switch per row and a row menu. */
function Schedules({ preset }: { preset: SchedulePreset | null }) {
  const { lastEvent } = useChaos();
  const list = useFetch(() => api.schedules(), 15_000, [lastEvent]);
  const scenarios = useFetch(() => api.scenarios(), 0);
  const runs = useFetch(() => api.runs(300), 15_000, [lastEvent]);
  const [dialog, setDialog] = useState<{ open: boolean; schedule: Schedule | null }>({
    open: preset !== null,
    schedule: null,
  });

  const lastRun = (s: Schedule) => (runs.data ?? []).find((r) => r.schedule_id === s.id);

  const toggle = async (s: Schedule, enabled: boolean) => {
    try {
      await api.scheduleUpdate(s.id, { name: s.name, cron: s.cron, job: s.job, enabled });
      list.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };
  const runNow = async (s: Schedule) => {
    try {
      const items = await api.scheduleRun(s.id);
      toast.success(`queued ${items.length === 1 ? items[0]!.name : `${items.length} runs`}`);
    } catch (e) {
      toast.error(describe(e));
    }
  };
  const remove = async (s: Schedule) => {
    if (!confirm(`Delete schedule ${s.name}?`)) return;
    try {
      await api.scheduleDelete(s.id);
      list.reload();
    } catch (e) {
      toast.error(describe(e));
    }
  };

  const items = list.data ?? [];
  const enabled = items.filter((s) => s.enabled).length;

  return (
    <>
      <PageTitle
        title="Schedules"
        description="Cron jobs that queue scenarios, load or validate runs on their own. Times are UTC; a schedule due while its last job is still going is skipped."
      >
        <Button onClick={() => setDialog({ open: true, schedule: null })}>
          <Plus /> New schedule
        </Button>
      </PageTitle>

      <QueuePanel />

      {!list.data ? (
        <Skeleton className="h-40" />
      ) : !items.length ? (
        <div className="text-muted-foreground flex flex-col items-center gap-3 rounded-xl border border-dashed p-10 text-center text-sm">
          <CalendarClock className="size-8" />
          <p>No schedules yet. Create one here, or use “Schedule…” on a scenario.</p>
        </div>
      ) : (
        <div className="overflow-x-auto rounded-xl border">
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead className="w-12">On</TableHead>
                <TableHead>Schedule</TableHead>
                <TableHead>Runs</TableHead>
                <TableHead>Every</TableHead>
                <TableHead>Next</TableHead>
                <TableHead>Last run</TableHead>
                <TableHead className="text-right">Fired</TableHead>
                <TableHead className="w-24 text-right" />
              </TableRow>
            </TableHeader>
            <TableBody>
              {items.map((s) => {
                const last = lastRun(s);
                return (
                  <TableRow key={s.id} className={s.enabled ? "" : "text-muted-foreground"}>
                    <TableCell>
                      <Switch
                        checked={s.enabled}
                        onCheckedChange={(v) => toggle(s, v)}
                        aria-label={`${s.enabled ? "Pause" : "Enable"} ${s.name}`}
                      />
                    </TableCell>
                    <TableCell>
                      <button
                        type="button"
                        className="font-medium hover:underline"
                        onClick={() => setDialog({ open: true, schedule: s })}
                      >
                        {s.name}
                      </button>
                      <div className="text-muted-foreground font-mono text-[11px]">{s.id.slice(0, 13)}</div>
                    </TableCell>
                    <TableCell>{describeJob(s.job, scenarios.data ?? undefined)}</TableCell>
                    <TableCell>
                      <div>{presetLabel(s.cron)}</div>
                      <div className="text-muted-foreground font-mono text-[11px]">{s.cron}</div>
                    </TableCell>
                    <TableCell title={s.next_at ?? ""}>
                      {s.next_at ? (
                        <>
                          <div>{when(s.next_at)}</div>
                          <div className="text-muted-foreground text-xs">{until(s.next_at)}</div>
                        </>
                      ) : (
                        <Badge variant="outline">paused</Badge>
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
                      {s.fired}
                      {s.skipped ? (
                        <span
                          className="text-muted-foreground text-xs"
                          title={`skipped ${s.skipped} times, last ${when(s.last_skipped_at)}`}
                        >
                          {" "}
                          / {s.skipped} skipped
                        </span>
                      ) : null}
                    </TableCell>
                    <TableCell className="text-right">
                      <div className="flex justify-end gap-1">
                        <Button size="sm" variant="outline" onClick={() => runNow(s)}>
                          <Play /> Run now
                        </Button>
                        <DropdownMenu>
                          <DropdownMenuTrigger asChild>
                            <Button size="icon-sm" variant="ghost" aria-label="More">
                              <MoreHorizontal />
                            </Button>
                          </DropdownMenuTrigger>
                          <DropdownMenuContent align="end">
                            <DropdownMenuItem onClick={() => setDialog({ open: true, schedule: s })}>
                              Edit
                            </DropdownMenuItem>
                            <DropdownMenuItem asChild>
                              <Link href="/runs/">Runs</Link>
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
            </TableBody>
          </Table>
          <div className="text-muted-foreground border-t px-4 py-2 text-xs">
            {enabled} of {items.length} enabled. Stored in {`‘schedules.json’`} next to the run records; the
            queue itself does not survive a restart.
          </div>
        </div>
      )}

      {dialog.open ? (
        <ScheduleDialog
          key={dialog.schedule?.id ?? "new"}
          open={dialog.open}
          onOpenChange={(open) => setDialog((d) => ({ ...d, open }))}
          schedule={dialog.schedule}
          preset={dialog.schedule ? null : preset}
          scenarios={scenarios.data ?? []}
          onSaved={() => list.reload()}
        />
      ) : null}
    </>
  );
}

function until(iso: string): string {
  const d = (new Date(iso).getTime() - Date.now()) / 1000;
  if (Number.isNaN(d)) return "";
  if (d <= 0) return "due now";
  if (d < 60) return `in ${Math.round(d)} s`;
  if (d < 3600) return `in ${Math.round(d / 60)} min`;
  if (d < 86400) return `in ${(d / 3600).toFixed(1)} h`;
  return `in ${Math.round(d / 86400)} d`;
}
