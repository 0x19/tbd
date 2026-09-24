"use client";

// The money charts, in the kit's style: a big number with an uppercase caption,
// the kit's --chart-* palette, tooltips through ChartContainer.
import { useState } from "react";
import {
  Area,
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  ComposedChart,
  Line,
  LineChart,
  Pie,
  PieChart,
  XAxis,
  type XAxisTickContentProps,
  YAxis,
} from "recharts";

import { type ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart";
import { money, monthShort } from "@/lib/format";
import { useT } from "@/lib/i18n";
import { type CategoryTotal, chartValue, type MonthTotals } from "@/lib/summary";
import { cn } from "@/lib/utils";

export function ChartHeadline({ value, caption }: { value: React.ReactNode; caption: string }) {
  return (
    <div className="mb-2">
      <div className="text-3xl font-semibold tracking-tight tabular-nums">{value}</div>
      <div className="text-muted-foreground text-[11px] font-medium tracking-widest uppercase">{caption}</div>
    </div>
  );
}

export function Legend({ items }: { items: { label: string; color: string }[] }) {
  return (
    <div className="text-muted-foreground flex flex-wrap items-center gap-4 text-xs">
      {items.map((i) => (
        <span key={i.label} className="inline-flex items-center gap-1.5">
          <span className="size-2 rounded-sm" style={{ background: i.color }} />
          {i.label}
        </span>
      ))}
    </div>
  );
}

/** The three series, labelled in the page's language. */
function useFlowConfig() {
  const t = useT();
  return {
    in: { label: t("nav.chart.in"), color: "var(--chart-2)" },
    spent: { label: t("nav.chart.spent"), color: "var(--chart-1)" },
    other: { label: t("nav.chart.other"), color: "var(--chart-4)" },
  } satisfies ChartConfig;
}

function axisMoney(v: number): string {
  const a = Math.abs(v);
  if (a >= 1_000_000) return `${(v / 1_000_000).toFixed(1)}M`;
  if (a >= 1000) return `${Math.round(v / 1000)}k`;
  return String(Math.round(v));
}

/** Money in against money spent, month by month. Transfers and capital
 *  movements are the third, muted series so they are visible but not
 *  mistaken for spending. */
function tickLabel(ym: string, first: boolean): string {
  const [y, m] = ym.split("-");
  return m === "01" || first ? `${monthShort(ym)} '${y!.slice(2)}` : monthShort(ym);
}

/** The bars are the month picker: click one and it is the selected month, the
 *  others step back. */
export function MoneyFlowChart({
  months,
  currency,
  selected,
  range,
  onSelect,
  height = 280,
}: {
  months: MonthTotals[];
  currency: string;
  selected?: string;
  /** Months to keep lit; the rest dim. Overrides `selected` when given. */
  range?: string[];
  onSelect?: (ym: string) => void;
  height?: number;
}) {
  const flowConfig = useFlowConfig();
  const lit = range ? new Set(range) : selected ? new Set([selected]) : null;
  const data = months.map((m, i) => ({
    month: tickLabel(m.month, i === 0),
    ym: m.month,
    in: chartValue(m.in),
    spent: chartValue(m.spent),
    other: chartValue(m.out - m.spent),
  }));
  const dim = (ym: string) => (lit && !lit.has(ym) ? 0.35 : 1);
  const cells = (key: string) =>
    data.map((d) => (
      <Cell
        key={`${key}-${d.ym}`}
        fillOpacity={dim(d.ym)}
        cursor={onSelect ? "pointer" : undefined}
        onClick={onSelect ? () => onSelect(d.ym) : undefined}
      />
    ));
  return (
    <ChartContainer config={flowConfig} className="w-full" style={{ height }}>
      <BarChart
        accessibilityLayer
        data={data}
        barCategoryGap="28%"
        margin={{ top: 6, right: 0, bottom: 0, left: 0 }}
      >
        <CartesianGrid stroke="var(--border)" strokeDasharray="4 4" strokeOpacity={0.55} vertical={false} />
        <XAxis
          axisLine={false}
          dataKey="month"
          tickLine={false}
          tickMargin={10}
          tick={({ x, y, payload, index }: XAxisTickContentProps) => {
            const ym = data[index]?.ym;
            const active = ym != null && (lit ? lit.has(ym) : false);
            return (
              <text
                x={Number(x)}
                y={Number(y) + 12}
                textAnchor="middle"
                fontSize={12}
                fontWeight={active ? 600 : 400}
                fill={active ? "var(--foreground)" : "var(--muted-foreground)"}
                style={{ cursor: onSelect ? "pointer" : undefined }}
                onClick={onSelect && ym ? () => onSelect(ym) : undefined}
              >
                {String(payload.value)}
              </text>
            );
          }}
        />
        <YAxis
          axisLine={false}
          tickLine={false}
          tick={{ fontSize: 12 }}
          width={44}
          tickFormatter={axisMoney}
        />
        <ChartTooltip
          content={
            <ChartTooltipContent
              formatter={(value, name) => (
                <div className="flex w-full items-center justify-between gap-4">
                  <span className="text-muted-foreground">
                    {flowConfig[name as keyof typeof flowConfig]?.label ?? name}
                  </span>
                  <span className="font-mono font-medium tabular-nums">
                    {money(Math.round(Number(value) * 100), currency)}
                  </span>
                </div>
              )}
            />
          }
          cursor={{ fill: "color-mix(in oklch, var(--muted) 45%, transparent)" }}
        />
        <Bar dataKey="in" fill="var(--color-in)" radius={[4, 4, 0, 0]}>
          {cells("in")}
        </Bar>
        <Bar dataKey="spent" fill="var(--color-spent)" radius={[4, 4, 0, 0]}>
          {cells("spent")}
        </Bar>
        <Bar dataKey="other" fill="var(--color-other)" radius={[4, 4, 0, 0]}>
          {cells("other")}
        </Bar>
      </BarChart>
    </ChartContainer>
  );
}

/** Horizontal bars: where the money went, biggest first. */
export function CategoryBars({
  items,
  currency,
  total,
  onPick,
  className,
}: {
  items: CategoryTotal[];
  currency: string;
  /** What 100% is, minor units. */
  total: bigint;
  onPick?: (category_id: string) => void;
  className?: string;
}) {
  const max = items.reduce((m, i) => (i.total < m ? i.total : m), 0n);
  return (
    <div className={cn("space-y-2.5", className)}>
      {items.map((c) => {
        const v = -c.total;
        const pct = total > 0n ? Number((v * 10000n) / total) / 100 : 0;
        const width = max < 0n ? Number((v * 1000n) / -max) / 10 : 0;
        return (
          <button
            key={c.category_id}
            type="button"
            onClick={onPick ? () => onPick(c.category_id) : undefined}
            className={cn(
              "group grid w-full grid-cols-[minmax(0,1fr)_auto] items-center gap-x-3 text-left",
              onPick && "cursor-pointer",
            )}
          >
            <div className="flex items-baseline justify-between gap-3 text-sm">
              <span className={cn("truncate", c.category_id === "none" && "text-muted-foreground italic")}>
                {c.category}
              </span>
              <span className="text-muted-foreground shrink-0 text-xs tabular-nums">
                {pct.toFixed(1)}% · {c.count}
              </span>
            </div>
            <span className="w-28 shrink-0 text-right font-mono text-sm tabular-nums">
              {money(v.toString(), currency)}
            </span>
            <div className="bg-muted col-span-2 h-1.5 overflow-hidden rounded-full">
              <div
                className={cn(
                  "h-full rounded-full transition-[width]",
                  c.kind === "tax"
                    ? "bg-[var(--chart-3)]"
                    : c.kind === "transfer" || c.kind === "capital"
                      ? "bg-[var(--chart-4)]"
                      : "bg-[var(--chart-1)]",
                )}
                style={{ width: `${width}%` }}
              />
            </div>
          </button>
        );
      })}
    </div>
  );
}

const sparkConfig = { value: { label: "value", color: "var(--chart-1)" } } satisfies ChartConfig;

/** A small line, no axes, for a KPI card. */
export function SparkLine({ values, height = 56 }: { values: number[]; height?: number }) {
  const data = values.map((value, i) => ({ i, value }));
  return (
    <ChartContainer config={sparkConfig} className="w-full" style={{ height }}>
      <LineChart accessibilityLayer data={data} margin={{ top: 4, right: 0, bottom: 0, left: 0 }}>
        <XAxis dataKey="i" hide />
        <YAxis hide domain={["dataMin", "dataMax"]} />
        <Line
          dataKey="value"
          dot={false}
          isAnimationActive={false}
          stroke="var(--color-value)"
          strokeWidth={1.75}
          type="monotone"
        />
      </LineChart>
    </ChartContainer>
  );
}

/** This year as a line over last year as a faint area, month by month: the
 *  kit's "total revenue" idiom. Missing months leave gaps rather than zeros. */
export function YearOverYearChart({
  data,
  currency,
  height = 280,
}: {
  data: { month: string; thisYear: number | null; lastYear: number | null }[];
  currency: string;
  height?: number;
}) {
  const t = useT();
  const config = {
    thisYear: { label: t("overview.yoy.this"), color: "var(--chart-1)" },
    lastYear: { label: t("overview.yoy.last"), color: "var(--chart-4)" },
  } satisfies ChartConfig;
  const rows = data.map((d) => ({ ...d, label: monthShort(d.month) }));
  return (
    <ChartContainer config={config} className="w-full" style={{ height }}>
      <ComposedChart accessibilityLayer data={rows} margin={{ top: 6, right: 0, bottom: 0, left: 0 }}>
        <CartesianGrid stroke="var(--border)" strokeDasharray="4 4" strokeOpacity={0.55} vertical={false} />
        <XAxis axisLine={false} dataKey="label" tickLine={false} tickMargin={10} tick={{ fontSize: 12 }} />
        <YAxis
          axisLine={false}
          tickLine={false}
          tick={{ fontSize: 12 }}
          width={44}
          tickFormatter={axisMoney}
        />
        <ChartTooltip
          content={
            <ChartTooltipContent
              formatter={(value, name) => (
                <div className="flex w-full items-center justify-between gap-4">
                  <span className="text-muted-foreground">
                    {config[name as keyof typeof config]?.label ?? name}
                  </span>
                  <span className="font-mono font-medium tabular-nums">
                    {money(Math.round(Number(value) * 100), currency)}
                  </span>
                </div>
              )}
            />
          }
        />
        <Area
          dataKey="lastYear"
          type="linear"
          stroke="var(--color-lastYear)"
          strokeWidth={1.5}
          strokeOpacity={0.6}
          fill="var(--color-lastYear)"
          fillOpacity={0.1}
          connectNulls={false}
          isAnimationActive={false}
        />
        <Line
          dataKey="thisYear"
          type="linear"
          stroke="var(--color-thisYear)"
          strokeWidth={2}
          dot={{ r: 2.5, strokeWidth: 0, fill: "var(--color-thisYear)" }}
          activeDot={{ r: 4 }}
          connectNulls={false}
          isAnimationActive={false}
        />
      </ComposedChart>
    </ChartContainer>
  );
}

export type Slice = { id: string; label: string; value: number; color: string; pct: number };

/** A donut with the biggest slice in the middle and a legend beside it that
 *  lights the slice it names: the kit's "categories" idiom. */
export function CategoryDonut({
  slices,
  currency,
  onPick,
}: {
  slices: Slice[];
  currency: string;
  onPick?: (id: string) => void;
}) {
  const [active, setActive] = useState<number | null>(null);
  const config = Object.fromEntries(
    slices.map((s) => [s.id, { label: s.label, color: s.color }]),
  ) satisfies ChartConfig;
  const top = slices[0];
  return (
    <div className="flex flex-1 flex-col items-center gap-4 sm:flex-row sm:gap-6">
      <div className="relative size-[170px] shrink-0 sm:size-[190px]">
        <ChartContainer config={config} className="h-full w-full">
          <PieChart>
            <Pie
              data={slices}
              dataKey="value"
              nameKey="label"
              innerRadius="62%"
              outerRadius="92%"
              paddingAngle={2}
              strokeWidth={0}
              isAnimationActive={false}
              onMouseEnter={(_: unknown, index: number) => setActive(index)}
              onMouseLeave={() => setActive(null)}
            >
              {slices.map((s, i) => (
                <Cell
                  key={s.id}
                  fill={s.color}
                  fillOpacity={active === null || active === i ? 1 : 0.35}
                  cursor={onPick ? "pointer" : undefined}
                  onClick={onPick ? () => onPick(s.id) : undefined}
                />
              ))}
            </Pie>
          </PieChart>
        </ChartContainer>
        {top ? (
          <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
            <span className="text-lg font-semibold tabular-nums">
              {(active !== null ? slices[active] : top)?.pct.toFixed(0)}%
            </span>
            <span className="text-muted-foreground max-w-[110px] truncate text-[10px]">
              {(active !== null ? slices[active] : top)?.label}
            </span>
          </div>
        ) : null}
      </div>
      <div className="flex min-w-0 flex-1 flex-col gap-2">
        {slices.map((s, i) => (
          <button
            key={s.id}
            type="button"
            className={cn(
              "flex w-full cursor-pointer items-center gap-2.5 rounded-md text-left transition-opacity",
              active !== null && active !== i && "opacity-50",
            )}
            onPointerEnter={() => setActive(i)}
            onPointerLeave={() => setActive(null)}
            onFocus={() => setActive(i)}
            onBlur={() => setActive(null)}
            onClick={onPick ? () => onPick(s.id) : undefined}
          >
            <span className="h-4 w-1 shrink-0 rounded-sm" style={{ backgroundColor: s.color }} />
            <span className="text-muted-foreground min-w-0 flex-1 truncate text-xs">{s.label}</span>
            <span className="shrink-0 font-mono text-xs tabular-nums">
              {money(Math.round(s.value * 100), currency)}
            </span>
            <span className="w-10 shrink-0 text-right text-xs font-semibold tabular-nums">
              {s.pct.toFixed(0)}%
            </span>
          </button>
        ))}
      </div>
    </div>
  );
}
