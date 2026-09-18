"use client";

import { usePathname } from "next/navigation";

import { useFinance } from "@/app/providers";
import { crumbs } from "@/data/sidebar-data";
import { site } from "@/data/site";
import { useT } from "@/lib/i18n";

import { ScopeToggle } from "../scope-toggle";

/** The kit's sub-header: root breadcrumb, section and tail; the party scope on the right.
 *  The language toggle lives in the header, which every width shows; this bar does not. */
export function SubHeader() {
  const pathname = usePathname();
  const trail = crumbs(pathname);
  const { multi } = useFinance();
  const t = useT();

  return (
    <header className="bg-background hidden shrink-0 border-b px-4 py-4 sm:px-6 md:block">
      <div className="flex flex-col gap-4 xl:flex-row xl:items-center xl:gap-6">
        <div className="flex min-w-0 flex-wrap items-center gap-3 text-sm">
          <span className="text-foreground font-medium">{site.title}</span>
          {trail.map((crumb, i) => (
            <span key={`${crumb}-${i}`} className="contents">
              <span className="text-muted-foreground">/</span>
              <span
                className={i === trail.length - 1 ? "text-muted-foreground" : "text-foreground font-medium"}
              >
                {t(crumb)}
              </span>
            </span>
          ))}
        </div>
        {multi ? (
          <div className="flex items-center gap-2 xl:ml-auto">
            <ScopeToggle />
          </div>
        ) : null}
      </div>
    </header>
  );
}
