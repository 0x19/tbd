"use client";

import Link from "next/link";
import { useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { Square } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { LoadChart } from "@/components/load-chart";
import { Empty, ErrorNote, PageHeader } from "@/components/page-header";
import { StatCard } from "@/components/stat-card";
import { BoolBadge, StatusBadge } from "@/components/status-badge";
import { api } from "@/lib/api/client";
import { describe, useRunFeed } from "@/lib/api/hooks";
import type { LoadSnapshot, RunRecord } from "@/lib/api/schema";
import { ms, num, pct, seconds, when } from "@/lib/format";

export default function RunViewPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <RunView />
    </Suspense>
  );
}

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

  if (!id) return <Empty>No run id.</Empty>;

  return (
    <>
      <PageHeader
        title={record?.name ?? "Run"}
        description={
          record
            ? `${record.kind}${record.scenario_id ? ` · scenarios/${record.scenario_id}.toml` : ""} · started ${when(record.started_at)}${running && live.phase ? ` · phase: ${live.phase}` : ""}`
            : id
        }
      >
        {record ? <StatusBadge status={running ? "running" : record.status} /> : null}
        {record?.scenario_id ? (
          <Button
            variant="outline"
            size="sm"
            render={<Link href={`/scenarios/view/?id=${encodeURIComponent(record.scenario_id)}`} />}
          >
            open scenario
          </Button>
        ) : null}
        {running ? (
          <Button variant="destructive" size="sm" onClick={cancel} disabled={cancelling}>
            <Square /> cancel
          </Button>
        ) : null}
      </PageHeader>
      <ErrorNote message={live.error} />
      {record?.error ? <ErrorNote message={record.error} /> : null}

      {record?.kind === "validate" && record.validate ? (
        <ValidateView record={record} />
      ) : (
        <>
          <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-5">
            <StatCard
              label="Requests"
              value={num(latest?.requests_total)}
              hint={latest ? `${num(latest.requests_failed)} failed` : undefined}
            />
            <StatCard
              label="Error rate"
              value={pct(latest?.error_rate)}
              tone={latest ? (latest.error_rate > 0 ? "bad" : "good") : undefined}
            />
            <StatCard label="Throughput" value={latest ? `${Math.round(latest.throughput_rps)} rps` : "–"} />
            <StatCard
              label="p50 / p99"
              value={latest ? `${ms(latest.latency.p50_ms)} / ${ms(latest.latency.p99_ms)}` : "–"}
            />
            <StatCard
              label="Took"
              value={running ? seconds(latest?.elapsed_s) : seconds(record?.duration_s)}
              hint={running ? "running" : record ? when(record.finished_at) : undefined}
            />
          </div>
          <Card>
            <CardHeader>
              <CardTitle>Load over time</CardTitle>
              <CardDescription>
                One sample per second: requests per second, p50 and p99 in ms, error percent.
              </CardDescription>
            </CardHeader>
            <CardContent>
              <LoadChart samples={live.samples} />
            </CardContent>
          </Card>
          <div className="grid gap-4 xl:grid-cols-2">
            {record?.scenario ? <Assertions record={record} /> : null}
            <Card>
              <CardHeader>
                <CardTitle>Timeline</CardTitle>
                <CardDescription>Actions as they fired, seconds after load start.</CardDescription>
              </CardHeader>
              <CardContent>
                {live.events.length ? (
                  <Table>
                    <TableHeader>
                      <TableRow>
                        <TableHead className="w-20">At</TableHead>
                        <TableHead>Action</TableHead>
                        <TableHead>Result</TableHead>
                      </TableRow>
                    </TableHeader>
                    <TableBody>
                      {live.events.map((e, i) => (
                        <TableRow key={i}>
                          <TableCell className="tabular-nums">{e.at_s.toFixed(2)} s</TableCell>
                          <TableCell className="font-mono text-xs">{e.action}</TableCell>
                          <TableCell>
                            {e.error ? <span className="text-destructive">{e.error}</span> : "ok"}
                          </TableCell>
                        </TableRow>
                      ))}
                    </TableBody>
                  </Table>
                ) : (
                  <Empty>
                    {record?.kind === "load" ? "Ad-hoc load has no timeline." : "No timeline events yet."}
                  </Empty>
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

function Assertions({ record }: { record: RunRecord }) {
  const s = record.scenario!;
  return (
    <Card>
      <CardHeader>
        <CardTitle>Assertions</CardTitle>
        <CardDescription>Each bound and what was observed.</CardDescription>
      </CardHeader>
      <CardContent>
        {s.assertions.length ? (
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
              {s.assertions.map((a) => (
                <TableRow key={a.name}>
                  <TableCell className="font-mono text-xs">{a.name}</TableCell>
                  <TableCell className="tabular-nums">{a.expected}</TableCell>
                  <TableCell className="tabular-nums">{a.actual}</TableCell>
                  <TableCell>
                    <BoolBadge ok={a.passed} />
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        ) : (
          <Empty>
            {record.status === "running" ? "Evaluated when load ends." : "No assertions in this scenario."}
          </Empty>
        )}
      </CardContent>
    </Card>
  );
}

function Breakdown({
  snapshot,
  services,
}: {
  snapshot: LoadSnapshot;
  services?: Record<string, { total: number; failed: number }>;
}) {
  return (
    <div className="grid gap-4 xl:grid-cols-3">
      <Card size="sm">
        <CardHeader>
          <CardTitle>Per operation</CardTitle>
        </CardHeader>
        <CardContent>
          <Table>
            <TableHeader>
              <TableRow>
                <TableHead>Op</TableHead>
                <TableHead className="text-right">Sent</TableHead>
                <TableHead className="text-right">Failed</TableHead>
                <TableHead className="text-right">p50</TableHead>
                <TableHead className="text-right">p99</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {Object.entries(snapshot.per_op).map(([op, s]) => (
                <TableRow key={op}>
                  <TableCell className="font-mono text-xs">{op}</TableCell>
                  <TableCell className="text-right tabular-nums">{num(s.total)}</TableCell>
                  <TableCell className={`text-right tabular-nums ${s.failed ? "text-destructive" : ""}`}>
                    {num(s.failed)}
                  </TableCell>
                  <TableCell className="text-right tabular-nums">{ms(s.latency.p50_ms)}</TableCell>
                  <TableCell className="text-right tabular-nums">{ms(s.latency.p99_ms)}</TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
      <Card size="sm">
        <CardHeader>
          <CardTitle>Errors by class</CardTitle>
        </CardHeader>
        <CardContent>
          {Object.keys(snapshot.errors).length ? (
            <Table>
              <TableBody>
                {Object.entries(snapshot.errors).map(([k, v]) => (
                  <TableRow key={k}>
                    <TableCell className="font-mono text-xs">{k}</TableCell>
                    <TableCell className="text-right tabular-nums">{num(v)}</TableCell>
                  </TableRow>
                ))}
              </TableBody>
            </Table>
          ) : (
            <Empty>No failures.</Empty>
          )}
        </CardContent>
      </Card>
      <Card size="sm">
        <CardHeader>
          <CardTitle>Per target and service</CardTitle>
          <CardDescription>Client side per protocol; engine side from its counters.</CardDescription>
        </CardHeader>
        <CardContent>
          <Table>
            <TableBody>
              {Object.entries(snapshot.per_target).map(([k, v]) => (
                <TableRow key={`t-${k}`}>
                  <TableCell className="font-mono text-xs">{k}</TableCell>
                  <TableCell className="text-right tabular-nums">
                    {num(v.total)} sent, {num(v.failed)} failed
                  </TableCell>
                </TableRow>
              ))}
              {Object.entries(services ?? {}).map(([k, v]) => (
                <TableRow key={`s-${k}`}>
                  <TableCell className="font-mono text-xs">{k}</TableCell>
                  <TableCell className="text-right tabular-nums">
                    {num(v.total)} served, {num(v.failed)} failed
                  </TableCell>
                </TableRow>
              ))}
            </TableBody>
          </Table>
        </CardContent>
      </Card>
    </div>
  );
}

function ValidateView({ record }: { record: RunRecord }) {
  const r = record.validate!;
  return (
    <>
      <div className="grid gap-4 md:grid-cols-3">
        <StatCard label="Passed" value={r.passed} tone="good" />
        <StatCard label="Failed" value={r.failed} tone={r.failed ? "bad" : "good"} />
        <StatCard label="Took" value={seconds(record.duration_s)} hint={JSON.stringify(record.request)} />
      </div>
      <ChecksTable checks={r.checks} />
    </>
  );
}

export function ChecksTable({
  checks,
}: {
  checks: RunRecord["validate"] extends infer V ? (V extends { checks: infer C } ? C : never) : never;
}) {
  return (
    <div className="overflow-x-auto rounded-lg border">
      <Table>
        <TableHeader>
          <TableRow>
            <TableHead>Check</TableHead>
            <TableHead>Surface</TableHead>
            <TableHead />
            <TableHead className="text-right">Latency</TableHead>
            <TableHead>Detail</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {checks.map((c) => (
            <TableRow key={c.name}>
              <TableCell className="font-mono text-xs">{c.name}</TableCell>
              <TableCell className="text-muted-foreground">{c.surface}</TableCell>
              <TableCell>
                <BoolBadge ok={c.passed} yes="pass" no="fail" />
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
        </TableBody>
      </Table>
    </div>
  );
}
