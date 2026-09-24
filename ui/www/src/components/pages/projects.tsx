"use client";

import { useId, useState } from "react";

import { Eyebrow, Frame, Reveal } from "@/components/kit";
import { TabList } from "@/components/tabs";
import { Button } from "@/components/ui/button";
import { company } from "@/data/site";
import { useT } from "@/lib/i18n";
import { type Site, useSite } from "@/lib/i18n/site";
import { cn } from "@/lib/utils";

type Project = Site["projects"][number];
type Domain = "all" | Project["domain"];

/**
 * The work page in the reader's language (`app/work/page.tsx` carries the
 * metadata): the open-source libraries, newest first, as tabs by domain over
 * a grid of cards, the same language as the playgrounds. A card is the
 * number, the domain, the year, the name, what it does, and its language
 * with a link to the source. No star or fork counts: they are about me, not
 * about the reader, and they go stale in a static build.
 */
export function ProjectsContent() {
  const t = useT();
  const id = useId();
  const { projects } = useSite();
  const [tab, setTab] = useState<Domain>("all");

  const sorted = [...projects].sort((a, b) => Number(b.year) - Number(a.year));
  const tabs: { key: Domain; label: string; n: number }[] = [
    { key: "all", label: t("projects.domain.all"), n: sorted.length },
    ...(["blockchain", "data", "tools"] as const)
      .map((key) => ({
        key,
        label: t(`projects.domain.${key}`),
        n: sorted.filter((p) => p.domain === key).length,
      }))
      .filter((x) => x.n > 0),
  ];
  const shown = sorted.filter((p) => tab === "all" || p.domain === tab);

  return (
    <>
      <Frame className="pt-20 pb-12 sm:pt-28">
        <Eyebrow>{t("projects.eyebrow")}</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("projects.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("projects.lead")}</p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Reveal>
          <TabList id={id} label={t("projects.domains")} tabs={tabs} value={tab} onChange={setTab} />
          <div
            role="tabpanel"
            id={`${id}-panel`}
            aria-labelledby={`${id}-tab-${tab}`}
            className="bg-border/70 grid gap-px border-b sm:grid-cols-2 lg:grid-cols-3"
          >
            {shown.map((p) => (
              <Card key={p.name} p={p} n={sorted.indexOf(p) + 1} />
            ))}
            {/* Blank cells that close the last row, so the hairline grid never
                shows its gap colour: two columns from sm, three from lg. */}
            {Array.from({ length: 2 }, (_, i) => (
              <div
                key={`fill-${i}`}
                aria-hidden
                className={cn(
                  "bg-background hidden",
                  i < (2 - (shown.length % 2)) % 2 && "sm:block",
                  i < (3 - (shown.length % 3)) % 3 ? "lg:block" : "lg:hidden",
                )}
              />
            ))}
          </div>
          <Button variant="outline" className="mt-8" asChild>
            <a href={company.github} target="_blank" rel="noreferrer">
              {t("projects.rest")}
            </a>
          </Button>
        </Reveal>
      </Frame>
    </>
  );
}

function Card({ p, n }: { p: Project; n: number }) {
  const t = useT();
  return (
    <a
      href={p.href}
      target="_blank"
      rel="noreferrer"
      className={cn("group bg-background hover:bg-muted/30 flex flex-col p-6 transition-colors sm:p-7")}
    >
      <div className="flex items-center gap-3 font-mono text-[11px] tracking-[0.18em] uppercase">
        <span className="text-muted-foreground/50 tabular-nums">{String(n).padStart(2, "0")}</span>
        <span className="text-muted-foreground">{t(`projects.domain.${p.domain}`)}</span>
        <span className="text-muted-foreground/70 ml-auto tabular-nums">{p.year}</span>
      </div>
      <h2 className="mt-7 font-mono text-lg font-medium tracking-tight">{p.name}</h2>
      <p className="text-muted-foreground mt-2 text-sm text-pretty">{p.what}</p>
      <div className="mt-auto flex items-end gap-4 pt-7">
        <p className="text-muted-foreground/70 flex-1 border-t pt-4 font-mono text-[11px]">
          {p.language}
          {"  ·  "}
          {t("projects.oss")}
        </p>
        <span className="text-muted-foreground group-hover:text-foreground flex items-center gap-1 font-mono text-[11px] tracking-[0.12em] uppercase transition-colors">
          {t("projects.source")}
          <span
            aria-hidden
            className="transition-transform group-hover:translate-x-0.5 group-hover:-translate-y-0.5"
          >
            ↗
          </span>
        </span>
      </div>
    </a>
  );
}
