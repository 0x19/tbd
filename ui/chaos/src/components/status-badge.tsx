import { Badge } from "@/components/ui/badge";
import type { RunStatus } from "@/lib/api/schema";

const STYLE: Record<RunStatus, string> = {
  running: "border-transparent bg-sky-500/15 text-sky-700 dark:text-sky-300",
  passed: "border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
  completed: "border-transparent bg-emerald-500/15 text-emerald-700 dark:text-emerald-300",
  failed: "border-transparent bg-red-500/15 text-red-700 dark:text-red-300",
  error: "border-transparent bg-amber-500/15 text-amber-700 dark:text-amber-300",
  cancelled: "border-transparent bg-muted text-muted-foreground",
};

export function StatusBadge({ status }: { status: RunStatus }) {
  return (
    <Badge variant="outline" className={STYLE[status]}>
      {status === "running" ? <span className="mr-1 size-1.5 animate-pulse rounded-full bg-current" /> : null}
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
