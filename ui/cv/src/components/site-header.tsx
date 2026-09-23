"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

import { useMe } from "@/app/providers";
import { Frame } from "@/components/kit";
import { LangToggle } from "@/components/lang-toggle";
import { HeaderUser } from "@/components/layout/header-user";
import { Logo } from "@/components/logo";
import { ThemeSwitch } from "@/components/theme-switch";
import { Button } from "@/components/ui/button";
import { company, publicNav, publicUrl } from "@/data/site";
import { useT } from "@/lib/i18n";
import { cn } from "@/lib/utils";

/**
 * The public site's header, with this app's two pages after a divider: the
 * mark goes home to the public site, the public pages are absolute links
 * back to it, and the person signed in is on the right. The same shape as
 * `ui/www/src/components/site-header.tsx`, on purpose.
 */
export function SiteHeader() {
  const t = useT();
  const me = useMe();
  const pathname = usePathname();
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  const owner = me.data?.role === "admin";
  const local = [
    { href: "/", label: t("nav.home") },
    ...(owner ? [{ href: "/admin/", label: t("nav.admin") }] : []),
  ];
  const item =
    "font-mono text-[10px] tracking-[0.12em] uppercase sm:text-[11px] sm:tracking-[0.14em] shrink-0 px-2 sm:px-3";

  return (
    <header
      className={cn(
        "sticky top-0 z-40 transition-colors duration-300",
        scrolled ? "bg-background/80 border-b backdrop-blur-md" : "border-b border-transparent",
      )}
    >
      <Frame className="flex h-16 items-center gap-2 sm:h-20 sm:gap-4">
        <a
          href={`${publicUrl}/`}
          className="hover:text-muted-foreground shrink-0 transition-colors"
          aria-label={`${company.name}, home`}
        >
          <Logo size={24} />
        </a>
        <nav className="ml-auto flex min-w-0 [scrollbar-width:none] items-center gap-0.5 overflow-x-auto [&::-webkit-scrollbar]:hidden">
          {publicNav.map((p) => (
            <Button
              key={p.href}
              variant="ghost"
              size="sm"
              className={cn(item, "text-muted-foreground")}
              asChild
            >
              <a href={p.href}>{t(`nav.${p.key}`)}</a>
            </Button>
          ))}
          <span className="bg-border mx-1 h-4 w-px shrink-0" />
          {local.map((l) => {
            const active = pathname === l.href;
            return (
              <Button key={l.href} variant="ghost" size="sm" className={item} asChild>
                <Link
                  href={l.href}
                  className={active ? "text-foreground" : "text-muted-foreground"}
                  aria-current={active ? "page" : undefined}
                >
                  {l.label}
                </Link>
              </Button>
            );
          })}
        </nav>
        <span className="bg-border mx-1 hidden h-4 w-px shrink-0 sm:block" />
        <div className="flex shrink-0 items-center gap-1.5">
          <LangToggle />
          <ThemeSwitch />
          <HeaderUser />
        </div>
      </Frame>
    </header>
  );
}
