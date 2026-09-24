"use client";

import { type Arena, fig, type Point, type TierState, useAge } from "@/lib/arena";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

/**
 * A small moving series as a hairline: the last two minutes, scaled to its own
 * maximum, with gaps where nothing was measured. SVG, no library.
 */
export function Spark({
  values,
  className,
  max: floor = 1,
}: {
  values: (number | null)[];
  className?: string;
  max?: number;
}) {
  const w = 120;
  const h = 28;
  const n = values.length;
  const max = Math.max(floor, ...values.filter((v): v is number => v !== null));
  const x = (i: number) => (n <= 1 ? w : (i / (n - 1)) * w);
  const y = (v: number) => h - 1 - (v / max) * (h - 2);
  const parts: string[] = [];
  let open = false;
  values.forEach((v, i) => {
    if (v === null) {
      open = false;
      return;
    }
    parts.push(`${open ? "L" : "M"}${x(i).toFixed(1)},${y(v).toFixed(1)}`);
    open = true;
  });
  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      preserveAspectRatio="none"
      className={cn("h-7 w-full", className)}
      aria-hidden
    >
      <line x1="0" x2={w} y1={h - 0.5} y2={h - 0.5} className="stroke-border" strokeWidth="1" />
      {parts.length ? (
        <path
          d={parts.join(" ")}
          fill="none"
          className="stroke-foreground"
          strokeWidth="1.25"
          vectorEffect="non-scaling-stroke"
        />
      ) : null}
    </svg>
  );
}

/** Slots as cells: filled for running, outlined for free, then the line waiting. */
function Slots({ t }: { t: TierState }) {
  const cells = Math.max(t.max_in_flight, t.in_flight);
  return (
    <span
      className="flex items-center gap-1"
      aria-label={`${t.in_flight} of ${t.max_in_flight} running, ${t.waiting} waiting`}
    >
      {Array.from({ length: Math.min(cells, 16) }, (_, i) => (
        <span
          key={i}
          className={cn(
            "size-2.5 rounded-[2px] border",
            i < t.in_flight ? "bg-foreground border-foreground" : "border-muted-foreground/40",
          )}
        />
      ))}
      {t.waiting > 0 ? (
        <span className="ml-1 flex items-center gap-0.5">
          {Array.from({ length: Math.min(t.waiting, 12) }, (_, i) => (
            <span key={i} className="size-1.5 rounded-full bg-amber-500/80" />
          ))}
          {t.waiting > 12 ? (
            <span className="text-muted-foreground ml-1 text-[10px]">+{t.waiting - 12}</span>
          ) : null}
        </span>
      ) : null}
    </span>
  );
}

function Dot({ on, className }: { on: boolean | null; className?: string }) {
  return (
    <span
      aria-hidden
      className={cn(
        "inline-block size-2 shrink-0 rounded-full",
        on === null ? "bg-muted-foreground/40" : on ? "bg-emerald-500" : "bg-destructive",
        className,
      )}
    />
  );
}

const series = (history: Point[], tier: string, key: "tps" | "inFlight" | "waiting") =>
  history.map((p) => (p.tiers[tier] ? (p.tiers[tier][key] as number | null) : null));

const since = (iso?: string | null) => {
  if (!iso) return null;
  const s = (Date.now() - Date.parse(iso)) / 1000;
  return Number.isFinite(s) ? Math.max(0, s) : null;
};

const ago = (s: number | null) =>
  s === null ? "—" : s < 1 ? "now" : s < 90 ? `${Math.round(s)} s ago` : `${Math.round(s / 60)} min ago`;

/**
 * What the platform is doing, from the arena: each tier's slots, queue and
 * speed with two minutes of history, every way in as the chaos tool last
 * checked it, the MCP tool count, and the chaos run when one is going. `wide`
 * lays it out across a page (the lab's own page); narrow is the workbench's
 * side column.
 */
export function LivePanel({ arena, wide = false }: { arena: Arena; wide?: boolean }) {
  const t = useT();
  const age = useAge(arena.receivedAt);
  const s = arena.snapshot;
  if (!s) {
    return (
      <div className="text-muted-foreground rounded-lg border border-dashed p-6 text-sm">
        {arena.error ?? t("wb.live.connecting")}
      </div>
    );
  }
  const stale = age !== null && age > 5;
  const chaos = s.chaos;
  const checked = since(s.surfaces_checked_at);
  return (
    <div
      className={cn("bg-border grid gap-px overflow-hidden rounded-lg border", wide ? "lg:grid-cols-3" : "")}
    >
      <div className="bg-background flex items-center gap-2 px-4 py-2.5 font-mono text-[11px] tracking-[0.12em] uppercase lg:col-span-full">
        <span className="relative flex size-1.5">
          {!stale ? (
            <span className="live-ping absolute inline-flex size-full rounded-full bg-emerald-500/60" />
          ) : null}
          <span
            className={cn(
              "relative inline-flex size-1.5 rounded-full",
              stale ? "bg-amber-500" : "bg-emerald-500",
            )}
          />
        </span>
        <span>{stale ? t("wb.live.stale") : t("wb.live.live")}</span>
        <span className="text-muted-foreground tracking-normal normal-case">
          {arena.via} · {age === null ? "—" : `${age.toFixed(0)} s`}
        </span>
      </div>

      {s.tiers.map((tier) => (
        <section key={tier.tier} className="bg-background flex flex-col gap-3 p-4">
          <header className="flex items-center gap-2">
            <Dot on={tier.up} />
            <span className="font-mono text-sm">{tier.tier}</span>
            <span className="text-muted-foreground truncate text-xs">
              {tier.model} · {tier.engine}
              {tier.stub ? " · stub" : ""}
            </span>
          </header>
          <Slots t={tier} />
          <dl className="grid grid-cols-2 gap-x-4 gap-y-1 font-mono text-xs tabular-nums">
            <dt className="text-muted-foreground">{t("wb.live.tps")}</dt>
            <dd className="text-right">{fig(tier.tokens_per_second, 1)}</dd>
            <dt className="text-muted-foreground">{t("wb.live.ttft")}</dt>
            <dd className="text-right">
              {fig(tier.ttft_p50_ms, 0)} / {fig(tier.ttft_p99_ms, 0)} ms
            </dd>
            <dt className="text-muted-foreground">{t("wb.live.refused")}</dt>
            <dd className="text-right">{fig(tier.refused_per_minute, 0)}</dd>
          </dl>
          <div>
            <p className="text-muted-foreground mb-1 font-mono text-[10px] tracking-[0.12em] uppercase">
              {t("wb.live.tps_2min")}
            </p>
            <Spark values={series(arena.history, tier.tier, "tps")} />
            <p className="text-muted-foreground mt-2 mb-1 font-mono text-[10px] tracking-[0.12em] uppercase">
              {t("wb.live.queue_2min")}
            </p>
            <Spark
              values={arena.history.map((p) =>
                p.tiers[tier.tier] ? p.tiers[tier.tier].inFlight + p.tiers[tier.tier].waiting : null,
              )}
              max={tier.max_in_flight + 1}
            />
          </div>
        </section>
      ))}

      <section className="bg-background flex flex-col gap-2 p-4">
        <header className="flex items-baseline justify-between gap-2">
          <span className="font-mono text-[11px] tracking-[0.12em] uppercase">{t("wb.live.surfaces")}</span>
          <span className="text-muted-foreground text-[11px]" title={s.surfaces_checked_at ?? undefined}>
            {t("wb.live.checked", { ago: ago(checked) })}
          </span>
        </header>
        {s.surfaces.length === 0 ? (
          <p className="text-muted-foreground text-xs">{t("wb.live.surfaces_none")}</p>
        ) : (
          <ul className="grid gap-1.5">
            {s.surfaces.map((x) => (
              <li key={x.name} className="flex items-center gap-2 text-xs" title={`${x.check}: ${x.detail}`}>
                <Dot on={x.up} />
                <span className="font-mono">{x.name}</span>
                <span className="text-muted-foreground ml-auto font-mono tabular-nums">
                  {fig(x.latency_ms, 0, " ms")}
                </span>
              </li>
            ))}
          </ul>
        )}
        <p className="border-t pt-2 text-xs">
          <span className="text-muted-foreground">{t("wb.live.mcp_tools")}</span>{" "}
          <span className="font-mono tabular-nums">{fig(s.mcp_tools)}</span>
        </p>
      </section>

      <section className={cn("bg-background flex flex-col gap-2 p-4", wide ? "lg:col-span-2" : "")}>
        <header className="flex items-baseline justify-between gap-2">
          <span className="font-mono text-[11px] tracking-[0.12em] uppercase">{t("wb.live.chaos")}</span>
          <span className="text-muted-foreground text-[11px]">
            {chaos ? t(`wb.chaos.${chaos.state}`) : "—"}
          </span>
        </header>
        {chaos?.state === "running" ? (
          <>
            <p className="text-sm">
              <span className="font-mono">{chaos.name}</span>{" "}
              <span className="text-muted-foreground text-xs">
                {chaos.kind}
                {chaos.phase ? ` · ${chaos.phase}` : ""} · {chaos.elapsed_s.toFixed(0)} s
              </span>
            </p>
            <dl className="grid grid-cols-2 gap-x-4 gap-y-1 font-mono text-xs tabular-nums sm:grid-cols-4">
              <dt className="text-muted-foreground">rps</dt>
              <dd>{fig(chaos.rps, 1)}</dd>
              <dt className="text-muted-foreground">{t("wb.live.errors")}</dt>
              <dd>{fig(chaos.error_rate * 100, 1, "%")}</dd>
              <dt className="text-muted-foreground">p99</dt>
              <dd>{fig(chaos.p99_ms, 0, " ms")}</dd>
              <dt className="text-muted-foreground">{t("wb.live.tps")}</dt>
              <dd>{fig(chaos.tokens_per_second, 1)}</dd>
            </dl>
            <Spark values={arena.history.map((p) => p.chaosRps)} />
          </>
        ) : (
          <p className="text-muted-foreground text-xs">{t("wb.live.chaos_quiet")}</p>
        )}
        <ul className="text-muted-foreground mt-auto flex flex-wrap gap-x-4 gap-y-1 border-t pt-2 text-[11px]">
          {s.sources.map((src) => (
            <li key={src.name} className="flex items-center gap-1.5" title={src.error || undefined}>
              <Dot on={src.ok} className="size-1.5" />
              {src.name} {src.ok ? ago(src.age_s ?? null) : src.error}
            </li>
          ))}
        </ul>
      </section>
    </div>
  );
}
