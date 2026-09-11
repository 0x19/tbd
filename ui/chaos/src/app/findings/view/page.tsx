"use client";

import { ArrowLeft, RotateCcw, Trash2 } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useState } from "react";
import { toast } from "sonner";

import { DetailList, LevelChip, PageTitle, StatRow } from "@/components/kit";
import { BoolBadge } from "@/components/status-badge";
import { StressTargetDialog } from "@/components/stress-target";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from "@/components/ui/table";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import type { Step } from "@/lib/api/schema";
import { ago, when } from "@/lib/format";

export default function FindingViewPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <FindingView />
    </Suspense>
  );
}

const json = (v: unknown) => JSON.stringify(v, null, 2);
const compact = (v: unknown, max = 160) => {
  const s = JSON.stringify(v);
  return s.length > max ? `${s.slice(0, max)}…` : s;
};

/** The request of a step without its `op`, one line. */
function requestOf(step: Step): string {
  const { op: _op, ...rest } = step.request;
  return compact(rest);
}

function outcomeOf(step: Step): { text: string; level: "ok" | "error" | "warn" } {
  if (step.error) {
    const e = step.error;
    const text =
      e.kind === "status"
        ? `${e.code}: ${e.message}`
        : e.kind === "timeout"
          ? "timeout"
          : `transport: ${String((e as { kind: string } & Record<string, unknown>)[0] ?? "")}`.trim();
    return { text, level: step.tolerated ? "warn" : "error" };
  }
  return { text: step.response === undefined ? "" : compact(step.response), level: "ok" };
}

/** One finding: what broke, where, the trace that got there, and the replays since. */
function FindingView() {
  const id = useSearchParams().get("id");
  const router = useRouter();
  const finding = useFetch(() => (id ? api.finding(id) : Promise.reject(new Error("no finding id"))), 0, [
    id,
  ]);
  const [replay, setReplay] = useState(false);
  const f = finding.data;

  const remove = async () => {
    if (!f || !confirm(`Delete finding ${f.id}?`)) return;
    try {
      await api.findingDelete(f.id);
      router.push("/findings/");
    } catch (e) {
      toast.error(describe(e));
    }
  };

  if (!id) return <p className="text-muted-foreground text-sm">No finding id.</p>;
  if (finding.error) return <p className="text-destructive text-sm">{finding.error}</p>;
  if (!f) return <Skeleton className="h-40" />;

  const reproduced = f.replays.filter((r) => r.reproduced).length;
  const last = f.trace.at(-1);

  return (
    <>
      <PageTitle
        back={
          <Button variant="outline" size="icon" aria-label="Back" asChild>
            <Link href="/findings/">
              <ArrowLeft />
            </Link>
          </Button>
        }
        title={
          <span className="flex flex-wrap items-center gap-3">
            <span className="font-mono">{f.invariant}</span>
            <Badge variant="destructive">finding</Badge>
            <Badge variant="outline">{f.worker}</Badge>
            {f.shrunk ? <Badge variant="outline">shrunk</Badge> : null}
          </span>
        }
        description={
          <>
            <span className="font-mono text-xs">{f.id}</span> · found {when(f.found_at)} in{" "}
            <Link className="underline" href={`/findings/?campaign=${encodeURIComponent(f.campaign)}`}>
              {f.campaign}
            </Link>
            {f.run_id ? (
              <>
                {" "}
                · run{" "}
                <Link className="font-mono text-xs underline" href={`/runs/view/?id=${f.run_id}`}>
                  {f.run_id.slice(0, 13)}
                </Link>
              </>
            ) : null}
          </>
        }
      >
        <Button variant="ghost" size="sm" onClick={remove}>
          <Trash2 /> Delete
        </Button>
        <Button size="sm" onClick={() => setReplay(true)}>
          <RotateCcw /> Replay
        </Button>
      </PageTitle>

      <div className="border-destructive/30 bg-destructive/5 rounded-lg border px-3 py-2 text-sm">
        {f.message}
      </div>

      <StatRow
        items={[
          {
            label: "Trace",
            value: `${f.trace.length} step${f.trace.length === 1 ? "" : "s"}`,
            sub: f.trace.length < f.original_len ? `shrunk from ${f.original_len}` : "as recorded",
          },
          {
            label: "Replays",
            value: f.replays.length,
            sub: f.replays.length ? `${reproduced} reproduced` : "none yet",
            tone: reproduced ? "bad" : undefined,
          },
          {
            label: "Target",
            value: <span className="font-mono text-base">{f.target}</span>,
            sub: f.store ?? undefined,
          },
          { label: "Signature", value: <span className="font-mono text-base">{f.signature}</span> },
        ]}
      />

      <div className="grid gap-4 xl:grid-cols-2">
        <Card>
          <CardHeader>
            <CardTitle>Expected</CardTitle>
            <CardDescription>
              What the model, built from the requests and their answers, says the ledger holds.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <pre className="bg-muted/40 max-h-96 overflow-auto rounded-lg p-3 font-mono text-xs">
              {json(f.expected)}
            </pre>
          </CardContent>
        </Card>
        <Card>
          <CardHeader>
            <CardTitle>Actual</CardTitle>
            <CardDescription>What the ledger answered on the last step.</CardDescription>
          </CardHeader>
          <CardContent>
            <pre className="bg-muted/40 max-h-96 overflow-auto rounded-lg p-3 font-mono text-xs">
              {json(f.actual)}
            </pre>
          </CardContent>
        </Card>
      </div>

      <div className="grid gap-4 xl:grid-cols-[1fr_20rem]">
        <Card>
          <CardHeader>
            <CardTitle>Trace</CardTitle>
            <CardDescription>
              The steps on the subject up to the violation; the last one broke the rule. Subjects and instants
              are symbolic, so a replay runs on a fresh subject.
            </CardDescription>
          </CardHeader>
          <CardContent>
            <div className="overflow-x-auto">
              <Table>
                <TableHeader>
                  <TableRow>
                    <TableHead className="w-12">#</TableHead>
                    <TableHead className="w-20 text-right">at</TableHead>
                    <TableHead>Request</TableHead>
                    <TableHead>Answer</TableHead>
                    <TableHead />
                  </TableRow>
                </TableHeader>
                <TableBody>
                  {f.trace.map((s) => {
                    const o = outcomeOf(s);
                    return (
                      <TableRow key={s.index} className={s === last ? "bg-destructive/5" : ""}>
                        <TableCell className="text-muted-foreground font-mono text-xs tabular-nums">
                          {s.index}
                        </TableCell>
                        <TableCell className="text-muted-foreground text-right font-mono text-xs tabular-nums">
                          {(s.at_ms / 1000).toFixed(2)} s
                        </TableCell>
                        <TableCell className="max-w-md">
                          <span className="font-mono text-xs font-medium">{s.request.op}</span>
                          <div
                            className="text-muted-foreground truncate font-mono text-[11px]"
                            title={json(s.request)}
                          >
                            {requestOf(s)}
                          </div>
                        </TableCell>
                        <TableCell className="max-w-md">
                          <div
                            className={`truncate font-mono text-[11px] ${o.level === "error" ? "text-destructive" : "text-muted-foreground"}`}
                            title={s.response === undefined ? o.text : json(s.response)}
                          >
                            {o.text || "–"}
                          </div>
                        </TableCell>
                        <TableCell className="text-right">
                          {s.tolerated ? <LevelChip level="warn" /> : <LevelChip level={o.level} />}
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
              <CardTitle>Where</CardTitle>
            </CardHeader>
            <CardContent>
              <DetailList
                rows={[
                  { k: "Campaign", v: f.campaign },
                  { k: "Target", v: <span className="font-mono text-xs">{f.target}</span> },
                  { k: "Store", v: f.store ?? "unknown" },
                  { k: "Subject", v: <span className="font-mono text-[11px]">{f.subject}</span> },
                  { k: "Worker", v: f.worker },
                  { k: "Found", v: when(f.found_at) },
                ]}
              />
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Shrink</CardTitle>
              <CardDescription>Delta debugging on fresh subjects after the workers stopped.</CardDescription>
            </CardHeader>
            <CardContent>
              <p className="text-sm">{f.shrink_note ?? (f.shrunk ? "shrunk" : "not shrunk")}</p>
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Replays</CardTitle>
              <CardDescription>Each replay of the trace, newest last.</CardDescription>
            </CardHeader>
            <CardContent>
              {f.replays.length ? (
                <ol className="divide-y">
                  {f.replays.map((r, i) => (
                    <li key={i} className="grid gap-1 py-2 text-xs">
                      <div className="flex items-center justify-between gap-2">
                        <span className="font-mono">{r.target}</span>
                        <BoolBadge ok={!r.reproduced} yes="not reproduced" no="reproduced" />
                      </div>
                      <div className="text-muted-foreground">
                        {ago(r.at)} · {r.steps_run} steps{r.message ? ` · ${r.message}` : ""}
                      </div>
                    </li>
                  ))}
                </ol>
              ) : (
                <p className="text-muted-foreground text-sm">
                  Not replayed yet. Replay it against the serve stack, the deployed ledger or a URL to see if
                  it recurs.
                </p>
              )}
            </CardContent>
          </Card>
        </div>
      </div>

      {replay ? (
        <StressTargetDialog
          open
          onOpenChange={setReplay}
          title="Replay this finding"
          description="The trace is sent again, on a fresh subject, and judged with the same model. A run of kind stress records the outcome."
          hasStack={false}
          attempts
          action="Replay"
          onRun={async (targets, attempts) => {
            try {
              const s = await api.findingReplay(f.id, { targets, attempts });
              toast.success(`replaying as ${s.name}`);
              router.push(`/runs/view/?id=${s.id}`);
            } catch (e) {
              toast.error(describe(e));
            }
          }}
        />
      ) : null}
    </>
  );
}
