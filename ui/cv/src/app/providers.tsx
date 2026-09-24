"use client";

import { createContext, useContext } from "react";

import { ThemeProvider } from "@/components/theme-provider";
import { api } from "@/lib/api/client";
import { type Loadable, useFetch } from "@/lib/api/hooks";
import type { Me } from "@/lib/api/schema";
import { LangProvider } from "@/lib/i18n";

const MeContext = createContext<Loadable<Me>>({
  data: null,
  error: null,
  loading: true,
  reload: () => {},
  setData: () => {},
});

/** Who is signed in, from the principal Envoy verified (`GET /v1/me`). */
export function useMe() {
  return useContext(MeContext);
}

/** Theme, language, and the signed-in person. */
export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
      <LangProvider>
        <MeProvider>{children}</MeProvider>
      </LangProvider>
    </ThemeProvider>
  );
}

function MeProvider({ children }: { children: React.ReactNode }) {
  const me = useFetch(() => api.me(), 5 * 60_000);
  return <MeContext.Provider value={me}>{children}</MeContext.Provider>;
}
