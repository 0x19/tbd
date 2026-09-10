"use client";

import {
  IconArrowRightDashed,
  IconDeviceLaptop,
  IconMoon,
  IconPlayerPlay,
  IconSun,
} from "@tabler/icons-react";
import { useRouter } from "next/navigation";
import { useTheme } from "next-themes";
import * as React from "react";
import { toast } from "sonner";

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
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";

import { useSearch } from "./search-provider";
import { ScrollArea } from "./ui/scroll-area";

/** The kit's ⌘K menu with the chaos pages, one entry per runnable scenario, and theme. */
export function CommandMenu() {
  const router = useRouter();
  const { setTheme } = useTheme();
  const { open, setOpen } = useSearch();
  const scenarios = useFetch(() => api.scenarios(), 0, [open]);

  const runCommand = React.useCallback(
    (command: () => unknown) => {
      setOpen(false);
      command();
    },
    [setOpen],
  );

  const runScenario = async (id: string) => {
    try {
      const s = await api.runScenario(id);
      router.push(`/runs/view/?id=${s.id}`);
    } catch (e) {
      toast.error(describe(e));
    }
  };

  return (
    <CommandDialog modal open={open} onOpenChange={setOpen}>
      <CommandInput placeholder="Type a command or search..." />
      <CommandList>
        <ScrollArea type="hover" className="h-80 pr-1">
          <CommandEmpty>No results found.</CommandEmpty>
          {navGroups.map((group) => (
            <CommandGroup key={group.title} heading={group.title}>
              {group.items.map((navItem, i) => {
                if (navItem.url)
                  return (
                    <CommandItem
                      key={`${navItem.url}-${i}`}
                      value={navItem.title}
                      onSelect={() => runCommand(() => router.push(navItem.url))}
                    >
                      <div className="mr-2 flex h-4 w-4 items-center justify-center">
                        <IconArrowRightDashed className="text-muted-foreground/80 size-2" />
                      </div>
                      {navItem.title}
                    </CommandItem>
                  );

                return navItem.items?.map((subItem, i) => (
                  <CommandItem
                    key={`${subItem.url}-${i}`}
                    value={`${navItem.title} ${subItem.title}`}
                    onSelect={() => runCommand(() => router.push(subItem.url))}
                  >
                    <div className="mr-2 flex h-4 w-4 items-center justify-center">
                      <IconArrowRightDashed className="text-muted-foreground/80 size-2" />
                    </div>
                    {subItem.title}
                  </CommandItem>
                ));
              })}
            </CommandGroup>
          ))}
          <CommandSeparator />
          <CommandGroup heading="Run a scenario">
            {(scenarios.data ?? [])
              .filter((s) => s.ok)
              .map((s) => (
                <CommandItem
                  key={s.id}
                  value={`run scenario ${s.id} ${s.name ?? ""}`}
                  onSelect={() => runCommand(() => runScenario(s.id))}
                >
                  <IconPlayerPlay className="text-muted-foreground/80 mr-2 size-4" />
                  <span className="font-mono text-xs">{s.id}</span>
                  <span className="text-muted-foreground ml-auto truncate text-xs">{s.description}</span>
                </CommandItem>
              ))}
          </CommandGroup>
          <CommandSeparator />
          <CommandGroup heading="Theme">
            <CommandItem onSelect={() => runCommand(() => setTheme("light"))}>
              <IconSun /> <span>Light</span>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => setTheme("dark"))}>
              <IconMoon className="scale-90" />
              <span>Dark</span>
            </CommandItem>
            <CommandItem onSelect={() => runCommand(() => setTheme("system"))}>
              <IconDeviceLaptop />
              <span>System</span>
            </CommandItem>
          </CommandGroup>
        </ScrollArea>
      </CommandList>
    </CommandDialog>
  );
}
