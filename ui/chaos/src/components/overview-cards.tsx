"use client";

// The overview's Stack and Environment cards, in the kit's dashboard style: an
// icon-button title, a donut with a centre label and a legend that doubles as
// the instance list, and link tiles with an icon box and an external arrow.
import {
  ArrowUpRight,
  BookOpen,
  Bug,
  Crosshair,
  Database,
  Flame,
  FlaskConical,
  Gauge,
  LayoutDashboard,
  ListVideo,
  type LucideIcon,
  ScrollText,
  Server,
  ShieldCheck,
  TestTube2,
  Waypoints,
} from "lucide-react";
import Link from "next/link";
import { useMemo, useState } from "react";
import { Cell, Pie, PieChart, type PieSectorShapeProps, Sector } from "recharts";

import { describeBehavior } from "@/components/instances-table";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import { Card, CardContent, CardDescription, CardHeader, CardTitle } from "@/components/ui/card";
import { ChartContainer, ChartTooltip, ChartTooltipContent } from "@/components/ui/chart";
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

/**
 * Where this environment points: the validate targets per kind, what is on
 * disk with counts, and the observability tools as tiles. Everything a
 * troubleshooter opens from here, one click each.
 */
export function EnvironmentCard({ overview }: { overview: Overview }) {
  const links = overview.config.links;
  const labelOf = (kind: string) => overview.kinds.find((k) => k.name === kind)?.label ?? kind;
  const targets = Object.entries(overview.config.targets);
  const sameHost = new Set(targets.map(([, u]) => u)).size === 1;

  const data: { icon: LucideIcon; title: string; count: string; path: string; href: string }[] = [
    {
      icon: FlaskConical,
      title: "Scenarios",
      count: String(overview.scenarios),
      path: overview.config.paths.scenarios,
      href: "/scenarios/",
    },
    {
      icon: TestTube2,
      title: "Campaigns",
      count: String(overview.campaigns),
      path: overview.config.paths.campaigns,
      href: "/stress/",
    },
    {
      icon: Bug,
      title: "Findings",
      count: overview.findings
        ? `${overview.findings} in ${overview.finding_signatures} signature${overview.finding_signatures === 1 ? "" : "s"}`
        : "none",
      path: overview.config.paths.findings,
      href: "/findings/",
    },
    {
      icon: ListVideo,
      title: "Run records",
      count: String(overview.runs),
      path: overview.config.paths.results,
      href: "/runs/",
    },
  ];

  const tools: { icon: LucideIcon; title: string; sub: string; href: string }[] = [
    {
      icon: LayoutDashboard,
      title: "Dashboards",
      sub: host(links.grafana),
      href: links.grafana ? `${links.grafana}/dashboards` : "",
    },
    {
      icon: Waypoints,
      title: "Traces",
      sub: "Grafana Explore · Tempo",
      href: links.grafana ? `${links.grafana}/explore` : "",
    },
    { icon: ScrollText, title: "Logs", sub: host(links.victorialogs), href: links.victorialogs },
    { icon: Gauge, title: "Metrics", sub: host(links.metrics), href: links.metrics },
    { icon: Flame, title: "Profiles", sub: host(links.pyroscope), href: links.pyroscope },
    { icon: ShieldCheck, title: "Envoy admin", sub: host(links.envoy_admin), href: links.envoy_admin },
  ].filter((t) => t.href);

  return (
    <Card className="flex flex-col">
      <CardHeader className="flex flex-row items-start justify-between gap-4 space-y-0">
        <div className="space-y-1.5">
          <CardTitle className="flex items-center gap-2">
            <span className="flex size-8 items-center justify-center rounded-lg border">
              <Crosshair className="size-4" />
            </span>
            Environment
          </CardTitle>
          <CardDescription>
            <span className="font-medium">{overview.env}</span>
            {links.domain ? <span className="font-mono text-xs"> · {links.domain}</span> : null}
          </CardDescription>
        </div>
        <Button variant="outline" size="sm" asChild>
          <Link href="/kb/view/?doc=docs%2Fchaos%2Frunbook">
            <BookOpen /> Runbook
          </Link>
        </Button>
      </CardHeader>
      <CardContent className="flex flex-1 flex-col gap-5">
        <section>
          <Eyebrow>Validate hits</Eyebrow>
          {sameHost && targets.length > 1 ? (
            <p className="text-sm">
              <span className="font-mono text-xs">{targets[0]![1]}</span>
              <span className="text-muted-foreground text-xs">
                {" "}
                for {targets.map(([k]) => labelOf(k).toLowerCase()).join(", ")}
              </span>
            </p>
          ) : (
            <ul className="grid gap-1 text-xs">
              {targets.map(([kind, url]) => (
                <li key={kind} className="flex items-baseline justify-between gap-3">
                  <span className="text-muted-foreground">{labelOf(kind)}</span>
                  <span className="truncate font-mono">{url}</span>
                </li>
              ))}
            </ul>
          )}
        </section>

        <section>
          <Eyebrow>On disk</Eyebrow>
          <div className="grid grid-cols-2 gap-2">
            {data.map((d) => (
              <Link
                key={d.title}
                href={d.href}
                className="hover:bg-muted/40 group rounded-lg border px-3 py-2 transition-colors"
              >
                <div className="flex items-center gap-2 text-xs">
                  <d.icon className="text-muted-foreground size-3.5" />
                  <span className="font-medium">{d.title}</span>
                  <span className="text-muted-foreground ml-auto tabular-nums">{d.count}</span>
                </div>
                <div className="text-muted-foreground mt-1 truncate font-mono text-[10px]" title={d.path}>
                  {d.path}
                </div>
              </Link>
            ))}
          </div>
        </section>

        <section>
          <Eyebrow>Open</Eyebrow>
          {tools.length ? (
            <div className="grid grid-cols-2 gap-2">
              {tools.map((t) => (
                <a
                  key={t.title}
                  href={t.href}
                  target="_blank"
                  rel="noreferrer"
                  className="hover:bg-muted/40 group flex items-center gap-2.5 rounded-lg border px-3 py-2 transition-colors"
                >
                  <span className="bg-muted flex size-8 shrink-0 items-center justify-center rounded-lg">
                    <t.icon className="size-4" />
                  </span>
                  <span className="min-w-0 flex-1">
                    <span className="block text-xs font-medium">{t.title}</span>
                    <span className="text-muted-foreground block truncate font-mono text-[10px]">
                      {t.sub}
                    </span>
                  </span>
                  <ArrowUpRight className="text-muted-foreground size-3.5 shrink-0 opacity-0 transition-opacity group-hover:opacity-100" />
                </a>
              ))}
            </div>
          ) : (
            <p className="text-muted-foreground text-xs">
              No observability links: set <span className="font-mono">[links]</span> in the chaos config.
            </p>
          )}
        </section>
        <p className="text-muted-foreground mt-auto flex items-center gap-1.5 text-[11px]">
          <Database className="size-3" /> Every request carries a trace id; logs, traces and profiles for the
          same second are one click apart.
        </p>
      </CardContent>
    </Card>
  );
}

function Eyebrow({ children }: { children: React.ReactNode }) {
  return (
    <div className="text-muted-foreground mb-1.5 text-[11px] font-medium tracking-wide uppercase">
      {children}
    </div>
  );
}

function host(url: string): string {
  return url.replace(/^https?:\/\//, "").replace(/\/$/, "");
}
