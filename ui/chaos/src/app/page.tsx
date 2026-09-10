"use client";

import Link from "next/link";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { Skeleton } from "@/components/ui/skeleton";
import { InstancesTable } from "@/components/instances-table";
import { Empty, ErrorNote, PageHeader } from "@/components/page-header";
import { RunsTable } from "@/components/runs-table";
import { StatCard } from "@/components/stat-card";
import { StatusBadge } from "@/components/status-badge";
import { useChaos } from "@/components/shell/providers";
import { ago, pct } from "@/lib/format";

export default function OverviewPage() {
  const { overview, error, reload } = useChaos();
  if (!overview) {
    return (
      <>
        <PageHeader title="Overview" description="State of the stack and the last runs." />
        <ErrorNote message={error} />
        <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
          {[0, 1, 2, 3].map((i) => (
            <Skeleton key={i} className="h-24" />
          ))}
        </div>
      </>
    );
  }
  const stack = overview.stack ?? [];
  const up = stack.filter((i) => i.running).length;
  const faulty = stack.filter((i) => i.behavior && i.behavior.type !== "healthy").length;
  const last = overview.recent_runs[0];
  const failedRecent = overview.recent_runs.filter(
    (r) => r.status === "failed" || r.status === "error",
  ).length;
  const lv = overview.last_validate;

  return (
    <>
      <PageHeader
        title="Overview"
        description={`${overview.env} · chaos v${overview.version} · ${overview.config_files.join(" + ")}`}
      >
        <Button variant="outline" size="sm" onClick={reload}>
          Refresh
        </Button>
      </PageHeader>
      <ErrorNote message={error} />
      <div className="grid gap-4 md:grid-cols-2 xl:grid-cols-4">
        <StatCard
          label="Stack"
          value={overview.stack ? `${up}/${stack.length} up` : "none"}
          tone={!overview.stack ? undefined : up === stack.length ? "good" : "bad"}
          hint={
            overview.stack
              ? faulty
                ? `${faulty} with an injected fault`
                : "all healthy"
              : "serve runs with --no-stack"
          }
        />
        <StatCard
          label="Last validate"
          value={lv ? (lv.passed ? `${lv.passed[0]}/${lv.passed[1]}` : lv.status) : "never"}
          tone={lv ? (lv.status === "passed" ? "good" : "bad") : "warn"}
          hint={lv ? `${ago(lv.started_at)} · ${overview.config.targets.protocol}` : "run one from Validate"}
        />
        <StatCard
          label="Last run"
          value={last ? <StatusBadge status={last.status} /> : "none"}
          hint={
            last
              ? `${last.name} · ${ago(last.started_at)}${last.error_rate !== null ? ` · ${pct(last.error_rate)} errors` : ""}`
              : `${overview.scenarios} scenarios ready`
          }
        />
        <StatCard
          label="Recent failures"
          value={failedRecent}
          tone={failedRecent ? "bad" : "good"}
          hint={`of the last ${overview.recent_runs.length} runs · ${overview.runs} recorded`}
        />
      </div>

      {overview.active_run ? (
        <Card size="sm">
          <CardHeader>
            <CardTitle className="flex items-center gap-2">
              <StatusBadge status={overview.active_run.status} /> {overview.active_run.name}
            </CardTitle>
            <CardDescription>A run is in progress.</CardDescription>
          </CardHeader>
          <CardContent>
            <Button size="sm" render={<Link href={`/runs/view/?id=${overview.active_run.id}`} />}>
              Follow it live
            </Button>
          </CardContent>
        </Card>
      ) : null}

      <div className="grid gap-4 xl:grid-cols-5">
        <Card className="xl:col-span-3">
          <CardHeader>
            <CardTitle>Stack</CardTitle>
            <CardDescription>
              {overview.config.paths.topology} in this process. Stop, start and inject faults on the Stack
              page.
            </CardDescription>
          </CardHeader>
          <CardContent>
            {overview.stack ? (
              <InstancesTable instances={overview.stack} compact />
            ) : (
              <Empty>serve runs with --no-stack</Empty>
            )}
          </CardContent>
        </Card>
        <Card className="xl:col-span-2">
          <CardHeader>
            <CardTitle>Where to look</CardTitle>
            <CardDescription>What this environment points at.</CardDescription>
          </CardHeader>
          <CardContent className="grid gap-2 text-sm">
            <Row
              k="validate targets (as seen from chaos serve)"
              v={`${overview.config.targets.protocol} · ${overview.config.targets.engine}`}
            />
            <Row k="scenarios" v={overview.config.paths.scenarios} />
            <Row k="run records" v={overview.config.paths.results} />
            <Row k="API" v={overview.config.serve.base_path} />
            {Object.entries(overview.config.links)
              .filter(([k, v]) => v && k !== "domain")
              .map(([k, v]) => (
                <Row
                  key={k}
                  k={k}
                  v={
                    <a className="underline" href={v} target="_blank" rel="noreferrer">
                      {v}
                    </a>
                  }
                />
              ))}
          </CardContent>
        </Card>
      </div>

      <Card>
        <CardHeader>
          <CardTitle>Recent runs</CardTitle>
          <CardDescription>
            Newest first.{" "}
            <Link className="underline" href="/runs/">
              All runs
            </Link>
            .
          </CardDescription>
        </CardHeader>
        <CardContent>
          <RunsTable runs={overview.recent_runs} />
        </CardContent>
      </Card>
    </>
  );
}

function Row({ k, v }: { k: string; v: React.ReactNode }) {
  return (
    <div className="grid grid-cols-[9rem_1fr] gap-2">
      <span className="text-muted-foreground">{k}</span>
      <span className="truncate font-mono text-xs leading-5">{v}</span>
    </div>
  );
}
