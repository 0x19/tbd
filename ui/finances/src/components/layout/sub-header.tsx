"use client";

import { usePathname } from "next/navigation";

import { useFinance } from "@/app/providers";
import { crumbs } from "@/data/sidebar-data";
import { site } from "@/data/site";

import { ScopeToggle } from "../scope-toggle";

/** The kit's sub-header: root breadcrumb, section and tail; the party scope on the right. */
export function SubHeader() {
  const pathname = usePathname();
  const trail = crumbs(pathname);
  const { multi } = useFinance();

  return (
    <header className="bg-background hidden shrink-0 border-b px-4 py-4 sm:px-6 md:block">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:gap-6">
        <div className="flex min-w-0 flex-wrap items-center gap-3 text-sm">
          <span className="text-foreground font-medium">{site.title}</span>
          {trail.map((t, i) => (
            <span key={`${t}-${i}`} className="contents">
              <span className="text-muted-foreground">/</span>
              <span
                className={i === trail.length - 1 ? "text-muted-foreground" : "text-foreground font-medium"}
              >
                {t}
              </span>
            </span>
          ))}
        </div>
        {multi ? (
          <div className="xl:ml-auto">
            <ScopeToggle />
          </div>
        ) : null}
      </div>
    </header>
  );
}
