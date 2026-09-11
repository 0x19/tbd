"use client";

import { ArrowLeft, Square } from "lucide-react";
import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { toast } from "sonner";

import { ChartHeadline, Legend, RunChart } from "@/components/charts";
import { DetailList, HeatGrid, LevelChip, PageTitle, StageBar, StatRow } from "@/components/kit";
import { BoolBadge, StatusBadge } from "@/components/status-badge";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import type { RunLive } from "@/lib/api/hooks";
import { describe, useRunFeed } from "@/lib/api/hooks";
import type { CheckResult, LoadSnapshot, RunRecord } from "@/lib/api/schema";
import { ms, num, pct, seconds, when } from "@/lib/format";

export default function RunViewPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <RunView />
    </Suspense>
  );
}

const STAGES = ["Setup", "Load + timeline", "Assert", "Teardown", "Done"];
const STRESS_STAGES = ["Setup", "Warmup", "Run + timeline", "Shrink", "Teardown", "Done"];

function stressStageIndex(phase: string | null, finished: boolean, record: RunRecord | null) {
  if (finished || (record && record.status !== "running")) return STRESS_STAGES.length;
  switch (phase) {
    case "setup":
      return 0;
    case "warmup":
      return 1;
    case "run":
      return 2;
    case "shrink":
      return 3;
    case "done":
    case "teardown":
      return 4;
    default:
      return record ? 2 : 0;
  }
}

function stageIndex(phase: string | null, finished: boolean, record: RunRecord | null) {
  if (finished || (record && record.status !== "running")) return STAGES.length;
  switch (phase) {
    case "setup":
      return 0;
    case "load":
      return 1;
    case "assert":
      return 2;
    case "teardown":
      return 3;
    default:
      return record ? 1 : 0;
  }
}

/** The kit's detail layout: back, big title with chips, lifecycle strip, stats, chart, tables. */
function RunView() {
  const id = useSearchParams().get("id");
  const live = useRunFeed(id);
  const [cancelling, setCancelling] = useState(false);
  const record = live.record;
  const running = record?.status === "running" && !live.finished;
  const latest: LoadSnapshot | null = live.samples.at(-1) ?? record?.scenario?.load ?? record?.load ?? null;

  const cancel = async () => {
    if (!id) return;
    setCancelling(true);
    try {
      await api.runCancel(id);
      toast.success("cancelling");
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setCancelling(false);
    }
  };

  if (!id) return <p className="text-muted-foreground text-sm">No run id.</p>;

  return (
    <>
      <PageTitle
        back={
          <Button variant="outline" size="icon" aria-label="Back" asChild>
            <Link href="/runs/">
              <ArrowLeft />
            </Link>
          </Button>
        }
        title={
          <span className="flex flex-wrap items-center gap-3">
            {record?.name ?? "Run"}
            {record ? <StatusBadge status={running ? "running" : record.status} /> : null}
            {record?.kind ? (
              <Badge variant="outline" className="capitalize">
                {record.kind}
              </Badge>
            ) : null}
          </span>
        }
        description={
          record ? (
            <>
              <span className="font-mono text-xs">{record.id}</span> · started {when(record.started_at)}
              {record.scenario_id ? (
                <>
                  {" "}
                  · from{" "}
                  <Link
                    className="underline"
                    href={`/scenarios/view/?id=${encodeURIComponent(record.scenario_id)}`}
                  >
                    scenarios/{record.scenario_id}.toml
                  </Link>
                </>
              ) : null}
              {record.campaign_id ? (
                <>
                  {" "}
                  · from{" "}
                  <Link
                    className="underline"
                    href={`/stress/view/?id=${encodeURIComponent(record.campaign_id)}`}
                  >
                    stress/{record.campaign_id}.toml
                  </Link>
                </>
              ) : null}
            </>
          ) : (
            id
          )
        }
      >
        {running ? (
          <Button variant="destructive" size="sm" onClick={cancel} disabled={cancelling}>
            <Square /> Cancel run
          </Button>
        ) : null}
      </PageTitle>
      {live.error ? <p className="text-destructive text-sm">{live.error}</p> : null}
      {record?.error ? (
        <div className="border-destructive/30 bg-destructive/5 text-destructive rounded-lg border px-3 py-2 text-sm">
          {record.error}
        </div>
      ) : null}

      {record?.kind === "validate" && record.validate ? (
        <ValidateView record={record} />
      ) : record?.kind === "stress" || live.stress ? (
        <StressView live={live} record={record} running={running} latest={latest} />
      ) : (
        <>
          {record?.kind !== "load" ? (
            <div className="bg-muted/30 rounded-xl border p-4">
              <div className="mb-3 flex justify-between text-sm">
                <span>
                  Lifecycle
                  {running && live.phase ? (
                    <span className="text-muted-foreground"> · in {live.phase}</span>
                  ) : null}
                </span>
                <span className="text-muted-foreground">
                  {running ? `${seconds(latest?.elapsed_s)} elapsed` : `took ${seconds(record?.duration_s)}`}
                </span>
              </div>
              <StageBar stages={STAGES} current={stageIndex(live.phase, live.finished, record)} />
            </div>
          ) : null}

          <StatRow
            items={[
              {
                label: "Requests",
                value: num(latest?.requests_total),
                sub: latest ? `${num(latest.requests_failed)} failed` : undefined,
              },
              {
                label: "Error rate",
                value: pct(latest?.error_rate),
                tone: latest ? (latest.error_rate > 0 ? "bad" : "good") : undefined,
              },
              {
                label: "Throughput",
                value: latest ? `${Math.round(latest.throughput_rps)} rps` : "–",
              },
              {
                label: "p50 / p99",
                value: latest ? `${ms(latest.latency.p50_ms)} / ${ms(latest.latency.p99_ms)}` : "–",
              },
            ]}
          />

          <Card>
            <CardHeader>
              <CardTitle>Load over time</CardTitle>
              <CardDescription>
                <Legend
                  items={[
                    {
                      label: "req/s",
                      color: "var(--foreground)",
                    },
                    {
                      label: "p50 ms",
                      color: "var(--chart-2)",
                    },
                    {
                      label: "p99 ms",
                      color: "var(--chart-4)",
                    },
                    {
                      label: "error %",
                      color: "var(--destructive)",
                    },
                  ]}
                />
              </CardDescription>
            </CardHeader>
            <CardContent>
              <ChartHeadline
                value={latest ? `${Math.round(latest.throughput_rps)} req/s` : "–"}
                caption={
                  record?.kind === "load"
                    ? "ad-hoc load, one sample per second"
                    : "this run, one sample per second"
                }
              />
              <RunChart samples={live.samples} />
            </CardContent>
          </Card>

          <div className="grid gap-4 xl:grid-cols-2">
            {record?.scenario ? (
              <Card>
                <CardHeader>
                  <CardTitle>Assertions</CardTitle>
                  <CardDescription>Each bound and what was observed.</CardDescription>
                </CardHeader>
                <CardContent>
                  {record.scenario.assertions.length ? (
                    <Table>
                      <TableHeader>
                        <TableRow>
                          <TableHead>Assertion</TableHead>
                          <TableHead>Expected</TableHead>
                          <TableHead>Actual</TableHead>
                          <TableHead />
                        </TableRow>
                      </TableHeader>
                      <TableBody>
                        {record.scenario.assertions.map((a) => (
                          <TableRow key={a.name}>
                            <TableCell className="font-mono text-xs">{a.name}</TableCell>
                            <TableCell className="tabular-nums">{a.expected}</TableCell>
                            <TableCell className="tabular-nums">{a.actual}</TableCell>
                            <TableCell className="text-right">
                              <BoolBadge ok={a.passed} />
                            </TableCell>
                          </TableRow>
                        ))}
                      </TableBody>
                    </Table>
                  ) : (
                    <p className="text-muted-foreground text-sm">
                      {running ? "Evaluated when load ends." : "No assertions in this scenario."}
                    </p>
                  )}
                </CardContent>
              </Card>
            ) : null}
            <Card>
              <CardHeader>
                <CardTitle>Timeline</CardTitle>
                <CardDescription>Actions as they fired, seconds after load start.</CardDescription>
              </CardHeader>
              <CardContent>
                {live.events.length ? (
                  <ol className="divide-y">
                    {live.events.map((e, i) => (
                      <li key={i} className="flex items-start gap-3 py-2.5 text-sm">
                        <span className="text-muted-foreground w-16 font-mono text-xs tabular-nums">
                          {e.at_s.toFixed(2)} s
                        </span>
                        <span className="flex-1 font-mono text-xs">{e.action}</span>
                        {e.error ? <LevelChip level="error" /> : <LevelChip level="ok" />}
                      </li>
                    ))}
                  </ol>
                ) : (
                  <p className="text-muted-foreground text-sm">
                    {record?.kind === "load" ? "Ad-hoc load has no timeline." : "No timeline events yet."}
                  </p>
                )}
              </CardContent>
            </Card>
          </div>

          {latest ? <Breakdown snapshot={latest} services={record?.scenario?.services} /> : null}
        </>
      )}
    </>
  );
}

/**
 * A campaign or a replay: the stages, the checks as they are evaluated, the
 * findings as they arrive, the load numbers, and for a replay its verdict.
 */
function StressView({
  live,
  record,
  running,
  latest,
}: {
  live: RunLive;
  record: RunRecord | null;
  running: boolean;
  latest: LoadSnapshot | null;
}) {
  const result = record?.stress ?? null;
  const checks = (!running && result?.checks) || live.stress?.checks || result?.checks || {};
  const rows = Object.entries(checks).sort(([a], [b]) => a.localeCompare(b));
  const evaluated = rows.reduce((n, [, c]) => n + c.passed + c.violated, 0);
  const violated = rows.reduce((n, [, c]) => n + c.violated, 0);
  const findings = live.findings.length ? live.findings : (result?.findings ?? []);
  const ops = live.stress?.ops_total ?? latest?.requests_total;
  const failed = live.stress?.ops_failed ?? latest?.requests_failed;
  const tolerated = live.stress?.tolerated ?? result?.tolerated ?? 0;
  const redriven = live.stress?.redriven ?? result?.redriven ?? 0;
  const replay = record?.replay ?? null;
  return (
    <>
      <div className="bg-muted/30 rounded-xl border p-4">
        <div className="mb-3 flex justify-between text-sm">
          <span>
            Lifecycle
            {running && live.phase ? <span className="text-muted-foreground"> · in {live.phase}</span> : null}
            {result?.store ? <span className="text-muted-foreground"> · {result.store} store</span> : null}
            {result?.targets.length ? (
              <span className="text-muted-foreground"> · {result.targets.join(", ")}</span>
            ) : null}
          </span>
          <span className="text-muted-foreground">
            {running
              ? `${seconds(live.stress?.elapsed_s ?? latest?.elapsed_s)} elapsed`
              : `took ${seconds(record?.duration_s)}`}
          </span>
        </div>
        <StageBar stages={STRESS_STAGES} current={stressStageIndex(live.phase, live.finished, record)} />
      </div>

      {replay ? (
        <div
          className={`rounded-lg border px-3 py-2 text-sm ${
            replay.reproduced
              ? "border-destructive/30 bg-destructive/5 text-destructive"
              : "border-emerald-500/30 bg-emerald-500/5 text-emerald-700 dark:text-emerald-400"
          }`}
        >
          {replay.reproduced ? "Reproduced" : "Not reproduced"} on {replay.target} after {replay.steps_run}{" "}
          steps
          {replay.message ? `: ${replay.message}` : ""}
        </div>
      ) : null}
      {result?.stopped_early ? (
        <p className="text-muted-foreground text-sm">Stopped early: `[stop] max_findings` was reached.</p>
      ) : null}

      <StatRow
        items={[
          {
            label: "Requests",
            value: num(ops),
            sub: ops !== undefined ? `${num(failed)} failed` : undefined,
          },
          {
            label: "Invariant evaluations",
            value: num(evaluated),
            sub: `${num(violated)} violated`,
            tone: evaluated ? (violated ? "bad" : "good") : undefined,
          },
          {
            label: "Findings",
            value: findings.length,
            tone: findings.length ? "bad" : record && !running ? "good" : undefined,
            sub: live.stress ? `${num(live.stress.subjects)} subjects` : undefined,
          },
          {
            label: "Tolerated / re-driven",
            value: `${num(tolerated)} / ${num(redriven)}`,
            sub: latest ? `${ms(latest.latency.p50_ms)} p50 · ${ms(latest.latency.p99_ms)} p99` : undefined,
          },
        ]}
      />

      <div className="grid gap-4 xl:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Invariants</CardTitle>
            <CardDescription>
              Every rule the workers judged, how often it was evaluated and how often it broke.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {rows.length ? (
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Invariant</TableHead>
                    <TableHead className="text-right">Evaluated</TableHead>
                    <TableHead className="text-right">Violated</TableHead>
                    <TableHead />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {rows.map(([name, c]) => (
                    <TableRow key={name}>
                      <TableCell className="font-mono text-xs">{name}</TableCell>
                      <TableCell className="text-right tabular-nums">{num(c.passed + c.violated)}</TableCell>
                      <TableCell
                        className={`text-right tabular-nums ${c.violated ? "text-destructive" : ""}`}
                      >
                        {num(c.violated)}
                      </TableCell>
                      <TableCell className="text-right">
                        {c.passed + c.violated ? (
                          <BoolBadge ok={!c.violated} yes="held" no="broken" />
                        ) : (
                          <Badge variant="outline">not evaluated</Badge>
                        )}
                      </TableCell>
                    </TableRow>
                  ))}
                </TableBody>
              </Table>
            ) : (
              <p className="text-muted-foreground text-sm">
                {running ? "Evaluations arrive once the measured phase starts." : "Nothing was evaluated."}
              </p>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Findings</CardTitle>
            <CardDescription>
              Each is one broken rule with the trace that got there; shrunk after the workers stop.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {findings.length ? (
              <ol className="divide-y">
                {findings.map((f) => (
                  <li key={f.id} className="grid gap-0.5 py-2.5 text-sm">
                    <div className="flex items-center justify-between gap-2">
                      <Link
                        href={`/findings/view/?id=${f.id}`}
                        className="font-mono text-xs font-medium hover:underline"
                      >
                        {f.invariant}
                      </Link>
                      <span className="text-muted-foreground text-xs tabular-nums">
                        {f.trace_len} steps{f.shrunk ? ", shrunk" : ""}
                      </span>
                    </div>
                    <Link
                      href={`/findings/view/?id=${f.id}`}
                      className="text-muted-foreground line-clamp-2 text-xs hover:underline"
                    >
                      {f.message}
                    </Link>
                  </li>
                ))}
              </ol>
            ) : (
              <p className="text-muted-foreground text-sm">
                {running ? "None so far." : "None. Every evaluated invariant held."}
              </p>
            )}
            {record?.id && findings.length ? (
              <Button variant="outline" size="sm" className="mt-3" asChild>
                <Link href={`/findings/?run=${record.id}`}>All findings of this run</Link>
              </Button>
            ) : null}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Load over time</CardTitle>
          <CardDescription>
            <Legend
              items={[
                { label: "req/s", color: "var(--foreground)" },
                { label: "p50 ms", color: "var(--chart-2)" },
                { label: "p99 ms", color: "var(--chart-4)" },
                { label: "error %", color: "var(--destructive)" },
              ]}
            />
          </CardDescription>
        </CardHeader>
        <CardContent>
          <ChartHeadline
            value={latest ? `${Math.round(latest.throughput_rps)} req/s` : "–"}
            caption="closed-loop workers, one sample per second; a tolerated fault counts as an error here"
          />
          <RunChart samples={live.samples} />
        </CardContent>
      </Card>

      {live.events.length ? (
        <Card>
          <CardHeader>
            <CardTitle>Timeline</CardTitle>
            <CardDescription>Actions as they fired, seconds after the measured phase began.</CardDescription>
          </CardHeader>
          <CardContent>
            <ol className="divide-y">
              {live.events.map((e, i) => (
                <li key={i} className="flex items-start gap-3 py-2.5 text-sm">
                  <span className="text-muted-foreground w-16 font-mono text-xs tabular-nums">
                    {e.at_s.toFixed(2)} s
                  </span>
                  <span className="flex-1 font-mono text-xs">{e.action}</span>
                  {e.error ? <LevelChip level="error" /> : <LevelChip level="ok" />}
                </li>
              ))}
            </ol>
          </CardContent>
        </Card>
      ) : null}

      {latest ? <Breakdown snapshot={latest} /> : null}
    </>
  );
}

function Breakdown({
  snapshot,
  services,
}: {
  snapshot: LoadSnapshot;
  services?: Record<string, { total: number; failed: number }>;
}) {
  const ops = Object.keys(snapshot.per_op);
  const metrics = ["p50", "p90", "p99", "max"] as const;
  return (
    <div className="grid gap-4 xl:grid-cols-3">
      <Card className="xl:col-span-2">
        <CardHeader>
          <CardTitle>Latency grid</CardTitle>
          <CardDescription>Milliseconds per operation. Darker is slower.</CardDescription>
        </CardHeader>
        <CardContent>
          <HeatGrid
            rows={ops}
            cols={[...metrics]}
            cell={(op, m) => {
              const l = snapshot.per_op[op]?.latency;
              if (!l) return null;
              return m === "p50" ? l.p50_ms : m === "p90" ? l.p90_ms : m === "p99" ? l.p99_ms : l.max_ms;
            }}
            format={(v) => `${v.toFixed(2)} ms`}
          />
          <div className="mt-4 overflow-x-auto">
            <Table>
              <TableHeader>
                <TableRow>
                  <TableHead>Operation</TableHead>
                  <TableHead className="text-right">Sent</TableHead>
                  <TableHead className="text-right">Failed</TableHead>
                  <TableHead className="text-right">Share</TableHead>
                </TableRow>
              </TableHeader>
              <TableBody>
                {ops.map((op) => {
                  const s = snapshot.per_op[op];
                  return (
                    <TableRow key={op}>
                      <TableCell className="font-mono text-xs">{op}</TableCell>
                      <TableCell className="text-right tabular-nums">{num(s.total)}</TableCell>
                      <TableCell className={`text-right tabular-nums ${s.failed ? "text-destructive" : ""}`}>
                        {num(s.failed)}
                      </TableCell>
                      <TableCell className="text-right tabular-nums">
                        {snapshot.requests_total
                          ? `${((s.total / snapshot.requests_total) * 100).toFixed(0)}%`
                          : "–"}
                      </TableCell>
                    </TableRow>
                  );
                })}
              </TableBody>
            </Table>
          </div>
        </CardContent>
      </Card>
      <div className="grid content-start gap-4">
        <Card>
          <CardHeader>
            <CardTitle>Errors by class</CardTitle>
          </CardHeader>
          <CardContent>
            {Object.keys(snapshot.errors).length ? (
              <DetailList
                rows={Object.entries(snapshot.errors).map(([k, v]) => ({
                  k,
                  v: <span className="tabular-nums">{num(v)}</span>,
                }))}
              />
            ) : (
              <p className="text-muted-foreground text-sm">No failures.</p>
            )}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Per target and service</CardTitle>
            <CardDescription>Client side per protocol; engine side from its counters.</CardDescription>
          </CardHeader>
          <CardContent>
            <DetailList
              rows={[
                ...Object.entries(snapshot.per_target).map(([k, v]) => ({
                  k,
                  v: (
                    <span className="tabular-nums">
                      {num(v.total)} sent · {num(v.failed)} failed
                    </span>
                  ),
                })),
                ...Object.entries(services ?? {}).map(([k, v]) => ({
                  k,
                  v: (
                    <span className="tabular-nums">
                      {num(v.total)} served · {num(v.failed)} failed
                    </span>
                  ),
                })),
              ]}
            />
          </CardContent>
        </Card>
      </div>
    </div>
  );
}

function ValidateView({ record }: { record: RunRecord }) {
  const r = record.validate!;
  return (
    <>
      <StatRow
        items={[
          { label: "Passed", value: r.passed, tone: "good" },
          {
            label: "Failed",
            value: r.failed,
            tone: r.failed ? "bad" : undefined,
          },
          { label: "Took", value: seconds(record.duration_s) },
          {
            label: "Targets",
            value: (
              <span className="grid text-sm font-normal">
                {Object.entries((record.request as Record<string, unknown> | null) ?? {})
                  .filter(([k, v]) => k !== "timeout" && typeof v === "string")
                  .map(([k, v]) => (
                    <span key={k}>
                      {String(v)} <span className="text-muted-foreground">({k})</span>
                    </span>
                  ))}
              </span>
            ),
          },
        ]}
      />
      <ChecksTable checks={r.checks} />
    </>
  );
}

export function ChecksTable({ checks }: { checks: CheckResult[] }) {
  return (
    <div className="overflow-x-auto rounded-xl border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Check</TableHead>
            <TableHead>Surface</TableHead>
            <TableHead>Level</TableHead>
            <TableHead className="text-right">Latency</TableHead>
            <TableHead>Detail</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {checks.map((c) => (
            <TableRow key={c.name}>
              <TableCell className="font-mono text-xs">{c.name}</TableCell>
              <TableCell className="text-muted-foreground uppercase">{c.surface}</TableCell>
              <TableCell>
                <LevelChip level={c.passed ? "pass" : "fail"} />
              </TableCell>
              <TableCell className="text-right tabular-nums">{ms(c.latency_ms)}</TableCell>
              <TableCell
                className={`max-w-lg truncate text-xs ${c.passed ? "text-muted-foreground" : "text-destructive"}`}
                title={c.detail}
              >
                {c.detail}
              </TableCell>
            </TableRow>
          ))}
          <TableRow className="bg-muted/40 font-medium">
            <TableCell>Total</TableCell>
            <TableCell colSpan={4} className="text-muted-foreground">
              {checks.filter((c) => c.passed).length} of {checks.length} passed
            </TableCell>
          </TableRow>
        </TableBody>
      </Table>
    </div>
  );
}
