import type { Metadata } from "next";

import { Eyebrow, Frame, Reveal, SectionHead, Tag } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { achievements, company, cv, earlier, experience, languages, projects } from "@/data/site";

export const metadata: Metadata = {
  title: "CV",
  description: `${company.person} — ${company.title}. The record, with a PDF to keep.`,
  alternates: { canonical: "/cv/" },
};

/**
 * The CV as a page: the same facts as the PDF (`public/cv/`, rendered by
 * `mise run www:cv` from the same data file), readable without a download.
 */
export default function CvPage() {
  // The about page compresses 2007–2014 into one "Earlier" entry; here the same
  // years are listed job by job (`earlier`), so that entry is left out.
  const jobs = experience.filter((job) => job.company !== "Earlier");
  return (
    <>
      <Frame className="pt-16 pb-10 sm:pt-24 sm:pb-14">
        <Eyebrow>CV</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.03em] text-balance sm:text-5xl">
          {company.person}
        </h1>
        <p className="mt-4 max-w-2xl text-xl text-pretty">{company.title}</p>
        <p className="text-muted-foreground mt-2 font-mono text-[11px] tracking-[0.14em] uppercase">
          {company.city}
        </p>
        <p className="mt-8 max-w-2xl border-l-2 pl-4 text-pretty">{company.availability}</p>
        <div className="mt-8 flex flex-wrap gap-2">
          <Button size="lg" asChild>
            <a href={cv.pdf} download>
              Download the PDF
            </a>
          </Button>
          <Button size="lg" variant="outline" asChild>
            <a href={cv.fullUrl}>Request the full CV</a>
          </Button>
          <Button size="lg" variant="outline" asChild>
            <a href={`mailto:${company.email}`}>{company.email}</a>
          </Button>
        </div>
        <p className="text-muted-foreground mt-4 max-w-2xl text-sm text-pretty">
          The full version, with a phone number, an address and references, is behind a sign-in: ask, I
          approve, and you get a PDF prepared for you.
        </p>
      </Frame>

      <Frame className="py-10 sm:py-14">
        <SectionHead n="01" label="Summary" />
        <Reveal className="mt-8">
          <p className="max-w-2xl text-lg text-pretty">{company.summary}</p>
        </Reveal>
      </Frame>

      <Frame className="py-10 sm:py-14">
        <SectionHead n="02" label="Selected work" />
        <Reveal className="mt-8">
          <ul className="max-w-2xl space-y-3">
            {achievements.map((a) => (
              <li key={a} className="border-l pl-4 text-pretty">
                {a}
              </li>
            ))}
          </ul>
        </Reveal>
      </Frame>

      <Frame className="py-10 sm:py-14">
        <SectionHead n="03" label="Experience" />
        <ol className="mt-8">
          {jobs.map((job, i) => (
            <Reveal key={job.company} delay={i * 40}>
              <li className="grid gap-3 border-t py-8 sm:grid-cols-[11rem_minmax(0,1fr)] sm:gap-10">
                <div>
                  <div className="text-muted-foreground/70 font-mono text-[11px] tracking-[0.14em] uppercase tabular-nums">
                    {job.when}
                  </div>
                  <div className="text-muted-foreground/60 mt-1.5 font-mono text-[11px]">{job.where}</div>
                </div>
                <div>
                  <h3 className="text-xl font-medium tracking-tight">
                    {job.href ? (
                      <a href={job.href} target="_blank" rel="noreferrer">
                        {job.company}
                      </a>
                    ) : (
                      job.company
                    )}
                  </h3>
                  <p className="text-muted-foreground mt-1 text-sm">{job.role}</p>
                  <p className="mt-4 max-w-2xl text-pretty">{job.body}</p>
                  {job.highlights.length ? (
                    <ul className="mt-4 max-w-2xl space-y-2">
                      {job.highlights.map((h) => (
                        <li key={h} className="text-muted-foreground border-l pl-4 text-sm text-pretty">
                          {h}
                        </li>
                      ))}
                    </ul>
                  ) : null}
                  {job.tags.length ? (
                    <div className="mt-4 flex flex-wrap gap-1.5">
                      {job.tags.map((tag) => (
                        <Tag key={tag}>{tag}</Tag>
                      ))}
                    </div>
                  ) : null}
                </div>
              </li>
            </Reveal>
          ))}
        </ol>
      </Frame>

      <Frame className="py-10 sm:py-14">
        <SectionHead
          n="04"
          label="Earlier"
          lead="Web work, mostly in PHP and JavaScript, before the systems work began."
        />
        <ol className="mt-8">
          {earlier.map((job) => (
            <li
              key={job.company}
              className="grid gap-2 border-t py-5 sm:grid-cols-[11rem_minmax(0,1fr)] sm:gap-10"
            >
              <div className="text-muted-foreground/70 font-mono text-[11px] tracking-[0.14em] uppercase tabular-nums">
                {job.when}
              </div>
              <div>
                <span className="font-medium">{job.company}</span>
                <span className="text-muted-foreground"> — {job.role}</span>
              </div>
            </li>
          ))}
        </ol>
      </Frame>

      <Frame className="py-10 sm:py-14">
        <SectionHead n="05" label="Open source" />
        <ol className="mt-8">
          {projects.map((p) => (
            <li
              key={p.name}
              className="grid gap-2 border-t py-5 sm:grid-cols-[11rem_minmax(0,1fr)] sm:gap-10"
            >
              <div>
                <a href={p.href} target="_blank" rel="noreferrer" className="font-medium">
                  {p.name}
                </a>
                <div className="text-muted-foreground/70 mt-1 font-mono text-[11px] tabular-nums">
                  {p.year}
                </div>
              </div>
              <p className="text-muted-foreground text-pretty">{p.what}</p>
            </li>
          ))}
        </ol>
      </Frame>

      <Frame className="py-10 pb-20 sm:py-14 sm:pb-28">
        <SectionHead n="06" label="Languages and education" />
        <p className="mt-8 max-w-2xl text-pretty">
          {languages.join(", ")}. {cv.education}
        </p>
      </Frame>
    </>
  );
}
