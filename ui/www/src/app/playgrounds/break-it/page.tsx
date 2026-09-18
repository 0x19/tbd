"use client";

import { useCallback, useEffect, useRef, useState } from "react";

import { Eyebrow, Frame, Tag } from "@/components/kit";
import { cn } from "@/lib/utils";

import {
  EMPTY_METER,
  inject,
  type Meter,
  open as openTransport,
  type Score,
  scores as fetchScores,
  type Transport,
  type World,
} from "./transport";

/** The moves, in the order they escalate. Costs mirror the service's own list. */
const MOVES = [
  {
    id: "MOVE_LATENCY",
    label: "Add latency",
    cost: 1,
    what: "an engine, 400 ms slower — until it is ejected",
  },
  {
    id: "MOVE_ERRORS",
    label: "Fail requests",
    cost: 2,
    what: "an engine refuses a third — until it is ejected",
  },
  {
    id: "MOVE_STALL_STORE",
    label: "Stall the store",
    cost: 2,
    what: "the ledger drops the odd write; nothing routes around it",
  },
  { id: "MOVE_KILL", label: "Kill a service", cost: 3, what: "an engine stops; its twin carries the load" },
] as const;

const TRANSPORTS: { id: Transport; label: string; note: string }[] = [
  { id: "ws", label: "WebSocket", note: "one socket, many calls" },
  { id: "sse", label: "SSE", note: "server-sent events" },
  { id: "poll", label: "Polling", note: "plain HTTP, once a second" },
];

const HEALTH: Record<string, { label: string; dot: string; text: string }> = {
  HEALTH_HEALED: { label: "holding", dot: "bg-emerald-500", text: "text-emerald-600 dark:text-emerald-400" },
  HEALTH_DEGRADED: { label: "degraded", dot: "bg-amber-500", text: "text-amber-600 dark:text-amber-400" },
  HEALTH_BREACHED: { label: "breached", dot: "bg-destructive", text: "text-destructive" },
  HEALTH_UNSPECIFIED: { label: "starting", dot: "bg-muted-foreground/40", text: "text-muted-foreground" },
};

export default function BreakItPage() {
  const [transport, setTransport] = useState<Transport>("ws");
  const [world, setWorld] = useState<World | null>(null);
  const [meter, setMeter] = useState<Meter>(EMPTY_METER);
  const [error, setError] = useState<string | null>(null);
  const [said, setSaid] = useState<string | null>(null);
  const [board, setBoard] = useState<Score[]>([]);
  const [actor, setActor] = useState("");
  const [busy, setBusy] = useState(false);
  const closer = useRef<(() => void) | null>(null);

  // One transport at a time; switching closes the old one first.
  useEffect(() => {
    setMeter(EMPTY_METER);
    closer.current?.();
    closer.current = openTransport(transport, {
      world: setWorld,
      meter: setMeter,
      error: setError,
    });
    return () => closer.current?.();
  }, [transport]);

  const refreshBoard = useCallback(() => {
    void fetchScores().then(setBoard);
  }, []);
  useEffect(refreshBoard, [refreshBoard]);
  // A breach is when the board changes, so reload it when one happens.
  useEffect(() => {
    if (world?.health === "HEALTH_BREACHED") refreshBoard();
  }, [world?.health, refreshBoard]);

  const play = async (move: string) => {
    setBusy(true);
    const result = await inject(move, actor);
    setBusy(false);
    setSaid(result.accepted ? null : (result.reason ?? "refused"));
    if (result.world) setWorld(result.world);
  };

  const health = HEALTH[world?.health ?? "HEALTH_UNSPECIFIED"] ?? HEALTH.HEALTH_UNSPECIFIED;
  const objective = world?.objective;
  const budget = world?.budget;
  const tokens = budget?.tokens ?? 0;
  const current = objective?.current ?? 1;
  const target = objective?.target ?? 0.99;
  const p99 = objective?.latency_current_ms ?? 0;
  const p99Target = objective?.latency_target_ms ?? 250;
  const successFailing = current < target;
  const latencyFailing = p99 > p99Target;
  const healing = world?.healing_seconds ?? 0;
  // Breached with nothing injected: the failures are aging out, not stuck.
  const isHealing = world?.health === "HEALTH_BREACHED" && !world?.faults?.length && healing > 0;
  // The bar shows the last stretch of the scale, where the game actually happens.
  const floor = Math.min(current, target) - 0.05;
  const span = Math.max(1 - floor, 0.001);
  const fill = Math.max(0, Math.min(1, (current - floor) / span)) * 100;
  const mark = Math.max(0, Math.min(1, (target - floor) / span)) * 100;

  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playground · Break it</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          A real cluster. Real faults. See if you can take it down.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-pretty">
          Four services are running under live traffic with an objective to hold: two engines behind a load
          balancer that ejects a replica the moment it starts failing or crawling, one ledger with nothing to
          hide behind, and the gateway in front. One fault is survivable by design. Breaking it takes several
          at once, faster than the budget refills — bring friends. Everyone shares one sandbox and one budget,
          and every fault expires on its own, so the world heals and the next person starts fresh.
        </p>
      </Frame>

      {/* ------------------------------------------------------- the world */}
      <Frame className="pb-10">
        <div className="bg-border/70 grid gap-px border-y sm:grid-cols-[minmax(0,2fr)_minmax(0,1fr)]">
          <div className="bg-background p-6 sm:p-8">
            <div className="flex items-baseline justify-between gap-4">
              <Eyebrow>Objective</Eyebrow>
              <span className={cn("flex items-center gap-2 font-mono text-[11px] uppercase", health.text)}>
                <span className={cn("size-2 rounded-full", health.dot, isHealing && "animate-pulse")} />
                {isHealing ? `healing · ${healing} s` : health.label}
              </span>
            </div>
            <div className="mt-4 flex items-baseline gap-3">
              <span
                className={cn(
                  "text-5xl font-semibold tracking-[-0.03em] tabular-nums",
                  successFailing && "text-destructive",
                )}
              >
                {(current * 100).toFixed(2)}%
              </span>
              <span className="text-muted-foreground text-sm">
                must stay above {(target * 100).toFixed(0)}% over {objective?.window_seconds ?? 30} s
              </span>
            </div>
            <div className="bg-muted relative mt-5 h-2 w-full overflow-hidden rounded-full">
              <div
                className={cn(
                  "h-full rounded-full transition-all duration-500",
                  world?.health === "HEALTH_BREACHED" ? "bg-destructive" : "bg-foreground",
                )}
                style={{ width: `${fill}%` }}
              />
              <div
                className="bg-foreground/40 absolute inset-y-0 w-px"
                style={{ left: `${mark}%` }}
                aria-hidden
              />
            </div>

            {/* The second clause. Either one failing breaks the objective. */}
            <div className="mt-5 flex items-baseline gap-3">
              <span
                className={cn(
                  "text-2xl font-semibold tracking-[-0.02em] tabular-nums",
                  latencyFailing && "text-destructive",
                )}
              >
                {p99.toFixed(0)} ms
              </span>
              <span className="text-muted-foreground text-sm">
                p99 must stay under {p99Target.toFixed(0)} ms, averaged over the window
              </span>
            </div>
            <div className="bg-muted relative mt-3 h-1.5 w-full overflow-hidden rounded-full">
              <div
                className={cn(
                  "h-full rounded-full transition-all duration-500",
                  latencyFailing ? "bg-destructive" : "bg-foreground",
                )}
                style={{ width: `${Math.max(0, Math.min(1, p99 / (p99Target * 2))) * 100}%` }}
              />
              <div className="bg-foreground/40 absolute inset-y-0 left-1/2 w-px" aria-hidden />
            </div>

            <dl className="text-muted-foreground mt-5 flex flex-wrap gap-x-8 gap-y-2 font-mono text-[11px]">
              <div>
                <dt className="inline">rps </dt>
                <dd className="text-foreground inline tabular-nums">
                  {(world?.traffic?.requests_per_second ?? 0).toFixed(1)}
                </dd>
              </div>
              <div>
                <dt className="inline">errors </dt>
                <dd className="text-foreground inline tabular-nums">
                  {((world?.traffic?.error_rate ?? 0) * 100).toFixed(1)}%
                </dd>
              </div>
              <div>
                <dt className="inline">p50 </dt>
                <dd className="text-foreground inline tabular-nums">
                  {(world?.traffic?.p50_ms ?? 0).toFixed(1)} ms
                </dd>
              </div>
              <div>
                <dt className="inline">p99 </dt>
                <dd className="text-foreground inline tabular-nums">
                  {(world?.traffic?.p99_ms ?? 0).toFixed(1)} ms
                </dd>
              </div>
            </dl>
          </div>

          <div className="bg-background p-6 sm:p-8">
            <Eyebrow>Budget</Eyebrow>
            <div className="mt-4 flex items-baseline gap-2">
              <span className="text-5xl font-semibold tracking-[-0.03em] tabular-nums">{tokens}</span>
              <span className="text-muted-foreground text-sm">of {budget?.max_tokens ?? 10}</span>
            </div>
            <div className="mt-4 flex gap-1" aria-hidden>
              {Array.from({ length: budget?.max_tokens ?? 10 }, (_, i) => (
                <span
                  key={i}
                  className={cn("h-6 flex-1 rounded-sm", i < tokens ? "bg-foreground" : "bg-muted")}
                />
              ))}
            </div>
            <p className="text-muted-foreground mt-4 text-xs text-pretty">
              Shared by everyone here. One token back every few seconds
              {budget?.refill_in_seconds ? `, next in ${budget.refill_in_seconds} s` : ""}.
            </p>
          </div>
        </div>
      </Frame>

      {/* -------------------------------------------------------- the moves */}
      <Frame className="pb-10">
        <Eyebrow>Your move</Eyebrow>
        <div className="mt-5 grid gap-2 sm:grid-cols-4">
          {MOVES.map((move) => {
            const afford = tokens >= move.cost;
            return (
              <button
                key={move.id}
                type="button"
                disabled={!afford || busy}
                onClick={() => void play(move.id)}
                className={cn(
                  "group rounded-xl border p-4 text-left transition-colors",
                  afford ? "hover:border-foreground/30 hover:bg-muted/40" : "opacity-40",
                )}
              >
                <div className="flex items-baseline justify-between gap-2">
                  <span className="font-medium">{move.label}</span>
                  <span className="text-muted-foreground font-mono text-[11px] tabular-nums">
                    {move.cost}
                  </span>
                </div>
                <span className="text-muted-foreground mt-1.5 block text-xs text-pretty">{move.what}</span>
              </button>
            );
          })}
        </div>
        <div className="mt-4 flex flex-wrap items-center gap-3">
          <label className="text-muted-foreground text-xs" htmlFor="actor">
            Name for the board
          </label>
          <input
            id="actor"
            value={actor}
            onChange={(event) => setActor(event.target.value)}
            maxLength={24}
            placeholder="optional"
            className="border-input bg-background focus-visible:ring-ring h-8 rounded-md border px-3 text-sm focus-visible:ring-1 focus-visible:outline-none"
          />
          {said ? <span className="text-muted-foreground text-xs">{said}</span> : null}
          {error ? <span className="text-destructive text-xs">{error}</span> : null}
        </div>
      </Frame>

      {/* --------------------------------------------------- the instances */}
      <Frame className="pb-10">
        <Eyebrow>The sandbox</Eyebrow>
        <ul className="mt-5 divide-y border-y">
          {(world?.instances ?? []).map((instance) => (
            <li key={instance.name} className="flex flex-wrap items-center gap-x-4 gap-y-1 py-3 text-sm">
              <span
                className={cn(
                  "size-2 shrink-0 rounded-full",
                  !instance.running
                    ? "bg-muted-foreground/40"
                    : instance.ailment
                      ? "bg-amber-500"
                      : "bg-emerald-500",
                )}
              />
              <span className="w-28 font-mono text-xs">{instance.name}</span>
              <Tag>{instance.kind}</Tag>
              <span className="text-muted-foreground flex-1 text-xs">
                {instance.running ? instance.ailment || "healthy" : "stopped"}
              </span>
              {instance.balanced ? (
                <span
                  className={cn(
                    "font-mono text-[11px] uppercase",
                    instance.ejected
                      ? "text-destructive"
                      : instance.in_rotation
                        ? "text-emerald-600 dark:text-emerald-400"
                        : "text-muted-foreground",
                  )}
                >
                  {instance.ejected ? "ejected" : instance.in_rotation ? "in rotation" : "out of rotation"}
                </span>
              ) : null}
              <span className="text-muted-foreground font-mono text-[11px] tabular-nums">
                {Number(instance.requests_total ?? 0).toLocaleString()} req
                {Number(instance.requests_failed ?? 0) > 0 ? (
                  <span className="text-destructive">
                    {" "}
                    · {Number(instance.requests_failed).toLocaleString()} failed
                  </span>
                ) : null}
              </span>
            </li>
          ))}
          {!world ? <li className="text-muted-foreground py-3 text-sm">connecting…</li> : null}
        </ul>
        {world?.faults?.length ? (
          <ul className="text-muted-foreground mt-4 space-y-1 text-xs">
            {world.faults.map((fault) => (
              <li key={fault.id}>
                <span className="text-foreground font-mono">{fault.instance}</span> — {fault.description},{" "}
                {fault.expires_in_seconds ?? 0} s left
                {fault.actor ? ` · ${fault.actor}` : ""}
              </li>
            ))}
          </ul>
        ) : null}
      </Frame>

      {/* -------------------------------------------------- the transports */}
      <Frame className="pb-10">
        <Eyebrow>Watching over</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          The same call, three ways. The gateway generates REST, server-sent events and the multiplexed socket
          from one proto definition, so switching here changes the pipe and nothing else.
        </p>
        <div className="mt-5 grid gap-2 sm:grid-cols-3">
          {TRANSPORTS.map((option) => (
            <button
              key={option.id}
              type="button"
              onClick={() => setTransport(option.id)}
              aria-pressed={transport === option.id}
              className={cn(
                "rounded-xl border p-4 text-left transition-colors",
                transport === option.id ? "border-foreground/40 bg-muted/40" : "hover:bg-muted/20",
              )}
            >
              <span className="font-medium">{option.label}</span>
              <span className="text-muted-foreground mt-1 block text-xs">{option.note}</span>
              {transport === option.id ? (
                <span className="text-muted-foreground mt-3 block font-mono text-[11px] tabular-nums">
                  {/* The two numbers are not the same measurement, so each says
                      which it is: a pushed frame is timed from the last one, a
                      poll from the request that fetched it. */}
                  {meter.latency === null
                    ? "—"
                    : option.id === "poll"
                      ? `${meter.latency} ms round trip`
                      : `${(meter.latency / 1000).toFixed(1)} s between frames`}
                  {" · "}
                  {meter.frames} frames · {(meter.bytes / 1024).toFixed(1)} KB · {meter.opens}{" "}
                  {option.id === "poll" ? "requests" : `connection${meter.opens === 1 ? "" : "s"}`}
                </span>
              ) : null}
            </button>
          ))}
        </div>
      </Frame>

      {/* ----------------------------------------------------- what it sees */}
      <Frame className="pb-10">
        <Eyebrow>What this page sends</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          The name you type, if you type one, so a breach has an author — it is trimmed to one line and
          capped, and it is the only thing you give it. Moves are public: everyone watching sees which
          instance was hit and by whom. Requests reach the sandbox through the same gateway as everything
          else, which counts them per address to keep one visitor from spending the whole budget. No cookies,
          no analytics, nothing from anyone else.
        </p>
      </Frame>

      {/* ------------------------------------------------------ the scores */}
      <Frame className="pb-24 sm:pb-32">
        <Eyebrow>Fastest breaches</Eyebrow>
        {board.length ? (
          <ol className="mt-5 divide-y border-y">
            {board.map((score, i) => (
              <li key={`${score.actor}-${score.at ?? i}`} className="flex items-baseline gap-4 py-3 text-sm">
                <span className="text-muted-foreground/60 w-6 font-mono text-[11px] tabular-nums">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <span className="flex-1 font-medium">{score.actor}</span>
                <span className="text-muted-foreground font-mono text-xs tabular-nums">
                  {score.faults_used ?? 0} faults
                </span>
                <span className="font-mono text-sm tabular-nums">
                  {(score.seconds_to_breach ?? 0).toFixed(1)} s
                </span>
              </li>
            ))}
          </ol>
        ) : (
          <p className="text-muted-foreground mt-5 border-y py-6 text-sm">
            Nobody has broken it yet. The clock starts from the moment the world is healed.
          </p>
        )}
      </Frame>
    </>
  );
}
