"use client";

import {
  Activity,
  AlertTriangle,
  BarChart3,
  CalendarClock,
  ClipboardCheck,
  Gauge,
  ListChecks,
  ListOrdered,
  RefreshCw,
  Server,
} from "lucide-react";
import Link from "next/link";
import { useEffect, useMemo, useState } from "react";

import { useChaos } from "@/app/providers";
import { ChartHeadline, CompareChart, LatencyBars, Legend, LoadTrend, PassStrip } from "@/components/charts";
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
  const [allActivity, setAllActivity] = useState(false);
  // One row per thing that happened: a run's "started" folds into its
  // "finished" once that arrives, and consecutive queue changes collapse into
  // the newest one, so a sweep of ten scenarios is ten rows, not forty.
  const feed = useMemo(() => {
    const finished = new Set(
      activity
        .filter((a) => a.event.type === "run_finished")
        .map((a) => (a.event as { run: { id: string } }).run.id),
    );
    const keep: typeof activity = [];
    for (const a of activity) {
      if (a.event.type === "run_started" && finished.has(a.event.run.id)) continue;
      if (a.event.type === "queue_changed" && keep.at(-1)?.event.type === "queue_changed") continue;
      keep.push(a);
    }
    return keep;
  }, [activity]);
  const visibleActivity = useMemo(() => {
    if (allActivity) return feed;
    const cutoff = Date.now() - ACTIVITY_WINDOW_MS;
    return feed.filter((a) => a.at >= cutoff).slice(0, ACTIVITY_ROWS);
  }, [feed, allActivity]);
  const hiddenActivity = feed.length - visibleActivity.length;
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
  const loadRuns = all.filter(
    (r) => r.kind === "load" && r.status !== "running" && r.throughput_rps !== null,
  );
  const lastLoad = loadRuns[0] ?? null;
  const loadTrend = loadRuns
    .slice(0, 12)
    .reverse()
    .map((r) => ({
      id: r.id,
      label: ago(r.started_at).replace(" ago", ""),
      rps: r.throughput_rps ?? 0,
      p99: r.p99_ms ?? 0,
      errors: (r.error_rate ?? 0) * 100,
    }));
  const validateRuns = all.filter((r) => r.kind === "validate" && r.status !== "running");
  const lastValidate = validateRuns[0] ?? null;
  const validateStrip = validateRuns
    .slice(0, 40)
    .reverse()
    .map((r) => ({
      id: r.id,
      ok: r.status === "passed",
      title: `${r.status}${r.passed ? ` · ${r.passed[0]}/${r.passed[1]} checks` : ""} · ${ago(r.started_at)}`,
    }));
  const validatePassRate = validateRuns.length
    ? validateRuns.slice(0, 40).filter((r) => r.status === "passed").length /
      Math.min(validateRuns.length, 40)
    : null;
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
            previous: lv
              ? `${ago(lv.started_at)} against ${Object.values(overview.config.targets)[0] ?? "the config targets"}`
              : "never run",
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
          <CardHeader className="flex flex-row items-center justify-between gap-4 space-y-0">
            <CardTitle className="flex items-center gap-2">
              <span className="flex size-8 items-center justify-center rounded-lg border">
                <BarChart3 className="size-4" />
              </span>
              Throughput
            </CardTitle>
            <div>
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
          <CardHeader>
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

      <div className="grid gap-4 xl:grid-cols-5">
        <Card className="xl:col-span-3">
          <CardHeader className="flex flex-row items-center justify-between gap-4 space-y-0">
            <CardTitle className="flex items-center gap-2">
              <span className="flex size-8 items-center justify-center rounded-lg border">
                <Gauge className="size-4" />
              </span>
              Load runs
            </CardTitle>
            <div className="flex items-center gap-3">
              <Legend
                items={[
                  { label: "req/s", color: "var(--foreground)" },
                  { label: "p99", color: "var(--chart-2)" },
                ]}
              />
              <Button variant="outline" size="sm" asChild>
                <Link href="/runs/?kind=load">All</Link>
              </Button>
            </div>
          </CardHeader>
          <CardContent>
            <ChartHeadline
              value={lastLoad ? `${Math.round(lastLoad.throughput_rps ?? 0)} req/s` : "–"}
              caption={
                lastLoad
                  ? `last load · ${lastLoad.name} · p99 ${(lastLoad.p99_ms ?? 0).toFixed(1)} ms · ${((lastLoad.error_rate ?? 0) * 100).toFixed(2)}% errors · ${ago(lastLoad.started_at)}`
                  : "no load run yet"
              }
            />
            <LoadTrend rows={loadTrend} />
          </CardContent>
        </Card>
        <Card className="xl:col-span-2">
          <CardHeader className="flex flex-row items-center justify-between gap-4 space-y-0">
            <CardTitle className="flex items-center gap-2">
              <span className="flex size-8 items-center justify-center rounded-lg border">
                <ClipboardCheck className="size-4" />
              </span>
              Validate
            </CardTitle>
            <Button variant="outline" size="sm" asChild>
              <Link href="/validate/">Run</Link>
            </Button>
          </CardHeader>
          <CardContent>
            <ChartHeadline
              value={
                lastValidate ? (
                  <span className="flex items-center gap-3">
                    {lastValidate.passed ? `${lastValidate.passed[0]}/${lastValidate.passed[1]}` : "–"}
                    <StatusBadge status={lastValidate.status} />
                  </span>
                ) : (
                  "–"
                )
              }
              caption={
                lastValidate
                  ? `last validate · ${ago(lastValidate.started_at)} · ${validatePassRate === null ? "" : `${Math.round(validatePassRate * 100)}% of the last ${Math.min(validateRuns.length, 40)} passed`}`
                  : "never run"
              }
            />
            <PassStrip items={validateStrip} />
            {lastValidate?.error ? (
              <p className="text-destructive mt-3 text-xs">{lastValidate.error}</p>
            ) : null}
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-3">
        <Card>
          <CardHeader className="flex flex-row items-start justify-between gap-4 space-y-0">
            <div className="space-y-1.5">
              <CardTitle>Stack</CardTitle>
              <CardDescription>{overview.config.paths.topology} in this process.</CardDescription>
            </div>
            <Button variant="outline" size="sm" asChild>
              <Link href="/stack/">Manage</Link>
            </Button>
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
          <CardHeader>
            <CardTitle>Recent activity</CardTitle>
            <CardDescription>
              Live feed of this session; the last ten minutes, older on request.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {feed.length ? (
              <ul className="divide-y">
                {visibleActivity.map((a) => (
                  <li key={a.id} className="flex items-start gap-3 py-2.5 text-sm">
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
                    ) : a.event.type === "queue_changed" ? (
                      <>
                        <ListOrdered className="text-muted-foreground mt-0.5 size-4" />
                        <div className="flex-1">
                          <Link href="/runs/" className="hover:underline">
                            Queue changed
                          </Link>
                          <div className="text-muted-foreground text-xs">{a.event.queue.length} waiting</div>
                        </div>
                      </>
                    ) : a.event.type === "schedules_changed" ? (
                      <>
                        <CalendarClock className="text-muted-foreground mt-0.5 size-4" />
                        <div className="flex-1">
                          <Link href="/schedules/" className="hover:underline">
                            Schedules changed
                          </Link>
                          <div className="text-muted-foreground text-xs">
                            {a.event.schedules.filter((x) => x.enabled).length}/{a.event.schedules.length}{" "}
                            enabled
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
                {!visibleActivity.length ? (
                  <li className="text-muted-foreground py-2.5 text-sm">Quiet for the last ten minutes.</li>
                ) : null}
              </ul>
            ) : (
              <p className="text-muted-foreground text-sm">Nothing yet. Start a run and it shows up here.</p>
            )}
            {hiddenActivity > 0 || allActivity ? (
              <Button variant="ghost" size="sm" className="mt-2" onClick={() => setAllActivity((v) => !v)}>
                {allActivity ? "Show less" : `Show ${hiddenActivity} more`}
              </Button>
            ) : null}
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
                  k: "Serve stack",
                  v: (
                    <Link href="/stack/" className="text-xs hover:underline">
                      {overview.stack ? describeStack(overview.stack) : "not started (--no-stack)"}
                    </Link>
                  ),
                },
                {
                  k: "Validate hits",
                  v: (
                    <span className="grid font-mono text-xs">
                      {Object.entries(overview.config.targets).map(([kind, url], i) => (
                        <span key={kind} className={i ? "text-muted-foreground" : ""}>
                          {url} <span className="text-muted-foreground">({kind})</span>
                        </span>
                      ))}
                    </span>
                  ),
                },
                {
                  k: "Scenarios",
                  v: <span className="font-mono text-xs">{overview.config.paths.scenarios}</span>,
                },
                {
                  k: "Campaigns",
                  v: (
                    <span className="font-mono text-xs">
                      {overview.config.paths.campaigns}{" "}
                      <span className="text-muted-foreground font-sans">({overview.campaigns})</span>
                    </span>
                  ),
                },
                {
                  k: "Findings",
                  v: (
                    <span className="font-mono text-xs">
                      {overview.config.paths.findings}{" "}
                      <span className="text-muted-foreground font-sans">
                        ({overview.findings} in {overview.finding_signatures} signature
                        {overview.finding_signatures === 1 ? "" : "s"})
                      </span>
                    </span>
                  ),
                },
                {
                  k: "Run records",
                  v: <span className="font-mono text-xs">{overview.config.paths.results}</span>,
                },
                ...Object.entries(overview.config.links)
                  .filter(([k, v]) => v && !["domain", "chaos", "auth"].includes(k))
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
        <CardHeader className="flex flex-row items-start justify-between gap-4 space-y-0">
          <div className="space-y-1.5">
            <CardTitle>Recent runs</CardTitle>
            <CardDescription>Newest first.</CardDescription>
          </div>
          <Button variant="outline" size="sm" asChild>
            <Link href="/runs/">All runs</Link>
          </Button>
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

const ACTIVITY_WINDOW_MS = 10 * 60_000;
const ACTIVITY_ROWS = 6;

/** "2 engines, 1 protocol · 3/3 running", counted live from the stack. */
function describeStack(stack: { kind: string; running: boolean }[]): string {
  const byKind = new Map<string, number>();
  for (const i of stack) byKind.set(i.kind, (byKind.get(i.kind) ?? 0) + 1);
  const parts = [...byKind.entries()].map(([k, n]) => `${n} ${k}${n === 1 ? "" : "s"}`);
  const up = stack.filter((i) => i.running).length;
  return `${parts.join(", ")} · ${up}/${stack.length} running`;
}
