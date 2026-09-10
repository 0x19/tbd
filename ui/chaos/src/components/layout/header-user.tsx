"use client";

import { LogOut, ShieldCheck } from "lucide-react";

import { Avatar, AvatarFallback } from "@/components/ui/avatar";
import { Badge } from "@/components/ui/badge";
import { Button } from "@/components/ui/button";
import {
  DropdownMenu,
  DropdownMenuContent,
  DropdownMenuItem,
  DropdownMenuLabel,
  DropdownMenuSeparator,
  DropdownMenuTrigger,
} from "@/components/ui/dropdown-menu";
import { api } from "@/lib/api/client";
import { useFetch } from "@/lib/api/hooks";

function initials(name: string, email: string): string {
  const src = name.trim() || email.split("@")[0] || "?";
  const parts = src.split(/[\s._-]+/).filter(Boolean);
  return (parts.length > 1 ? `${parts[0]![0]}${parts[1]![0]}` : src.slice(0, 2)).toUpperCase();
}

/** Who is signed in, from Envoy's verified identity headers, and how to sign out.
 *  Renders nothing on the open local host, where there is no login. */
export function HeaderUser() {
  const me = useFetch(() => api.me(), 5 * 60_000);
  const user = me.data?.user;
  if (!user) return null;
  const signout = me.data!.signout;
  const signoutAll = me.data!.signout_all;
  const label = user.name || user.email;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="icon"
          className="size-9 rounded-full p-0"
          aria-label={`Signed in as ${label}`}
        >
          <Avatar className="size-7">
            <AvatarFallback className="text-[11px] font-semibold">
              {initials(user.name, user.email)}
            </AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel className="font-normal">
          <div className="grid gap-1">
            <div className="truncate text-sm font-semibold">{label}</div>
            {user.name && user.email ? (
              <div className="text-muted-foreground truncate text-xs">{user.email}</div>
            ) : null}
            {user.role ? (
              <div className="pt-1">
                <Badge variant="outline" className="gap-1 text-[10px] capitalize">
                  <ShieldCheck className="size-3" /> {user.role}
                </Badge>
              </div>
            ) : null}
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <a href={signout}>
            <LogOut /> Sign out of Chaos Admin
          </a>
        </DropdownMenuItem>
        {signoutAll ? (
          <DropdownMenuItem asChild>
            <a href={signoutAll}>
              <LogOut /> Sign out everywhere
            </a>
          </DropdownMenuItem>
        ) : null}
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
