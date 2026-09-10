"use client";

import { ThemeProvider } from "next-themes";
import { createContext, useContext, useMemo, useState } from "react";
import { api } from "@/lib/api/client";
import { useFetch, useGlobalFeed } from "@/lib/api/hooks";
import type { GlobalEvent, Overview } from "@/lib/api/schema";

type ChaosContext = {
  overview: Overview | null;
  error: string | null;
  connected: boolean;
  reload: () => void;
  lastEvent: GlobalEvent | null;
};

const Ctx = createContext<ChaosContext>({
  overview: null,
  error: null,
  connected: false,
  reload: () => {},
  lastEvent: null,
});

export function useChaos() {
  return useContext(Ctx);
}

/** Theme plus the overview every page and the shell share. */
export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
      <ChaosProvider>{children}</ChaosProvider>
    </ThemeProvider>
  );
}

function ChaosProvider({ children }: { children: React.ReactNode }) {
  const overview = useFetch(() => api.overview(), 10_000);
  const [lastEvent, setLastEvent] = useState<GlobalEvent | null>(null);
  const { connected } = useGlobalFeed((e) => {
    setLastEvent(e);
    if (e.type === "run_started" || e.type === "run_finished") overview.reload();
  });
  const value = useMemo<ChaosContext>(
    () => ({
      overview: overview.data,
      error: overview.error,
      connected,
      reload: overview.reload,
      lastEvent,
    }),
    [overview.data, overview.error, overview.reload, connected, lastEvent],
  );
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}
