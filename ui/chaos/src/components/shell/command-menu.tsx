"use client";

import { useRouter } from "next/navigation";
import { useEffect } from "react";
import { FlaskConical, ListChecks, Moon, Play, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { toast } from "sonner";
import {
  Command,
  CommandDialog,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
} from "@/components/ui/command";
import { api } from "@/lib/api/client";
import { describe, useFetch } from "@/lib/api/hooks";
import { NAV } from "./nav";
import { useChaos } from "./providers";

/** ⌘K: pages, scenarios to run, and a few actions. */
export function CommandMenu() {
  const router = useRouter();
  const { commandOpen, setCommandOpen } = useChaos();
  const { setTheme } = useTheme();
  const scenarios = useFetch(() => api.scenarios(), 0, [commandOpen]);

  useEffect(() => {
    const down = (e: KeyboardEvent) => {
      if (e.key === "k" && (e.metaKey || e.ctrlKey)) {
        e.preventDefault();
        setCommandOpen(!commandOpen);
      }
    };
    document.addEventListener("keydown", down);
    return () => document.removeEventListener("keydown", down);
  }, [commandOpen, setCommandOpen]);

  const go = (href: string) => {
    setCommandOpen(false);
    router.push(href);
  };

  const run = async (id: string) => {
    setCommandOpen(false);
    try {
      const s = await api.runScenario(id);
      router.push(`/runs/view/?id=${s.id}`);
    } catch (e) {
      toast.error(describe(e));
    }
  };

  return (
    <CommandDialog
      open={commandOpen}
      onOpenChange={setCommandOpen}
      title="Search"
      description="Pages, scenarios and actions"
    >
      <Command>
        <CommandInput placeholder="Search pages or run commands…" />
        <CommandList>
          <CommandEmpty>No results.</CommandEmpty>
          {NAV.map((group) => (
            <CommandGroup key={group.label} heading={group.label}>
              {group.items.map((item) => (
                <CommandItem
                  key={item.href}
                  value={`${group.label} ${item.title}`}
                  onSelect={() => go(item.href)}
                >
                  <item.icon />
                  <span>{item.title}</span>
                  <span className="ml-auto text-xs text-muted-foreground">{item.blurb}</span>
                </CommandItem>
              ))}
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
                  onSelect={() => run(s.id)}
                >
                  <Play />
                  <span className="font-mono text-xs">{s.id}</span>
                  <span className="ml-auto truncate text-xs text-muted-foreground">{s.description}</span>
                </CommandItem>
              ))}
          </CommandGroup>
          <CommandSeparator />
          <CommandGroup heading="Actions">
            <CommandItem value="run validate" onSelect={() => go("/validate/")}>
              <ListChecks />
              <span>Run validate</span>
            </CommandItem>
            <CommandItem value="new scenario" onSelect={() => go("/scenarios/view/?id=new")}>
              <FlaskConical />
              <span>New scenario</span>
            </CommandItem>
            <CommandItem
              value="light theme"
              onSelect={() => {
                setTheme("light");
                setCommandOpen(false);
              }}
            >
              <Sun />
              <span>Light theme</span>
            </CommandItem>
            <CommandItem
              value="dark theme"
              onSelect={() => {
                setTheme("dark");
                setCommandOpen(false);
              }}
            >
              <Moon />
              <span>Dark theme</span>
            </CommandItem>
          </CommandGroup>
        </CommandList>
      </Command>
    </CommandDialog>
  );
}
