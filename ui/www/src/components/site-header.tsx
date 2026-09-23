"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

import { Frame } from "@/components/kit";
import { LangToggle } from "@/components/lang-toggle";
import { Logo } from "@/components/logo";
import { ThemeSwitch } from "@/components/theme-switch";
import { Button } from "@/components/ui/button";
import { company, cv, nav, navVisible } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useIsAdmin } from "@/lib/me";
import { cn } from "@/lib/utils";

/** Transparent over the hero, and a hairline under it once the page moves. */
export function SiteHeader() {
  const pathname = usePathname();
  const t = useT();
  const admin = useIsAdmin();
  const [scrolled, setScrolled] = useState(false);

  useEffect(() => {
    const onScroll = () => setScrolled(window.scrollY > 8);
    onScroll();
    window.addEventListener("scroll", onScroll, { passive: true });
    return () => window.removeEventListener("scroll", onScroll);
  }, []);

  return (
    <header
      className={cn(
        "sticky top-0 z-40 transition-colors duration-300",
        scrolled ? "bg-background/80 border-b backdrop-blur-md" : "border-b border-transparent",
      )}
    >
      <Frame className="flex h-16 items-center gap-2 sm:h-20 sm:gap-4">
        {/* The mark alone: the name is in the title, the tab and the footer, and
            the monogram is the name anyway. */}
        <Link
          href="/"
          className="hover:text-muted-foreground shrink-0 transition-colors"
          aria-label={`${company.name}, home`}
        >
          <Logo size={24} />
        </Link>
        {/* Five items and a theme control do not fit a phone; let the row scroll
            rather than overlap the name or hide behind a menu nobody opens. */}
        <nav className="ml-auto flex min-w-0 [scrollbar-width:none] items-center gap-0.5 overflow-x-auto [&::-webkit-scrollbar]:hidden">
          {nav
            .filter((item) => item.href !== "/" && navVisible(item, admin))
            .map((item) => {
              const active = pathname.startsWith(item.href);
              return (
                <Button key={item.href} variant="ghost" size="sm" className="shrink-0 px-2 sm:px-3" asChild>
                  <Link
                    href={item.href}
                    className={cn(
                      "font-mono text-[10px] tracking-[0.12em] uppercase sm:text-[11px] sm:tracking-[0.14em]",
                      active ? "text-foreground" : "text-muted-foreground",
                    )}
                    aria-current={active ? "page" : undefined}
                  >
                    {t(`common.${item.key}`)}
                  </Link>
                </Button>
              );
            })}
          {/* The gated room: the same site behind a sign-in, where the full CV is.
              A plain anchor, since it is another host; whoever is signed in
              there lands on their standing, everyone else on the sign-in. */}
          <Button variant="ghost" size="sm" className="shrink-0 px-2 sm:px-3" asChild>
            <a
              href={cv.fullUrl}
              className="text-muted-foreground font-mono text-[10px] tracking-[0.12em] uppercase sm:text-[11px] sm:tracking-[0.14em]"
            >
              {t("common.full_cv")} ↗
            </a>
          </Button>
        </nav>
        <span className="bg-border mx-1 hidden h-4 w-px shrink-0 sm:block" />
        {/* Outside the scrolling nav, so they are reachable on a phone without scrolling. */}
        <div className="flex shrink-0 items-center gap-1">
          <LangToggle />
          <ThemeSwitch />
        </div>
      </Frame>
    </header>
  );
}
