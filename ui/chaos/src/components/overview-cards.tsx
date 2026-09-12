"use client";

// The overview's header pieces and Stack card, in the kit's dashboard style:
// the tools as one segmented button group beside Refresh, a context strip with
// live instance dots, and a status card whose donut legend is the instance list.
import {
  BookOpen,
  Crosshair,
  Flame,
  Gauge,
  LayoutDashboard,
  type LucideIcon,
  RefreshCw,
  ScrollText,
  Server,
  ShieldCheck,
  Waypoints,
} from "lucide-react";
import Link from "next/link";
import { useMemo, useState } from "react";
import { Cell, Pie, PieChart, type PieSectorShapeProps, Sector } from "recharts";

import { describeBehavior } from "@/components/instances-table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { ButtonGroup } from "@/components/ui/button-group";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart";
import { Separator } from "@/components/ui/separator";
import type { InstanceInfo, KindDescriptor, Overview } from "@/lib/api/schema";
import { num } from "@/lib/format";
import { cn } from "@/lib/utils";

/** The categorical order for instance slices, fixed, never cycled: a sixth instance folds into "other". */
const SLICE = ["var(--chart-2)", "var(--chart-4)", "var(--chart-1)", "var(--chart-5)", "var(--chart-3)"];
const OTHER = "var(--muted-foreground)";

type Tone = "good" | "warn" | "off";

function toneOf(i: InstanceInfo): Tone {
  if (!i.running) return "off";
  if (
    (i.behavior && i.behavior.type !== "healthy") ||
    (i.store_behavior && i.store_behavior.type !== "healthy")
  )
    return "warn";
  return "good";
}

const DOT: Record<Tone, string> = {
  good: "bg-emerald-500",
  warn: "bg-amber-500",
  off: "bg-muted-foreground/40",
};

/** One line per instance: what it is doing, or why it is not healthy. */
function stateOf(i: InstanceInfo): string {
  if (!i.running) return "stopped";
  const parts: string[] = [];
  if (i.behavior && i.behavior.type !== "healthy") parts.push(describeBehavior(i.behavior));
  if (i.store_behavior && i.store_behavior.type !== "healthy")
    parts.push(`store ${describeBehavior(i.store_behavior)}`);
  return parts.length ? parts.join(" · ") : "healthy";
}

/**
 * The serve stack: how many are up, faulted or stopped; a donut of the
 * requests each instance served since it started (the services' own counters,
 * so a load run against the ledger shows up here), and one row per instance
 * with its state, dependency and failed count.
 */
export function StackCard({
  stack,
  kinds,
  topology,
}: {
  stack: InstanceInfo[] | null;
  kinds: KindDescriptor[];
  topology: string;
}) {
  const [active, setActive] = useState<string | null>(null);
  const rows = useMemo(() => {
    if (!stack) return [];
    const counted = stack.filter((i) => i.requests);
    const ranked = [...counted].sort((a, b) => (b.requests?.total ?? 0) - (a.requests?.total ?? 0));
    const color = new Map<string, string>();
    ranked.forEach((i, n) => color.set(i.name, n < SLICE.length ? SLICE[n]! : OTHER));
    return stack.map((i) => ({ i, color: color.get(i.name) ?? null }));
  }, [stack]);
  const total = rows.reduce((n, r) => n + (r.i.requests?.total ?? 0), 0);
  const failedTotal = rows.reduce((n, r) => n + (r.i.requests?.failed ?? 0), 0);
  const slices = rows
    .filter((r) => r.color && (r.i.requests?.total ?? 0) > 0)
    .map((r) => ({ name: r.i.name, value: r.i.requests?.total ?? 0, fill: r.color! }));
  const up = rows.filter((r) => r.i.running).length;
  const faulted = rows.filter((r) => toneOf(r.i) === "warn").length;
  const stopped = rows.length - up;
  const labelOf = (kind: string) => kinds.find((k) => k.name === kind)?.label ?? kind;
  const config = Object.fromEntries(
    rows.map((r) => [r.i.name, { label: r.i.name, color: r.color ?? OTHER }]),
  );

  return (
    <Card className="flex flex-col">
      <CardHeader className="flex flex-row items-start justify-between gap-4 space-y-0">
        <div className="space-y-1.5">
          <CardTitle className="flex items-center gap-2">
            <span className="flex size-8 items-center justify-center rounded-lg border">
              <Server className="size-4" />
            </span>
            Stack
          </CardTitle>
          <CardDescription>
            <span className="font-mono text-xs">{topology}</span> in this process.
          </CardDescription>
        </div>
        <Button variant="outline" size="sm" asChild>
          <Link href="/stack/">Manage</Link>
        </Button>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col gap-4">
        {!stack ? (
          <p className="text-muted-foreground text-sm">
            serve runs with --no-stack; nothing to fault from here.
          </p>
        ) : (
          <>
            <div className="flex items-center gap-4">
              <div className="grid flex-1 content-start gap-2">
                <div className="flex flex-wrap items-center gap-1.5 text-xs">
                  <Chip tone="good" label={`${up} up`} />
                  {faulted ? <Chip tone="warn" label={`${faulted} faulted`} /> : null}
                  {stopped ? <Chip tone="off" label={`${stopped} stopped`} /> : null}
                </div>
                <div className="text-muted-foreground text-xs tabular-nums">
                  {num(total)} served since start ·{" "}
                  <span className={failedTotal ? "text-destructive font-medium" : ""}>
                    {num(failedTotal)} failed
                  </span>
                </div>
                <div className="text-muted-foreground text-[11px]">
                  Share of requests per instance; the services&apos; own counters, reset on restart.
                </div>
              </div>
              <div className="relative w-[6.5rem] shrink-0">
                <ChartContainer config={config} className="aspect-square w-full">
                  <PieChart>
                    {slices.length ? (
                      <ChartTooltip
                        cursor={false}
                        content={
                          <ChartTooltipContent
                            hideLabel
                            formatter={(value, name) => (
                              <span className="flex w-full justify-between gap-4">
                                <span className="text-muted-foreground">{String(name)}</span>
                                <span className="font-mono tabular-nums">
                                  {num(Number(value))} ·{" "}
                                  {total ? Math.round((Number(value) / total) * 100) : 0}%
                                </span>
                              </span>
                            )}
                          />
                        }
                      />
                    ) : null}
                    <Pie
                      data={slices.length ? slices : [{ name: "none", value: 1, fill: "var(--muted)" }]}
                      dataKey="value"
                      nameKey="name"
                      innerRadius="68%"
                      outerRadius="94%"
                      paddingAngle={slices.length > 1 ? 2 : 0}
                      strokeWidth={0}
                      isAnimationActive={false}
                      shape={(props: PieSectorShapeProps) => {
                        const { outerRadius = 0, ...rest } = props;
                        const grow = String(rest.name) === active ? 3 : 0;
                        return <Sector {...rest} outerRadius={outerRadius + grow} />;
                      }}
                      onMouseEnter={(_, n) => setActive(slices[n]?.name ?? null)}
                      onMouseLeave={() => setActive(null)}
                    >
                      {(slices.length ? slices : [{ name: "none", fill: "var(--muted)" }]).map((s) => (
                        <Cell
                          key={s.name}
                          fill={s.fill}
                          fillOpacity={active && active !== s.name ? 0.35 : 1}
                        />
                      ))}
                    </Pie>
                  </PieChart>
                </ChartContainer>
                <div className="pointer-events-none absolute inset-0 flex flex-col items-center justify-center">
                  <span className="text-base font-semibold tabular-nums">{compact(total)}</span>
                  <span className="text-muted-foreground text-[9px] tracking-wider uppercase">req</span>
                </div>
              </div>
            </div>

            <div>
              <ul className="divide-y">
                {rows.map(({ i, color }) => {
                  const tone = toneOf(i);
                  const share = total && i.requests ? (i.requests.total / total) * 100 : 0;
                  return (
                    <li
                      key={i.name}
                      className={cn(
                        "grid gap-x-3 py-2 text-sm transition-opacity",
                        active && active !== i.name && "opacity-50",
                      )}
                      onMouseEnter={() => color && setActive(i.name)}
                      onMouseLeave={() => setActive(null)}
                    >
                      <div className="flex items-center gap-2">
                        <span className={cn("size-2 shrink-0 rounded-full", DOT[tone])} title={stateOf(i)} />
                        <Link
                          href="/stack/"
                          className="truncate font-mono text-xs font-medium hover:underline"
                        >
                          {i.name}
                        </Link>
                        <span className="text-muted-foreground text-xs">{labelOf(i.kind)}</span>
                        {i.added ? (
                          <Badge variant="outline" className="h-4 px-1 text-[10px]">
                            added
                          </Badge>
                        ) : null}
                        <span
                          className={cn(
                            "ml-auto text-right text-xs",
                            tone === "warn" ? "text-amber-600 dark:text-amber-400" : "text-muted-foreground",
                          )}
                        >
                          {stateOf(i)}
                        </span>
                      </div>
                      <div className="text-muted-foreground mt-1 flex items-center gap-2 pl-4 text-[11px]">
                        <span className="font-mono">{i.addr}</span>
                        {i.depends_on.length ? <span>→ {i.depends_on.join(", ")}</span> : null}
                        {i.requests ? (
                          <span className="ml-auto flex items-center gap-2 tabular-nums">
                            {i.requests.failed ? (
                              <span className="text-destructive font-medium">
                                {num(i.requests.failed)} failed
                              </span>
                            ) : null}
                            <span className="text-foreground">{num(i.requests.total)} req</span>
                            <span className="bg-muted h-1.5 w-12 overflow-hidden rounded-full" aria-hidden>
                              <span
                                className="block h-full rounded-full"
                                style={{
                                  width: `${Math.max(share ? 4 : 0, share)}%`,
                                  background: color ?? OTHER,
                                }}
                              />
                            </span>
                          </span>
                        ) : (
                          <span className="ml-auto">no counters</span>
                        )}
                      </div>
                    </li>
                  );
                })}
              </ul>
            </div>
            <p className="text-muted-foreground mt-auto pt-2 text-[11px]">
              {describeStack(
                rows.map((r) => r.i),
                labelOf,
              )}
            </p>
          </>
        )}
      </CardContent>
    </Card>
  );
}

/**
 * What validate hits, as short as it can be said: one host for every kind
 * behind the same Envoy, else the host once and a port per kind, else the
 * URLs. The scheme never carries information here.
 */
function describeTargets(targets: [string, string][], labelOf: (kind: string) => string): React.ReactNode {
  if (!targets.length) return <span>no validate targets</span>;
  const bare = targets.map(([k, u]) => [k, u.replace(/^https?:\/\//, "").replace(/\/$/, "")] as const);
  const kinds = bare.map(([k]) => labelOf(k).toLowerCase()).join(", ");
  if (new Set(bare.map(([, u]) => u)).size === 1) {
    return (
      <span className="truncate">
        <span className="font-mono">{bare[0]![1]}</span> for {kinds}
      </span>
    );
  }
  const hosts = new Set(bare.map(([, u]) => u.split(":")[0]));
  if (hosts.size === 1) {
    const host = [...hosts][0]!;
    return (
      <span className="truncate">
        <span className="font-mono">{host}</span>{" "}
        {bare.map(([k, u], n) => (
          <span key={k}>
            {n ? " · " : ""}
            <span className="font-mono">:{u.split(":")[1]}</span> {labelOf(k).toLowerCase()}
          </span>
        ))}
      </span>
    );
  }
  return <span className="truncate font-mono">{bare.map(([k, u]) => `${u} (${k})`).join(" · ")}</span>;
}

/** "1 engine, 1 ledger, 2 protocols · 3 of 4 running", counted live. */
function describeStack(stack: InstanceInfo[], labelOf: (kind: string) => string): string {
  const byKind = new Map<string, number>();
  for (const i of stack) byKind.set(i.kind, (byKind.get(i.kind) ?? 0) + 1);
  const parts = [...byKind.entries()].map(
    ([k, n]) => `${n} ${labelOf(k).toLowerCase()}${n === 1 ? "" : "s"}`,
  );
  const up = stack.filter((i) => i.running).length;
  return `${parts.join(", ")} · ${up} of ${stack.length} running`;
}

function Chip({ tone, label }: { tone: Tone; label: string }) {
  return (
    <span className="inline-flex items-center gap-1.5 rounded-md border px-2 py-0.5 font-medium">
      <span className={cn("size-1.5 rounded-full", DOT[tone])} />
      {label}
    </span>
  );
}

function compact(n: number): string {
  if (n >= 1_000_000) return `${(n / 1_000_000).toFixed(1)}M`;
  if (n >= 10_000) return `${(n / 1000).toFixed(0)}K`;
  if (n >= 1000) return `${(n / 1000).toFixed(1)}K`;
  return String(n);
}

/** The observability tools of this environment, in the order a failure is read. */
export function toolLinks(
  links: Overview["config"]["links"],
): { icon: LucideIcon; title: string; href: string }[] {
  return [
    { icon: LayoutDashboard, title: "Dashboards", href: links.grafana ? `${links.grafana}/dashboards` : "" },
    { icon: Waypoints, title: "Traces", href: links.grafana ? `${links.grafana}/explore` : "" },
    { icon: ScrollText, title: "Logs", href: links.victorialogs },
    { icon: Gauge, title: "Metrics", href: links.metrics },
    { icon: Flame, title: "Profiles", href: links.pyroscope },
    { icon: ShieldCheck, title: "Envoy", href: links.envoy_admin },
  ].filter((t) => t.href);
}

/** The page's action row: the tools as one segmented group, then Refresh. */
export function OverviewActions({ overview, onRefresh }: { overview: Overview; onRefresh: () => void }) {
  const tools = toolLinks(overview.config.links);
  return (
    <>
      {tools.length ? (
        <ButtonGroup>
          {tools.map((t) => (
            <Button key={t.title} variant="outline" size="sm" className="gap-1.5" asChild title={t.href}>
              <a href={t.href} target="_blank" rel="noreferrer">
                <t.icon />
                <span className="hidden lg:inline">{t.title}</span>
              </a>
            </Button>
          ))}
        </ButtonGroup>
      ) : null}
      <Button variant="outline" size="sm" className="gap-1.5" onClick={onRefresh}>
        <RefreshCw /> Refresh
      </Button>
    </>
  );
}

/**
 * The context line under the title: which environment this is, every instance
 * as a live dot, what validate hits, the topology and what is on disk. One
 * line that says where you are before any number on the page is read.
 */
export function EnvironmentStrip({ overview }: { overview: Overview }) {
  const links = overview.config.links;
  const stack = overview.stack ?? [];
  const labelOf = (kind: string) => overview.kinds.find((k) => k.name === kind)?.label ?? kind;
  const up = stack.filter((i) => i.running).length;
  const faulted = stack.filter((i) => toneOf(i) === "warn").length;
  const health: Tone = !overview.stack ? "off" : up < stack.length ? "off" : faulted ? "warn" : "good";
  const targets = Object.entries(overview.config.targets);
  const counts = [
    { label: "scenarios", n: overview.scenarios, href: "/scenarios/" },
    { label: "campaigns", n: overview.campaigns, href: "/stress/" },
    { label: "findings", n: overview.findings, href: "/findings/", bad: overview.findings > 0 },
    { label: "runs", n: overview.runs, href: "/runs/" },
  ];

  return (
    <div className="bg-card text-card-foreground rounded-xl border shadow-xs">
      <div className="flex flex-wrap items-center gap-x-4 gap-y-3 px-4 py-3 text-sm">
        <span className="flex items-center gap-2">
          <span className={cn("size-2 rounded-full", DOT[health])} />
          <span className="font-semibold">{overview.env}</span>
          {links.domain ? (
            <span className="text-muted-foreground font-mono text-xs">{links.domain}</span>
          ) : null}
        </span>

        <Separator orientation="vertical" className="hidden h-4 sm:block" />

        {overview.stack ? (
          <Link
            href="/stack/"
            className="hover:text-foreground text-muted-foreground flex items-center gap-2 text-xs"
          >
            <span className="flex items-center gap-1">
              {stack.map((i) => (
                <span
                  key={i.name}
                  className={cn("size-2 rounded-full", DOT[toneOf(i)])}
                  title={`${i.name} (${labelOf(i.kind)}) · ${stateOf(i)}`}
                />
              ))}
            </span>
            <span>
              {up}/{stack.length} up{faulted ? `, ${faulted} faulted` : ""}
            </span>
          </Link>
        ) : (
          <span className="text-muted-foreground text-xs">no stack (--no-stack)</span>
        )}

        <Separator orientation="vertical" className="hidden h-4 sm:block" />

        <span
          className="text-muted-foreground flex min-w-0 flex-1 items-center gap-1.5 text-xs"
          title={`validate hits ${targets.map(([k, u]) => `${u} (${labelOf(k)})`).join(", ") || "nothing"}`}
        >
          <Crosshair className="size-3.5 shrink-0" />
          {describeTargets(targets, labelOf)}
        </span>

        <span
          className="text-muted-foreground hidden items-center gap-1.5 text-xs xl:flex"
          title="the serve topology"
        >
          <Server className="size-3.5 shrink-0" />
          <span className="font-mono">{overview.config.paths.topology}</span>
        </span>

        <div className="flex flex-wrap items-center gap-1.5">
          {counts.map((c) => (
            <Link
              key={c.label}
              href={c.href}
              className="bg-muted/60 ring-border hover:bg-muted inline-flex items-center gap-1.5 rounded-md px-2 py-1 text-xs ring-1 transition-colors ring-inset"
            >
              <span className={cn("font-medium tabular-nums", c.bad && "text-destructive")}>{c.n}</span>
              <span className="text-muted-foreground">{c.label}</span>
            </Link>
          ))}
          <Button variant="ghost" size="sm" className="gap-1.5" asChild>
            <Link href="/kb/view/?doc=docs%2Fchaos%2Frunbook">
              <BookOpen /> Runbook
            </Link>
          </Button>
        </div>
      </div>
    </div>
  );
}
