import type { RunSummary } from "@/lib/api/schema";

/** Runs whose start falls in the last `hours` hours. */
export function inWindow(runs: RunSummary[], hours: number, offsetHours = 0): RunSummary[] {
  const now = Date.now();
  const end = now - offsetHours * 3600_000;
  const start = end - hours * 3600_000;
  return runs.filter((r) => {
    const t = new Date(r.started_at).getTime();
    return t >= start && t < end;
  });
}

/** Percentage change from `prev` to `cur`; 0 when both are 0. */
export function delta(cur: number, prev: number): number {
  if (prev === 0) return cur === 0 ? 0 : 100;
  return ((cur - prev) / prev) * 100;
}

export function failed(r: RunSummary): boolean {
  return r.status === "failed" || r.status === "error";
}

/** The last finished scenario run per scenario id. */
export function latestPerScenario(runs: RunSummary[]): RunSummary[] {
  const seen = new Set<string>();
  const out: RunSummary[] = [];
  for (const r of runs) {
    if (r.kind !== "scenario" || !r.scenario_id || r.status === "running") continue;
    if (seen.has(r.scenario_id)) continue;
    seen.add(r.scenario_id);
    out.push(r);
  }
  return out;
}
