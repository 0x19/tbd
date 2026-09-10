"use client";

import Link from "next/link";
import { usePathname } from "next/navigation";

import { useChaos } from "@/app/providers";
import { crumbs } from "@/data/sidebar-data";
import { site } from "@/data/site";

import { StatusBadge } from "../status-badge";

/** The kit's sub-header: root breadcrumb, section and tail; the active run on the right. */
export function SubHeader() {
  const pathname = usePathname();
  const trail = crumbs(pathname);
  const { overview, queue } = useChaos();
  const active = overview?.active_run;

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

        {active || queue.length ? (
          <div className="flex items-center gap-3 text-sm xl:ml-auto">
            {active ? (
              <Link href={`/runs/view/?id=${active.id}`} className="flex items-center gap-2">
                <StatusBadge status={active.status} />
                <span className="text-muted-foreground">{active.name}</span>
              </Link>
            ) : null}
            {queue.length ? (
              <Link href="/runs/" className="text-muted-foreground hover:text-foreground text-xs">
                +{queue.length} queued
              </Link>
            ) : null}
          </div>
        ) : null}
      </div>
    </header>
  );
}
