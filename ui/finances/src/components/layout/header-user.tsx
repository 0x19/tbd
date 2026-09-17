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

/** Who is signed in, from the principal Envoy verified (`GET /v1/me`), and how
 *  to sign out. Sign-out is Envoy's OAuth2 filter path on this host. */
export function HeaderUser() {
  const me = useFetch(() => api.me(), 5 * 60_000);
  const user = me.data;
  if (!user) return null;
  const label = user.subject.length > 16 ? `${user.subject.slice(0, 8)}…` : user.subject;

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button variant="outline" size="icon" className="size-9 rounded-full p-0" aria-label="Signed in">
          <Avatar className="size-7">
            <AvatarFallback className="text-[11px] font-semibold">NV</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel className="font-normal">
          <div className="grid gap-1">
            <div className="truncate font-mono text-xs">{label}</div>
            <div className="text-muted-foreground truncate text-xs">{user.kind}</div>
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
          <a href="/oauth2/signout">
            <LogOut /> Sign out
          </a>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}
