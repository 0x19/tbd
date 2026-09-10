// One fetch layer for the chaos API (docs/chaos/api.md). Every response is
// parsed through its Zod schema so pages never touch untyped JSON.
import type { z } from "zod";

import {
  type Behavior,
  CheckReply,
  GlobalEvent,
  InstanceInfo,
  type Job,
  type LoadRequest,
  Overview,
  QueuedRun,
  RunFeed,
  RunRecord,
  RunSummary,
  ScenarioDetail,
  ScenarioEntry,
  Schedule,
  type ScheduleSpec,
} from "./schema";

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
  stack: () => call(InstanceInfo.array(), "/stack"),
  stackStart: (name: string) =>
    call(InstanceInfo.array(), `/stack/${encodeURIComponent(name)}/start`, {
      method: "POST",
    }),
  stackStop: (name: string) =>
    call(InstanceInfo.array(), `/stack/${encodeURIComponent(name)}/stop`, {
      method: "POST",
    }),
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
  run: (id: string) => call(RunRecord, `/runs/${id}`),
  runScenario: (scenario: string) => call(RunSummary, "/runs", { method: "POST", json: { scenario } }),
  runLoad: (req: LoadRequest) => call(RunSummary, "/runs", { method: "POST", json: req }),
  runCancel: (id: string) => call(none, `/runs/${id}/cancel`, { method: "POST" }),
  runDelete: (id: string) => call(none, `/runs/${id}`, { method: "DELETE" }),
  validate: (body?: { protocol?: string; engine?: string; timeout?: string }) =>
    call(RunRecord, "/validate", { method: "POST", json: body ?? {} }),
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
    "finished",
    "run_started",
    "run_finished",
    "stack_changed",
    "queue_changed",
    "schedules_changed",
  ]) {
    source.addEventListener(type, handler as EventListener);
  }
  if (onError) source.onerror = onError;
  return () => source.close();
}

export const feeds = {
  run: (id: string, onItem: (f: RunFeed) => void, onError?: (e: Event) => void) =>
    subscribe(`/runs/${id}/events`, RunFeed, onItem, onError),
  global: (onItem: (f: GlobalEvent) => void, onError?: (e: Event) => void, onOpen?: () => void) =>
    subscribe("/events", GlobalEvent, onItem, onError, onOpen),
};
