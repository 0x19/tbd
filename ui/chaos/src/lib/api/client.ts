// One fetch layer for the chaos API (docs/chaos/api.md). Every response is
// parsed through its Zod schema so pages never touch untyped JSON.
import type { z } from "zod";

import {
  type AddInstance,
  type Behavior,
  CampaignDetail,
  CampaignEntry,
  CheckReply,
  Finding,
  FindingGroup,
  FindingSummary,
  GlobalEvent,
  InstanceInfo,
  type Job,
  type LoadRequest,
  Me,
  Overview,
  QueuedRun,
  type ReplayRequest,
  RunFeed,
  RunRecord,
  RunSummary,
  ScenarioDetail,
  ScenarioEntry,
  Schedule,
  type ScheduleSpec,
  type StressRequest,
  type ValidateRequest,
} from "./schema";

/** What `GET /findings` filters on; every key optional. */
export type FindingQuery = { run?: string; invariant?: string; campaign?: string; limit?: number };

function findingQuery(q: FindingQuery, grouped: boolean): string {
  const params = new URLSearchParams();
  if (grouped) params.set("grouped", "true");
  if (q.limit) params.set("limit", String(q.limit));
  for (const k of ["run", "invariant", "campaign"] as const) if (q[k]) params.set(k, q[k]);
  const s = params.toString();
  return s ? `?${s}` : "";
}

/**
 * Where the API is. Same origin in every deployment (chaos serve and Envoy
 * both serve the UI at the root and the API at /api/chaos/v1). `next dev` has no
 * API of its own, so it talks to `chaos serve` on its default port unless
 * NEXT_PUBLIC_CHAOS_API says otherwise.
 */
export function apiBase(): string {
  const configured = process.env.NEXT_PUBLIC_CHAOS_API;
  if (configured) return configured.replace(/\/$/, "");
  if (process.env.NODE_ENV === "development") return "http://127.0.0.1:7700/api/chaos/v1";
  if (typeof window !== "undefined") return `${window.location.origin}/api/chaos/v1`;
  return "/api/chaos/v1";
}

export class ApiError extends Error {
  constructor(
    public status: number,
    message: string,
  ) {
    super(message);
  }
}

const RELOGIN_KEY = "chaos-relogin-at";

/**
 * The browser session behind Envoy is gone (401 on an API call: the ID-token
 * cookie expired and the refresh failed, or the person signed out elsewhere).
 * A fetch cannot follow the sign-in redirect, a page load can: reload, and
 * Envoy sends the browser through auth and back to this URL. At most once a
 * minute, so a broken deployment shows its error instead of a reload loop.
 */
function relogin(): boolean {
  if (typeof window === "undefined") return false;
  const now = Date.now();
  let last = 0;
  try {
    last = Number(window.sessionStorage.getItem(RELOGIN_KEY) ?? 0);
  } catch {
    // storage unavailable: still reload once per page life
  }
  if (now - last < 60_000) return false;
  try {
    window.sessionStorage.setItem(RELOGIN_KEY, String(now));
  } catch {
    // ignore
  }
  window.location.reload();
  return true;
}

async function call<T>(
  schema: z.ZodType<T>,
  path: string,
  init?: RequestInit & { json?: unknown },
): Promise<T> {
  const { json, ...rest } = init ?? {};
  const res = await fetch(`${apiBase()}${path}`, {
    ...rest,
    headers: {
      accept: "application/json",
      ...(json !== undefined ? { "content-type": "application/json" } : {}),
      ...(rest.headers ?? {}),
    },
    body: json !== undefined ? JSON.stringify(json) : rest.body,
  });
  if (res.status === 401 && relogin()) {
    // The page is reloading; keep the caller waiting instead of showing an error.
    await new Promise<never>(() => {});
  }
  if (!res.ok) {
    let message = `${res.status} ${res.statusText}`;
    try {
      const body = (await res.json()) as { error?: string };
      if (body.error) message = body.error;
    } catch {
      // not JSON
    }
    throw new ApiError(res.status, message);
  }
  if (res.status === 204) return schema.parse(undefined);
  return schema.parse(await res.json());
}

const none: z.ZodType<void> = {
  parse: () => undefined,
} as unknown as z.ZodType<void>;

export const api = {
  overview: () => call(Overview, "/overview"),
  me: () => call(Me, "/me"),
  notifyTest: () => call(none, "/notify/test", { method: "POST", json: {} }),
  stack: () => call(InstanceInfo.array(), "/stack"),
  stackStart: (name: string) =>
    call(InstanceInfo.array(), `/stack/${encodeURIComponent(name)}/start`, {
      method: "POST",
    }),
  stackStop: (name: string) =>
    call(InstanceInfo.array(), `/stack/${encodeURIComponent(name)}/stop`, {
      method: "POST",
    }),
  stackClone: (name: string, count = 1) =>
    call(InstanceInfo.array(), `/stack/${encodeURIComponent(name)}/clone`, {
      method: "POST",
      json: { count },
    }),
  stackAdd: (body: AddInstance) => call(InstanceInfo.array(), "/stack", { method: "POST", json: body }),
  stackRemove: (name: string) =>
    call(InstanceInfo.array(), `/stack/${encodeURIComponent(name)}`, { method: "DELETE" }),
  stackBehavior: (name: string, behavior: Behavior) =>
    call(InstanceInfo.array(), `/stack/${encodeURIComponent(name)}/behavior`, {
      method: "PUT",
      json: behavior,
    }),
  scenarios: () => call(ScenarioEntry.array(), "/scenarios"),
  scenario: (id: string) => call(ScenarioDetail, `/scenarios/${id}`),
  scenarioSave: (id: string, text: string) =>
    call(ScenarioEntry, `/scenarios/${id}`, {
      method: "PUT",
      json: { text },
    }),
  scenarioDelete: (id: string) => call(none, `/scenarios/${id}`, { method: "DELETE" }),
  scenarioCheck: (text: string) =>
    call(CheckReply, "/scenarios/check", {
      method: "POST",
      json: { text },
    }),
  runs: (limit = 50) => call(RunSummary.array(), `/runs?limit=${limit}`),
  campaigns: () => call(CampaignEntry.array(), "/stress"),
  campaign: (id: string) => call(CampaignDetail, `/stress/${id}`),
  campaignSave: (id: string, text: string) =>
    call(CampaignEntry, `/stress/${id}`, { method: "PUT", json: { text } }),
  campaignDelete: (id: string) => call(none, `/stress/${id}`, { method: "DELETE" }),
  campaignCheck: (text: string) => call(CheckReply, "/stress/check", { method: "POST", json: { text } }),
  runStress: (req: StressRequest) => call(RunSummary, "/runs", { method: "POST", json: { stress: req } }),
  runReplay: (req: ReplayRequest) => call(RunSummary, "/runs", { method: "POST", json: { replay: req } }),
  findings: (q: FindingQuery = {}) => call(FindingSummary.array(), `/findings${findingQuery(q, false)}`),
  findingGroups: (q: FindingQuery = {}) => call(FindingGroup.array(), `/findings${findingQuery(q, true)}`),
  finding: (id: string) => call(Finding, `/findings/${id}`),
  findingDelete: (id: string) => call(none, `/findings/${id}`, { method: "DELETE" }),
  findingReplay: (id: string, body: { targets?: StressRequest["targets"]; attempts?: number } = {}) =>
    call(RunSummary, `/findings/${id}/replay`, { method: "POST", json: body }),
  run: (id: string) => call(RunRecord, `/runs/${id}`),
  runScenario: (scenario: string) => call(RunSummary, "/runs", { method: "POST", json: { scenario } }),
  runLoad: (req: LoadRequest) => call(RunSummary, "/runs", { method: "POST", json: req }),
  runCancel: (id: string) => call(none, `/runs/${id}/cancel`, { method: "POST" }),
  runDelete: (id: string) => call(none, `/runs/${id}`, { method: "DELETE" }),
  validate: (body?: ValidateRequest) => call(RunRecord, "/validate", { method: "POST", json: body ?? {} }),
  queue: () => call(QueuedRun.array(), "/queue"),
  enqueue: (jobs: Job[]) => call(QueuedRun.array(), "/queue", { method: "POST", json: { jobs } }),
  queueRemove: (id: string) => call(none, `/queue/${id}`, { method: "DELETE" }),
  queueClear: () => call(none, "/queue", { method: "DELETE" }),
  schedules: () => call(Schedule.array(), "/schedules"),
  scheduleCreate: (spec: ScheduleSpec) => call(Schedule, "/schedules", { method: "POST", json: spec }),
  scheduleUpdate: (id: string, spec: ScheduleSpec) =>
    call(Schedule, `/schedules/${id}`, { method: "PUT", json: spec }),
  scheduleDelete: (id: string) => call(none, `/schedules/${id}`, { method: "DELETE" }),
  scheduleRun: (id: string) => call(QueuedRun.array(), `/schedules/${id}/run`, { method: "POST", json: {} }),
};

/** Subscribe to an SSE endpoint; frames are parsed through `schema`. */
export function subscribe<T>(
  path: string,
  schema: z.ZodType<T>,
  onItem: (item: T) => void,
  onError?: (e: Event) => void,
  onOpen?: () => void,
): () => void {
  const source = new EventSource(`${apiBase()}${path}`);
  if (onOpen) source.onopen = () => onOpen();
  const handler = (ev: MessageEvent<string>) => {
    try {
      onItem(schema.parse(JSON.parse(ev.data)));
    } catch (e) {
      console.warn("bad SSE frame", ev.type, e);
    }
  };
  // Frames arrive with `event: <type>`; EventSource dispatches them by name.
  for (const type of [
    "started",
    "phase",
    "load",
    "timeline",
    "stress",
    "finding",
    "finished",
    "run_started",
    "run_finished",
    "stack_changed",
    "queue_changed",
    "schedules_changed",
  ]) {
    source.addEventListener(type, handler as EventListener);
  }
  source.onerror = (e) => {
    // EventSource hides the status; a stream that drops because the session is
    // gone is told apart by asking `/me`, which answers 401 in that case.
    void fetch(`${apiBase()}/me`, { headers: { accept: "application/json" } })
      .then((res) => {
        if (res.status === 401 && relogin()) return;
        onError?.(e);
      })
      .catch(() => onError?.(e));
  };
  return () => source.close();
}

export const feeds = {
  run: (id: string, onItem: (f: RunFeed) => void, onError?: (e: Event) => void) =>
    subscribe(`/runs/${id}/events`, RunFeed, onItem, onError),
  global: (onItem: (f: GlobalEvent) => void, onError?: (e: Event) => void, onOpen?: () => void) =>
    subscribe("/events", GlobalEvent, onItem, onError, onOpen),
};
