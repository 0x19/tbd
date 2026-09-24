"use client";

import { useCallback, useEffect, useState } from "react";

/**
 * The agents the model service speaks as (RFC 0011), from `GET /v1/llm/agents`
 * on this origin: who each is, the tier it runs on, and whether this caller may
 * use it. Never its instructions; the service does not send them.
 */
export type Agent = {
  id: string;
  name: string;
  persona: string;
  tier: "TIER_FAST" | "TIER_DEEP";
  available: boolean;
};

let cached: Promise<Agent[] | null> | null = null;

function fetchAgents(): Promise<Agent[] | null> {
  cached ??= fetch("/v1/llm/agents", { credentials: "include", headers: { accept: "application/json" } })
    .then(async (res) => {
      if (!res.ok) return null;
      const body = (await res.json()) as { agents?: Agent[] };
      return (body.agents ?? []).map((a) => ({ ...a, available: !!a.available }));
    })
    .catch(() => null);
  return cached;
}

/** The agents, once per page life when `enabled`; `null` until known, and when the service does not answer. */
export function useAgents(enabled: boolean): Agent[] | null {
  const [agents, setAgents] = useState<Agent[] | null>(null);
  useEffect(() => {
    if (!enabled) return;
    let live = true;
    void fetchAgents().then((a) => {
      if (live) setAgents(a);
    });
    return () => {
      live = false;
    };
  }, [enabled]);
  return agents;
}

export type Budget = { used: number; limit: number; unlimited: boolean };

/** This caller's token budget for today (`GET /v1/llm/budget`), and a refresh for after a turn. */
export function useBudget(enabled: boolean): { budget: Budget | null; refresh: () => Promise<void> } {
  const [budget, setBudget] = useState<Budget | null>(null);
  const refresh = useCallback(async () => {
    try {
      const res = await fetch("/v1/llm/budget", { credentials: "include" });
      if (!res.ok) return;
      const b = (await res.json()) as { used_today: string; tokens_per_day: string; unlimited: boolean };
      setBudget({ used: Number(b.used_today), limit: Number(b.tokens_per_day), unlimited: b.unlimited });
    } catch {
      /* the status line shows a dash */
    }
  }, []);
  useEffect(() => {
    if (enabled) void refresh();
  }, [enabled, refresh]);
  return { budget, refresh };
}
