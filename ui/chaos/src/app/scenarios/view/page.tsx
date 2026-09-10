"use client";

import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useState } from "react";
import { Play, Save, Trash2 } from "lucide-react";
import { toast } from "sonner";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Skeleton } from "@/components/ui/skeleton";
import { Textarea } from "@/components/ui/textarea";
import { ErrorNote, PageHeader } from "@/components/page-header";
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

export default function ScenarioViewPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <ScenarioView />
    </Suspense>
  );
}

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

  // Check on every pause in typing; never writes.
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
      else {
        const d = await api.scenario(id);
        setDetail(d);
      }
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
      const summary = await api.runScenario(id.trim());
      router.push(`/runs/view/?id=${summary.id}`);
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
      toast.success("deleted");
      router.push("/scenarios/");
    } catch (e) {
      toast.error(describe(e));
    }
  };

  return (
    <>
      <PageHeader
        title={isNew ? "New scenario" : (detail?.name ?? requested)}
        description={
          isNew
            ? "Write the TOML, check it as you type, save it into the scenarios directory, run it."
            : detail?.description || detail?.file
        }
      >
        {!isNew ? (
          <Button variant="outline" size="sm" onClick={remove}>
            <Trash2 /> delete
          </Button>
        ) : null}
        <Button variant="outline" size="sm" onClick={save} disabled={busy || !check?.ok}>
          <Save /> {isNew ? "save" : dirty ? "save changes" : "saved"}
        </Button>
        <Button size="sm" onClick={run} disabled={busy || !check?.ok || !id.trim()}>
          <Play /> {dirty ? "save and run" : "run"}
        </Button>
      </PageHeader>
      <ErrorNote message={error} />
      <div className="grid gap-4 xl:grid-cols-3">
        <Card className="xl:col-span-2">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              {isNew ? (
                <Input
                  value={id}
                  onChange={(e) => setId(e.target.value)}
                  placeholder="id, e.g. ws/burst → scenarios/ws/burst.toml"
                  className="max-w-sm font-mono text-xs"
                />
              ) : (
                <span className="font-mono text-xs">{detail?.file ?? requested}</span>
              )}
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
            </CardTitle>
            {check && !check.ok ? (
              <CardDescription className="text-destructive">{check.error}</CardDescription>
            ) : null}
          </CardHeader>
          <CardContent>
            <Textarea
              value={text}
              onChange={(e) => setText(e.target.value)}
              spellCheck={false}
              className="min-h-[32rem] font-mono text-xs leading-5"
            />
          </CardContent>
        </Card>
        <div className="grid content-start gap-4">
          {detail?.last_run ? (
            <Card size="sm">
              <CardHeader>
                <CardTitle>Last run</CardTitle>
                <CardDescription>{ago(detail.last_run.started_at)}</CardDescription>
              </CardHeader>
              <CardContent className="flex items-center gap-2 text-sm">
                <StatusBadge status={detail.last_run.status} />
                <Link className="underline" href={`/runs/view/?id=${detail.last_run.id}`}>
                  open
                </Link>
              </CardContent>
            </Card>
          ) : null}
          <Card size="sm">
            <CardHeader>
              <CardTitle>What a scenario proves</CardTitle>
              <CardDescription>Read it top down.</CardDescription>
            </CardHeader>
            <CardContent className="grid gap-2 text-sm text-muted-foreground">
              <p>
                <b className="text-foreground">[stack]</b> is what gets started, on free ports. Names are what
                the timeline and the assertions refer to.
              </p>
              <p>
                <b className="text-foreground">[load]</b> is open loop: the rate is held whatever the latency
                does, up to <code>max_in_flight</code>.
              </p>
              <p>
                <b className="text-foreground">[[timeline]]</b> fires at offsets from load start:{" "}
                <code>set_behavior</code>, <code>stop</code>, <code>start</code>, <code>log</code>.
              </p>
              <p>
                <b className="text-foreground">[assertions]</b> are bounds on the whole window; per-service
                ones use the engine&apos;s own counters.
              </p>
              <p>
                Bounds come from what the scenario proves, not the fastest machine: loopback p99 is about 3
                ms, CI is slower.
              </p>
            </CardContent>
          </Card>
          {check?.parsed ? (
            <Card size="sm">
              <CardHeader>
                <CardTitle>Parsed</CardTitle>
                <CardDescription>What the executor will run.</CardDescription>
              </CardHeader>
              <CardContent>
                <pre className="max-h-80 overflow-auto rounded-md bg-muted p-2 text-xs">
                  {JSON.stringify(check.parsed, null, 1)}
                </pre>
              </CardContent>
            </Card>
          ) : null}
        </div>
      </div>
    </>
  );
}
