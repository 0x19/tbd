"use client";

import { useT } from "@/lib/i18n";
import type { LabStatus } from "@/lib/lab";
import { cn } from "@/lib/utils";

/** The colour says how settled a page is: argued, settled, replaced; running, measured, published. */
const TONE: Record<LabStatus, string> = {
  open: "text-amber-700 ring-amber-700/30 dark:text-amber-400 dark:ring-amber-400/30",
  decided: "text-emerald-700 ring-emerald-700/30 dark:text-emerald-400 dark:ring-emerald-400/30",
  superseded: "text-muted-foreground ring-border line-through",
  running: "text-amber-700 ring-amber-700/30 dark:text-amber-400 dark:ring-amber-400/30",
  measured: "text-sky-700 ring-sky-700/30 dark:text-sky-400 dark:ring-sky-400/30",
  published: "text-emerald-700 ring-emerald-700/30 dark:text-emerald-400 dark:ring-emerald-400/30",
};

/** A rubber stamp: mono, uppercase, one word, in the reader's language. */
export function StatusStamp({ status, className }: { status: LabStatus; className?: string }) {
  const t = useT();
  return (
    <span
      className={cn(
        "rounded-sm px-1.5 py-0.5 font-mono text-[10px] tracking-[0.16em] uppercase ring-1 ring-inset",
        TONE[status],
        className,
      )}
    >
      {t(`lab.status.${status}`)}
    </span>
  );
}
