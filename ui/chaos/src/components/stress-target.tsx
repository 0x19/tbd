"use client";

import { Play } from "lucide-react";
import { useState } from "react";

import { useChaos } from "@/app/providers";
import { Button } from "@/components/ui/button";
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from "@/components/ui/dialog";
import { Input } from "@/components/ui/input";
import type { StressTarget } from "@/lib/api/schema";
import { stackInstances } from "@/lib/load";

/** Where a campaign or a replay sends its workers. */
export type StressWhere = "own" | "stack" | "deployed" | "url";

/**
 * The ledgers a campaign or a replay runs against, as the API's `targets`:
 * empty for the campaign's own `[stack]`, the serve stack's running ledgers by
 * address, the deployed ledger from `[targets]`, or a typed gRPC URL.
 */
export function stressTargets(
  where: StressWhere,
  url: string,
  overview: ReturnType<typeof useChaos>["overview"],
): StressTarget[] {
  switch (where) {
    case "own":
      return [];
    case "stack":
      return stackInstances(overview, "ledger").map((i) => ({
        name: i.name,
        http_url: `http://${i.addr}`,
        kind: "ledger",
      }));
    case "deployed":
      return [{ name: "deployed-ledger", http_url: overview?.config.targets.ledger ?? "", kind: "ledger" }];
    case "url":
      return [{ name: "ledger-url", http_url: url.trim(), kind: "ledger" }];
  }
}

/**
 * The dialog behind "Run against…" on a campaign and "Replay" on a finding:
 * one target picker, the same choices as the load form, plus the attempt count
 * for a replay (a race needs more than one).
 */
export function StressTargetDialog({
  open,
  onOpenChange,
  title,
  description,
  hasStack,
  attempts: withAttempts = false,
  action = "Run",
  onRun,
}: {
  open: boolean;
  onOpenChange: (open: boolean) => void;
  title: string;
  description: string;
  /** The campaign has a `[stack]` of its own (the default choice). */
  hasStack: boolean;
  /** Offer an attempt count (replays). */
  attempts?: boolean;
  action?: string;
  onRun: (targets: StressTarget[], attempts: number) => Promise<void>;
}) {
  const { overview } = useChaos();
  const running = stackInstances(overview, "ledger");
  const deployed = overview?.config.targets.ledger;
  const [where, setWhere] = useState<StressWhere>(hasStack ? "own" : running.length ? "stack" : "deployed");
  const [url, setUrl] = useState("");
  const [attempts, setAttempts] = useState("1");
  const [busy, setBusy] = useState(false);

  const problem =
    where === "own" && !hasStack
      ? "this campaign has no [stack]"
      : where === "stack" && !running.length
        ? "no ledger is running in the serve stack"
        : where === "deployed" && !deployed
          ? "no deployed ledger is configured ([targets] ledger)"
          : where === "url" && !url.trim()
            ? "give a gRPC URL"
            : null;

  const go = async () => {
    setBusy(true);
    try {
      await onRun(stressTargets(where, url, overview), Math.max(1, Number(attempts) || 1));
      onOpenChange(false);
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onOpenChange={onOpenChange}>
      <DialogContent className="sm:max-w-md">
        <DialogHeader>
          <DialogTitle>{title}</DialogTitle>
          <DialogDescription>{description}</DialogDescription>
        </DialogHeader>
        <div className="grid gap-3">
          <label className="grid gap-1.5 text-sm">
            <span className="text-muted-foreground text-xs">Ledger</span>
            <select
              aria-label="Ledger target"
              className="bg-background h-8 w-full rounded-lg border px-2 text-sm"
              value={where}
              onChange={(e) => setWhere(e.target.value as StressWhere)}
            >
              {hasStack ? (
                <option value="own">The campaign&apos;s own stack (booted for this run)</option>
              ) : null}
              <option value="stack">
                Serve stack ({running.map((i) => i.name).join(", ") || "no ledger running"})
              </option>
              <option value="deployed" disabled={!deployed}>
                Deployed ledger ({deployed ?? "no target configured"})
              </option>
              <option value="url">A gRPC URL</option>
            </select>
          </label>
          {where === "url" ? (
            <Input
              aria-label="Ledger URL"
              value={url}
              onChange={(e) => setUrl(e.target.value)}
              placeholder={deployed ?? "http://localhost:18080"}
              className="font-mono text-xs"
            />
          ) : null}
          {where !== "own" && hasStack ? (
            <p className="text-muted-foreground text-xs">
              Against explicit ledgers the campaign&apos;s timeline is skipped; the workers and invariants are
              the same.
            </p>
          ) : null}
          {withAttempts ? (
            <label className="grid gap-1.5 text-sm">
              <span className="text-muted-foreground text-xs">Attempts (a race needs more than one)</span>
              <Input
                aria-label="Attempts"
                type="number"
                min={1}
                max={100}
                value={attempts}
                onChange={(e) => setAttempts(e.target.value)}
                className="max-w-24"
              />
            </label>
          ) : null}
          {problem ? <p className="text-destructive text-xs">{problem}</p> : null}
        </div>
        <DialogFooter>
          <Button variant="outline" onClick={() => onOpenChange(false)}>
            Cancel
          </Button>
          <Button onClick={go} disabled={busy || !!problem}>
            <Play /> {action}
          </Button>
        </DialogFooter>
      </DialogContent>
    </Dialog>
  );
}
