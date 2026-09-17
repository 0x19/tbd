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

import { useSearch } from "./search-provider";

/** ⌘K: pages, the party scope, and the theme. */
export function CommandMenu() {
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
      <CommandInput placeholder="Go to…" />
      <CommandList>
        <CommandEmpty>Nothing found.</CommandEmpty>
        {navGroups.map((group) => (
          <CommandGroup key={group.title} heading={group.title}>
            {group.items
              .flatMap((item) => (item.url ? [{ title: item.title, url: item.url }] : []))
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
        <CommandGroup heading="Show">
          <CommandItem onSelect={() => run(() => setScope("all"))}>Combined</CommandItem>
          {parties.map((p) => (
            <CommandItem key={p.id} onSelect={() => run(() => setScope(p.id))}>
              {p.display_name}
            </CommandItem>
          ))}
        </CommandGroup>
        <CommandSeparator />
        <CommandGroup heading="Theme">
          <CommandItem onSelect={() => run(() => setTheme("light"))}>
            <IconSun /> <span>Light</span>
          </CommandItem>
          <CommandItem onSelect={() => run(() => setTheme("dark"))}>
            <IconMoon className="scale-90" />
            <span>Dark</span>
          </CommandItem>
          <CommandItem onSelect={() => run(() => setTheme("system"))}>
            <IconDeviceLaptop />
            <span>System</span>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </CommandDialog>
  );
}
