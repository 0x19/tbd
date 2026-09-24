"use client";

// The small building blocks the overview's widgets share: a card with the
// kit's icon-in-a-box header and a "view all" link, a mini chart card with a
// value, a delta and room for a sparkline, and the empty/loading states so
// every widget degrades the same way.
import { ArrowDownRight, ArrowUpRight, ArrowUpRight as Open, type LucideIcon } from "lucide-react";
import Link from "next/link";

import { Button } from "@/components/ui/button";
import { Skeleton } from "@/components/ui/skeleton";
import { cn } from "@/lib/utils";

export function WidgetCard({
  icon: Icon,
  title,
  description,
  href,
  hrefLabel,
  actions,
  className,
  children,
}: {
  icon: LucideIcon;
  title: string;
  description?: React.ReactNode;
  href?: string;
  hrefLabel?: string;
  actions?: React.ReactNode;
  className?: string;
  children: React.ReactNode;
}) {
  return (
    <div
      className={cn(
        "bg-card text-card-foreground flex flex-col gap-4 rounded-xl border p-4 shadow-xs sm:p-5",
        className,
      )}
    >
      <div className="flex items-start justify-between gap-3">
        <div className="flex min-w-0 items-start gap-3">
          <div className="bg-muted/40 flex size-8 shrink-0 items-center justify-center rounded-md border">
            <Icon className="text-muted-foreground size-4" aria-hidden="true" />
          </div>
          <div className="min-w-0">
            <p className="text-sm font-medium">{title}</p>
            {description ? <p className="text-muted-foreground text-xs">{description}</p> : null}
          </div>
        </div>
        <div className="flex shrink-0 items-center gap-2">
          {actions}
          {href ? (
            <Button asChild variant="ghost" size="sm" className="h-8 px-2 text-xs">
              <Link href={href}>
                {hrefLabel} <Open className="size-3.5" />
              </Link>
            </Button>
          ) : null}
        </div>
      </div>
      {children}
    </div>
  );
}

/** Coloured ↑↓ percentage, green when the direction is the good one. */
export function Delta({
  value,
  label,
  goodWhen = "up",
  className,
}: {
  value: number | undefined;
  label: string;
  goodWhen?: "up" | "down";
  className?: string;
}) {
  if (value == null) return <span className={cn("text-muted-foreground text-xs", className)}>{label}</span>;
  const good = goodWhen === "down" ? value <= 0 : value >= 0;
  return (
    <span
      className={cn(
        "flex items-center gap-1 text-xs",
        good ? "text-emerald-600 dark:text-emerald-400" : "text-red-600 dark:text-red-400",
        className,
      )}
    >
      {value >= 0 ? <ArrowUpRight className="size-3.5" /> : <ArrowDownRight className="size-3.5" />}
      <span className="font-medium tabular-nums">
        {value >= 0 ? "+" : ""}
        {value.toFixed(1)}%
      </span>
      <span className="text-muted-foreground">{label}</span>
    </span>
  );
}

/** Icon, caption, big value, delta, then whatever chart the caller draws. */
export function MiniChartCard({
  icon: Icon,
  title,
  value,
  delta,
  deltaLabel,
  goodWhen,
  hint,
  loading,
  children,
}: {
  icon: LucideIcon;
  title: string;
  value: React.ReactNode;
  delta?: number;
  deltaLabel: string;
  goodWhen?: "up" | "down";
  hint?: string;
  loading?: boolean;
  children?: React.ReactNode;
}) {
  return (
    <div className="bg-card text-card-foreground flex flex-col gap-3 rounded-xl border p-4 shadow-xs sm:p-5">
      <div className="flex items-start gap-3">
        <div className="bg-muted/40 flex size-8 shrink-0 items-center justify-center rounded-md border">
          <Icon className="text-muted-foreground size-4" aria-hidden="true" />
        </div>
        <div className="min-w-0 space-y-1">
          <p className="text-muted-foreground text-xs font-medium">{title}</p>
          {loading ? (
            <Skeleton className="h-7 w-32" />
          ) : (
            <div className="flex flex-wrap items-center gap-x-3 gap-y-1">
              <span className="text-lg font-semibold tabular-nums">{value}</span>
              <Delta value={delta} label={deltaLabel} goodWhen={goodWhen} />
            </div>
          )}
          {hint ? <p className="text-muted-foreground text-[11px]">{hint}</p> : null}
        </div>
      </div>
      <div className="min-h-14 w-full flex-1">
        {loading ? <Skeleton className="h-14 w-full" /> : children}
      </div>
    </div>
  );
}

export function Empty({ children }: { children: React.ReactNode }) {
  return <p className="text-muted-foreground py-6 text-center text-sm">{children}</p>;
}

export function Rows({ n = 4 }: { n?: number }) {
  return (
    <div className="space-y-2.5">
      {Array.from({ length: n }, (_, i) => (
        <Skeleton key={i} className="h-5 w-full" />
      ))}
    </div>
  );
}
