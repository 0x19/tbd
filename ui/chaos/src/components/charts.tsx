"use client";

// Monochrome charts in the kit's style: a big number with an uppercase
// caption above, greys for series, a legend on the right.
import {
  Area,
  AreaChart,
  Bar,
  BarChart,
  CartesianGrid,
  ComposedChart,
  Line,
  LineChart,
  XAxis,
  type XAxisTickContentProps,
  YAxis,
} from "recharts";

import { type ChartConfig, ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart";
import type { LoadSnapshot } from "@/lib/api/schema";

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
        <span key={i.label} className="flex items-center gap-1.5">
          <span className="size-2 rounded-full" style={{ background: i.color }} />
          {i.label}
        </span>
      ))}
    </div>
  );
}

function perSecond(samples: LoadSnapshot[]) {
  // The last snapshot of a phase arrives a few milliseconds after the last
  // tick; its per-second delta would read as a collapse. Drop it.
  const last = samples.at(-1);
  const beforeLast = samples.at(-2);
  const trimmed =
    last && beforeLast && last.elapsed_s - beforeLast.elapsed_s < 0.5 ? samples.slice(0, -1) : samples;
  return trimmed.map((s, i) => {
    const prev = i > 0 ? trimmed[i - 1] : null;
    const dt = prev ? s.elapsed_s - prev.elapsed_s : s.elapsed_s;
    const dn = prev ? s.requests_total - prev.requests_total : s.requests_total;
    return {
      t: Number(s.elapsed_s.toFixed(1)),
      rps: dt > 0 ? Math.round(dn / dt) : Math.round(s.throughput_rps),
      p50: s.latency.p50_ms,
      p99: s.latency.p99_ms,
      err: s.error_rate * 100,
    };
  });
}

const twoRuns = {
  cur: { label: "This run", color: "var(--foreground)" },
  prev: { label: "Previous run", color: "var(--chart-2)" },
} satisfies ChartConfig;

/** Throughput of a run over a previous run of the same scenario. */
export function CompareChart({
  current,
  previous,
  height = 240,
}: {
  current: LoadSnapshot[];
  previous: LoadSnapshot[];
  height?: number;
}) {
  const a = perSecond(current);
  const b = perSecond(previous);
  const n = Math.max(a.length, b.length);
  const data = Array.from({ length: n }, (_, i) => ({
    t: a[i]?.t ?? b[i]?.t ?? i,
    cur: a[i]?.rps ?? null,
    prev: b[i]?.rps ?? null,
  }));
  if (!n)
    return (
      <div className="text-muted-foreground flex h-40 items-center justify-center text-sm">
        No load samples yet.
      </div>
    );
  return (
    <ChartContainer config={twoRuns} className="aspect-auto w-full" style={{ height }}>
      <AreaChart data={data} margin={{ left: 0, right: 8, top: 8, bottom: 0 }}>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis dataKey="t" tickLine={false} axisLine={false} unit="s" minTickGap={24} />
        <YAxis tickLine={false} axisLine={false} width={44} />
        <ChartTooltip content={<ChartTooltipContent labelFormatter={(v) => `${v} s`} />} />
        <Area
          type="monotone"
          dataKey="prev"
          stroke="var(--color-prev)"
          fill="var(--color-prev)"
          fillOpacity={0.15}
          dot={false}
          isAnimationActive={false}
          connectNulls
        />
        <Line
          type="monotone"
          dataKey="cur"
          stroke="var(--color-cur)"
          strokeWidth={2}
          dot={false}
          isAnimationActive={false}
          connectNulls
        />
      </AreaChart>
    </ChartContainer>
  );
}

const live = {
  rps: { label: "req/s", color: "var(--foreground)" },
  p50: { label: "p50 ms", color: "var(--chart-2)" },
  p99: { label: "p99 ms", color: "var(--chart-4)" },
  err: { label: "error %", color: "var(--destructive)" },
} satisfies ChartConfig;

/** One run's samples: req/s on the left axis, latency on the right, error % on the left. */
export function RunChart({ samples, height = 260 }: { samples: LoadSnapshot[]; height?: number }) {
  const data = perSecond(samples);
  if (!data.length)
    return (
      <div className="text-muted-foreground flex h-40 items-center justify-center text-sm">
        No samples yet.
      </div>
    );
  return (
    <ChartContainer config={live} className="aspect-auto w-full" style={{ height }}>
      <LineChart data={data} margin={{ left: 0, right: 8, top: 8, bottom: 0 }}>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis dataKey="t" tickLine={false} axisLine={false} unit="s" minTickGap={24} />
        <YAxis yAxisId="l" tickLine={false} axisLine={false} width={44} />
        <YAxis yAxisId="r" orientation="right" tickLine={false} axisLine={false} width={48} unit="ms" />
        <ChartTooltip content={<ChartTooltipContent labelFormatter={(v) => `${v} s`} />} />
        <Line
          yAxisId="l"
          type="monotone"
          dataKey="rps"
          stroke="var(--color-rps)"
          strokeWidth={2}
          dot={false}
          isAnimationActive={false}
        />
        <Line
          yAxisId="r"
          type="monotone"
          dataKey="p50"
          stroke="var(--color-p50)"
          dot={false}
          isAnimationActive={false}
        />
        <Line
          yAxisId="r"
          type="monotone"
          dataKey="p99"
          stroke="var(--color-p99)"
          dot={false}
          isAnimationActive={false}
        />
        <Line
          yAxisId="l"
          type="monotone"
          dataKey="err"
          stroke="var(--color-err)"
          dot={false}
          isAnimationActive={false}
        />
      </LineChart>
    </ChartContainer>
  );
}

const latency = {
  p50: { label: "p50", color: "var(--foreground)" },
  p90: { label: "p90", color: "var(--chart-3)" },
  p99: { label: "p99", color: "var(--chart-1)" },
} satisfies ChartConfig;

/** Horizontal stacked bars per row, like "Revenue by Channel". */
export function LatencyBars({
  rows,
  height,
}: {
  rows: { name: string; p50: number; p90: number; p99: number }[];
  height?: number;
}) {
  if (!rows.length)
    return (
      <div className="text-muted-foreground flex h-40 items-center justify-center text-sm">
        No finished scenario runs yet.
      </div>
    );
  const data = rows.map((r) => ({
    name: r.name,
    p50: r.p50,
    p90: Math.max(0, r.p90 - r.p50),
    p99: Math.max(0, r.p99 - r.p90),
    _p90: r.p90,
    _p99: r.p99,
  }));
  return (
    <ChartContainer
      config={latency}
      className="aspect-auto w-full"
      style={{ height: height ?? 40 + rows.length * 40 }}
    >
      <BarChart data={data} layout="vertical" margin={{ left: 0, right: 16, top: 0, bottom: 0 }} barSize={16}>
        <CartesianGrid horizontal={false} strokeDasharray="3 3" />
        <XAxis type="number" tickLine={false} axisLine={false} unit="ms" />
        <YAxis
          type="category"
          dataKey="name"
          tickLine={false}
          axisLine={false}
          width={110}
          tick={{ fontSize: 11 }}
        />
        <ChartTooltip
          content={
            <ChartTooltipContent
              formatter={(_value, name, item) => {
                const p = item.payload as {
                  p50: number;
                  _p90: number;
                  _p99: number;
                };
                const v = name === "p50" ? p.p50 : name === "p90" ? p._p90 : p._p99;
                return (
                  <span className="flex w-full justify-between gap-4">
                    <span className="text-muted-foreground">{String(name)}</span>
                    <span className="font-mono tabular-nums">{v.toFixed(2)} ms</span>
                  </span>
                );
              }}
            />
          }
        />
        <Bar dataKey="p50" stackId="a" fill="var(--color-p50)" isAnimationActive={false} />
        <Bar dataKey="p90" stackId="a" fill="var(--color-p90)" isAnimationActive={false} />
        <Bar
          dataKey="p99"
          stackId="a"
          fill="var(--color-p99)"
          radius={[0, 4, 4, 0]}
          isAnimationActive={false}
        />
      </BarChart>
    </ChartContainer>
  );
}

const loadTrend = {
  ok: { label: "req/s ok", color: "var(--foreground)" },
  err: { label: "req/s failed", color: "var(--destructive)" },
  p99: { label: "p99", color: "var(--chart-2)" },
} satisfies ChartConfig;

/**
 * Throughput of the last load runs as stacked bars, the failed share in red so
 * an error rate is visible without the tooltip, and p99 as a line; oldest
 * first. Runs with errors get a marker under the label.
 */
export function LoadTrend({
  rows,
  height = 200,
}: {
  rows: { id: string; label: string; rps: number; p99: number; errors: number; failed: boolean }[];
  height?: number;
}) {
  if (!rows.length)
    return (
      <div className="text-muted-foreground flex h-40 items-center justify-center text-sm">
        No finished load runs yet. Start one from Load.
      </div>
    );
  const data = rows.map((r) => {
    const err = (r.rps * r.errors) / 100;
    return { ...r, err, ok: Math.max(0, r.rps - err) };
  });
  return (
    <ChartContainer config={loadTrend} className="aspect-auto w-full" style={{ height }}>
      <ComposedChart data={data} margin={{ left: 0, right: 8, top: 8, bottom: 0 }} barSize={18}>
        <CartesianGrid vertical={false} strokeDasharray="3 3" />
        <XAxis
          dataKey="label"
          tickLine={false}
          axisLine={false}
          tick={(props: XAxisTickContentProps) => {
            const row = data[props.index];
            const bad = !!row && (row.errors > 0 || row.failed);
            return (
              <text
                x={props.x}
                y={Number(props.y) + 10}
                textAnchor="middle"
                fontSize={11}
                className="fill-muted-foreground"
              >
                {String(props.payload.value)}
                {bad ? (
                  <tspan className="fill-destructive" dx={3}>
                    ●
                  </tspan>
                ) : null}
              </text>
            );
          }}
        />
        <YAxis yAxisId="rps" tickLine={false} axisLine={false} width={40} tick={{ fontSize: 11 }} />
        <YAxis
          yAxisId="p99"
          orientation="right"
          tickLine={false}
          axisLine={false}
          width={48}
          unit="ms"
          tick={{ fontSize: 11 }}
        />
        <ChartTooltip
          content={
            <ChartTooltipContent
              formatter={(value, name, item) => {
                const p = item.payload as { errors: number; rps: number; failed: boolean };
                const v = Number(value);
                if (name === "err") {
                  return (
                    <span className="flex w-full justify-between gap-4">
                      <span className="text-muted-foreground">errors</span>
                      <span className={`font-mono tabular-nums ${p.errors ? "text-destructive" : ""}`}>
                        {p.errors.toFixed(2)}%{p.failed ? " · run failed" : ""}
                      </span>
                    </span>
                  );
                }
                return (
                  <span className="flex w-full justify-between gap-4">
                    <span className="text-muted-foreground">{name === "p99" ? "p99" : "req/s"}</span>
                    <span className="font-mono tabular-nums">
                      {name === "p99" ? `${v.toFixed(1)} ms` : `${Math.round(p.rps)} req/s`}
                    </span>
                  </span>
                );
              }}
            />
          }
        />
        <Bar yAxisId="rps" dataKey="ok" stackId="rps" fill="var(--color-ok)" isAnimationActive={false} />
        <Bar
          yAxisId="rps"
          dataKey="err"
          stackId="rps"
          fill="var(--color-err)"
          radius={[3, 3, 0, 0]}
          isAnimationActive={false}
        />
        <Line
          yAxisId="p99"
          dataKey="p99"
          stroke="var(--color-p99)"
          strokeWidth={2}
          dot={{ r: 2 }}
          isAnimationActive={false}
        />
      </ComposedChart>
    </ChartContainer>
  );
}

/** Pass/fail history as a row of squares, oldest first; each links to its run. */
export function PassStrip({ items }: { items: { id: string; ok: boolean; title: string }[] }) {
  if (!items.length)
    return (
      <div className="text-muted-foreground flex h-24 items-center justify-center text-sm">
        No validate runs yet. Run one from Validate.
      </div>
    );
  return (
    <div className="flex flex-wrap gap-1">
      {items.map((i) => (
        <a
          key={i.id}
          href={`/runs/view/?id=${i.id}`}
          title={i.title}
          className={`size-4 rounded-[3px] ${i.ok ? "bg-emerald-500" : "bg-destructive"} opacity-90 hover:opacity-100`}
        />
      ))}
    </div>
  );
}
