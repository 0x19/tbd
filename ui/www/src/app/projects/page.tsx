import type { Metadata } from "next";

import { Frame, IndexRow, Reveal, SectionHead } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { company, projects } from "@/data/site";

export const metadata: Metadata = {
  title: "Projects",
  description: "Libraries and tools I have written and left in the open.",
  alternates: { canonical: "/projects/" },
};

export default function ProjectsPage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <h1 className="max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          Things I have built and left in the open.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">
          Mostly Go, mostly infrastructure, mostly written because I needed them and there was nothing there.
          A few have outlived the reason they were written.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <SectionHead n="01" label="Open source" />
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
              The rest is on GitHub ↗
            </a>
          </Button>
        </Reveal>
      </Frame>
    </>
  );
}
