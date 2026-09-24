"use client";

import { ThemeProvider } from "@/components/theme-provider";
import { LangProvider } from "@/lib/i18n";

export function Providers({ children }: { children: React.ReactNode }) {
  return (
    <ThemeProvider attribute="class" defaultTheme="system" enableSystem disableTransitionOnChange>
      <LangProvider>{children}</LangProvider>
    </ThemeProvider>
  );
}
