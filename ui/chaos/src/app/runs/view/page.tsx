"use client";

import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { ArrowLeft, Square } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { ChartHeadline, Legend, RunChart } from "@/components/charts";
import { DetailList, HeatGrid, LevelChip, PageTitle, StageBar, StatRow } from "@/components/kit";
import { BoolBadge, StatusBadge } from "@/components/status-badge";
import { api } from "@/lib/api/client";
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

  if (!id) return <p className="text-sm text-muted-foreground">No run id.</p>;

  return (
    <>
      <PageTitle
        back={
          <Button variant="outline" size="icon" render={<Link href="/runs/" />} aria-label="Back">
            <ArrowLeft />
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
      {live.error ? <p className="text-sm text-destructive">{live.error}</p> : null}
      {record?.error ? (
        <div className="rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive">
          {record.error}
        </div>
      ) : null}

      {record?.kind === "validate" && record.validate ? (
        <ValidateView record={record} />
      ) : (
        <>
          {record?.kind !== "load" ? (
            <div className="rounded-xl border bg-muted/30 p-4">
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
              { label: "Throughput", value: latest ? `${Math.round(latest.throughput_rps)} rps` : "–" },
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
                    <p className="text-sm text-muted-foreground">
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
                        <span className="w-16 font-mono text-xs tabular-nums text-muted-foreground">
                          {e.at_s.toFixed(2)} s
                        </span>
                        <span className="flex-1 font-mono text-xs">{e.action}</span>
                        {e.error ? <LevelChip level="error" /> : <LevelChip level="ok" />}
                      </li>
                    ))}
                  </ol>
                ) : (
                  <p className="text-sm text-muted-foreground">
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
        <Card size="sm">
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
              <p className="text-sm text-muted-foreground">No failures.</p>
            )}
          </CardContent>
        </Card>
        <Card size="sm">
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
          { label: "Failed", value: r.failed, tone: r.failed ? "bad" : undefined },
          { label: "Took", value: seconds(record.duration_s) },
          {
            label: "Targets",
            value: (
              <span className="text-sm font-normal">
                {String((record.request as { protocol?: string } | null)?.protocol ?? "")}
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
