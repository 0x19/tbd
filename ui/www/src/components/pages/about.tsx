"use client";

import { IconBrandGithub, IconBrandLinkedin, IconBrandX } from "@tabler/icons-react";

import { Eyebrow, Frame, Reveal, SectionHead, Tag } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { company as facts, cv, previously } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";

/**
 * The person and the record on one page: who, the story, the selected work,
 * every position with its lines, the earlier years, the open-source work.
 * The PDF (`public/cv/`, `mise run www:cv`) is the same facts; the full
 * version, with contact details and references, is behind `cv.fullUrl`.
 * `app/about/page.tsx` carries the metadata.
 */
export function AboutContent() {
  const t = useT();
  const { about, achievements, company, earlier, education, experience, languages, projects } = useSite();
  const elsewhere = [
    { icon: IconBrandGithub, label: "GitHub", value: "0x19", href: facts.github },
    { icon: IconBrandX, label: "X", value: "@vesicnevio", href: facts.x },
    { icon: IconBrandLinkedin, label: "LinkedIn", value: "neviovesic", href: facts.linkedin },
  ];

  const rows = [
    { k: t("about.facts.where"), v: t("about.facts.remote", { city: company.city }) },
    { k: t("about.facts.now"), v: company.now },
    { k: t("about.facts.before"), v: previously.join(", ") },
    { k: t("about.facts.languages"), v: languages.join(", ") },
    { k: t("about.facts.email"), v: company.email, email: true },
  ];

  // The about page compresses 2007–2014 into one "Earlier" entry in the
  // timeline; the same years are listed job by job below, so that entry is
  // left out of the positions.
  const jobs = experience.filter((job) => job.company !== "Earlier");

  return (
    <>
      {/* ------------------------------------------------------------ who */}
      <Frame className="pt-20 pb-12 sm:pt-28">
        <Eyebrow>{t("about.eyebrow")}</Eyebrow>
        <h1 className="mt-6 text-5xl font-semibold tracking-[-0.04em] text-balance sm:text-7xl">
          {company.person}
        </h1>
        <p className="mt-6 max-w-2xl text-xl text-pretty">{company.title}</p>
        <p className="text-muted-foreground mt-1 text-sm">{company.focus}</p>
        <p className="text-muted-foreground mt-2 font-mono text-[11px] tracking-[0.14em] uppercase">
          {company.city}
        </p>
        <p className="mt-8 max-w-2xl border-l-2 pl-4 text-pretty">{company.availability}</p>
        <div className="mt-8 flex flex-wrap gap-2">
          <Button size="lg" asChild>
            <a href={cv.pdf} download>
              {t("about.download")}
            </a>
          </Button>
          <Button size="lg" variant="outline" asChild>
            <a href={cv.fullUrl}>{t("about.request")}</a>
          </Button>
          <Button size="lg" variant="outline" asChild>
            <a href={`mailto:${company.email}`}>{company.email}</a>
          </Button>
        </div>
        <p className="text-muted-foreground mt-4 max-w-2xl text-sm text-pretty">{t("about.full_note")}</p>
      </Frame>

      {/* ---------------------------------------------------------- story */}
      <Frame className="pb-14 sm:pb-20">
        <div className="grid gap-10 border-t pt-10 sm:grid-cols-[minmax(0,1.5fr)_minmax(0,1fr)] sm:gap-16">
          <div className="space-y-5">
            {about.map((paragraph) => (
              <p key={paragraph.slice(0, 24)} className="text-lg leading-relaxed text-pretty">
                {paragraph}
              </p>
            ))}
          </div>

          <div className="self-start">
            <dl className="divide-y border-y">
              {rows.map((f) => (
                <div key={f.k} className="flex items-baseline justify-between gap-6 py-3">
                  <dt className="text-muted-foreground/70 shrink-0 font-mono text-[11px] tracking-[0.18em] uppercase">
                    {f.k}
                  </dt>
                  <dd className="text-right text-sm text-pretty">
                    {f.email ? (
                      <a className="underline-offset-4 hover:underline" href={`mailto:${company.email}`}>
                        {f.v}
                      </a>
                    ) : (
                      f.v
                    )}
                  </dd>
                </div>
              ))}
            </dl>
            <ul className="mt-6 space-y-1">
              {elsewhere.map((e) => (
                <li key={e.label}>
                  <a
                    href={e.href}
                    target="_blank"
                    rel="noreferrer"
                    className="group flex items-center gap-3 py-1.5 text-sm"
                  >
                    <e.icon className="text-muted-foreground size-4" />
                    <span className="font-medium">{e.label}</span>
                    <span className="text-muted-foreground font-mono text-xs">{e.value}</span>
                    <span className="text-muted-foreground ml-auto opacity-0 transition-opacity group-hover:opacity-100">
                      ↗
                    </span>
                  </a>
                </li>
              ))}
            </ul>
          </div>
        </div>
      </Frame>

      {/* --------------------------------------------------- selected work */}
      <Frame className="py-10 sm:py-14">
        <SectionHead
          n="01"
          label={t("about.work.label")}
          title={t("about.work.title")}
          lead={t("about.work.lead")}
        />
        <ol className="bg-border/70 mt-12 grid gap-px border-y sm:grid-cols-2">
          {achievements.map((a, i) => (
            <Reveal key={a.slice(0, 24)} delay={i * 40} className="h-full">
              <li className="bg-background flex h-full gap-4 px-6 py-6">
                <span className="text-muted-foreground/50 font-mono text-[11px] tabular-nums">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <p className="text-pretty">{a}</p>
              </li>
            </Reveal>
          ))}
        </ol>
      </Frame>

      {/* ----------------------------------------------------- experience */}
      <Frame className="py-10 sm:py-14">
        <SectionHead n="02" label={t("about.experience.label")} title={t("about.experience.title")} />
        <ol className="mt-12">
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
                      <a
                        href={job.href}
                        target="_blank"
                        rel="noreferrer"
                        className="group inline-flex items-baseline gap-1.5"
                      >
                        {job.company}
                        <span className="text-muted-foreground text-xs opacity-0 transition-opacity group-hover:opacity-100">
                          ↗
                        </span>
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

      {/* -------------------------------------------------------- earlier */}
      <Frame className="py-10 sm:py-14">
        <SectionHead n="03" label={t("about.earlier.label")} lead={t("about.earlier.lead")} />
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

      {/* ---------------------------------------------------- open source */}
      <Frame className="py-10 sm:py-14">
        <SectionHead n="04" label={t("about.oss.label")} lead={t("about.oss.lead")} />
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

      {/* ------------------------------------------------------ education */}
      <Frame className="py-10 pb-24 sm:py-14 sm:pb-32">
        <SectionHead n="05" label={t("about.education.label")} />
        <p className="mt-8 max-w-2xl text-pretty">
          {languages.join(", ")}. {education}
        </p>
      </Frame>
    </>
  );
}
