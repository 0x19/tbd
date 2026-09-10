"use client";

import { CartesianGrid, Line, LineChart, XAxis, YAxis } from "recharts";
import { ChartContainer, ChartTooltip, ChartTooltipContent, type ChartConfig } from "@/components/ui/chart";
import type { LoadSnapshot } from "@/lib/api/schema";

const config = {
  rps: { label: "req/s", color: "var(--chart-2)" },
  p50: { label: "p50 ms", color: "var(--chart-3)" },
  p99: { label: "p99 ms", color: "var(--chart-5)" },
  errors: { label: "error %", color: "var(--destructive)" },
} satisfies ChartConfig;

/**
 * Per-second samples of a run: throughput on the left axis, latency on the
 * right, error rate as a third line. Same shape whether the run is live or
 * loaded from its record.
 */
export function LoadChart({ samples, height = 220 }: { samples: LoadSnapshot[]; height?: number }) {
  // Each sample is cumulative since load start; show the delta per interval
  // for throughput so a stall is visible, and the cumulative percentiles.
  const data = samples.map((s, i) => {
    const prev = i > 0 ? samples[i - 1] : null;
    const dt = prev ? s.elapsed_s - prev.elapsed_s : s.elapsed_s;
    const dn = prev ? s.requests_total - prev.requests_total : s.requests_total;
    return {
      t: Number(s.elapsed_s.toFixed(1)),
      rps: dt > 0 ? Math.round(dn / dt) : Math.round(s.throughput_rps),
      p50: Number(s.latency.p50_ms.toFixed(2)),
      p99: Number(s.latency.p99_ms.toFixed(2)),
      errors: Number((s.error_rate * 100).toFixed(2)),
    };
  });
  if (!data.length) {
    return (
      <div className="flex h-40 items-center justify-center text-sm text-muted-foreground">
        no samples yet
      </div>
    );
  }
  return (
    <ChartContainer config={config} className="aspect-auto w-full" style={{ height }}>
      <LineChart data={data} margin={{ left: 0, right: 8, top: 8, bottom: 0 }}>
        <CartesianGrid vertical={false} />
        <XAxis dataKey="t" tickLine={false} axisLine={false} unit="s" minTickGap={24} />
        <YAxis yAxisId="left" tickLine={false} axisLine={false} width={44} />
        <YAxis yAxisId="right" orientation="right" tickLine={false} axisLine={false} width={44} unit="ms" />
        <ChartTooltip content={<ChartTooltipContent labelFormatter={(v) => `${v} s`} />} />
        <Line
          yAxisId="left"
          type="monotone"
          dataKey="rps"
          stroke="var(--color-rps)"
          dot={false}
          strokeWidth={2}
          isAnimationActive={false}
        />
        <Line
          yAxisId="right"
          type="monotone"
          dataKey="p50"
          stroke="var(--color-p50)"
          dot={false}
          isAnimationActive={false}
        />
        <Line
          yAxisId="right"
          type="monotone"
          dataKey="p99"
          stroke="var(--color-p99)"
          dot={false}
          isAnimationActive={false}
        />
        <Line
          yAxisId="left"
          type="monotone"
          dataKey="errors"
          stroke="var(--color-errors)"
          dot={false}
          isAnimationActive={false}
        />
      </LineChart>
    </ChartContainer>
  );
}
