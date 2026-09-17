"use client";

import { ChevronLeft, ChevronRight } from "lucide-react";

import { Button } from "@/components/ui/button";
import { monthLabel } from "@/lib/format";
import { cn } from "@/lib/utils";

/** ‹ Sep 2026 › over an ordered list of months, with "Latest" to jump back. */
export function MonthStepper({
  months,
  value,
  onChange,
  className,
}: {
  months: string[];
  value: string;
  onChange: (ym: string) => void;
  className?: string;
}) {
  const i = months.indexOf(value);
  const prev = i > 0 ? months[i - 1] : undefined;
  const next = i >= 0 && i < months.length - 1 ? months[i + 1] : undefined;
  const latest = months.at(-1);
  return (
    <div className={cn("flex items-center gap-1", className)}>
      <Button
        variant="outline"
        size="icon"
        className="size-8"
        disabled={!prev}
        onClick={() => prev && onChange(prev)}
        aria-label="Previous month"
      >
        <ChevronLeft />
      </Button>
      <span className="min-w-24 text-center text-sm font-medium tabular-nums">{monthLabel(value)}</span>
      <Button
        variant="outline"
        size="icon"
        className="size-8"
        disabled={!next}
        onClick={() => next && onChange(next)}
        aria-label="Next month"
      >
        <ChevronRight />
      </Button>
      {latest && latest !== value ? (
        <Button variant="ghost" size="sm" className="h-8 px-2 text-xs" onClick={() => onChange(latest)}>
          Latest
        </Button>
      ) : null}
    </div>
  );
}
