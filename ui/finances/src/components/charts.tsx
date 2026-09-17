"use client";

// The money charts, in the kit's style: a big number with an uppercase caption,
// the kit's --chart-* palette, tooltips through ChartContainer.
import {
  Bar,
  BarChart,
  CartesianGrid,
  Cell,
  Line,
  LineChart,
  XAxis,
  type XAxisTickContentProps,
  YAxis,
} from "recharts";

import { type ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart";
import { money, monthShort } from "@/lib/format";
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

const flowConfig = {
  in: { label: "In", color: "var(--chart-2)" },
  spent: { label: "Spent", color: "var(--chart-1)" },
  other: { label: "Transfers & capital", color: "var(--chart-4)" },
} satisfies ChartConfig;

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
  onSelect,
  height = 280,
}: {
  months: MonthTotals[];
  currency: string;
  selected?: string;
  onSelect?: (ym: string) => void;
  height?: number;
}) {
  const data = months.map((m, i) => ({
    month: tickLabel(m.month, i === 0),
    ym: m.month,
    in: chartValue(m.in),
    spent: chartValue(m.spent),
    other: chartValue(m.out - m.spent),
  }));
  const dim = (ym: string) => (selected && ym !== selected ? 0.35 : 1);
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
            const active = ym === selected;
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
