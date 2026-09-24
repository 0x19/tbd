"use client";

import { Check, ChevronDown, Shuffle } from "lucide-react";
import { useTheme } from "next-themes";
import * as React from "react";

import { Button } from "@/components/ui/button";
import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
} from "@/components/ui/command";
import { Popover, PopoverContent, PopoverTrigger } from "@/components/ui/popover";
import { useT } from "@/lib/i18n";
import {
  applyThemePresetToDocument,
  getPresetLabel,
  getPresetPreviewColors,
  listPresetIdsSorted,
  THEME_PRESET_STORAGE_KEY,
} from "@/lib/theme-preset-apply";
import { defaultPresets } from "@/lib/theme-presets";
import { cn } from "@/lib/utils";

/** one / few / other, the way both English and Croatian count: 1 tema,
 *  2–4 teme, 5+ tema (English folds few into other). */
function plural(n: number): "one" | "few" | "other" {
  if (n % 10 === 1 && n % 100 !== 11) return "one";
  if (n % 10 >= 2 && n % 10 <= 4 && (n % 100 < 12 || n % 100 > 14)) return "few";
  return "other";
}

function ThemeSwatches({
  colors,
  className,
}: {
  colors: [string, string, string, string];
  className?: string;
}) {
  return (
    <span className={cn("inline-flex gap-0.5", className)} aria-hidden>
      {colors.map((c, i) => (
        <span
          key={i}
          className={cn("border-border size-4 shrink-0 rounded-sm border", i > 0 && "hidden sm:inline-block")}
          style={{ backgroundColor: c }}
        />
      ))}
    </span>
  );
}

export function ThemePresetSelector({
  className,
  align = "end",
  triggerId,
  contentId,
}: {
  className?: string;
  align?: "start" | "center" | "end";
  triggerId?: string;
  contentId?: string;
}) {
  const t = useT();
  const { resolvedTheme } = useTheme();
  const [mounted, setMounted] = React.useState(false);
  const [open, setOpen] = React.useState(false);
  const [presetId, setPresetId] = React.useState<string | null>(null);

  React.useEffect(() => {
    setMounted(true);
  }, []);

  React.useLayoutEffect(() => {
    const saved = localStorage.getItem(THEME_PRESET_STORAGE_KEY);
    const id = saved && (saved === "default" || defaultPresets[saved]) ? saved : "default";
    setPresetId(id);
  }, []);

  React.useLayoutEffect(() => {
    if (presetId === null) return;
    applyThemePresetToDocument(presetId);
    localStorage.setItem(THEME_PRESET_STORAGE_KEY, presetId);
  }, [presetId]);

  const activeId = presetId ?? "default";
  const mode: "light" | "dark" = mounted && resolvedTheme === "dark" ? "dark" : "light";

  const previewColors = React.useMemo(() => getPresetPreviewColors(activeId, mode), [activeId, mode]);

  const allIds = React.useMemo(() => listPresetIdsSorted(), []);

  const [search, setSearch] = React.useState("");

  const filteredIds = React.useMemo(() => {
    const q = search.trim().toLowerCase();
    if (!q) return allIds;
    return allIds.filter((id) => {
      const label = getPresetLabel(id).toLowerCase();
      return label.includes(q) || id.toLowerCase().includes(q);
    });
  }, [allIds, search]);

  const shuffle = () => {
    const pool = allIds.filter((id) => id !== "default");
    const pick = pool[Math.floor(Math.random() * pool.length)] ?? "default";
    setPresetId(pick);
  };

  return (
    <Popover open={open} onOpenChange={setOpen}>
      <PopoverTrigger asChild>
        <Button
          {...(triggerId ? { id: triggerId } : {})}
          {...(contentId ? { "aria-controls": contentId } : {})}
          variant="outline"
          size="sm"
          className={cn(
            "border-border h-9 max-w-full min-w-0 shrink gap-2 overflow-hidden rounded-lg px-2.5 font-normal",
            className,
          )}
          aria-label={t("nav.choose_color_theme")}
        >
          <ThemeSwatches colors={previewColors} />
          <span className="hidden max-w-[min(11rem,calc(100vw-7rem))] min-w-0 truncate text-sm sm:block">
            {getPresetLabel(activeId)}
          </span>
          <ChevronDown className="text-muted-foreground size-4 shrink-0 opacity-70" />
        </Button>
      </PopoverTrigger>
      <PopoverContent
        {...(contentId ? { id: contentId } : {})}
        align={align}
        className="w-[min(100vw-1.5rem,22rem)] p-0"
        sideOffset={8}
      >
        <Command shouldFilter={false}>
          <CommandInput placeholder={t("nav.search_themes")} value={search} onValueChange={setSearch} />
          <div className="text-muted-foreground flex items-center justify-between border-b px-3 py-2 text-xs">
            <span>{t(`nav.themes_${plural(filteredIds.length)}`, { n: filteredIds.length })}</span>
            <Button
              type="button"
              variant="ghost"
              size="icon"
              className="size-8"
              onClick={shuffle}
              aria-label={t("nav.random_theme")}
            >
              <Shuffle className="size-4" />
            </Button>
          </div>
          <CommandList className="max-h-[min(50vh,320px)]">
            <CommandEmpty>{t("nav.no_themes_found")}</CommandEmpty>
            <CommandGroup heading={t("nav.builtin_themes")}>
              {filteredIds.map((id) => {
                const colors = getPresetPreviewColors(id, mode);
                const label = getPresetLabel(id);
                return (
                  <CommandItem
                    key={id}
                    value={`${id} ${label}`}
                    onSelect={() => {
                      setPresetId(id);
                      setOpen(false);
                      setSearch("");
                    }}
                    className="min-w-0 gap-2 py-2"
                  >
                    <ThemeSwatches colors={colors} />
                    <span className="min-w-0 flex-1 truncate text-left">{label}</span>
                    {id === activeId ? <Check className="text-primary size-4 shrink-0" /> : null}
                  </CommandItem>
                );
              })}
            </CommandGroup>
          </CommandList>
        </Command>
      </PopoverContent>
    </Popover>
  );
}
