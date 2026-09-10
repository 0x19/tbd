"use client";

// Data hooks: polling for lists, SSE for anything live. Small on purpose;
// swap for TanStack Query when the page count justifies it.
import { useCallback, useEffect, useRef, useState } from "react";
import { ApiError, api, feeds } from "./client";
import type { GlobalEvent, InstanceInfo, RunFeed, RunRecord, LoadSnapshot } from "./schema";

export type Loadable<T> = {
  data: T | null;
  error: string | null;
  loading: boolean;
  reload: () => void;
  setData: (d: T) => void;
};

/** Fetch once, refetch every `intervalMs` (0 = never), refetch on `deps`. */
export function useFetch<T>(fetcher: () => Promise<T>, intervalMs = 0, deps: unknown[] = []): Loadable<T> {
  const [data, setData] = useState<T | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [loading, setLoading] = useState(true);
  const [tick, setTick] = useState(0);
  const fetcherRef = useRef(fetcher);
  useEffect(() => {
    fetcherRef.current = fetcher;
  });

  useEffect(() => {
    let cancelled = false;
    const go = () =>
      fetcherRef
        .current()
        .then((d) => {
          if (cancelled) return;
          setData(d);
          setError(null);
        })
        .catch((e: unknown) => {
          if (cancelled) return;
          setError(describe(e));
        })
        .finally(() => {
          if (!cancelled) setLoading(false);
        });
    void go();
    const timer = intervalMs > 0 ? setInterval(go, intervalMs) : undefined;
    return () => {
      cancelled = true;
      if (timer) clearInterval(timer);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [tick, intervalMs, ...deps]);

  const reload = useCallback(() => setTick((t) => t + 1), []);
  return { data, error, loading, reload, setData };
}

export function describe(e: unknown): string {
  if (e instanceof ApiError) return e.message;
  if (e instanceof Error) return e.message.includes("fetch") ? "chaos API unreachable" : e.message;
  return String(e);
}

/** The global feed. Returns the last event and the connection state. */
export function useGlobalFeed(onEvent?: (e: GlobalEvent) => void) {
  const [last, setLast] = useState<GlobalEvent | null>(null);
  const [connected, setConnected] = useState(false);
  const cb = useRef(onEvent);
  useEffect(() => {
    cb.current = onEvent;
  });
  useEffect(() => {
    const close = feeds.global(
      (e) => {
        setLast(e);
        cb.current?.(e);
      },
      () => setConnected(false),
      () => setConnected(true),
    );
    return close;
  }, []);
  return { last, connected };
}

/** Stack instances, refreshed by polling and by `stack_changed` events. */
export function useStack(): Loadable<InstanceInfo[]> {
  const loadable = useFetch(() => api.stack(), 2000);
  const { setData } = loadable;
  useGlobalFeed((e) => {
    if (e.type === "stack_changed") setData(e.instances);
  });
  return loadable;
}

export type RunLive = {
  record: RunRecord | null;
  phase: string | null;
  samples: LoadSnapshot[];
  events: { at_s: number; action: string; error: string | null }[];
  finished: boolean;
  error: string | null;
};

const EMPTY: RunLive = {
  record: null,
  phase: null,
  samples: [],
  events: [],
  finished: false,
  error: null,
};

/** Follow one run: replay plus live frames until `finished`. */
export function useRunFeed(id: string | null): RunLive {
  // State is tagged with the run id it belongs to, so switching runs shows a
  // clean slate without a setState in an effect.
  const [state, setState] = useState<{ id: string | null; live: RunLive }>({
    id: null,
    live: EMPTY,
  });
  const update = useCallback(
    (runId: string, f: (s: RunLive) => RunLive) =>
      setState((cur) => ({
        id: runId,
        live: f(cur.id === runId ? cur.live : EMPTY),
      })),
    [],
  );

  useEffect(() => {
    if (!id) return;
    // A finished run has its record already; load it so the page is complete
    // even if the SSE replay is a single frame.
    api
      .run(id)
      .then((record) =>
        update(id, (s) =>
          s.record
            ? s
            : {
                ...s,
                record,
                samples: record.samples,
                events: record.events,
              },
        ),
      )
      .catch((e: unknown) => update(id, (s) => ({ ...s, error: describe(e) })));
    const close = feeds.run(
      id,
      (f: RunFeed) =>
        update(id, (s) => {
          switch (f.type) {
            case "started":
              return { ...s, samples: [], events: [] };
            case "phase":
              return { ...s, phase: f.name };
            case "load":
              return {
                ...s,
                samples: [...s.samples, f.snapshot],
              };
            case "timeline":
              return { ...s, events: [...s.events, f.event] };
            case "finished":
              return {
                ...s,
                record: f.run,
                samples: f.run.samples.length ? f.run.samples : s.samples,
                events: f.run.events.length ? f.run.events : s.events,
                finished: true,
                phase: null,
                error: null,
              };
          }
        }),
      () => update(id, (s) => (s.finished ? s : { ...s, error: s.error ?? "feed disconnected" })),
    );
    return close;
  }, [id, update]);

  return state.id === id ? state.live : EMPTY;
}
