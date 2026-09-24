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
import { useT } from "@/lib/i18n";

/** Who is signed in, from the principal Envoy verified (`GET /v1/me`), and how
 *  to sign out. Sign-out is Envoy's OAuth2 filter path on this host. */
export function HeaderUser() {
  const t = useT();
  const me = useFetch(() => api.me(), 5 * 60_000);
  const user = me.data;
  if (!user) return null;
  const label =
    user.name || user.email || (user.subject.length > 16 ? `${user.subject.slice(0, 8)}…` : user.subject);
  const initials = initialsOf(user.name ?? "", user.email ?? "");

  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="icon"
          className="size-9 rounded-full p-0"
          aria-label={t("nav.signed_in")}
        >
          <Avatar className="size-7">
            <AvatarFallback className="text-[11px] font-semibold">{initials}</AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel className="font-normal">
          <div className="grid gap-1">
            <div className="truncate text-xs font-medium">{label}</div>
            <div className="text-muted-foreground truncate text-xs">{user.email ?? user.kind}</div>
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
            <LogOut /> {t("nav.sign_out")}
          </a>
        </DropdownMenuItem>
      </DropdownMenuContent>
    </DropdownMenu>
  );
}

/** Two letters from the name, else one from the e-mail, else a dot. */
function initialsOf(name: string, email: string): string {
  const parts = name.split(/\s+/).filter(Boolean);
  if (parts.length >= 2) return (parts[0][0] + parts[parts.length - 1][0]).toUpperCase();
  if (parts.length === 1) return parts[0].slice(0, 2).toUpperCase();
  if (email) return email[0].toUpperCase();
  return "·";
}
