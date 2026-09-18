"use client";

// Data hooks: fetch once, poll when asked. Small on purpose; swap for
// TanStack Query when the page count justifies it.
import { useCallback, useEffect, useRef, useState } from "react";
import type { ZodType } from "zod";

import { ApiError } from "./client";

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
  if (e instanceof Error) return e.message.includes("fetch") ? "finance API unreachable" : e.message;
  return String(e);
}

export type Feed = {
  /** Connected and receiving. False while (re)connecting. */
  live: boolean;
  /** The last connection error, if the browser gave one. */
  error: string | null;
};

/**
 * A server-sent event stream (a transcoded server-streaming RPC). Every
 * `data:` frame is JSON, validated by `schema` and handed to `onEvent`;
 * an `event: error` frame ends the stream with the protocol's envelope.
 * The browser reconnects on its own; `live` says whether it is connected.
 *
 * In development the API needs a bearer token, which EventSource cannot
 * carry, so there the caller should fall back to polling when `live`
 * stays false.
 */
export function useEvents<T>(url: string | null, schema: ZodType<T>, onEvent: (event: T) => void): Feed {
  const [live, setLive] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const handler = useRef(onEvent);
  useEffect(() => {
    handler.current = onEvent;
  });
  useEffect(() => {
    if (!url || typeof window === "undefined" || !("EventSource" in window)) return;
    const source = new EventSource(url, { withCredentials: true });
    source.onopen = () => {
      setLive(true);
      setError(null);
    };
    source.onmessage = (m) => {
      try {
        handler.current(schema.parse(JSON.parse(m.data as string)));
      } catch (e) {
        setError(describe(e));
      }
    };
    source.addEventListener("error", (ev) => {
      setLive(false);
      const data = (ev as MessageEvent).data as string | undefined;
      if (data) {
        try {
          const env = JSON.parse(data) as { error?: string; code?: string };
          setError(env.error ?? env.code ?? "stream error");
        } catch {
          setError("stream error");
        }
      }
    });
    return () => {
      source.close();
      setLive(false);
    };
    // The schema is a module constant; only the URL decides the stream.
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [url]);
  return { live, error };
}
