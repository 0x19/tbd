"use client";

import { useState } from "react";
import { Pause, Play, Wand2 } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { BehaviorDialog } from "@/components/behavior-dialog";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { Behavior, InstanceInfo } from "@/lib/api/schema";
import { num } from "@/lib/format";

export function describeBehavior(b: Behavior | null): string {
  if (!b) return "–";
  switch (b.type) {
    case "healthy":
      return "healthy";
    case "hang":
      return "hang";
    case "slow":
      return `slow ${b.latency}${b.jitter ? ` ± ${b.jitter}` : ""}`;
    case "error":
      return `error ${b.kind} ${Math.round((b.rate ?? 1) * 100)}%`;
    case "delayed_failure":
      return `healthy ${b.healthy_for}, then ${describeBehavior(b.then)}`;
  }
}

/** The stack with stop, start and behaviour controls per instance. */
export function InstancesTable({
  instances,
  onChange,
  compact = false,
}: {
  instances: InstanceInfo[];
  onChange?: (i: InstanceInfo[]) => void;
  compact?: boolean;
}) {
  const [editing, setEditing] = useState<InstanceInfo | null>(null);
  const [busy, setBusy] = useState<string | null>(null);

  const act = async (name: string, f: () => Promise<InstanceInfo[]>, what: string) => {
    setBusy(name);
    try {
      const next = await f();
      onChange?.(next);
      toast.success(`${what} ${name}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(null);
    }
  };

  return (
    <>
      <div className="overflow-x-auto rounded-lg border">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead>Instance</TableHead>
              <TableHead>Kind</TableHead>
              <TableHead>Address</TableHead>
              <TableHead>State</TableHead>
              <TableHead>Behaviour</TableHead>
              {!compact ? <TableHead className="text-right">Served</TableHead> : null}
              {!compact ? <TableHead className="text-right">Failed</TableHead> : null}
              {onChange ? <TableHead className="text-right">Actions</TableHead> : null}
            </TableRow>
          </TableHeader>
          <TableBody>
            {instances.map((i) => (
              <TableRow key={i.name}>
                <TableCell className="font-medium">
                  {i.name}
                  {i.depends_on.length ? (
                    <span className="ml-2 text-xs text-muted-foreground">→ {i.depends_on.join(", ")}</span>
                  ) : null}
                </TableCell>
                <TableCell className="text-muted-foreground">{i.kind}</TableCell>
                <TableCell className="font-mono text-xs">{i.addr}</TableCell>
                <TableCell>
                  <Badge
                    variant="outline"
                    className={i.running ? "text-emerald-600 dark:text-emerald-400" : "text-muted-foreground"}
                  >
                    {i.running ? "running" : "stopped"}
                  </Badge>
                </TableCell>
                <TableCell>
                  {i.behavior ? (
                    <span
                      className={i.behavior.type === "healthy" ? "" : "text-amber-600 dark:text-amber-400"}
                    >
                      {describeBehavior(i.behavior)}
                    </span>
                  ) : (
                    <span className="text-muted-foreground">{i.running ? "no fault injection" : "–"}</span>
                  )}
                </TableCell>
                {!compact ? (
                  <TableCell className="text-right tabular-nums">
                    {i.requests ? num(i.requests.total) : "–"}
                  </TableCell>
                ) : null}
                {!compact ? (
                  <TableCell
                    className={`text-right tabular-nums ${i.requests?.failed ? "text-destructive" : ""}`}
                  >
                    {i.requests ? num(i.requests.failed) : "–"}
                  </TableCell>
                ) : null}
                {onChange ? (
                  <TableCell className="text-right">
                    <div className="flex justify-end gap-1">
                      {i.running ? (
                        <Button
                          size="sm"
                          variant="outline"
                          disabled={busy === i.name}
                          onClick={() => act(i.name, () => api.stackStop(i.name), "stopped")}
                        >
                          <Pause /> stop
                        </Button>
                      ) : (
                        <Button
                          size="sm"
                          variant="outline"
                          disabled={busy === i.name}
                          onClick={() => act(i.name, () => api.stackStart(i.name), "started")}
                        >
                          <Play /> start
                        </Button>
                      )}
                      <Button
                        size="sm"
                        variant="outline"
                        disabled={!i.running || i.behavior === null}
                        title={i.behavior === null ? "no fault injection on this service" : "set behaviour"}
                        onClick={() => setEditing(i)}
                      >
                        <Wand2 /> fault
                      </Button>
                    </div>
                  </TableCell>
                ) : null}
              </TableRow>
            ))}
          </TableBody>
        </Table>
      </div>
      {editing ? (
        <BehaviorDialog
          key={editing.name}
          open
          onOpenChange={(o) => !o && setEditing(null)}
          instance={editing.name}
          current={editing.behavior}
          onApply={async (b) => {
            const next = await api.stackBehavior(editing.name, b);
            onChange?.(next);
            toast.success(`${editing.name}: ${describeBehavior(b)}`);
          }}
        />
      ) : null}
    </>
  );
}
