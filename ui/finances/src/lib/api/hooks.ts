"use client";

// Data hooks: fetch once, poll when asked. Small on purpose; swap for
// TanStack Query when the page count justifies it.
import { useCallback, useEffect, useRef, useState } from "react";

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
