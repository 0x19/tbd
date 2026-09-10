"use client";

import { Activity, AlertTriangle, BarChart3, ListChecks, RefreshCw, Server } from "lucide-react";
import Link from "next/link";
import { useEffect, useState } from "react";

import { useChaos } from "@/app/providers";
import { ChartHeadline, CompareChart, LatencyBars, Legend } from "@/components/charts";
import { describeBehavior } from "@/components/instances-table";
import { DetailList, KpiStrip, PageTitle } from "@/components/kit";
import { RunsTable } from "@/components/runs-table";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { RunRecord, RunSummary } from "@/lib/api/schema";
import { ago, ms, num, pct } from "@/lib/format";
import { delta, failed, inWindow, latestPerScenario } from "@/lib/runs";

export default function OverviewPage() {
  const { overview, error, reload, lastEvent, activity } = useChaos();
  const runs = useFetch(() => api.runs(300), 15_000, [lastEvent]);
  const all = runs.data ?? [];

  const today = inWindow(all, 24);
  const yesterday = inWindow(all, 24, 24);
  const failToday = today.filter(failed).length;
  const failYesterday = yesterday.filter(failed).length;
  const rateToday = today.length ? (failToday / today.length) * 100 : 0;
  const rateYesterday = yesterday.length ? (failYesterday / yesterday.length) * 100 : 0;

  const stack = overview?.stack ?? [];
  const up = stack.filter((i) => i.running).length;
  const faulty = stack.filter((i) => i.behavior && i.behavior.type !== "healthy").length;
  const lv = overview?.last_validate ?? null;
  const latest = latestPerScenario(all);
  const lastScenario = latest[0] ?? null;
  const previousOfSame = lastScenario
    ? (all.find(
        (r) =>
          r.kind === "scenario" &&
          r.scenario_id === lastScenario.scenario_id &&
          r.id !== lastScenario.id &&
          r.status !== "running",
      ) ?? null)
    : null;

  if (!overview) {
    return (
      <>
        <PageTitle title="Overview" description="What the stack and the last runs look like right now." />
        {error ? <p className="text-destructive text-sm">{error}</p> : null}
        <Skeleton className="h-36" />
        <div className="grid gap-4 xl:grid-cols-5">
          <Skeleton className="h-80 xl:col-span-3" />
          <Skeleton className="h-80 xl:col-span-2" />
        </div>
      </>
    );
  }

  return (
    <>
      <PageTitle title="Overview" description="What the stack and the last runs look like right now.">
        <Button
          variant="outline"
          size="sm"
          onClick={() => {
            reload();
            runs.reload();
          }}
        >
          <RefreshCw /> Refresh
        </Button>
      </PageTitle>

      <KpiStrip
        items={[
          {
            icon: Server,
            label: "Stack health",
            previous: overview.stack
              ? `${stack.length} instances in ${overview.config.paths.topology.split("/").pop()}`
              : "serve runs with --no-stack",
            value: overview.stack ? `${up}/${stack.length} up` : "none",
            hint: faulty ? `${faulty} with an injected fault` : "no faults injected",
          },
          {
            icon: ListChecks,
            label: "Last validate",
            previous: lv ? `${ago(lv.started_at)} against ${overview.config.targets.protocol}` : "never run",
            value: lv?.passed ? `${lv.passed[0]}/${lv.passed[1]}` : (lv?.status ?? "–"),
            hint: lv ? `${lv.status} in ${lv.duration_s.toFixed(1)} s` : "run one from Validate",
          },
          {
            icon: Activity,
            label: "Runs, last 24 h",
            previous: `${yesterday.length} the 24 h before`,
            value: today.length,
            ...(yesterday.length
              ? {
                  delta: {
                    value: delta(today.length, yesterday.length),
                    label: "vs previous day",
                  },
                }
              : { hint: "no runs the day before" }),
          },
          {
            icon: AlertTriangle,
            label: "Failure rate, 24 h",
            previous: `${failYesterday} of ${yesterday.length} failed before`,
            value: `${rateToday.toFixed(1)}%`,
            ...(yesterday.length
              ? {
                  delta: {
                    value: rateToday - rateYesterday,
                    label: "points vs previous day",
                    goodWhen: "down",
                  },
                }
              : {
                  hint: `${failToday} of ${today.length} failed today`,
                }),
          },
        ]}
      />

      <div className="grid gap-4 xl:grid-cols-5">
        <Card className="xl:col-span-3">
          <CardHeader className="relative">
            <CardTitle className="flex items-center gap-2">
              <span className="flex size-8 items-center justify-center rounded-lg border">
                <BarChart3 className="size-4" />
              </span>
              Throughput
            </CardTitle>
            <div className="absolute top-6 right-6">
              <Legend
                items={[
                  {
                    label: "This run",
                    color: "var(--foreground)",
                  },
                  {
                    label: "Previous run",
                    color: "var(--chart-2)",
                  },
                ]}
              />
            </div>
          </CardHeader>
          <CardContent>
            <ThroughputCompare current={lastScenario} previous={previousOfSame} />
          </CardContent>
        </Card>
        <Card className="xl:col-span-2">
          <CardHeader className="relative">
            <CardTitle className="flex items-center gap-2">
              <span className="flex size-8 items-center justify-center rounded-lg border">
                <BarChart3 className="size-4" />
              </span>
              Latency by scenario
            </CardTitle>
            <CardDescription>Latest finished run of each scenario.</CardDescription>
          </CardHeader>
          <CardContent>
            <Legend
              items={[
                { label: "p50", color: "var(--foreground)" },
                { label: "p90", color: "var(--chart-3)" },
                { label: "p99", color: "var(--chart-1)" },
              ]}
            />
            <div className="mt-3">
              <LatencyBars
                rows={latest
                  .filter((r) => r.p99_ms !== null)
                  .map((r) => ({
                    name: r.scenario_id ?? r.name,
                    p50: r.p50_ms ?? 0,
                    p90: r.p90_ms ?? 0,
                    p99: r.p99_ms ?? 0,
                  }))}
              />
            </div>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Stack</CardTitle>
            <CardDescription>{overview.config.paths.topology} in this process.</CardDescription>
            <div className="absolute top-6 right-6">
              <Button variant="outline" size="sm" asChild>
                <Link href="/stack/">Manage</Link>
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            {overview.stack ? (
              <ul className="divide-y">
                {stack.map((i) => (
                  <li key={i.name} className="flex items-center gap-3 py-2.5 text-sm">
                    <span
                      className={`size-2 rounded-full ${!i.running ? "bg-muted-foreground/40" : i.behavior && i.behavior.type !== "healthy" ? "bg-amber-500" : "bg-emerald-500"}`}
                    />
                    <span className="font-mono text-xs">{i.name}</span>
                    <span className="text-muted-foreground">{i.kind}</span>
                    <span className="text-muted-foreground ml-auto text-xs">
                      {!i.running ? "stopped" : i.behavior ? describeBehavior(i.behavior) : "running"}
                    </span>
                    {i.requests ? (
                      <span className="w-20 text-right text-xs tabular-nums">
                        {num(i.requests.total)} req
                      </span>
                    ) : null}
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-muted-foreground text-sm">serve runs with --no-stack</p>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader className="relative">
            <CardTitle>Recent activity</CardTitle>
            <CardDescription>Live feed of this session.</CardDescription>
          </CardHeader>
          <CardContent>
            {activity.length ? (
              <ul className="divide-y">
                {activity.slice(0, 6).map((a) => (
                  <li key={a.at} className="flex items-start gap-3 py-2.5 text-sm">
                    {a.event.type === "stack_changed" ? (
                      <>
                        <Server className="text-muted-foreground mt-0.5 size-4" />
                        <div className="flex-1">
                          <div>Stack changed</div>
                          <div className="text-muted-foreground text-xs">
                            {a.event.instances.filter((i) => i.running).length}/{a.event.instances.length}{" "}
                            running
                          </div>
                        </div>
                      </>
                    ) : (
                      <>
                        <Activity className="text-muted-foreground mt-0.5 size-4" />
                        <div className="min-w-0 flex-1">
                          <Link href={`/runs/view/?id=${a.event.run.id}`} className="hover:underline">
                            {a.event.type === "run_started" ? "Started" : "Finished"} {a.event.run.name}
                          </Link>
                          <div className="text-muted-foreground text-xs">{a.event.run.kind}</div>
                        </div>
                        <StatusBadge status={a.event.run.status} />
                      </>
                    )}
                    <span className="text-muted-foreground text-xs">{ago(new Date(a.at).toISOString())}</span>
                  </li>
                ))}
              </ul>
            ) : (
              <p className="text-muted-foreground text-sm">Nothing yet. Start a run and it shows up here.</p>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Where to look</CardTitle>
            <CardDescription>What this environment points at.</CardDescription>
          </CardHeader>
          <CardContent>
            <DetailList
              rows={[
                {
                  k: "Validate targets",
                  v: <span className="font-mono text-xs">{overview.config.targets.protocol}</span>,
                },
                {
                  k: "Engine target",
                  v: <span className="font-mono text-xs">{overview.config.targets.engine}</span>,
                },
                {
                  k: "Scenarios",
                  v: <span className="font-mono text-xs">{overview.config.paths.scenarios}</span>,
                },
                {
                  k: "Run records",
                  v: <span className="font-mono text-xs">{overview.config.paths.results}</span>,
                },
                ...Object.entries(overview.config.links)
                  .filter(([k, v]) => v && k !== "domain")
                  .map(([k, v]) => ({
                    k: k.replace("_", " "),
                    v: (
                      <a className="font-mono text-xs underline" href={v} target="_blank" rel="noreferrer">
                        {v.replace(/^https?:\/\//, "")}
                      </a>
                    ),
                  })),
              ]}
            />
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent runs</CardTitle>
          <CardDescription>Newest first.</CardDescription>
          <div className="absolute top-6 right-6">
            <Button variant="outline" size="sm" asChild>
              <Link href="/runs/">All runs</Link>
            </Button>
          </div>
        </CardHeader>
        <CardContent>
          <RunsTable runs={all.slice(0, 8)} />
        </CardContent>
      </Card>
    </>
  );
}

function ThroughputCompare({
  current,
  previous,
}: {
  current: RunSummary | null;
  previous: RunSummary | null;
}) {
  const [cur, setCur] = useState<RunRecord | null>(null);
  const [prev, setPrev] = useState<RunRecord | null>(null);
  useEffect(() => {
    if (!current) return;
    api
      .run(current.id)
      .then(setCur)
      .catch(() => setCur(null));
    if (previous)
      api
        .run(previous.id)
        .then(setPrev)
        .catch(() => setPrev(null));
  }, [current, previous]);
  // Without a previous run there is nothing to compare against.
  const prevSamples = previous && prev ? prev.samples : [];
  const load = cur?.scenario?.load ?? cur?.load ?? null;
  return (
    <>
      <ChartHeadline
        value={load ? `${Math.round(load.throughput_rps)} req/s` : "–"}
        caption={
          current
            ? `${current.name} · p99 ${ms(load?.latency.p99_ms)} · ${pct(load?.error_rate)} errors`
            : "no scenario run yet"
        }
      />
      <CompareChart current={cur?.samples ?? []} previous={prevSamples} />
    </>
  );
}
