"use client";

import { IconArrowRightDashed, IconDeviceLaptop, IconMoon, IconSun } from "@tabler/icons-react";
import { useRouter } from "next/navigation";
import { useTheme } from "next-themes";
import * as React from "react";

import { useFinance } from "@/app/providers";
import {
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import { navGroups } from "@/data/sidebar-data";
import { useT } from "@/lib/i18n";

import { useSearch } from "./search-provider";

/** ⌘K: pages, the party scope, and the theme. */
export function CommandMenu() {
  const t = useT();
  const router = useRouter();
  const { setTheme } = useTheme();
  const { open, setOpen } = useSearch();
  const { parties, setScope } = useFinance();

  const run = React.useCallback(
    (command: () => unknown) => {
      setOpen(false);
      command();
    },
    [setOpen],
  );

  return (
    <CommandDialog modal open={open} onOpenChange={setOpen}>
      <CommandInput placeholder={t("nav.go_to")} />
      <CommandList>
        <CommandEmpty>{t("nav.no_results")}</CommandEmpty>
        {navGroups.map((group) => (
          <CommandGroup key={group.title} heading={t(group.title)}>
            {group.items
              .flatMap((item) => (item.url ? [{ title: t(item.title), url: item.url }] : []))
              .map((item) => (
                <CommandItem
                  key={item.url}
                  value={item.title}
                  onSelect={() => run(() => router.push(item.url))}
                >
                  <div className="mr-2 flex size-4 items-center justify-center">
                    <IconArrowRightDashed className="text-muted-foreground/80 size-2" />
                  </div>
                  {item.title}
                </CommandItem>
              ))}
          </CommandGroup>
        ))}
        <CommandSeparator />
        <CommandGroup heading={t("nav.show")}>
          <CommandItem onSelect={() => run(() => setScope("all"))}>{t("common.combined")}</CommandItem>
          {parties.map((p) => (
            <CommandItem key={p.id} onSelect={() => run(() => setScope(p.id))}>
              {p.display_name}
            </CommandItem>
          ))}
        </CommandGroup>
        <CommandSeparator />
        <CommandGroup heading={t("nav.theme")}>
          <CommandItem onSelect={() => run(() => setTheme("light"))}>
            <IconSun /> <span>{t("nav.theme.light")}</span>
          </CommandItem>
          <CommandItem onSelect={() => run(() => setTheme("dark"))}>
            <IconMoon className="scale-90" />
            <span>{t("nav.theme.dark")}</span>
          </CommandItem>
          <CommandItem onSelect={() => run(() => setTheme("system"))}>
            <IconDeviceLaptop />
            <span>{t("nav.theme.system")}</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
