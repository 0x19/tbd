"use client";

import { IconCheck, IconMoon, IconSun } from "@tabler/icons-react";
import { useTheme } from "next-themes";
import * as React from "react";

import { Button, type ButtonProps } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

interface ThemeSwitchProps {
  align?: "start" | "center" | "end";
  contentClassName?: string;
  triggerClassName?: string;
  triggerId?: string;
  triggerSize?: ButtonProps["size"];
  triggerVariant?: ButtonProps["variant"];
}

export function ThemeSwitch({
  align = "end",
  contentClassName,
  triggerClassName,
  triggerId,
  triggerSize = "icon",
  triggerVariant = "ghost",
}: ThemeSwitchProps = {}) {
  const t = useT();
  const { theme, setTheme } = useTheme();

  return (
    <DropdownMenu modal={false}>
      <DropdownMenuTrigger asChild>
        <Button
          {...(triggerId ? { id: triggerId } : {})}
          variant={triggerVariant}
          size={triggerSize}
          className={cn("scale-95 rounded-full", triggerClassName)}
          aria-label={t("nav.toggle_theme")}
        >
          <IconSun className="size-[1.2rem] scale-100 rotate-0 transition-all dark:scale-0 dark:-rotate-90" />
          <IconMoon className="absolute size-[1.2rem] scale-0 rotate-90 transition-all dark:scale-100 dark:rotate-0" />
          <span className="sr-only">{t("nav.toggle_theme")}</span>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align={align} className={contentClassName}>
        <DropdownMenuItem onClick={() => setTheme("light")}>
          {t("nav.theme.light")}{" "}
          <IconCheck size={14} className={cn("ml-auto", theme !== "light" && "hidden")} />
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("dark")}>
          {t("nav.theme.dark")}
          <IconCheck size={14} className={cn("ml-auto", theme !== "dark" && "hidden")} />
        </DropdownMenuItem>
        <DropdownMenuItem onClick={() => setTheme("system")}>
          {t("nav.theme.system")}
          <IconCheck size={14} className={cn("ml-auto", theme !== "system" && "hidden")} />
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
