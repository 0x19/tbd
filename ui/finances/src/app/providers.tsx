"use client";

import { createContext, useCallback, useContext, useEffect, useMemo, useState } from "react";

import SearchProvider from "@/components/search-provider";
import { ThemeProvider } from "@/components/theme-provider";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";
import type { Party } from "@/lib/api/schema";
import { LangProvider } from "@/lib/i18n";

/** Which of the caller's parties the pages show: a subset of the grant. */
export type Scope = "all" | string;

type FinanceContext = {
  /** The parties the caller may read, from the server. Never more. */
  parties: Party[];
  error: string | null;
  loading: boolean;
  reload: () => void;
  /** "all", or one party id. */
  scope: Scope;
  setScope: (s: Scope) => void;
  /** The party ids the current scope resolves to, for API calls. */
  partyIds: string[];
  /** Whether more than one party is readable, i.e. whether a toggle makes sense. */
  multi: boolean;
  partyName: (id: string) => string;
};

const Ctx = createContext<FinanceContext>({
  parties: [],
  error: null,
  loading: true,
  reload: () => {},
  scope: "all",
  setScope: () => {},
  partyIds: [],
  multi: false,
  partyName: (id) => id,
});

export function useFinance() {
  return useContext(Ctx);
}

const SCOPE_KEY = "finance-scope";

interface Props {
  children: React.ReactNode;
}

/** Theme, ⌘K search state (the kit's SearchProvider) and the finance scope. */
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
      <LangProvider>
        <FinanceProvider>
          <SearchProvider value={{ open, setOpen }}>{children}</SearchProvider>
        </FinanceProvider>
      </LangProvider>
    </ThemeProvider>
  );
}

function FinanceProvider({ children }: Props) {
  const parties = useFetch(() => api.parties(), 5 * 60_000);
  const [scope, setScopeState] = useState<Scope>(() => {
    if (typeof window === "undefined") return "all";
    try {
      return window.localStorage.getItem(SCOPE_KEY) ?? "all";
    } catch {
      return "all";
    }
  });
  const setScope = useCallback((s: Scope) => {
    setScopeState(s);
    try {
      window.localStorage.setItem(SCOPE_KEY, s);
    } catch {
      // ignore
    }
  }, []);

  const list = useMemo(() => parties.data?.parties ?? [], [parties.data]);
  // A remembered scope that the grant no longer covers falls back to all:
  // the server decides what is readable, the browser only remembers a choice.
  const effective = scope !== "all" && list.length && !list.some((p) => p.id === scope) ? "all" : scope;
  const partyIds = useMemo(
    () => (effective === "all" ? list.map((p) => p.id) : [effective]),
    [effective, list],
  );
  const partyName = useCallback(
    (id: string) => list.find((p) => p.id === id)?.display_name ?? id.slice(0, 8),
    [list],
  );

  const value = useMemo<FinanceContext>(
    () => ({
      parties: list,
      error: parties.error,
      loading: parties.loading,
      reload: parties.reload,
      scope: effective,
      setScope,
      partyIds,
      multi: list.length > 1,
      partyName,
    }),
    [list, parties.error, parties.loading, parties.reload, effective, setScope, partyIds, partyName],
  );
  return <Ctx.Provider value={value}>{children}</Ctx.Provider>;
}
