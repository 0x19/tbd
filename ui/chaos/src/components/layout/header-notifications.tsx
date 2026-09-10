"use client";

import { Bell } from "lucide-react";
import Link from "next/link";
import * as React from "react";

import { useChaos } from "@/app/providers";
import { StatusBadge } from "@/components/status-badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { ScrollArea } from "@/components/ui/scroll-area";
import { ago } from "@/lib/format";
import { cn } from "@/lib/utils";

/** The kit's notification bell, fed by the live feed: runs that finished in this session. */
export function HeaderNotifications() {
  const { activity } = useChaos();
  const [seen, setSeen] = React.useState<Set<number>>(() => new Set());
  const finished = activity.filter((a) => a.event.type === "run_finished").slice(0, 12);
  const unreadCount = finished.filter((a) => !seen.has(a.at)).length;

  const markAllRead = () => setSeen(new Set(finished.map((a) => a.at)));

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          id="header-notifications-trigger"
          variant="outline"
          size="icon"
          className="relative size-9"
          aria-label={unreadCount ? `Notifications, ${unreadCount} unread` : "Notifications"}
        >
          <Bell className="size-4" aria-hidden="true" />
          {unreadCount > 0 ? (
            <span className="bg-destructive text-destructive-foreground absolute -top-0.5 -right-0.5 flex h-4 min-w-4 items-center justify-center rounded-full px-1 text-[10px] font-medium tabular-nums">
              {unreadCount > 9 ? "9+" : unreadCount}
            </span>
          ) : null}
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent className="w-80 p-0" align="end">
        <DropdownMenuLabel className="px-3 py-2 text-sm font-normal">
          <div className="flex items-center justify-between gap-2">
            <span className="font-semibold">Finished runs</span>
            {unreadCount > 0 ? (
              <button
                type="button"
                className="text-muted-foreground hover:text-foreground text-xs font-medium underline-offset-4 hover:underline"
                onClick={(e) => {
                  e.preventDefault();
                  markAllRead();
                }}
              >
                Mark all as read
              </button>
            ) : null}
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <ScrollArea className="h-[min(320px,50vh)]">
          <div className="flex flex-col py-1">
            {finished.length ? (
              finished.map((a) =>
                a.event.type === "run_finished" ? (
                  <DropdownMenuItem
                    key={a.at}
                    asChild
                    className={cn(
                      "focus:bg-accent cursor-pointer items-start rounded-none px-3 py-2.5",
                      !seen.has(a.at) && "bg-accent/40",
                    )}
                    onSelect={() => setSeen((s) => new Set(s).add(a.at))}
                  >
                    <Link href={`/runs/view/?id=${a.event.run.id}`}>
                      <div className="min-w-0 flex-1 space-y-1">
                        <div className="flex items-start justify-between gap-2">
                          <p
                            className={cn(
                              "truncate text-sm leading-tight",
                              !seen.has(a.at) && "font-semibold",
                            )}
                          >
                            {a.event.run.name}
                          </p>
                          <span className="text-muted-foreground shrink-0 text-[10px] whitespace-nowrap">
                            {ago(new Date(a.at).toISOString())}
                          </span>
                        </div>
                        <div className="flex items-center gap-2">
                          <StatusBadge status={a.event.run.status} />
                          <span className="text-muted-foreground text-xs capitalize">{a.event.run.kind}</span>
                        </div>
                      </div>
                    </Link>
                  </DropdownMenuItem>
                ) : null,
              )
            ) : (
              <p className="text-muted-foreground px-3 py-6 text-center text-xs">
                Nothing finished in this session yet.
              </p>
            )}
          </div>
        </ScrollArea>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
