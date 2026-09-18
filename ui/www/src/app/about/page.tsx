import { IconBrandGithub, IconBrandLinkedin, IconBrandX } from "@tabler/icons-react";
import type { Metadata } from "next";

import { Eyebrow, Frame, Reveal, SectionHead, Tag } from "@/components/kit";
import { about, achievements, company, experience, languages, previously } from "@/data/site";

export const metadata: Metadata = {
  title: "About",
  description: `${company.person} — ${company.title}. Twenty years of telecom, real-time networks and blockchain protocols.`,
  alternates: { canonical: "/about/" },
};

export default function AboutPage() {
  const elsewhere = [
    { icon: IconBrandGithub, label: "GitHub", value: "0x19", href: company.github },
    { icon: IconBrandX, label: "X", value: "@vesicnevio", href: company.x },
    { icon: IconBrandLinkedin, label: "LinkedIn", value: "neviovesic", href: company.linkedin },
  ];

  const facts = [
    { k: "Where", v: `${company.city}, remote` },
    { k: "Now", v: `${company.role} at ${company.employer.name}` },
    { k: "Before", v: previously.join(", ") },
    { k: "Languages", v: languages.join(", ") },
    { k: "Email", v: company.email },
  ];

  return (
    <>
      {/* ------------------------------------------------------------ who */}
      <Frame className="pt-20 pb-12 sm:pt-28">
        <Eyebrow>About</Eyebrow>
        <h1 className="mt-6 text-5xl font-semibold tracking-[-0.04em] text-balance sm:text-7xl">
          {company.person}
        </h1>
        <p className="mt-6 max-w-2xl text-xl text-pretty">{company.title}</p>
        <p className="text-muted-foreground mt-3 max-w-2xl text-pretty">
          Twenty years on the half of the stack nobody demos: the part that has to be awake at four in the
          morning.
        </p>
      </Frame>

      {/* -------------------------------------------------------- summary */}
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
              {facts.map((f) => (
                <div key={f.k} className="flex items-baseline justify-between gap-6 py-3">
                  <dt className="text-muted-foreground/70 shrink-0 font-mono text-[11px] tracking-[0.18em] uppercase">
                    {f.k}
                  </dt>
                  <dd className="text-right text-sm text-pretty">
                    {f.k === "Email" ? (
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

      {/* ----------------------------------------------------- experience */}
      <Frame className="py-10 sm:py-14">
        <SectionHead n="01" label="Experience" title="Where the work happened." />
        <ol className="mt-12">
          {experience.map((job, i) => (
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

      {/* --------------------------------------------------- achievements */}
      <Frame className="py-10 pb-24 sm:py-14 sm:pb-32">
        <SectionHead
          n="02"
          label="Worth pulling out"
          title="Six things, specifically."
          lead="Every one of these had a date, a team and something that had to work at the end of it."
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
    </>
  );
}
