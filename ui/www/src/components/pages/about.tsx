"use client";

import { IconBrandGithub, IconBrandLinkedin, IconBrandX } from "@tabler/icons-react";

import { Eyebrow, Frame, Reveal, SectionHead } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { company as facts, cv, previously } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";

/**
 * The person: who, the story with the facts beside it, the path in chapters,
 * and a colophon. The record itself, position by position with the lines
 * under each, is the PDF (`public/cv/`, `mise run www:cv`) and is not
 * repeated here; the full version, with contact details and references, is
 * behind `cv.fullUrl`. `app/about/page.tsx` carries the metadata.
 */
export function AboutContent() {
  const t = useT();
  const { about, chapters, company, languages } = useSite();
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

  const colophon = [
    { k: t("about.colophon.built"), v: t("about.colophon.built_v") },
    { k: t("about.colophon.type"), v: t("about.colophon.type_v") },
    { k: t("about.colophon.tracking"), v: t("about.colophon.tracking_v") },
    { k: t("about.colophon.pdf"), v: t("about.colophon.pdf_v") },
  ];

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
          <div className="space-y-6">
            {about.map((paragraph, i) => (
              <p
                key={paragraph.slice(0, 24)}
                className={
                  i === 0
                    ? "text-2xl leading-snug font-medium tracking-[-0.01em] text-pretty sm:text-3xl"
                    : "text-muted-foreground text-lg leading-relaxed text-pretty"
                }
              >
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

      {/* ----------------------------------------------------------- path */}
      <Frame className="py-10 sm:py-14">
        <SectionHead
          n="01"
          label={t("about.path.label")}
          title={t("about.path.title")}
          lead={t("about.path.lead")}
        />
        <ol className="mt-12 border-t">
          {chapters.map((c, i) => (
            <Reveal key={c.when} delay={i * 40}>
              <li className="grid gap-3 border-b py-8 sm:grid-cols-[8rem_minmax(0,1fr)_minmax(0,14rem)] sm:gap-10">
                <div className="text-muted-foreground/70 font-mono text-[11px] tracking-[0.14em] uppercase tabular-nums">
                  {c.when}
                </div>
                <div>
                  <h3 className="text-xl font-medium tracking-tight text-balance">{c.title}</h3>
                  <p className="text-muted-foreground mt-3 max-w-2xl text-pretty">{c.body}</p>
                </div>
                <div className="text-muted-foreground/60 font-mono text-[11px] leading-relaxed text-pretty sm:text-right">
                  {c.where}
                </div>
              </li>
            </Reveal>
          ))}
        </ol>
        <Button variant="ghost" className="mt-6 -ml-4" asChild>
          <a href={cv.pdf} download>
            {t("about.path.pdf")}
          </a>
        </Button>
      </Frame>

      {/* ------------------------------------------------------- colophon */}
      <Frame className="py-10 pb-24 sm:py-14 sm:pb-32">
        <SectionHead n="02" label={t("about.colophon.label")} />
        <dl className="mt-8 max-w-3xl divide-y border-y">
          {colophon.map((r) => (
            <div key={r.k} className="grid gap-2 py-4 sm:grid-cols-[8rem_minmax(0,1fr)] sm:gap-10">
              <dt className="text-muted-foreground/70 font-mono text-[11px] tracking-[0.18em] uppercase">
                {r.k}
              </dt>
              <dd className="text-muted-foreground text-sm text-pretty">{r.v}</dd>
            </div>
          ))}
        </dl>
      </Frame>
    </>
  );
}
