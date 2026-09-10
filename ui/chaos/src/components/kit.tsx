"use client";

// The Admin Kit's recurring building blocks, reproduced from its pages:
// a page title with subtitle and actions, a KPI strip (one card, columns
// separated by dividers, "previous" line, big value, coloured delta), a
// stat row with dividers, a stage progress bar, a filter rail, a heat grid,
// a sparkline, and a key/value detail list.
import { ArrowDownRight, ArrowUpRight, ChevronDown, type LucideIcon } from "lucide-react";
import { Button } from "@/components/ui/button";
import { Checkbox } from "@/components/ui/checkbox";
import { Collapsible, CollapsibleContent, CollapsibleTrigger } from "@/components/ui/collapsible";
import { cn } from "@/lib/utils";

export function PageTitle({
  title,
  description,
  children,
  back,
}: {
  title: React.ReactNode;
  description?: React.ReactNode;
  children?: React.ReactNode;
  back?: React.ReactNode;
}) {
  return (
    <div className="flex flex-wrap items-start justify-between gap-4">
      <div className="flex items-start gap-3">
        {back}
        <div>
          <h1 className="text-3xl font-semibold tracking-tight">{title}</h1>
          {description ? <p className="mt-1 text-sm text-muted-foreground">{description}</p> : null}
        </div>
      </div>
      {children ? <div className="flex flex-wrap items-center gap-2">{children}</div> : null}
    </div>
  );
}

export function SectionTitle({
  title,
  description,
  children,
}: {
  title: string;
  description?: string;
  children?: React.ReactNode;
}) {
  return (
    <div className="flex items-end justify-between gap-4">
      <div>
        <h2 className="text-base font-semibold">{title}</h2>
        {description ? <p className="text-sm text-muted-foreground">{description}</p> : null}
      </div>
      {children}
    </div>
  );
}

export type Kpi = {
  icon: LucideIcon;
  label: string;
  previous?: string;
  value: React.ReactNode;
  delta?: { value: number; label: string; goodWhen?: "up" | "down" };
  hint?: string;
};

/** One card, N columns with vertical dividers. */
export function KpiStrip({ items }: { items: Kpi[] }) {
  return (
    <div className="rounded-xl border bg-card text-card-foreground shadow-xs">
      <div className="grid grid-cols-1 divide-y sm:grid-cols-2 sm:divide-x sm:divide-y-0 xl:grid-cols-4">
        {items.map((k) => {
          const good = k.delta
            ? k.delta.goodWhen === "down"
              ? k.delta.value <= 0
              : k.delta.value >= 0
            : true;
          return (
            <div key={k.label} className="flex flex-col gap-2 px-6 py-5">
              <div className="flex items-center gap-2 text-sm text-muted-foreground">
                <k.icon className="size-4" />
                <span className="text-foreground">{k.label}</span>
              </div>
              <div className="text-xs text-muted-foreground">{k.previous ?? " "}</div>
              <div className="text-3xl font-semibold tracking-tight tabular-nums">{k.value}</div>
              {k.delta ? (
                <div
                  className={cn(
                    "flex items-center gap-1 text-xs",
                    good ? "text-emerald-600 dark:text-emerald-400" : "text-red-600 dark:text-red-400",
                  )}
                >
                  {k.delta.value >= 0 ? (
                    <ArrowUpRight className="size-3.5" />
                  ) : (
                    <ArrowDownRight className="size-3.5" />
                  )}
                  <span>
                    {k.delta.value >= 0 ? "+" : ""}
                    {k.delta.value.toFixed(1)}%
                  </span>
                  <span className="text-muted-foreground">{k.delta.label}</span>
                </div>
              ) : k.hint ? (
                <div className="text-xs text-muted-foreground">{k.hint}</div>
              ) : null}
            </div>
          );
        })}
      </div>
    </div>
  );
}

/** Label over big value, columns separated by dividers ("Delivery window"). */
export function StatRow({
  items,
}: {
  items: { label: string; value: React.ReactNode; sub?: string; tone?: "good" | "bad" }[];
}) {
  return (
    <div className="grid grid-cols-2 divide-x md:grid-cols-4">
      {items.map((s) => (
        <div key={s.label} className="px-4 py-2 first:pl-0">
          <div className="text-xs text-muted-foreground">{s.label}</div>
          <div
            className={cn(
              "mt-1 text-2xl font-semibold tracking-tight tabular-nums",
              s.tone === "good" && "text-emerald-600 dark:text-emerald-400",
              s.tone === "bad" && "text-red-600 dark:text-red-400",
            )}
          >
            {s.value}
          </div>
          {s.sub ? <div className="text-xs text-muted-foreground">{s.sub}</div> : null}
        </div>
      ))}
    </div>
  );
}

/** Stage progress, like the order lifecycle strip: dot, name, filled bar. */
export function StageBar({ stages, current }: { stages: string[]; current: number }) {
  return (
    <div className="grid gap-3" style={{ gridTemplateColumns: `repeat(${stages.length}, minmax(0, 1fr))` }}>
      {stages.map((s, i) => {
        const state = i < current ? "done" : i === current ? "active" : "todo";
        return (
          <div key={s} className="grid gap-2">
            <div className="flex items-center gap-2 text-sm">
              <span
                className={cn(
                  "size-2 rounded-full",
                  state === "todo" ? "bg-muted-foreground/30" : "bg-foreground",
                )}
              />
              <span
                className={cn(
                  state === "todo" && "text-muted-foreground",
                  state === "active" && "font-medium",
                )}
              >
                {s}
              </span>
            </div>
            <div className="h-1 overflow-hidden rounded-full bg-muted">
              <div
                className={cn(
                  "h-full bg-foreground transition-all",
                  state === "done" ? "w-full" : state === "active" ? "w-1/2 animate-pulse" : "w-0",
                )}
              />
            </div>
          </div>
        );
      })}
    </div>
  );
}

export type FilterGroup = {
  title: string;
  options: { value: string; label: string; count?: number }[];
};

/** The "Filters / Reset" rail with collapsible checkbox groups. */
export function FilterRail({
  groups,
  selected,
  onChange,
  onReset,
}: {
  groups: FilterGroup[];
  selected: Record<string, string[]>;
  onChange: (group: string, values: string[]) => void;
  onReset: () => void;
}) {
  return (
    <div className="grid content-start gap-4 border-r pr-4">
      <div className="flex items-center justify-between">
        <span className="text-sm font-semibold">Filters</span>
        <Button variant="outline" size="xs" onClick={onReset}>
          Reset
        </Button>
      </div>
      {groups.map((g) => (
        <Collapsible key={g.title} defaultOpen className="group/filter">
          <CollapsibleTrigger className="flex w-full items-center gap-2 text-sm font-medium">
            <ChevronDown className="size-4 transition-transform group-data-[panel-closed]/filter:-rotate-90" />
            {g.title}
          </CollapsibleTrigger>
          <CollapsibleContent>
            <div className="mt-2 grid gap-1 rounded-lg border p-2">
              {g.options.map((o) => {
                const on = (selected[g.title] ?? []).includes(o.value);
                return (
                  <label
                    key={o.value}
                    className="flex cursor-pointer items-center gap-2 rounded-md px-2 py-1.5 text-sm hover:bg-muted"
                  >
                    <Checkbox
                      checked={on}
                      onCheckedChange={(v) => {
                        const cur = selected[g.title] ?? [];
                        onChange(g.title, v ? [...cur, o.value] : cur.filter((x) => x !== o.value));
                      }}
                    />
                    <span className="flex-1">{o.label}</span>
                    {o.count !== undefined ? (
                      <span className="rounded-md bg-muted px-1.5 text-xs tabular-nums text-muted-foreground">
                        {o.count}
                      </span>
                    ) : null}
                  </label>
                );
              })}
            </div>
          </CollapsibleContent>
        </Collapsible>
      ))}
    </div>
  );
}

/** Darker is worse: values scaled to the max of the grid. */
export function HeatGrid({
  rows,
  cols,
  cell,
  format,
}: {
  rows: string[];
  cols: string[];
  cell: (row: string, col: string) => number | null;
  format: (v: number) => string;
}) {
  const values = rows.flatMap((r) => cols.map((c) => cell(r, c))).filter((v): v is number => v !== null);
  const max = Math.max(1e-9, ...values);
  return (
    <div className="overflow-x-auto">
      <table className="w-full text-xs">
        <thead>
          <tr>
            <th />
            {cols.map((c) => (
              <th key={c} className="pb-2 text-center font-normal text-muted-foreground">
                {c}
              </th>
            ))}
          </tr>
        </thead>
        <tbody>
          {rows.map((r) => (
            <tr key={r}>
              <td className="pr-3 text-right font-mono text-muted-foreground">{r}</td>
              {cols.map((c) => {
                const v = cell(r, c);
                const t = v === null ? 0 : v / max;
                const dark = t > 0.55;
                return (
                  <td key={c} className="p-0">
                    <div
                      className={cn(
                        "flex h-11 items-center justify-center border border-background text-xs tabular-nums",
                        dark ? "text-background" : "text-foreground",
                      )}
                      style={{
                        background:
                          v === null
                            ? "var(--muted)"
                            : `color-mix(in oklch, var(--foreground) ${Math.round(10 + t * 85)}%, var(--muted))`,
                      }}
                    >
                      {v === null ? "–" : format(v)}
                    </div>
                  </td>
                );
              })}
            </tr>
          ))}
        </tbody>
      </table>
    </div>
  );
}

/** Tiny bar sparkline, like the "attempt spread" column. */
export function Sparkline({ values, className }: { values: number[]; className?: string }) {
  const max = Math.max(1e-9, ...values);
  const shown =
    values.length > 24 ? values.filter((_, i) => i % Math.ceil(values.length / 24) === 0) : values;
  return (
    <div className={cn("flex h-5 items-end gap-px", className)} aria-hidden>
      {shown.map((v, i) => (
        <span
          key={i}
          className="w-1 rounded-sm bg-foreground/80"
          style={{ height: `${Math.max(8, (v / max) * 100)}%` }}
        />
      ))}
    </div>
  );
}

/** Key on the left, value on the right, hairline between rows. */
export function DetailList({ rows }: { rows: { k: string; v: React.ReactNode }[] }) {
  return (
    <dl className="divide-y">
      {rows.map((r) => (
        <div key={r.k} className="flex items-center justify-between gap-4 py-2.5 text-sm">
          <dt className="text-muted-foreground">{r.k}</dt>
          <dd className="truncate text-right">{r.v}</dd>
        </div>
      ))}
    </dl>
  );
}

/** Level chips in the kit's Events & Logs style. */
export function LevelChip({ level }: { level: "pass" | "fail" | "info" | "warn" | "error" | "ok" }) {
  const cls =
    level === "fail" || level === "error"
      ? "bg-destructive text-white"
      : level === "warn"
        ? "bg-foreground text-background"
        : level === "pass" || level === "ok"
          ? "bg-emerald-600 text-white"
          : "bg-foreground text-background";
  return (
    <span
      className={cn(
        "inline-flex rounded-md px-1.5 py-0.5 text-[10px] font-semibold uppercase tracking-wide",
        cls,
      )}
    >
      {level}
    </span>
  );
}
