"use client";

import { FlaskConical, LogOut, ShieldCheck } from "lucide-react";

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
import { lab } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useMe } from "@/lib/me";

/**
 * The right end of the header, the same widget the CV host has: a "Sign in"
 * button for whoever is not signed in on this host (a plain anchor to the gated
 * `/account/`, so the gateway can send the browser to the sign-in), and for
 * whoever is, their initials with a menu: who, the role, the lab for an admin,
 * and the sign-out that ends the session everywhere.
 */
export function HeaderUser() {
  const t = useT();
  const me = useMe();
  if (!me) {
    return (
      <Button variant="outline" size="sm" className="h-8 shrink-0 px-3" asChild>
        <a href="/account/" className="font-mono text-[10px] tracking-[0.12em] uppercase sm:text-[11px]">
          {t("common.sign_in")}
        </a>
      </Button>
    );
  }
  const label = me.name || me.email || me.subject.slice(0, 8);
  const admin = me.role === "admin";
  return (
    <DropdownMenu>
      <DropdownMenuTrigger asChild>
        <Button
          variant="outline"
          size="icon"
          className="size-8 shrink-0 rounded-full p-0"
          aria-label={t("common.account")}
        >
          <Avatar className="size-7">
            <AvatarFallback className="text-[11px] font-semibold">
              {initialsOf(me.name ?? "", me.email ?? "")}
            </AvatarFallback>
          </Avatar>
        </Button>
      </DropdownMenuTrigger>
      <DropdownMenuContent align="end" className="w-64">
        <DropdownMenuLabel className="font-normal">
          <div className="grid gap-1">
            <div className="truncate text-xs font-medium">{label}</div>
            {me.email && me.name ? (
              <div className="text-muted-foreground truncate text-xs">{me.email}</div>
            ) : null}
            {me.role ? (
              <div className="pt-1">
                <Badge variant="outline" className="gap-1 text-[10px] capitalize">
                  <ShieldCheck className="size-3" /> {me.role}
                </Badge>
              </div>
            ) : null}
          </div>
        </DropdownMenuLabel>
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <a href="/account/">{t("common.account")}</a>
        </DropdownMenuItem>
        {admin && !lab.public ? (
          <DropdownMenuItem asChild>
            <a href={lab.href}>
              <FlaskConical /> {t("common.lab")}
            </a>
          </DropdownMenuItem>
        ) : null}
        <DropdownMenuSeparator />
        <DropdownMenuItem asChild>
          <a href="/oauth2/signout">
            <LogOut /> {t("common.sign_out")}
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
