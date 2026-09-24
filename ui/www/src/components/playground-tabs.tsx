"use client";

import { useEffect, useId, useState } from "react";

import { TabList } from "@/components/tabs";
import { useT } from "@/lib/i18n";
import type { Site } from "@/lib/i18n/site";
import { cn } from "@/lib/utils";

type Playground = Site["playgrounds"][number];
type Tab = "all" | Playground["category"];

/**
 * The playgrounds as a tabbed index, shared by the home page and
 * `/playgrounds/`: a tab per category that has something in it, each with its
 * count, and a grid of cards under them. A card carries its number, its tag,
 * whether it is live, the name, one line of what it does and two or three
 * facts in its foot. On `/playgrounds/` the first systems piece is featured
 * across the full width with its longer description, and the chosen tab
 * lives in the URL's hash so a link can open on it. The tab bar is `TabList`.
 */
export function PlaygroundTabs({
  items,
  limit,
  feature = false,
  hash = false,
  columns = "lg:grid-cols-4",
}: {
  items: Playground[];
  limit?: number;
  feature?: boolean;
  hash?: boolean;
  columns?: string;
}) {
  const t = useT();
  const id = useId();
  const [tab, setTab] = useState<Tab>("all");
  const tabs: { key: Tab; label: string; n: number }[] = [
    { key: "all", label: t("playgrounds.cat.all"), n: items.length },
    ...(["systems", "music"] as const)
      .map((key) => ({
        key,
        label: t(`playgrounds.cat.${key}`),
        n: items.filter((p) => p.category === key).length,
      }))
      .filter((x) => x.n > 0),
  ];

  // The hash opens a tab on first load, and follows the choice after it.
  useEffect(() => {
    if (!hash) return;
    const h = window.location.hash.slice(1);
    if (h === "systems" || h === "music") setTab(h);
  }, [hash]);
  const choose = (key: Tab) => {
    setTab(key);
    if (hash) window.history.replaceState(null, "", key === "all" ? window.location.pathname : `#${key}`);
  };

  const shown = items.filter((p) => tab === "all" || p.category === tab);
  const featured = feature ? shown.find((p) => p.category === "systems" && !p.paused) : undefined;
  const rest = (featured ? shown.filter((p) => p !== featured) : shown).slice(0, limit);

  return (
    <div>
      <TabList id={id} label={t("playgrounds.cat.label")} tabs={tabs} value={tab} onChange={choose} />

      <div
        role="tabpanel"
        id={`${id}-panel`}
        aria-labelledby={`${id}-tab-${tab}`}
        className={cn("bg-border/70 grid gap-px border-b sm:grid-cols-2", columns)}
      >
        {featured ? <Card p={featured} n={items.indexOf(featured) + 1} big /> : null}
        {rest.map((p) => (
          <Card key={p.name} p={p} n={items.indexOf(p) + 1} />
        ))}
      </div>
    </div>
  );
}

function Card({ p, n, big }: { p: Playground; n: number; big?: boolean }) {
  const t = useT();
  const body = (
    <>
      <div className="flex items-center gap-3 font-mono text-[11px] tracking-[0.18em] uppercase">
        <span className="text-muted-foreground/50 tabular-nums">{String(n).padStart(2, "0")}</span>
        <span className="text-muted-foreground">
          {big ? `${t("playgrounds.featured")} · ${p.tag}` : p.tag}
        </span>
        <span className="text-muted-foreground ml-auto flex items-center gap-1.5">
          {p.paused ? (
            t("playgrounds.paused")
          ) : p.href ? (
            <>
              <span aria-hidden className="relative flex size-1.5">
                <span className="live-ping bg-foreground/40 absolute inline-flex size-full rounded-full" />
                <span className="bg-foreground/70 relative inline-flex size-1.5 rounded-full" />
              </span>
              {t("playgrounds.live")}
            </>
          ) : (
            t("playgrounds.building")
          )}
        </span>
      </div>
      <h3 className={cn("font-medium tracking-tight", big ? "mt-8 text-2xl sm:text-3xl" : "mt-7 text-lg")}>
        {p.name}
      </h3>
      <p className={cn("text-muted-foreground mt-2 text-pretty", big ? "max-w-2xl text-base" : "text-sm")}>
        {p.summary}
      </p>
      {p.paused ? (
        <p className="text-muted-foreground/80 mt-2 text-sm italic">{t("playgrounds.paused_note")}</p>
      ) : null}
      {big ? <p className="text-muted-foreground/80 mt-3 max-w-2xl text-sm text-pretty">{p.what}</p> : null}
      <div className="mt-auto flex items-end gap-4 pt-7">
        <p className="text-muted-foreground/70 flex-1 border-t pt-4 font-mono text-[11px] text-pretty">
          {p.specs.join("  ·  ")}
        </p>
        {p.href && !p.paused ? (
          <span aria-hidden className="text-muted-foreground transition-transform group-hover:translate-x-1">
            →
          </span>
        ) : null}
      </div>
    </>
  );
  const cls = cn("bg-background flex flex-col", big ? "p-6 sm:col-span-full sm:p-10" : "p-6 sm:p-7");
  // A paused card is listed but links nowhere, dimmed, so it never reads as playable.
  if (p.paused) return <div className={cn(cls, "opacity-70")}>{body}</div>;
  return p.href ? (
    <a href={p.href} className={cn("group hover:bg-muted/30 transition-colors", cls)}>
      {body}
    </a>
  ) : (
    <div className={cls}>{body}</div>
  );
}
