"use client";

import { useRouter } from "next/navigation";
import { useState } from "react";
import { Play } from "lucide-react";
import { toast } from "sonner";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Field } from "@/components/field";
import { PageHeader } from "@/components/page-header";
import { RunsTable } from "@/components/runs-table";
import { useChaos } from "@/components/shell/providers";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import { OpKind, type LoadRequest } from "@/lib/api/schema";

const OPS = OpKind.options;

/** An ad-hoc load run: the `[load]` table as a form, against the serve stack or any URL. */
export default function LoadPage() {
  const router = useRouter();
  const { overview } = useChaos();
  const recent = useFetch(() => api.runs(100), 5000);
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

  const request = (): LoadRequest => ({
    name: name || undefined,
    targets: targetMode === "url" && targetUrl ? [{ name: "url", http_url: targetUrl }] : [],
    load: {
      rate: Number(rate),
      duration,
      warmup: warmup || undefined,
      timeout: timeout || undefined,
      max_in_flight: Number(maxInFlight) || undefined,
      pattern: ramp
        ? {
            type: "ramp",
            start_rate: Number(startRate),
            end_rate: Number(endRate),
          }
        : undefined,
      operations: OPS.map((op) => ({
        op,
        weight: Number(weights[op] ?? 0),
      })).filter((o) => o.weight > 0),
    },
  });

  const start = async () => {
    setBusy(true);
    try {
      const summary = await api.runLoad(request());
      toast.success(`started ${summary.name}`);
      router.push(`/runs/view/?id=${summary.id}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  return (
    <>
      <PageHeader
        title="Load"
        description="Open-loop load with live throughput, percentiles and error classes. Against the serve stack, or any protocol URL such as Envoy in the cluster."
      >
        <Button
          size="sm"
          onClick={start}
          disabled={
            busy || (targetMode === "url" && !targetUrl) || (targetMode === "stack" && !stackProtocols.length)
          }
        >
          <Play /> start
        </Button>
      </PageHeader>
      <div className="grid gap-4 xl:grid-cols-3">
        <Card>
          <CardHeader>
            <CardTitle>Shape</CardTitle>
            <CardDescription>The same keys as a scenario&apos;s [load] table.</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3">
            <Field label="Name">
              <Input value={name} onChange={(e) => setName(e.target.value)} />
            </Field>
            <div className="grid grid-cols-2 gap-3">
              <Field label="Rate (req/s)">
                <Input
                  value={rate}
                  onChange={(e) => setRate(e.target.value)}
                  inputMode="numeric"
                  disabled={ramp}
                />
              </Field>
              <Field label="Duration">
                <Input value={duration} onChange={(e) => setDuration(e.target.value)} placeholder="10s" />
              </Field>
              <Field label="Warmup">
                <Input value={warmup} onChange={(e) => setWarmup(e.target.value)} placeholder="500ms" />
              </Field>
              <Field label="Timeout">
                <Input value={timeout} onChange={(e) => setTimeoutValue(e.target.value)} placeholder="5s" />
              </Field>
              <Field label="Max in flight">
                <Input
                  value={maxInFlight}
                  onChange={(e) => setMaxInFlight(e.target.value)}
                  inputMode="numeric"
                />
              </Field>
              <Field label="Pattern">
                <select
                  className="h-8 w-full rounded-md border bg-background px-2 text-sm"
                  value={ramp ? "ramp" : "constant"}
                  onChange={(e) => setRamp(e.target.value === "ramp")}
                >
                  <option value="constant">constant</option>
                  <option value="ramp">ramp</option>
                </select>
              </Field>
              {ramp ? (
                <>
                  <Field label="Start rate">
                    <Input
                      value={startRate}
                      onChange={(e) => setStartRate(e.target.value)}
                      inputMode="numeric"
                    />
                  </Field>
                  <Field label="End rate">
                    <Input value={endRate} onChange={(e) => setEndRate(e.target.value)} inputMode="numeric" />
                  </Field>
                </>
              ) : null}
            </div>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Operations</CardTitle>
            <CardDescription>Relative weights; zero leaves an operation out.</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3">
            {OPS.map((op) => (
              <Field key={op} label={op}>
                <Input
                  value={weights[op] ?? "0"}
                  onChange={(e) =>
                    setWeights({
                      ...weights,
                      [op]: e.target.value,
                    })
                  }
                  inputMode="numeric"
                />
              </Field>
            ))}
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Target</CardTitle>
            <CardDescription>Where requests go.</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-3">
            <Field label="Mode">
              <select
                className="h-8 w-full rounded-md border bg-background px-2 text-sm"
                value={targetMode}
                onChange={(e) => setTargetMode(e.target.value as "stack" | "url")}
              >
                <option value="stack">
                  serve stack ({stackProtocols.map((p) => p.name).join(", ") || "no protocol running"})
                </option>
                <option value="url">a protocol URL</option>
              </select>
            </Field>
            {targetMode === "url" ? (
              <Field label="Protocol base URL">
                <Input
                  value={targetUrl}
                  onChange={(e) => setTargetUrl(e.target.value)}
                  placeholder={overview?.config.targets.protocol ?? "http://localhost:18080"}
                />
              </Field>
            ) : null}
            <p className="text-xs text-muted-foreground">
              Through Envoy the URL is the edge: locally <code>http://localhost:18080</code> from a serve on
              this machine, or <code>{overview?.config.targets.protocol ?? "http://envoy:8080"}</code> from
              the serve inside the cluster.
            </p>
            <pre className="overflow-x-auto rounded-md bg-muted p-2 text-xs">
              {JSON.stringify(request(), null, 1)}
            </pre>
          </CardContent>
        </Card>
      </div>
      <Card>
        <CardHeader>
          <CardTitle>Previous load runs</CardTitle>
        </CardHeader>
        <CardContent>
          <RunsTable runs={(recent.data ?? []).filter((r) => r.kind === "load")} />
        </CardContent>
      </Card>
    </>
  );
}
