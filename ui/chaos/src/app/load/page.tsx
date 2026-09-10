"use client";

import Link from "next/link";
import { useRouter } from "next/navigation";
import { useState } from "react";
import { Info, Play } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardHeader, CardTitle } from "@/components/ui/card";
import { InputGroup, InputGroupAddon, InputGroupInput, InputGroupText } from "@/components/ui/input-group";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { Tooltip, TooltipContent, TooltipTrigger } from "@/components/ui/tooltip";
import { PageTitle, Sparkline, StatRow } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { useChaos } from "@/components/shell/providers";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import { OpKind, type LoadRequest, type RunRecord } from "@/lib/api/schema";
import { ago, ms, num, pct } from "@/lib/format";
import { useEffect } from "react";

const OPS = OpKind.options;

function secondsOf(s: string): number {
  const m = /^(\d+(?:\.\d+)?)\s*(ms|s|m|h)?$/.exec(s.trim());
  if (!m) return 0;
  const n = Number(m[1]);
  return m[2] === "ms" ? n / 1000 : m[2] === "m" ? n * 60 : m[2] === "h" ? n * 3600 : n;
}

/** The kit's delivery simulator: a policy rail on the left, the window and past results on the right. */
export default function LoadPage() {
  const router = useRouter();
  const { overview, lastEvent } = useChaos();
  const recent = useFetch(() => api.runs(200), 5000, [lastEvent]);
  const [name, setName] = useState("adhoc");
  const [rate, setRate] = useState("200");
  const [duration, setDuration] = useState("10s");
  const [warmup, setWarmup] = useState("500ms");
  const [timeout, setTimeoutValue] = useState("5s");
  const [maxInFlight, setMaxInFlight] = useState("256");
  const [ramp, setRamp] = useState(false);
  const [startRate, setStartRate] = useState("10");
  const [endRate, setEndRate] = useState("500");
  const [weights, setWeights] = useState<Record<string, string>>({
    rest_evaluate: "4",
    graphql_evaluate: "2",
    ws_echo: "2",
    grpc_ping: "1",
  });
  const [targetMode, setTargetMode] = useState<"stack" | "url">("stack");
  const [targetUrl, setTargetUrl] = useState("");
  const [busy, setBusy] = useState(false);

  const stackProtocols = (overview?.stack ?? []).filter((i) => i.kind === "protocol" && i.running);
  const dur = secondsOf(duration);
  const r = Number(rate) || 0;
  const expected = ramp ? Math.round(((Number(startRate) + Number(endRate)) / 2) * dur) : Math.round(r * dur);
  const concurrency = Math.round((ramp ? Number(endRate) : r) * 0.01); // 10 ms per request at loopback
  const capped = concurrency > (Number(maxInFlight) || 256);
  const totalWeight = OPS.reduce((n, op) => n + (Number(weights[op]) || 0), 0);

  const request = (): LoadRequest => ({
    name: name || undefined,
    targets: targetMode === "url" && targetUrl ? [{ name: "url", http_url: targetUrl }] : [],
    load: {
      rate: r,
      duration,
      warmup: warmup || undefined,
      timeout: timeout || undefined,
      max_in_flight: Number(maxInFlight) || undefined,
      pattern: ramp ? { type: "ramp", start_rate: Number(startRate), end_rate: Number(endRate) } : undefined,
      operations: OPS.map((op) => ({ op, weight: Number(weights[op] ?? 0) })).filter((o) => o.weight > 0),
    },
  });

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
        description="Model the shape before you push it: open-loop rate, duration, mix, concurrency cap. Then watch it live."
      >
        <Button
          onClick={start}
          disabled={
            busy ||
            (targetMode === "url" && !targetUrl) ||
            (targetMode === "stack" && !stackProtocols.length) ||
            !totalWeight
          }
        >
          <Play /> Run load
        </Button>
      </PageTitle>

      <div className="grid gap-8 xl:grid-cols-[22rem_1fr]">
        <div className="grid content-start gap-6">
          <section>
            <h2 className="mb-3 text-base font-semibold">Shape</h2>
            <div className="grid gap-3 rounded-xl border p-4">
              <Field label="Name" help="Shown in the run list.">
                <InputGroup>
                  <InputGroupInput value={name} onChange={(e) => setName(e.target.value)} />
                </InputGroup>
              </Field>
              <Field
                label="Pattern"
                help="Constant holds the rate; ramp interpolates from start to end over the duration."
              >
                <select
                  className="h-8 w-full rounded-lg border bg-background px-2 text-sm"
                  value={ramp ? "ramp" : "constant"}
                  onChange={(e) => setRamp(e.target.value === "ramp")}
                >
                  <option value="constant">Constant</option>
                  <option value="ramp">Ramp</option>
                </select>
              </Field>
              {ramp ? (
                <div className="grid grid-cols-2 gap-3">
                  <Field label="Start rate" help="Requests per second at t = 0.">
                    <InputGroup>
                      <InputGroupInput
                        value={startRate}
                        onChange={(e) => setStartRate(e.target.value)}
                        inputMode="numeric"
                      />
                      <InputGroupAddon align="inline-end">
                        <InputGroupText>req/s</InputGroupText>
                      </InputGroupAddon>
                    </InputGroup>
                  </Field>
                  <Field label="End rate" help="Requests per second at the end.">
                    <InputGroup>
                      <InputGroupInput
                        value={endRate}
                        onChange={(e) => setEndRate(e.target.value)}
                        inputMode="numeric"
                      />
                      <InputGroupAddon align="inline-end">
                        <InputGroupText>req/s</InputGroupText>
                      </InputGroupAddon>
                    </InputGroup>
                  </Field>
                </div>
              ) : (
                <Field
                  label="Rate"
                  help="Requests per second across every target, open loop: latency does not slow the pacer."
                >
                  <InputGroup>
                    <InputGroupInput
                      value={rate}
                      onChange={(e) => setRate(e.target.value)}
                      inputMode="numeric"
                    />
                    <InputGroupAddon align="inline-end">
                      <InputGroupText>req/s</InputGroupText>
                    </InputGroupAddon>
                  </InputGroup>
                </Field>
              )}
              <div className="grid grid-cols-2 gap-3">
                <Field label="Duration" help="Measured window, excluding warmup.">
                  <InputGroup>
                    <InputGroupInput
                      value={duration}
                      onChange={(e) => setDuration(e.target.value)}
                      placeholder="10s"
                    />
                  </InputGroup>
                </Field>
                <Field label="Warmup" help="Same load first; its numbers are discarded.">
                  <InputGroup>
                    <InputGroupInput
                      value={warmup}
                      onChange={(e) => setWarmup(e.target.value)}
                      placeholder="500ms"
                    />
                  </InputGroup>
                </Field>
                <Field label="Timeout" help="Per request; a timeout counts as a failure.">
                  <InputGroup>
                    <InputGroupInput
                      value={timeout}
                      onChange={(e) => setTimeoutValue(e.target.value)}
                      placeholder="5s"
                    />
                  </InputGroup>
                </Field>
                <Field label="Max in flight" help="Concurrency cap; the pacer stalls when reached.">
                  <InputGroup>
                    <InputGroupInput
                      value={maxInFlight}
                      onChange={(e) => setMaxInFlight(e.target.value)}
                      inputMode="numeric"
                    />
                  </InputGroup>
                </Field>
              </div>
              <div className="border-t pt-3 text-sm">
                <div className="text-xs text-muted-foreground">Expected</div>
                <div className="tabular-nums">
                  ≈ {num(expected)} requests ·{" "}
                  {capped ? (
                    <span className="text-destructive">cap reached at ~10 ms latency</span>
                  ) : (
                    `~${concurrency || 1} in flight at 10 ms`
                  )}
                </div>
              </div>
            </div>
          </section>

          <section>
            <h2 className="mb-3 text-base font-semibold">Operations</h2>
            <div className="grid gap-3 rounded-xl border p-4">
              {OPS.map((op) => (
                <Field key={op} label={op} help="Relative weight in the mix; zero leaves it out.">
                  <InputGroup>
                    <InputGroupInput
                      value={weights[op] ?? "0"}
                      onChange={(e) => setWeights({ ...weights, [op]: e.target.value })}
                      inputMode="numeric"
                    />
                    <InputGroupAddon align="inline-end">
                      <InputGroupText>
                        {totalWeight
                          ? `${Math.round(((Number(weights[op]) || 0) / totalWeight) * 100)}%`
                          : "–"}
                      </InputGroupText>
                    </InputGroupAddon>
                  </InputGroup>
                </Field>
              ))}
            </div>
          </section>

          <section>
            <h2 className="mb-3 text-base font-semibold">Target</h2>
            <div className="grid gap-3 rounded-xl border p-4">
              <Field
                label="Where"
                help="The serve stack's protocols, or any protocol base URL such as Envoy."
              >
                <select
                  className="h-8 w-full rounded-lg border bg-background px-2 text-sm"
                  value={targetMode}
                  onChange={(e) => setTargetMode(e.target.value as "stack" | "url")}
                >
                  <option value="stack">
                    Serve stack ({stackProtocols.map((p) => p.name).join(", ") || "no protocol running"})
                  </option>
                  <option value="url">A protocol URL</option>
                </select>
              </Field>
              {targetMode === "url" ? (
                <Field
                  label="Protocol base URL"
                  help="From the chaos pod inside the cluster this is Envoy; from a serve on your machine it is the edge."
                >
                  <InputGroup>
                    <InputGroupInput
                      value={targetUrl}
                      onChange={(e) => setTargetUrl(e.target.value)}
                      placeholder={overview?.config.targets.protocol ?? "http://localhost:18080"}
                    />
                  </InputGroup>
                </Field>
              ) : null}
            </div>
          </section>
        </div>

        <div className="grid content-start gap-8">
          <section>
            <h2 className="mb-1 flex items-center gap-2 text-base font-semibold">
              Load window <Hint text="What this shape means before anything runs." />
            </h2>
            <StatRow
              items={[
                { label: "Warmup", value: warmup || "none" },
                { label: "Measured", value: duration || "–" },
                { label: "Requests", value: `≈ ${num(expected)}` },
                { label: "Mix", value: `${OPS.filter((op) => Number(weights[op]) > 0).length} ops` },
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
                      <TableCell colSpan={7} className="py-8 text-center text-muted-foreground">
                        No load runs yet.
                      </TableCell>
                    </TableRow>
                  ) : null}
                </TableBody>
              </Table>
            </div>
          </section>

          <Card size="sm">
            <CardHeader>
              <CardTitle>Request body</CardTitle>
            </CardHeader>
            <CardContent>
              <pre className="overflow-x-auto rounded-lg bg-muted p-3 text-xs">
                {JSON.stringify(request(), null, 1)}
              </pre>
            </CardContent>
          </Card>
        </div>
      </div>
    </>
  );
}

function Field({ label, help, children }: { label: string; help?: string; children: React.ReactNode }) {
  return (
    <label className="grid gap-1.5">
      <span className="flex items-center gap-1 text-sm font-medium">
        {label}
        {help ? <Hint text={help} /> : null}
      </span>
      {children}
    </label>
  );
}

function Hint({ text }: { text: string }) {
  return (
    <Tooltip>
      <TooltipTrigger render={<span className="inline-flex" />}>
        <Info className="size-3.5 text-muted-foreground" />
      </TooltipTrigger>
      <TooltipContent>{text}</TooltipContent>
    </Tooltip>
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
  if (!s.length) return <span className="text-xs text-muted-foreground">–</span>;
  const values = s.map((x, i) => {
    const p = i > 0 ? s[i - 1] : null;
    const dt = p ? x.elapsed_s - p.elapsed_s : x.elapsed_s;
    return dt > 0 ? (x.requests_total - (p?.requests_total ?? 0)) / dt : x.throughput_rps;
  });
  return <Sparkline values={values} />;
}
