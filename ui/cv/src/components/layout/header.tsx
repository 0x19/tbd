"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { useMe } from "@/app/providers";
import { LangToggle } from "@/components/lang-toggle";
import { HeaderUser } from "@/components/layout/header-user";
import { ThemeSwitch } from "@/components/theme-switch";
import { Button } from "@/components/ui/button";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

export function Header() {
  const t = useT();
  const me = useMe();
  const pathname = usePathname();
  const owner = me.data?.role === "admin";
  const links = [
    { href: "/", label: t("nav.home") },
    ...(owner ? [{ href: "/admin/", label: t("nav.admin") }] : []),
  ];
  return (
    <header className="bg-background border-b">
      <div className="mx-auto flex w-full max-w-3xl items-center gap-2 px-4 py-3 sm:px-6">
        <Link href="/" className="mr-2 text-sm font-semibold tracking-tight">
          {t("nav.title")}
        </Link>
        <nav className="flex items-center gap-0.5">
          {links.map((l) => (
            <Button
              key={l.href}
              variant="ghost"
              size="sm"
              className={cn("px-2", pathname === l.href && "bg-muted")}
              asChild
            >
              <Link href={l.href}>{l.label}</Link>
            </Button>
          ))}
        </nav>
        <div className="ml-auto flex items-center gap-1.5">
          <LangToggle />
          <ThemeSwitch triggerClassName="size-9 scale-100 rounded-lg" />
          <HeaderUser />
        </div>
      </div>
    </header>
  );
}
