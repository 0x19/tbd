"use client";

import { Search } from "lucide-react";
import Link from "next/link";
import { usePathname } from "next/navigation";

import { useChaos } from "@/app/providers";
import { useSearch } from "@/components/search-provider";
import { Button } from "@/components/ui/button";
import { crumbs } from "@/data/sidebar-data";
import { site } from "@/data/site";

import { StatusBadge } from "../status-badge";

/** The kit's sub-header: root breadcrumb, section, tail, and the command search. */
export function SubHeader() {
  const pathname = usePathname();
  const trail = crumbs(pathname);
  const { setOpen } = useSearch();
  const { overview } = useChaos();
  const active = overview?.active_run;

  return (
    <header className="bg-background hidden shrink-0 border-b px-4 py-4 sm:px-6 md:block">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:gap-6">
        <div className="flex min-w-0 flex-wrap items-center gap-3 text-sm">
          <span className="text-foreground font-medium">{site.title}</span>
          {trail.map((t, i) => (
            <span key={`${t}-${i}`} className="contents">
              <span className="text-muted-foreground">/</span>
              <span
                className={i === trail.length - 1 ? "text-muted-foreground" : "text-foreground font-medium"}
              >
                {t}
              </span>
            </span>
          ))}
        </div>

        <div className="relative xl:mx-auto xl:w-full xl:max-w-md">
          <Button
            type="button"
            variant="outline"
            onClick={() => setOpen(true)}
            aria-label="Open command palette"
            aria-keyshortcuts="Meta+K Control+K"
            className="bg-muted/70 text-muted-foreground hover:bg-muted/90 hover:text-foreground h-10 w-full justify-start rounded-full border-none px-3 font-normal shadow-none"
          >
            <Search className="size-4 shrink-0" aria-hidden="true" />
            <span className="min-w-0 flex-1 truncate text-left">Search pages or run commands</span>
            <kbd className="bg-background pointer-events-none hidden h-6 shrink-0 items-center gap-1 rounded-full border px-2 font-mono text-[10px] font-medium opacity-100 select-none sm:inline-flex">
              <span className="text-xs">⌘</span>K
            </kbd>
          </Button>
        </div>

        {active ? (
          <Link href={`/runs/view/?id=${active.id}`} className="flex items-center gap-2 text-sm xl:ml-auto">
            <StatusBadge status={active.status} />
            <span className="text-muted-foreground">{active.name}</span>
          </Link>
        ) : null}
      </div>
    </header>
  );
}
