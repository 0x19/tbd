import { Badge } from "@/components/ui/badge";
import type { RunStatus } from "@/lib/api/schema";
import { cn } from "@/lib/utils";

/** Status chips the way the kit does them: solid black for running/info,
 *  green for good, red for bad, muted for cancelled. */
const STYLE: Record<RunStatus, string> = {
  running: "border-transparent bg-foreground text-background",
  passed:
    "border-transparent bg-emerald-600/12 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
  completed:
    "border-transparent bg-emerald-600/12 text-emerald-700 dark:bg-emerald-500/15 dark:text-emerald-300",
  failed: "border-transparent bg-destructive/12 text-destructive",
  error: "border-transparent bg-amber-500/15 text-amber-700 dark:text-amber-300",
  cancelled: "border-transparent bg-muted text-muted-foreground",
};

export function StatusBadge({ status, className }: { status: RunStatus; className?: string }) {
  return (
    <Badge variant="outline" className={cn("gap-1.5 capitalize", STYLE[status], className)}>
      {status === "running" ? <span className="size-1.5 animate-pulse rounded-full bg-current" /> : null}
      {status}
    </Badge>
  );
}

export function BoolBadge({ ok, yes = "ok", no = "failed" }: { ok: boolean; yes?: string; no?: string }) {
  return (
    <Badge variant="outline" className={ok ? STYLE.passed : STYLE.failed}>
      {ok ? yes : no}
    </Badge>
  );
}

/** Green / amber / grey dot with a word, like "● Healthy" in the kit lists. */
export function Dot({ tone, label }: { tone: "good" | "warn" | "bad" | "off"; label: string }) {
  const color =
    tone === "good"
      ? "bg-emerald-500"
      : tone === "warn"
        ? "bg-amber-500"
        : tone === "bad"
          ? "bg-destructive"
          : "bg-muted-foreground/40";
  return (
    <span className="inline-flex items-center gap-1.5 text-sm">
      <span className={cn("size-2 rounded-full", color)} />
      {label}
    </span>
  );
}
