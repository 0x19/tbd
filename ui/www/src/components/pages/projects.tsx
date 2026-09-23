"use client";

import { Frame, IndexRow, Reveal, SectionHead } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { company } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";

/** The projects page in the reader's language; `app/projects/page.tsx` carries the metadata. */
export function ProjectsContent() {
  const t = useT();
  const { projects } = useSite();
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <h1 className="max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("projects.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("projects.lead")}</p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <SectionHead n="01" label={t("projects.oss")} />
        <Reveal className="mt-10">
          <div>
            {projects.map((p, i) => (
              <IndexRow
                key={p.name}
                n={String(i + 1).padStart(2, "0")}
                name={p.name}
                meta={p.language}
                year={p.year}
                href={p.href}
              >
                {p.what}
              </IndexRow>
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
