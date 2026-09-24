"use client";

import { ThemePresetSelector } from "@/components/theme-preset-selector";
import { ThemeSwitch } from "@/components/theme-switch";
import { cn } from "@/lib/utils";

export function HeaderThemeControls({ className }: { className?: string }) {
  return (
    <div className={cn("flex shrink-0 items-center gap-1.5", className)}>
      <ThemeSwitch triggerClassName="size-9 scale-100 rounded-lg" triggerId="header-theme-trigger" />
      <ThemePresetSelector triggerId="header-theme-preset-trigger" contentId="header-theme-preset-content" />
    </div>
  );
}
