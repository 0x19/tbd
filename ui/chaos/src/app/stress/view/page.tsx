"use client";

import { ArrowLeft, BookOpen, Play, Save, Trash2 } from "lucide-react";
import Link from "next/link";
import { useRouter, useSearchParams } from "next/navigation";
import { Suspense, useEffect, useRef, useState } from "react";
import { toast } from "sonner";

import { CampaignReference } from "@/components/campaign-reference";
import { DetailList, PageTitle, StageBar } from "@/components/kit";
import { StatusBadge } from "@/components/status-badge";
import { StressTargetDialog } from "@/components/stress-target";
import { TomlEditor, type TomlEditorHandle } from "@/components/toml-editor";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Input } from "@/components/ui/input";
import { Sheet, SheetContent, SheetDescription, SheetHeader, SheetTitle } from "@/components/ui/sheet";
import { Skeleton } from "@/components/ui/skeleton";
import { api } from "@/lib/api/client";
import { describe } from "@/lib/api/hooks";
import type { CampaignDetail, CheckReply, StressTarget } from "@/lib/api/schema";
import { ago } from "@/lib/format";

const TEMPLATE = `# New campaign. Every table rejects unknown keys; open Reference (top right) for every key.
[campaign]
name = "my_campaign"
description = "What this proves"
duration = "5s"
warmup = "500ms"
seed = 1

# The ledger the workers hit, booted for the run. Delete this table to run
# against the serve stack's ledgers (or pick a target when running).
[stack.ledgers.ledger-1]
grace = "0s"

[workload]
max_in_flight = 32

[workload.owner]
workers = 4
subjects = 2

[faults]
tolerate = []

[stop]
max_findings = 0
`;

/** `tbd_stress::Campaign` as JSON (durations are humantime strings). */
type Parsed = {
  campaign?: {
    name?: string;
    description?: string;
    skip?: boolean;
    duration?: string;
    warmup?: string | null;
    seed?: number;
    timeout?: string;
  };
  stack?: Record<string, Record<string, Record<string, unknown>>>;
  timeline?: { at?: string; action?: string; service?: string; message?: string }[];
  workload?: {
    max_in_flight?: number;
    paths?: string[];
    relation_paths?: string[];
    scopes?: string[];
    owner?: {
      workers?: number;
      subjects?: number;
      pace?: string;
      limit?: number;
      mix?: Record<string, number> | null;
    };
  };
  faults?: { tolerate?: string[]; settle?: string; clock_skew?: string };
  invariants?: Record<string, boolean>;
  stop?: { max_findings?: number; shrink?: boolean; shrink_attempts?: number; shrink_timeout?: string };
};

export default function CampaignViewPage() {
  return (
    <Suspense fallback={<Skeleton className="h-40" />}>
      <CampaignView />
    </Suspense>
  );
}

/** The kit's order-detail layout, as the scenario editor: back arrow, name with chips, stage strip, editor plus a right rail. */
function CampaignView() {
  const params = useSearchParams();
  const router = useRouter();
  const requested = params.get("id") ?? "new";
  const isNew = requested === "new";
  const [id, setId] = useState(isNew ? "" : requested);
  const [detail, setDetail] = useState<CampaignDetail | null>(null);
  const [text, setText] = useState(isNew ? TEMPLATE : "");
  const [check, setCheck] = useState<CheckReply | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [reference, setReference] = useState(false);
  const [against, setAgainst] = useState(false);
  const editor = useRef<TomlEditorHandle>(null);

  useEffect(() => {
    if (isNew) return;
    api
      .campaign(requested)
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
        .campaignCheck(text)
        .then(setCheck)
        .catch(() => setCheck(null));
    }, 400);
    return () => clearTimeout(t);
  }, [text]);

  const parsed = (check?.parsed ?? null) as Parsed | null;
  const dirty = detail ? text !== detail.text : true;
  const hasStack = Object.keys(parsed?.stack ?? {}).length > 0;

  const save = async () => {
    if (!id.trim()) {
      toast.error("give the campaign an id, e.g. ledger/soak");
      return;
    }
    setBusy(true);
    try {
      await api.campaignSave(id.trim(), text);
      toast.success(`saved stress/${id.trim()}.toml`);
      if (isNew) router.replace(`/stress/view/?id=${encodeURIComponent(id.trim())}`);
      else setDetail(await api.campaign(id));
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const run = async (targets?: StressTarget[]) => {
    setBusy(true);
    try {
      if (dirty) await api.campaignSave(id.trim(), text);
      const s = await api.runStress({ campaign: id.trim(), targets });
      router.push(`/runs/view/?id=${s.id}`);
    } catch (e) {
      toast.error(describe(e));
    } finally {
      setBusy(false);
    }
  };

  const remove = async () => {
    if (!confirm(`Delete stress/${id}.toml?`)) return;
    try {
      await api.campaignDelete(id);
      router.push("/stress/");
    } catch (e) {
      toast.error(describe(e));
    }
  };

  const meta = parsed?.campaign;
  const owner = parsed?.workload?.owner;
  const timeline = parsed?.timeline ?? [];
  const stackRows = Object.entries(parsed?.stack ?? {}).flatMap(([plural, instances]) =>
    Object.entries(instances ?? {}).map(([name, spec]) => ({
      k: name,
      v: `${plural.replace(/s$/, "")}${spec?.database_url ? " on Postgres" : " in memory"}${
        spec?.grace ? `, grace ${String(spec.grace)}` : ""
      }`,
    })),
  );
  const mix = owner?.mix ? Object.entries(owner.mix).filter(([, w]) => w > 0) : null;
  const off = Object.entries(parsed?.invariants ?? {})
    .filter(([, on]) => !on)
    .map(([k]) => k);
  const faults = parsed?.faults;
  const stop = parsed?.stop;

  return (
    <>
      <PageTitle
        back={
          <Button variant="outline" size="icon" aria-label="Back" asChild>
            <Link href="/stress/">
              <ArrowLeft />
            </Link>
          </Button>
        }
        title={
          <span className="flex flex-wrap items-center gap-3">
            {isNew ? "New campaign" : (meta?.name ?? detail?.name ?? requested)}
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
            {meta?.skip ? <Badge variant="outline">skipped in CI</Badge> : null}
            {dirty && !isNew ? <Badge variant="outline">unsaved</Badge> : null}
          </span>
        }
        description={
          isNew ? (
            "Write the TOML, it is checked as you type; save it into the stress directory, then run it."
          ) : (
            <>
              <span className="font-mono">{detail?.file ?? requested}</span>
              {meta?.description ? ` · ${meta.description}` : ""}
              {detail?.last_run ? ` · last run ${ago(detail.last_run.started_at)}` : ""}
            </>
          )
        }
      >
        <Button variant="ghost" size="sm" onClick={() => setReference(true)}>
          <BookOpen /> Reference
        </Button>
        {!isNew ? (
          <Button variant="ghost" size="sm" onClick={remove}>
            <Trash2 /> Delete
          </Button>
        ) : null}
        <Button variant="outline" size="sm" onClick={save} disabled={busy || !check?.ok}>
          <Save /> {isNew ? "Save" : dirty ? "Save changes" : "Saved"}
        </Button>
        <Button
          variant="outline"
          size="sm"
          onClick={() => setAgainst(true)}
          disabled={busy || !check?.ok || !id.trim()}
        >
          Run against…
        </Button>
        <Button size="sm" onClick={() => run()} disabled={busy || !check?.ok || !id.trim()}>
          <Play /> {dirty ? "Save and run" : "Run"}
        </Button>
      </PageTitle>
      {error ? <p className="text-destructive text-sm">{error}</p> : null}

      <div className="grid gap-6 xl:grid-cols-[1fr_20rem]">
        <div className="grid content-start gap-6">
          <div className="bg-muted/30 rounded-xl border p-4">
            <div className="mb-3 flex flex-wrap justify-between gap-2 text-sm">
              <span>
                {hasStack
                  ? `Stack ${stackRows.map((r) => r.k).join(", ")}`
                  : "No stack: runs against the serve stack's ledgers or a chosen target"}
              </span>
              <span className="text-muted-foreground">
                {meta
                  ? `${owner?.workers ?? 4} workers × ${owner?.subjects ?? 2} subjects for ${meta.duration ?? "?"}${
                      meta.warmup ? `, warmup ${meta.warmup}` : ""
                    }, seed ${meta.seed ?? 1}`
                  : ""}
              </span>
            </div>
            <StageBar stages={["Setup", "Warmup", "Run + timeline", "Shrink", "Teardown"]} current={-1} />
            {timeline.length ? (
              <ol className="text-muted-foreground mt-4 grid gap-1 text-xs">
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
              <span className="text-muted-foreground text-xs">Id, becomes stress/&lt;id&gt;.toml</span>
              <Input
                value={id}
                onChange={(e) => setId(e.target.value)}
                placeholder="ledger/soak"
                className="max-w-sm font-mono text-xs"
              />
            </div>
          ) : null}
          {check && !check.ok ? (
            <div className="border-destructive/30 bg-destructive/5 text-destructive rounded-lg border px-3 py-2 text-sm">
              {check.error}
            </div>
          ) : null}
          <TomlEditor
            ref={editor}
            value={text}
            onChange={setText}
            ariaLabel="Campaign TOML"
            className="min-h-[34rem]"
          />
        </div>

        <div className="grid content-start gap-4">
          {detail?.last_run ? (
            <Card>
              <CardHeader>
                <CardTitle>Last run</CardTitle>
                <CardDescription>
                  {ago(detail.last_run.started_at)}
                  {detail.last_run.findings ? ` · ${detail.last_run.findings} findings` : ""}
                </CardDescription>
              </CardHeader>
              <CardContent className="flex items-center justify-between">
                <StatusBadge status={detail.last_run.status} />
                <Button variant="outline" size="sm" asChild>
                  <Link href={`/runs/view/?id=${detail.last_run.id}`}>Open</Link>
                </Button>
              </CardContent>
            </Card>
          ) : null}
          <Card>
            <CardHeader>
              <CardTitle>Workload</CardTitle>
            </CardHeader>
            <CardContent>
              <DetailList
                rows={[
                  { k: "Owner workers", v: String(owner?.workers ?? 4) },
                  { k: "Subjects each", v: String(owner?.subjects ?? 2) },
                  { k: "In flight", v: String(parsed?.workload?.max_in_flight ?? 64) },
                  { k: "Pace", v: owner?.pace ?? "0ms" },
                  { k: "Page size", v: owner?.limit ? String(owner.limit) : "drawn per read" },
                  ...(mix
                    ? mix.map(([op, w]) => ({ k: op, v: `weight ${w}` }))
                    : [{ k: "Mix", v: "balanced default" }]),
                ]}
              />
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Faults</CardTitle>
            </CardHeader>
            <CardContent>
              <DetailList
                rows={[
                  {
                    k: "Tolerated",
                    v: faults?.tolerate?.length
                      ? faults.tolerate.join(", ")
                      : "none: every failure is a finding",
                  },
                  { k: "Settle", v: faults?.settle ?? "1500ms" },
                  { k: "Clock skew", v: faults?.clock_skew ?? "500ms" },
                ]}
              />
            </CardContent>
          </Card>
          <Card>
            <CardHeader>
              <CardTitle>Invariants and stop</CardTitle>
            </CardHeader>
            <CardContent>
              <DetailList
                rows={[
                  { k: "Switched off", v: off.length ? off.join(", ") : "none, every rule is on" },
                  {
                    k: "Stop after",
                    v: stop?.max_findings ? `${stop.max_findings} findings` : "never",
                  },
                  {
                    k: "Shrink",
                    v:
                      stop?.shrink === false
                        ? "off"
                        : `${stop?.shrink_attempts ?? 200} replays, ${stop?.shrink_timeout ?? "60s"}`,
                  },
                ]}
              />
            </CardContent>
          </Card>
        </div>
      </div>

      <Sheet open={reference} onOpenChange={setReference}>
        <SheetContent side="right" className="w-full overflow-y-auto sm:max-w-2xl">
          <SheetHeader>
            <SheetTitle>Campaign reference</SheetTitle>
            <SheetDescription>
              What a campaign file can say, key by key, and what every invariant checks. Insert a block and it
              lands at the cursor.
            </SheetDescription>
          </SheetHeader>
          <div className="mt-4">
            <CampaignReference
              onInsert={(toml) => {
                editor.current?.insert(toml);
                setReference(false);
              }}
            />
          </div>
        </SheetContent>
      </Sheet>

      {against ? (
        <StressTargetDialog
          open
          onOpenChange={setAgainst}
          title="Run against…"
          description="Which ledger the workers hit. The campaign's own stack is booted for the run; anything else is an existing ledger and the timeline is skipped."
          hasStack={hasStack}
          onRun={(targets) => run(targets)}
        />
      ) : null}
    </>
  );
}
