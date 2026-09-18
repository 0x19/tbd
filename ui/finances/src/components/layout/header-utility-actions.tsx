"use client";

import { HeaderThemeControls } from "@/components/layout/header-theme-controls";
import { HeaderUser } from "@/components/layout/header-user";
import { LangToggle } from "@/components/lang-toggle";
import { cn } from "@/lib/utils";

export function HeaderUtilityActions({ className }: { className?: string }) {
  return (
    <div className={cn("flex shrink-0 items-center gap-2", className)}>
      {/* Language first: it changes everything on the page, and unlike the
          theme it stays reachable on a phone. */}
      <LangToggle />
      <HeaderThemeControls className="hidden sm:flex" />
      <HeaderUser />
    </div>
  );
}
