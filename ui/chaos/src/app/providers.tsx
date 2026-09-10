"use client";

import { createContext, useContext, useEffect, useMemo, useState } from "react";

import SearchProvider from "@/components/search-provider";
import { ThemeProvider } from "@/components/theme-provider";
import { api } from "@/lib/api/client";
import { useFetch, useGlobalFeed } from "@/lib/api/hooks";
import type { GlobalEvent, Overview } from "@/lib/api/schema";

export type ActivityItem = { at: number; event: GlobalEvent };

type ChaosContext = {
  overview: Overview | null;
  error: string | null;
  connected: boolean;
  reload: () => void;
  lastEvent: GlobalEvent | null;
  /** The last events of this session, newest first. */
  activity: ActivityItem[];
};

const Ctx = createContext<ChaosContext>({
  overview: null,
  error: null,
  connected: false,
  reload: () => {},
  lastEvent: null,
  activity: [],
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
  const { connected } = useGlobalFeed((e) => {
    setLastEvent(e);
    setActivity((a) => [{ at: Date.now(), event: e }, ...a].slice(0, 30));
    if (e.type === "run_started" || e.type === "run_finished") overview.reload();
  });
  const value = useMemo<ChaosContext>(
    () => ({
      overview: overview.data,
      error: overview.error,
      connected,
      reload: overview.reload,
      lastEvent,
      activity,
    }),
    [overview.data, overview.error, overview.reload, connected, lastEvent, activity],
  );
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}
