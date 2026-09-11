"use client";

import { Play } from "lucide-react";
import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { useEffect } from "react";
import { toast } from "sonner";

import { useChaos } from "@/app/providers";
import { PageTitle, Sparkline, StatRow } from "@/components/kit";
import { Hint, LoadShapeFields, ShapeField } from "@/components/load-shape";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { InputGroup, InputGroupInput } from "@/components/ui/input-group";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { RunRecord } from "@/lib/api/schema";
import { ago, ms, num, pct } from "@/lib/format";
import { DEFAULT_SHAPE, loadProblem, secondsOf, toLoadRequest, weightedOps } from "@/lib/load";

/** The kit's delivery simulator: a policy rail on the left, the window and past results on the right. */
export default function LoadPage() {
  const router = useRouter();
  const { overview, kinds, lastEvent } = useChaos();
  const recent = useFetch(() => api.runs(200), 5000, [lastEvent]);
  const [name, setName] = useState("adhoc");
  const [shape, setShape] = useState(DEFAULT_SHAPE);
  const [busy, setBusy] = useState(false);

  const dur = secondsOf(shape.duration);
  const r = Number(shape.rate) || 0;
  const expected = shape.ramp
    ? Math.round(((Number(shape.startRate) + Number(shape.endRate)) / 2) * dur)
    : Math.round(r * dur);
  const concurrency = Math.round((shape.ramp ? Number(shape.endRate) : r) * 0.01); // 10 ms per request at loopback
  const capped = concurrency > (Number(shape.maxInFlight) || 256);
  const problem = loadProblem(shape, overview, kinds);
  const request = () => toLoadRequest(shape, name, overview);

  const start = async () => {
    setBusy(true);
    try {
      const s = await api.runLoad(request());
      toast.success(`started ${s.name}`);
      router.push(`/runs/view/?id=${s.id}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const loadRuns = (recent.data ?? []).filter((x) => x.kind === "load").slice(0, 8);

  return (
    <>
      <PageTitle
        title="Run new load"
        description="Model the shape before you push it: open-loop rate, duration, mix, targets, concurrency cap. Then watch it live."
      >
        <Button onClick={start} disabled={busy || problem !== null} title={problem ?? undefined}>
          <Play /> Run load
        </Button>
      </PageTitle>

      <div className="grid gap-8 xl:grid-cols-[22rem_1fr]">
        <div className="grid content-start gap-6">
          <section>
            <div className="grid gap-3 rounded-xl border p-4">
              <ShapeField label="Name" help="Shown in the run list.">
                <InputGroup>
                  <InputGroupInput value={name} onChange={(e) => setName(e.target.value)} />
                </InputGroup>
              </ShapeField>
              <div className="border-t pt-3 text-sm">
                <div className="text-muted-foreground text-xs">Expected</div>
                <div className="tabular-nums">
                  ≈ {num(expected)} requests ·{" "}
                  {capped ? (
                    <span className="text-destructive">cap reached at ~10 ms latency</span>
                  ) : (
                    `~${concurrency || 1} in flight at 10 ms`
                  )}
                </div>
                {problem ? <div className="text-destructive mt-1 text-xs">{problem}</div> : null}
              </div>
            </div>
          </section>
          <LoadShapeFields shape={shape} onChange={setShape} />
        </div>

        <div className="grid content-start gap-8">
          <section>
            <h2 className="mb-1 flex items-center gap-2 text-base font-semibold">
              Load window <Hint text="What this shape means before anything runs." />
            </h2>
            <StatRow
              items={[
                { label: "Warmup", value: shape.warmup || "none" },
                { label: "Measured", value: shape.duration || "–" },
                {
                  label: "Requests",
                  value: `≈ ${num(expected)}`,
                },
                {
                  label: "Mix",
                  value: `${weightedOps(shape).length} ops`,
                },
              ]}
            />
          </section>
          <section>
            <h2 className="mb-1 flex items-center gap-2 text-base font-semibold">
              Previous load runs <Hint text="The last runs of this kind, with their per-second throughput." />
            </h2>
            <div className="overflow-x-auto rounded-xl border">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead>Run</TableHead>
                    <TableHead>Status</TableHead>
                    <TableHead className="text-right">Requests</TableHead>
                    <TableHead className="text-right">Errors</TableHead>
                    <TableHead className="text-right">p99</TableHead>
                    <TableHead>Throughput</TableHead>
                    <TableHead>When</TableHead>
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {loadRuns.map((run) => (
                    <TableRow key={run.id}>
                      <TableCell>
                        <Link href={`/runs/view/?id=${run.id}`} className="font-medium hover:underline">
                          {run.name}
                        </Link>
                      </TableCell>
                      <TableCell>
                        <StatusBadge status={run.status} />
                      </TableCell>
                      <TableCell className="text-right tabular-nums">{num(run.requests_total)}</TableCell>
                      <TableCell className="text-right tabular-nums">{pct(run.error_rate)}</TableCell>
                      <TableCell className="text-right tabular-nums">{ms(run.p99_ms)}</TableCell>
                      <TableCell>
                        <RunSpark id={run.id} />
                      </TableCell>
                      <TableCell className="text-muted-foreground">{ago(run.started_at)}</TableCell>
                    </TableRow>
                  ))}
                  {!loadRuns.length ? (
                    <TableRow>
                      <TableCell colSpan={7} className="text-muted-foreground py-8 text-center">
                        No load runs yet.
                      </TableCell>
                    </TableRow>
                  ) : null}
                </TableBody>
              </Table>
            </div>
          </section>

          <Card>
            <CardHeader>
              <CardTitle>Request body</CardTitle>
            </CardHeader>
            <CardContent>
              <pre className="bg-muted overflow-x-auto rounded-lg p-3 text-xs">
                {JSON.stringify(request(), null, 1)}
              </pre>
            </CardContent>
          </Card>
        </div>
      </div>
    </>
  );
}

/** Per-second throughput of a finished load run, as a tiny bar chart. */
function RunSpark({ id }: { id: string }) {
  const [record, setRecord] = useState<RunRecord | null>(null);
  useEffect(() => {
    api
      .run(id)
      .then(setRecord)
      .catch(() => setRecord(null));
  }, [id]);
  const s = record?.samples ?? [];
  if (!s.length) return <span className="text-muted-foreground text-xs">–</span>;
  const values = s.map((x, i) => {
    const p = i > 0 ? s[i - 1] : null;
    const dt = p ? x.elapsed_s - p.elapsed_s : x.elapsed_s;
    return dt > 0 ? (x.requests_total - (p?.requests_total ?? 0)) / dt : x.throughput_rps;
  });
  return <Sparkline values={values} />;
}
