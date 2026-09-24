"use client";

import { useEffect, useRef, useState } from "react";

/**
 * The arena's snapshot as the gateway renders `tbd.arena.v1.Snapshot`: proto
 * field names, timestamps as RFC 3339, an unset optional absent (or null).
 * Absent means no source could say; the page shows "—", never a zero.
 */
export type TierState = {
  tier: string;
  engine: string;
  model: string;
  up: boolean;
  stub: boolean;
  in_flight: number;
  max_in_flight: number;
  waiting: number;
  tokens_per_second?: number | null;
  ttft_p50_ms?: number | null;
  ttft_p99_ms?: number | null;
  refused_per_minute?: number | null;
};
export type SurfaceState = { name: string; up: boolean; latency_ms: number; check: string; detail: string };
export type ChaosRun = {
  state: "idle" | "running" | "absent" | string;
  id: string;
  kind: string;
  name: string;
  phase: string;
  elapsed_s: number;
  rps: number;
  error_rate: number;
  p99_ms: number;
  tokens_per_second?: number | null;
};
export type SourceState = { name: string; ok: boolean; age_s?: number | null; error: string };
/** The sandbox runner (RFC 0010): whether it answered, how full it is, and its runs over five minutes. */
export type RunnerState = {
  up?: boolean;
  stub?: boolean;
  in_flight?: number;
  max_in_flight?: number;
  languages?: string[];
  runs_per_minute?: number | null;
  unavailable_per_minute?: number | null;
  p50_ms?: number | null;
  p99_ms?: number | null;
};
export type Snapshot = {
  now?: string | null;
  tiers: TierState[];
  surfaces: SurfaceState[];
  surfaces_checked_at?: string | null;
  mcp_tools?: number | null;
  chaos?: ChaosRun | null;
  sources: SourceState[];
  /** Absent when no runner is configured. */
  runner?: RunnerState | null;
};

/** One point of the moving series, per tier and for the chaos run. */
export type Point = {
  t: number;
  tiers: Record<string, { tps: number | null; inFlight: number; waiting: number }>;
  chaosRps: number | null;
};

export type Via = "websocket" | "sse" | "poll";

export type Arena = {
  snapshot: Snapshot | null;
  /** The last two minutes, oldest first. */
  history: Point[];
  via: Via | null;
  /** When the last frame arrived, by this browser's clock. */
  receivedAt: number | null;
  /** Why there is no live view, when there is none. */
  error: string | null;
};

const HISTORY = 120;
const SOCKET = "/v1/ws";
const EVENTS = "/v1/arena/events";
const SNAPSHOT = "/v1/arena/snapshot";
const METHOD = "tbd.arena.v1.ArenaService/Watch";

function point(s: Snapshot): Point {
  const tiers: Point["tiers"] = {};
  for (const t of s.tiers) {
    tiers[t.tier] = { tps: t.tokens_per_second ?? null, inFlight: t.in_flight, waiting: t.waiting };
  }
  return {
    t: Date.now(),
    tiers,
    chaosRps: s.chaos?.state === "running" ? s.chaos.rps : null,
  };
}

/**
 * The arena, live, while the component is mounted: the multiplexed socket
 * first (one frame to call, a snapshot a second back), server-sent events if
 * the socket will not carry it, and a poll every two seconds as the last
 * resort. The session is refreshed first (`/v1/me`), because the socket takes
 * the caller from the cookie once, at the upgrade.
 */
export function useArena(enabled = true): Arena {
  const [state, setState] = useState<Arena>({
    snapshot: null,
    history: [],
    via: null,
    receivedAt: null,
    error: null,
  });
  const closer = useRef<(() => void) | null>(null);

  useEffect(() => {
    if (!enabled) return;
    let stopped = false;
    const take = (snapshot: Snapshot, via: Via) => {
      if (stopped) return;
      setState((s) => ({
        snapshot,
        via,
        receivedAt: Date.now(),
        error: null,
        history: [...s.history, point(snapshot)].slice(-HISTORY),
      }));
    };
    const fail = (error: string) => !stopped && setState((s) => ({ ...s, error }));

    const poll = () => {
      const timer = window.setInterval(async () => {
        try {
          const res = await fetch(SNAPSHOT, { credentials: "include" });
          if (!res.ok) return fail(`the live view answered ${res.status}`);
          const body = (await res.json()) as { snapshot?: Snapshot };
          if (body.snapshot) take(body.snapshot, "poll");
        } catch {
          fail("the live view is unreachable");
        }
      }, 2000);
      closer.current = () => window.clearInterval(timer);
    };

    const sse = () => {
      const source = new EventSource(EVENTS, { withCredentials: true });
      let got = false;
      source.onmessage = (e) => {
        try {
          const body = JSON.parse(e.data) as { snapshot?: Snapshot };
          if (body.snapshot) {
            got = true;
            take(body.snapshot, "sse");
          }
        } catch {
          /* a frame that does not parse is skipped; the next one is whole */
        }
      };
      source.onerror = () => {
        if (!got) {
          source.close();
          poll();
        }
      };
      closer.current = () => source.close();
    };

    const socket = () => {
      const url = new URL(SOCKET, window.location.href);
      url.protocol = url.protocol.replace("http", "ws");
      const ws = new WebSocket(url);
      let got = false;
      ws.onopen = () => ws.send(JSON.stringify({ type: "call", id: "arena", method: METHOD, body: {} }));
      ws.onmessage = (e) => {
        try {
          const frame = JSON.parse(String(e.data)) as {
            type: string;
            body?: { snapshot?: Snapshot };
            error?: string;
          };
          if (frame.type === "data" && frame.body?.snapshot) {
            got = true;
            take(frame.body.snapshot, "websocket");
          } else if (frame.type === "error" || frame.type === "end") {
            ws.close();
          }
        } catch {
          /* skipped, as above */
        }
      };
      ws.onclose = () => {
        if (stopped) return;
        // Refused or never carried a frame: the socket is not the way today.
        if (!got) return sse();
        // It carried frames and dropped: come back on the same transport.
        window.setTimeout(() => !stopped && socket(), 1500);
      };
      closer.current = () => ws.close();
    };

    void fetch("/v1/me", { credentials: "include" })
      .catch(() => null)
      .finally(() => !stopped && socket());

    return () => {
      stopped = true;
      closer.current?.();
    };
  }, [enabled]);

  return state;
}

/** Seconds since the last frame, re-rendered every second. */
export function useAge(receivedAt: number | null): number | null {
  const [now, setNow] = useState(() => Date.now());
  useEffect(() => {
    const t = window.setInterval(() => setNow(Date.now()), 1000);
    return () => window.clearInterval(t);
  }, []);
  return receivedAt === null ? null : Math.max(0, (now - receivedAt) / 1000);
}

/** A figure or the dash: absent is never shown as zero. */
export function fig(v: number | null | undefined, digits = 0, unit = ""): string {
  if (v === null || v === undefined || !Number.isFinite(v)) return "—";
  return `${v.toFixed(digits)}${unit}`;
}
