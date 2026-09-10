"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { Bell, Moon, Search, Sun } from "lucide-react";
import { useTheme } from "next-themes";
import { Badge } from "@/components/ui/badge";
import {
  Breadcrumb,
  BreadcrumbItem,
  BreadcrumbLink,
  BreadcrumbList,
  BreadcrumbPage,
  BreadcrumbSeparator,
} from "@/components/ui/breadcrumb";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { Kbd } from "@/components/ui/kbd";
import { Separator } from "@/components/ui/separator";
import { SidebarTrigger } from "@/components/ui/sidebar";
import { StatusBadge } from "@/components/status-badge";
import { ago } from "@/lib/format";
import { crumbs } from "./nav";
import { useChaos } from "./providers";

/**
 * Two bars, as in the kit: the app bar (sidebar trigger, app name, search,
 * notifications, theme) and a breadcrumb bar with the command search.
 */
export function SiteHeader() {
  const pathname = usePathname();
  const { overview, error, activity, setCommandOpen } = useChaos();
  const trail = crumbs(pathname);
  const active = overview?.active_run;
  const unread = activity.filter((a) => a.event.type === "run_finished").slice(0, 9);

  return (
    <div className="sticky top-0 z-20 bg-background">
      <header className="flex h-16 shrink-0 items-center gap-2 border-b px-4">
        <SidebarTrigger className="-ml-1" />
        <span className="text-base font-semibold">Chaos Admin</span>
        {active ? (
          <Link
            href={`/runs/view/?id=${active.id}`}
            className="ml-3 hidden items-center gap-2 text-xs md:flex"
          >
            <StatusBadge status={active.status} />
            <span className="text-muted-foreground">{active.name}</span>
          </Link>
        ) : null}
        <div className="ml-auto flex items-center gap-1.5">
          {error ? (
            <Badge variant="destructive" className="mr-2">
              {error}
            </Badge>
          ) : null}
          <Button
            variant="outline"
            size="sm"
            className="gap-2 text-muted-foreground"
            onClick={() => setCommandOpen(true)}
          >
            <Search className="size-4" />
            <Kbd>⌘K</Kbd>
          </Button>
          <DropdownMenu>
            <DropdownMenuTrigger
              render={<Button variant="ghost" size="icon" aria-label="Notifications" className="relative" />}
            >
              <Bell className="size-4" />
              {unread.length ? (
                <span className="absolute -top-0.5 -right-0.5 flex size-4 items-center justify-center rounded-full bg-destructive text-[10px] font-medium text-white">
                  {unread.length}
                </span>
              ) : null}
            </DropdownMenuTrigger>
            <DropdownMenuContent align="end" className="w-80">
              <DropdownMenuLabel>Recent runs</DropdownMenuLabel>
              <DropdownMenuSeparator />
              {unread.length ? (
                unread.map((a) =>
                  a.event.type === "run_finished" ? (
                    <DropdownMenuItem key={a.at} render={<Link href={`/runs/view/?id=${a.event.run.id}`} />}>
                      <StatusBadge status={a.event.run.status} />
                      <span className="truncate">{a.event.run.name}</span>
                      <span className="ml-auto text-xs text-muted-foreground">
                        {ago(new Date(a.at).toISOString())}
                      </span>
                    </DropdownMenuItem>
                  ) : null,
                )
              ) : (
                <div className="px-2 py-4 text-center text-xs text-muted-foreground">
                  Nothing finished in this session yet.
                </div>
              )}
            </DropdownMenuContent>
          </DropdownMenu>
          <ThemeToggle />
          <Badge variant="outline" className="hidden gap-1.5 md:inline-flex">
            <span className="size-2 rounded-full bg-primary" />
            {overview?.env ?? "…"}
          </Badge>
        </div>
      </header>
      <div className="flex h-12 items-center gap-4 border-b px-4">
        <Breadcrumb>
          <BreadcrumbList>
            <BreadcrumbItem>
              <BreadcrumbLink render={<Link href="/" />}>Chaos Admin</BreadcrumbLink>
            </BreadcrumbItem>
            {trail.map((t, i) => (
              <span key={`${t}-${i}`} className="contents">
                <BreadcrumbSeparator />
                <BreadcrumbItem>
                  {i === trail.length - 1 ? (
                    <BreadcrumbPage>{t}</BreadcrumbPage>
                  ) : (
                    <span className="text-muted-foreground">{t}</span>
                  )}
                </BreadcrumbItem>
              </span>
            ))}
          </BreadcrumbList>
        </Breadcrumb>
        <Separator orientation="vertical" className="hidden data-[orientation=vertical]:h-4 md:block" />
        <button
          type="button"
          onClick={() => setCommandOpen(true)}
          className="hidden h-8 max-w-md flex-1 items-center gap-2 rounded-lg bg-muted px-3 text-sm text-muted-foreground md:flex"
        >
          <Search className="size-4" />
          <span className="flex-1 text-left">Search pages or run commands</span>
          <Kbd>⌘K</Kbd>
        </button>
      </div>
    </div>
  );
}

function ThemeToggle() {
  const { resolvedTheme, setTheme } = useTheme();
  return (
    <Button
      variant="ghost"
      size="icon"
      aria-label="Toggle theme"
      onClick={() => setTheme(resolvedTheme === "dark" ? "light" : "dark")}
    >
      <Sun className="size-4 dark:hidden" />
      <Moon className="hidden size-4 dark:block" />
    </Button>
  );
}
