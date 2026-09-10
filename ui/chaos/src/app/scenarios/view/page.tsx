"use client";

import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";
import { ArrowLeft, Play, Save, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { DetailList, PageTitle, StageBar } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { CheckReply, ScenarioDetail } from "@/lib/api/schema";
import { ago } from "@/lib/format";

const TEMPLATE = `# New scenario. Every table rejects unknown keys; docs/chaos/scenarios.md is the reference.
[scenario]
name = "my_scenario"
description = "What this proves"

[stack.engines.engine-1]
heartbeat = "100ms"

[stack.protocols.protocol-1]
engine = "engine-1"

[load]
rate = 100
duration = "3s"
warmup = "500ms"

[[load.operations]]
op = "rest_evaluate"
weight = 3

[[load.operations]]
op = "ws_echo"
weight = 1

[[timeline]]
at = "1s"
action = "set_behavior"
service = "engine-1"
behavior = { type = "error", kind = "unavailable", rate = 0.5 }

[[timeline]]
at = "2s"
action = "set_behavior"
service = "engine-1"
behavior = { type = "healthy" }

[assertions]
max_error_rate = 0.3
max_p99_ms = 100
min_requests = 200
`;

type Parsed = {
  scenario?: { name?: string; description?: string; skip?: boolean };
  stack?: { engines?: Record<string, unknown>; protocols?: Record<string, { engine?: string }> };
  load?: {
    rate?: number;
    duration?: string;
    warmup?: string;
    operations?: { op: string; weight: number }[];
    pattern?: { type: string };
  };
  timeline?: { at: string; action: string; service?: string; message?: string }[];
  assertions?: Record<string, unknown>;
};

export default function ScenarioViewPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <ScenarioView />
    </Suspense>
  );
}

/** The kit's order-detail layout: back arrow, big id with status chips, a stage strip, content plus a right rail. */
function ScenarioView() {
  const params = useSearchParams();
  const router = useRouter();
  const requested = params.get("id") ?? "new";
  const isNew = requested === "new";
  const [id, setId] = useState(isNew ? "" : requested);
  const [detail, setDetail] = useState<ScenarioDetail | null>(null);
  const [text, setText] = useState(isNew ? TEMPLATE : "");
  const [check, setCheck] = useState<CheckReply | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);

  useEffect(() => {
    if (isNew) return;
    api
      .scenario(requested)
      .then((d) => {
        setDetail(d);
        setText(d.text);
        setError(null);
      })
      .catch((e: unknown) => setError(describe(e)));
  }, [requested, isNew]);

  useEffect(() => {
    if (!text.trim()) return;
    const t = setTimeout(() => {
      api
        .scenarioCheck(text)
        .then(setCheck)
        .catch(() => setCheck(null));
    }, 400);
    return () => clearTimeout(t);
  }, [text]);

  const parsed = (check?.parsed ?? null) as Parsed | null;
  const dirty = detail ? text !== detail.text : true;

  const save = async () => {
    if (!id.trim()) {
      toast.error("give the scenario an id, e.g. ws/burst");
      return;
    }
    setBusy(true);
    try {
      await api.scenarioSave(id.trim(), text);
      toast.success(`saved scenarios/${id.trim()}.toml`);
      if (isNew) router.replace(`/scenarios/view/?id=${encodeURIComponent(id.trim())}`);
      else setDetail(await api.scenario(id));
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const run = async () => {
    setBusy(true);
    try {
      if (dirty) await api.scenarioSave(id.trim(), text);
      const s = await api.runScenario(id.trim());
      router.push(`/runs/view/?id=${s.id}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (!confirm(`Delete scenarios/${id}.toml?`)) return;
    try {
      await api.scenarioDelete(id);
      router.push("/scenarios/");
    } catch (e) {
      toast.error(describe(e));
    }
  };

  const timeline = parsed?.timeline ?? [];
  const engines = Object.keys(parsed?.stack?.engines ?? {});
  const protocols = Object.entries(parsed?.stack?.protocols ?? {});
  const load = parsed?.load;
  const assertions = Object.entries(parsed?.assertions ?? {}).filter(
    ([k, v]) => k !== "services" && v !== null,
  );
  const serviceAssertions = Object.entries(
    (parsed?.assertions?.services as Record<string, Record<string, unknown>> | undefined) ?? {},
  );

  return (
    <>
      <PageTitle
        back={
          <Button variant="outline" size="icon" render={<Link href="/scenarios/" />} aria-label="Back">
            <ArrowLeft />
          </Button>
        }
        title={
          <span className="flex flex-wrap items-center gap-3">
            {isNew ? "New scenario" : (parsed?.scenario?.name ?? detail?.name ?? requested)}
            {check ? (
              check.ok ? (
                <Badge variant="outline" className="text-emerald-600 dark:text-emerald-400">
                  checks
                </Badge>
              ) : (
                <Badge variant="outline" className="text-destructive">
                  does not check
                </Badge>
              )
            ) : null}
            {parsed?.scenario?.skip ? <Badge variant="outline">skipped in CI</Badge> : null}
            {dirty && !isNew ? <Badge variant="outline">unsaved</Badge> : null}
          </span>
        }
        description={
          isNew ? (
            "Write the TOML, it is checked as you type; save it into the scenarios directory, then run it."
          ) : (
            <>
              <span className="font-mono">{detail?.file ?? requested}</span>
              {parsed?.scenario?.description ? ` · ${parsed.scenario.description}` : ""}
              {detail?.last_run ? ` · last run ${ago(detail.last_run.started_at)}` : ""}
            </>
          )
        }
      >
        {!isNew ? (
          <Button variant="ghost" size="sm" onClick={remove}>
            <Trash2 /> Delete
          </Button>
        ) : null}
        <Button variant="outline" size="sm" onClick={save} disabled={busy || !check?.ok}>
          <Save /> {isNew ? "Save" : dirty ? "Save changes" : "Saved"}
        </Button>
        <Button size="sm" onClick={run} disabled={busy || !check?.ok || !id.trim()}>
          <Play /> {dirty ? "Save and run" : "Run"}
        </Button>
      </PageTitle>
      {error ? <p className="text-sm text-destructive">{error}</p> : null}

      <div className="grid gap-6 xl:grid-cols-[1fr_20rem]">
        <div className="grid content-start gap-6">
          <div className="rounded-xl border bg-muted/30 p-4">
            <div className="mb-3 flex flex-wrap justify-between gap-2 text-sm">
              <span>
                Stack <b>{engines.length}</b> engine{engines.length === 1 ? "" : "s"},{" "}
                <b>{protocols.length}</b> protocol{protocols.length === 1 ? "" : "s"}
              </span>
              <span className="text-muted-foreground">
                {load
                  ? `${load.rate ?? 50} req/s for ${load.duration ?? "?"}${load.warmup ? `, warmup ${load.warmup}` : ""}`
                  : "no load, timeline only"}
              </span>
            </div>
            <StageBar
              stages={["Setup", load?.warmup ? "Warmup" : "Ready", "Load + timeline", "Assert", "Teardown"]}
              current={-1}
            />
            {timeline.length ? (
              <ol className="mt-4 grid gap-1 text-xs text-muted-foreground">
                {timeline.map((t, i) => (
                  <li key={i} className="flex gap-3">
                    <span className="w-14 font-mono tabular-nums">{t.at}</span>
                    <span className="font-mono">
                      {t.action}
                      {t.service ? ` ${t.service}` : ""}
                      {t.message ? ` "${t.message}"` : ""}
                    </span>
                  </li>
                ))}
              </ol>
            ) : null}
          </div>

          {isNew ? (
            <div className="grid gap-1.5">
              <span className="text-xs text-muted-foreground">Id, becomes scenarios/&lt;id&gt;.toml</span>
              <Input
                value={id}
                onChange={(e) => setId(e.target.value)}
                placeholder="ws/burst"
                className="max-w-sm font-mono text-xs"
              />
            </div>
          ) : null}
          {check && !check.ok ? (
            <div className="rounded-lg border border-destructive/30 bg-destructive/5 px-3 py-2 text-sm text-destructive">
              {check.error}
            </div>
          ) : null}
          <Textarea
            value={text}
            onChange={(e) => setText(e.target.value)}
            spellCheck={false}
            className="min-h-[34rem] font-mono text-xs leading-5"
          />
        </div>

        <div className="grid content-start gap-4">
          {detail?.last_run ? (
            <Card size="sm">
              <CardHeader>
                <CardTitle>Last run</CardTitle>
                <CardDescription>{ago(detail.last_run.started_at)}</CardDescription>
              </CardHeader>
              <CardContent className="flex items-center justify-between">
                <StatusBadge status={detail.last_run.status} />
                <Button
                  variant="outline"
                  size="xs"
                  render={<Link href={`/runs/view/?id=${detail.last_run.id}`} />}
                >
                  Open
                </Button>
              </CardContent>
            </Card>
          ) : null}
          <Card size="sm">
            <CardHeader>
              <CardTitle>Stack</CardTitle>
            </CardHeader>
            <CardContent>
              <DetailList
                rows={[
                  ...engines.map((e) => ({ k: e, v: "engine" })),
                  ...protocols.map(([p, spec]) => ({ k: p, v: `protocol → ${spec.engine ?? "?"}` })),
                ]}
              />
            </CardContent>
          </Card>
          <Card size="sm">
            <CardHeader>
              <CardTitle>Load</CardTitle>
            </CardHeader>
            <CardContent>
              {load ? (
                <DetailList
                  rows={[
                    { k: "Rate", v: `${load.rate ?? 50} req/s` },
                    { k: "Duration", v: load.duration ?? "–" },
                    { k: "Pattern", v: load.pattern?.type ?? "constant" },
                    ...(load.operations ?? []).map((o) => ({ k: o.op, v: `weight ${o.weight}` })),
                  ]}
                />
              ) : (
                <p className="text-sm text-muted-foreground">No [load] table.</p>
              )}
            </CardContent>
          </Card>
          <Card size="sm">
            <CardHeader>
              <CardTitle>Assertions</CardTitle>
            </CardHeader>
            <CardContent>
              {assertions.length || serviceAssertions.length ? (
                <DetailList
                  rows={[
                    ...assertions.map(([k, v]) => ({ k, v: String(v) })),
                    ...serviceAssertions.flatMap(([svc, m]) =>
                      Object.entries(m)
                        .filter(([, v]) => v !== null && v !== undefined)
                        .map(([k, v]) => ({ k: `${svc}.${k}`, v: String(v) })),
                    ),
                  ]}
                />
              ) : (
                <p className="text-sm text-muted-foreground">
                  No assertions; the run passes if nothing breaks.
                </p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>
    </>
  );
}
