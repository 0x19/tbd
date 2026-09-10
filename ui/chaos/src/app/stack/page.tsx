"use client";

import { useState } from "react";
import { Activity, Pause, Play, ShieldCheck, Wand2 } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { BehaviorDialog } from "@/components/behavior-dialog";
import { describeBehavior } from "@/components/instances-table";
import { DetailList, PageTitle, SectionTitle } from "@/components/kit";
import { Dot } from "@/components/status-badge";
import { useChaos } from "@/components/shell/providers";
import { api } from "@/lib/api/client";
import { describe, useStack } from "@/lib/api/hooks";
import type { Behavior, InstanceInfo } from "@/lib/api/schema";
import { ago, num } from "@/lib/format";

/**
 * The kit's Webhooks page: two summary cards, a filter-less list of
 * endpoints with health dots, and a detail panel for the selected one.
 */
export default function StackPage() {
  const stack = useStack();
  const { activity } = useChaos();
  const [selected, setSelected] = useState<string | null>(null);
  const [editing, setEditing] = useState<InstanceInfo | null>(null);
  const [busy, setBusy] = useState<string | null>(null);
  const instances = stack.data ?? [];
  const up = instances.filter((i) => i.running).length;
  const faulty = instances.filter((i) => i.behavior && i.behavior.type !== "healthy");
  const served = instances.reduce((n, i) => n + (i.requests?.total ?? 0), 0);
  const failedReq = instances.reduce((n, i) => n + (i.requests?.failed ?? 0), 0);
  const current = instances.find((i) => i.name === selected) ?? null;

  const act = async (name: string, f: () => Promise<InstanceInfo[]>, what: string) => {
    setBusy(name);
    try {
      stack.setData(await f());
      toast.success(`${what} ${name}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(null);
    }
  };

  const tone = (i: InstanceInfo) =>
    !i.running ? "off" : i.behavior && i.behavior.type !== "healthy" ? "warn" : "good";
  const word = (i: InstanceInfo) =>
    !i.running ? "Stopped" : i.behavior && i.behavior.type !== "healthy" ? "Degraded" : "Healthy";

  return (
    <>
      <PageTitle
        title="Stack"
        description="The topology this serve runs in-process. Stop, start and inject faults, and watch the protocol react."
      >
        <Badge variant="outline">
          {up}/{instances.length} running
        </Badge>
      </PageTitle>
      {stack.error ? <p className="text-sm text-destructive">{stack.error}</p> : null}

      <div className="grid gap-4 md:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <ShieldCheck className="size-4" /> Stack integrity
            </CardTitle>
          </CardHeader>
          <CardContent className="grid gap-3">
            <div className="flex items-center justify-between text-xs text-muted-foreground">
              <span>Running {up}</span>
              <span>Target {instances.length}</span>
            </div>
            <div className="flex gap-0.5">
              {instances.map((i) => (
                <span
                  key={i.name}
                  className={`h-3 flex-1 rounded-sm ${tone(i) === "good" ? "bg-emerald-500" : tone(i) === "warn" ? "bg-amber-500" : "bg-muted"}`}
                  title={`${i.name}: ${word(i)}`}
                />
              ))}
            </div>
            <div className="text-sm">
              {faulty.length
                ? `${faulty.length} instance${faulty.length > 1 ? "s" : ""} with an injected fault`
                : "every instance healthy"}
            </div>
            <div className="flex flex-wrap gap-2 border-t pt-3 text-xs">
              <span className="text-muted-foreground">Try</span>
              {instances
                .filter((i) => i.behavior !== null)
                .map((i) => (
                  <Button key={i.name} size="xs" variant="outline" onClick={() => setEditing(i)}>
                    fault {i.name}
                  </Button>
                ))}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <Activity className="size-4" /> Engine traffic
            </CardTitle>
            <CardDescription className="text-[11px] tracking-widest uppercase">
              Requests served · since start
            </CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3">
            <div className="flex items-baseline gap-3">
              <span className="text-3xl font-semibold tabular-nums">{num(served)}</span>
              {served ? (
                <Badge
                  variant="outline"
                  className={failedReq ? "text-destructive" : "text-emerald-600 dark:text-emerald-400"}
                >
                  {failedReq ? `${((failedReq / served) * 100).toFixed(1)}% failed` : "0 failed"}
                </Badge>
              ) : null}
            </div>
            <DetailList
              rows={instances
                .filter((i) => i.requests)
                .map((i) => ({
                  k: i.name,
                  v: (
                    <span className="tabular-nums">
                      {num(i.requests?.total)} served ·{" "}
                      <span className={i.requests?.failed ? "text-destructive" : ""}>
                        {num(i.requests?.failed)} failed
                      </span>
                    </span>
                  ),
                }))}
            />
          </CardContent>
        </Card>
      </div>

      <SectionTitle
        title="Instances"
        description="Dependencies start first; a stopped instance keeps its port."
      />
      {!stack.data ? (
        <Skeleton className="h-40" />
      ) : (
        <div className="divide-y rounded-xl border">
          {instances.map((i) => (
            <div key={i.name} className="flex flex-wrap items-center gap-x-6 gap-y-2 px-4 py-3 text-sm">
              <button type="button" className="min-w-40 text-left" onClick={() => setSelected(i.name)}>
                <div className="font-mono font-medium hover:underline">{i.name}</div>
                <div className="text-xs text-muted-foreground">
                  {i.kind}
                  {i.depends_on.length ? ` · needs ${i.depends_on.join(", ")}` : ""}
                </div>
              </button>
              <div className="font-mono text-xs text-muted-foreground">{i.addr}</div>
              <div className="ml-auto flex items-center gap-4">
                <Dot tone={tone(i)} label={word(i)} />
                {i.behavior && i.behavior.type !== "healthy" ? (
                  <Badge variant="outline">{describeBehavior(i.behavior)}</Badge>
                ) : null}
                {i.requests ? (
                  <span className="w-24 text-right text-xs tabular-nums text-muted-foreground">
                    {num(i.requests.total)} req
                  </span>
                ) : null}
                <div className="flex gap-1">
                  {i.running ? (
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={busy === i.name}
                      onClick={() => act(i.name, () => api.stackStop(i.name), "stopped")}
                    >
                      <Pause /> Stop
                    </Button>
                  ) : (
                    <Button
                      size="sm"
                      variant="outline"
                      disabled={busy === i.name}
                      onClick={() => act(i.name, () => api.stackStart(i.name), "started")}
                    >
                      <Play /> Start
                    </Button>
                  )}
                  <Button
                    size="sm"
                    variant="outline"
                    disabled={!i.running || i.behavior === null}
                    onClick={() => setEditing(i)}
                  >
                    <Wand2 /> Fault
                  </Button>
                </div>
              </div>
            </div>
          ))}
        </div>
      )}

      <Sheet open={current !== null} onOpenChange={(o) => !o && setSelected(null)}>
        <SheetContent className="w-full sm:max-w-md">
          {current ? (
            <>
              <SheetHeader>
                <SheetTitle className="font-mono">{current.name}</SheetTitle>
                <SheetDescription>
                  {current.kind} · {current.addr}
                </SheetDescription>
              </SheetHeader>
              <div className="grid gap-4 px-4">
                <div className="flex gap-2">
                  {current.running ? (
                    <Button
                      variant="outline"
                      onClick={() => act(current.name, () => api.stackStop(current.name), "stopped")}
                    >
                      <Pause /> Stop
                    </Button>
                  ) : (
                    <Button
                      variant="outline"
                      onClick={() => act(current.name, () => api.stackStart(current.name), "started")}
                    >
                      <Play /> Start
                    </Button>
                  )}
                  <Button
                    variant="outline"
                    disabled={!current.running || current.behavior === null}
                    onClick={() => setEditing(current)}
                  >
                    <Wand2 /> Fault
                  </Button>
                </div>
                <DetailList
                  rows={[
                    { k: "Status", v: <Dot tone={tone(current)} label={word(current)} /> },
                    {
                      k: "Behaviour",
                      v: current.behavior ? describeBehavior(current.behavior) : "no fault injection",
                    },
                    { k: "Depends on", v: current.depends_on.join(", ") || "–" },
                    { k: "Served", v: current.requests ? num(current.requests.total) : "–" },
                    { k: "Failed", v: current.requests ? num(current.requests.failed) : "–" },
                  ]}
                />
                <div>
                  <div className="mb-2 text-sm font-semibold">Activity</div>
                  <ul className="grid gap-2 text-xs text-muted-foreground">
                    {activity
                      .filter((a) => a.event.type === "stack_changed")
                      .slice(0, 6)
                      .map((a) => (
                        <li key={a.at} className="flex justify-between gap-2">
                          <span>
                            stack changed ·{" "}
                            {a.event.type === "stack_changed"
                              ? `${a.event.instances.filter((i) => i.running).length}/${a.event.instances.length} running`
                              : ""}
                          </span>
                          <span>{ago(new Date(a.at).toISOString())}</span>
                        </li>
                      ))}
                    {!activity.some((a) => a.event.type === "stack_changed") ? (
                      <li>No changes in this session.</li>
                    ) : null}
                  </ul>
                </div>
              </div>
            </>
          ) : null}
        </SheetContent>
      </Sheet>

      {editing ? (
        <BehaviorDialog
          key={editing.name}
          open
          onOpenChange={(o) => !o && setEditing(null)}
          instance={editing.name}
          current={editing.behavior}
          onApply={async (b: Behavior) => {
            stack.setData(await api.stackBehavior(editing.name, b));
            toast.success(`${editing.name}: ${describeBehavior(b)}`);
          }}
        />
      ) : null}
    </>
  );
}
