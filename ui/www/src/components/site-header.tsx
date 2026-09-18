"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";
import { useEffect, useState } from "react";

import { Frame } from "@/components/kit";
import { Logo } from "@/components/logo";
import { ThemeSwitch } from "@/components/theme-switch";
import { Button } from "@/components/ui/button";
import { company, nav } from "@/data/site";
import { cn } from "@/lib/utils";

/** Transparent over the hero, and a hairline under it once the page moves. */
export function SiteHeader() {
  const pathname = usePathname();
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
            .filter((item) => item.href !== "/")
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
                    {item.label}
                  </Link>
                </Button>
              );
            })}
        </nav>
        <span className="bg-border mx-1 hidden h-4 w-px shrink-0 sm:block" />
        {/* Outside the scrolling nav, so it is reachable on a phone without scrolling. */}
        <div className="shrink-0">
          <ThemeSwitch />
        </div>
      </Frame>
    </header>
  );
}
