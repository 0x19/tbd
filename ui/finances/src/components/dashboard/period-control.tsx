"use client";

// The period the whole overview follows: month, quarter or year, which one,
// and ‹ › to step. Months come from the data, so the picker never offers a
// period nothing was booked in; "Latest" jumps back to the newest.
import { ChevronLeft, ChevronRight } from "lucide-react";
import { useMemo } from "react";

import { Button } from "@/components/ui/button";
import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from "@/components/ui/select";
import { Tabs, TabsList, TabsTrigger } from "@/components/ui/tabs";
import { periodLabel } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { type Period, type PeriodKind, periodOf } from "@/lib/summary";
import { cn } from "@/lib/utils";

const KINDS: PeriodKind[] = ["month", "quarter", "year"];

export function PeriodControl({
  period,
  months,
  onChange,
  className,
}: {
  period: Period;
  /** Every month with data, oldest first. */
  months: string[];
  onChange: (period: Period) => void;
  className?: string;
}) {
  const t = useT();

  // The periods of the current kind that hold at least one month with data,
  // oldest first, keyed by their first month.
  const options = useMemo(() => {
    const seen = new Map<string, Period>();
    for (const m of months) {
      const p = periodOf(period.kind, m);
      if (!seen.has(p.start)) seen.set(p.start, p);
    }
    return [...seen.values()].sort((a, b) => a.start.localeCompare(b.start));
  }, [months, period.kind]);

  const index = options.findIndex((p) => p.start === period.start);
  const latest = options.at(-1);
  const prev = index > 0 ? options[index - 1] : undefined;
  const next = index >= 0 && index < options.length - 1 ? options[index + 1] : undefined;

  const setKind = (kind: PeriodKind) => onChange(periodOf(kind, period.months.at(-1) ?? period.start));

  return (
    <div className={cn("flex flex-wrap items-center gap-2", className)}>
      <Tabs value={period.kind} onValueChange={(v) => setKind(v as PeriodKind)}>
        <TabsList>
          {KINDS.map((k) => (
            <TabsTrigger key={k} value={k} className="text-xs">
              {t(`overview.period.${k}`)}
            </TabsTrigger>
          ))}
        </TabsList>
      </Tabs>
      <div className="flex items-center gap-1">
        <Button
          variant="outline"
          size="icon"
          className="size-8"
          disabled={!prev}
          onClick={() => prev && onChange(prev)}
          aria-label={t("overview.period.previous")}
        >
          <ChevronLeft />
        </Button>
        <Select
          value={index >= 0 ? period.start : ""}
          onValueChange={(start) => onChange(periodOf(period.kind, start))}
        >
          <SelectTrigger className="h-8 min-w-32" aria-label={t("overview.period.pick")}>
            <SelectValue placeholder={periodLabel(period.kind, period.start)} />
          </SelectTrigger>
          <SelectContent>
            {[...options].reverse().map((p) => (
              <SelectItem key={p.start} value={p.start}>
                {periodLabel(p.kind, p.start)}
              </SelectItem>
            ))}
          </SelectContent>
        </Select>
        <Button
          variant="outline"
          size="icon"
          className="size-8"
          disabled={!next}
          onClick={() => next && onChange(next)}
          aria-label={t("overview.period.next")}
        >
          <ChevronRight />
        </Button>
        {latest && latest.start !== period.start ? (
          <Button variant="ghost" size="sm" className="h-8 px-2 text-xs" onClick={() => onChange(latest)}>
            {t("overview.period.latest")}
          </Button>
        ) : null}
      </div>
    </div>
  );
}
