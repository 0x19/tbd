"use client";

import { createContext, useContext, useEffect, useMemo, useState } from "react";

import SearchProvider from "@/components/search-provider";
import { ThemeProvider } from "@/components/theme-provider";
import { api } from "@/lib/api/client";
import { useFetch, useGlobalFeed } from "@/lib/api/hooks";
import type { GlobalEvent, KindDescriptor, Overview, QueuedRun } from "@/lib/api/schema";
import { kindOf } from "@/lib/kinds";

export type ActivityItem = { at: number; event: GlobalEvent };

type ChaosContext = {
  overview: Overview | null;
  error: string | null;
  connected: boolean;
  reload: () => void;
  lastEvent: GlobalEvent | null;
  /** The last events of this session, newest first. */
  activity: ActivityItem[];
  /** What waits for the active slot, front first. */
  queue: QueuedRun[];
  /** The registered service kinds, registry order; empty until the overview loads. */
  kinds: KindDescriptor[];
  /** The descriptor of a kind by name; synthesised when the API did not list it. */
  kindOf: (name: string) => KindDescriptor;
};

const Ctx = createContext<ChaosContext>({
  overview: null,
  error: null,
  connected: false,
  reload: () => {},
  lastEvent: null,
  activity: [],
  queue: [],
  kinds: [],
  kindOf: (name) => kindOf([], name),
});

export function useChaos() {
  return useContext(Ctx);
}

interface Props {
  children: React.ReactNode;
}

/** Theme, ⌘K search state (the kit's SearchProvider) and the chaos overview context. */
export function Providers({ children }: Props) {
  const [open, setOpen] = useState(false);

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setOpen((open) => !open);
      }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, []);

  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
      <ChaosProvider>
        <SearchProvider value={{ open, setOpen }}>{children}</SearchProvider>
      </ChaosProvider>
    </ThemeProvider>
  );
}

function ChaosProvider({ children }: Props) {
  const overview = useFetch(() => api.overview(), 10_000);
  const [lastEvent, setLastEvent] = useState<GlobalEvent | null>(null);
  const [activity, setActivity] = useState<ActivityItem[]>([]);
  const [liveQueue, setLiveQueue] = useState<QueuedRun[] | null>(null);
  const { connected } = useGlobalFeed((e) => {
    setLastEvent(e);
    setActivity((a) => [{ at: Date.now(), event: e }, ...a].slice(0, 30));
    if (e.type === "queue_changed") setLiveQueue(e.queue);
    if (e.type === "run_started" || e.type === "run_finished" || e.type === "schedules_changed")
      overview.reload();
  });
  const overviewQueue = overview.data?.queue;
  const queue = useMemo(() => liveQueue ?? overviewQueue ?? [], [liveQueue, overviewQueue]);
  const overviewKinds = overview.data?.kinds;
  const kinds = useMemo(() => overviewKinds ?? [], [overviewKinds]);
  const value = useMemo<ChaosContext>(
    () => ({
      overview: overview.data,
      error: overview.error,
      connected,
      reload: overview.reload,
      lastEvent,
      activity,
      queue,
      kinds,
      kindOf: (name: string) => kindOf(kinds, name),
    }),
    [overview.data, overview.error, overview.reload, connected, lastEvent, activity, queue, kinds],
  );
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}
