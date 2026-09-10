import Link from "next/link";
import type { ReactNode } from "react";

import { Logo } from "@/components/logo";
import { ThemeSwitch } from "@/components/theme-switch";
import { site } from "@/data/site";

/** The page frame every flow shares: brand row, theme switch, the card in the middle. */
export function AuthShell({ children }: { children: ReactNode }) {
  return (
    <div className="bg-background text-foreground flex min-h-svh flex-col">
      <header className="flex items-center justify-between px-6 py-4">
        <Link href="/" className="flex items-center gap-2 text-sm font-medium">
          <Logo width={20} height={20} />
          {site.title}
        </Link>
        <ThemeSwitch />
      </header>
      <main className="flex flex-1 items-start justify-center px-4 pt-6 pb-12 sm:items-center sm:pt-0">
        {children}
      </main>
    </div>
  );
}

export function Loading() {
  return (
    <AuthShell>
      <p className="text-muted-foreground text-sm">Loading…</p>
    </AuthShell>
  );
}
