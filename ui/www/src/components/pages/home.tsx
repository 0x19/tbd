"use client";

import Link from "next/link";

import { HeroMesh } from "@/components/hero-mesh";
import { Eyebrow, Frame, Reveal, SectionHead, Tag } from "@/components/kit";
import { Pipeline } from "@/components/pipeline";
import { Button } from "@/components/ui/button";
import { clients, company as facts, lab } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";

/**
 * The front door: every section shows something no other page shows and
 * links out instead of restating it. Who and whether I can be hired, three
 * facts about now, the playgrounds to click, the one drawing of how I build,
 * the lab once it is public, and the email. The story is `/about/`, the
 * libraries `/projects/`. `app/page.tsx` carries the metadata.
 */
export function HomeContent() {
  const t = useT();
  const { company, playgrounds, principles } = useSite();
  const open = playgrounds.filter((p) => p.href);

  const now = [
    { k: t("home.now.now"), v: company.now },
    { k: t("home.now.building"), v: t("home.now.building_v") },
    { k: t("home.now.based"), v: `${facts.city}, ${t("home.now.hours")}` },
  ];

  return (
    <>
      {/* ---------------------------------------------------------- hero */}
      <section className="relative overflow-hidden">
        <Grid />
        <HeroMesh />
        <Frame className="relative pt-20 pb-14 sm:pt-32 sm:pb-20">
          <Eyebrow className="flex flex-wrap items-center gap-x-3 gap-y-1">
            <span className="text-foreground">{facts.person}</span>
            <span className="bg-border h-px w-6" />
            <span>{company.title}</span>
          </Eyebrow>

          <h1 className="mt-8 max-w-4xl text-[2.75rem] leading-[1.02] font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
            {company.headline}
          </h1>

          <p className="mt-8 max-w-2xl border-l-2 pl-4 text-lg text-pretty">{company.availability}</p>

          <div className="mt-9 flex flex-wrap gap-2">
            <Button size="lg" asChild>
              <Link href="/about/">{t("home.read_cv")}</Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="/playgrounds/">{t("home.see_playgrounds")}</Link>
            </Button>
          </div>
        </Frame>

        {/* Three facts about now, where the ticker of tags used to run. */}
        <div className="border-y">
          <Frame>
            <dl className="grid divide-y sm:grid-cols-3 sm:divide-x sm:divide-y-0">
              {now.map((f) => (
                <div key={f.k} className="py-4 sm:px-6 sm:first:pl-0 sm:last:pr-0">
                  <dt className="text-muted-foreground/70 font-mono text-[11px] tracking-[0.18em] uppercase">
                    {f.k}
                  </dt>
                  <dd className="mt-1 text-sm text-pretty">{f.v}</dd>
                </div>
              ))}
            </dl>
          </Frame>
        </div>
      </section>

      {/* --------------------------------------------------- playgrounds */}
      <Frame className="py-14 sm:py-20">
        <SectionHead
          n="01"
          label={t("home.playgrounds.label")}
          title={t("home.playgrounds.title")}
          lead={t("home.playgrounds.lead")}
        />
        <Reveal className="mt-10">
          <div className="bg-border/70 grid gap-px border-y sm:grid-cols-2 lg:grid-cols-4">
            {open.slice(0, 4).map((p) => (
              <a
                key={p.href}
                href={p.href!}
                className="group bg-background hover:bg-muted/30 flex flex-col p-6 transition-colors"
              >
                <div className="flex items-center gap-3">
                  <h3 className="font-medium tracking-tight">{p.name}</h3>
                  <Tag>{p.tag}</Tag>
                  <span className="text-muted-foreground ml-auto transition-transform group-hover:translate-x-1">
                    →
                  </span>
                </div>
                <p className="text-muted-foreground mt-3 line-clamp-3 text-sm text-pretty">{p.what}</p>
              </a>
            ))}
          </div>
          <Button variant="ghost" className="mt-6 -ml-4" asChild>
            <Link href="/playgrounds/">{t("home.playgrounds.all", { n: open.length })}</Link>
          </Button>
        </Reveal>
      </Frame>

      {/* ---------------------------------------------------------- method */}
      <Frame className="py-10 sm:py-14">
        <SectionHead
          n="02"
          label={t("home.method.label")}
          title={t("home.method.title")}
          lead={t("home.method.lead")}
        />
        <Reveal className="mt-10">
          <Pipeline />
        </Reveal>
        <dl className="mt-px divide-y border-b">
          {principles.map((p, i) => (
            <div key={p.title} className="grid gap-1 py-4 sm:grid-cols-[3rem_14rem_minmax(0,1fr)] sm:gap-6">
              <dt className="text-muted-foreground/50 font-mono text-[11px] tabular-nums">
                {String(i + 1).padStart(2, "0")}
              </dt>
              <dt className="font-medium tracking-tight">{p.title}</dt>
              <dd className="text-muted-foreground text-sm text-pretty">{p.body}</dd>
            </div>
          ))}
        </dl>
      </Frame>

      {/* --------------------------------------------------------- clients */}
      {clients.length > 0 && (
        <Frame className="py-10 sm:py-14">
          <SectionHead n="03" label={t("home.clients.label")} />
          {clients.map((c) => (
            <Reveal key={c.name} className="mt-10">
              <article className="border-y">
                <div className="grid gap-8 p-8 sm:grid-cols-[minmax(0,1fr)_minmax(0,1.3fr)] sm:gap-16 sm:p-10">
                  <div>
                    <a
                      href={c.href}
                      target="_blank"
                      rel="noreferrer"
                      className="group inline-flex items-baseline gap-2 text-3xl font-semibold tracking-[-0.03em] sm:text-4xl"
                    >
                      {c.name}
                      <span className="text-muted-foreground text-sm transition-transform group-hover:translate-x-1">
                        ↗
                      </span>
                    </a>
                    <p className="text-muted-foreground mt-3 font-mono text-[11px] tracking-[0.18em] uppercase">
                      {c.role}
                    </p>
                    <p className="mt-6 text-lg font-medium tracking-tight text-balance">
                      &ldquo;{c.tagline}&rdquo;
                    </p>
                  </div>
                  <div>
                    <p className="text-muted-foreground text-pretty">{c.what}</p>
                    <p className="mt-4 text-pretty">{c.mine}</p>
                  </div>
                </div>
                <dl className="bg-border/70 grid gap-px border-t sm:grid-cols-4">
                  {c.facts.map((f) => (
                    <div key={f.label} className="bg-background px-6 py-5">
                      <dt className="sr-only">{f.label}</dt>
                      <dd>
                        <span className="block text-2xl font-semibold tracking-[-0.02em] tabular-nums">
                          {f.value}
                        </span>
                        <span className="text-muted-foreground mt-1.5 block text-xs text-pretty">
                          {f.label}
                        </span>
                      </dd>
                    </div>
                  ))}
                </dl>
              </article>
              <p className="text-muted-foreground/70 mt-4 font-mono text-[11px] tracking-[0.14em] uppercase">
                {t("common.figures_by", { name: c.name })}
              </p>
            </Reveal>
          ))}
        </Frame>
      )}

      {/* ------------------------------------------------------------- lab */}
      {lab.public ? (
        <Frame className="py-10 sm:py-14">
          <SectionHead
            n={clients.length > 0 ? "04" : "03"}
            label={t("home.lab.label")}
            title={t("home.lab.title")}
            lead={t("home.lab.lead")}
          />
          <Button variant="ghost" className="mt-6 -ml-4" asChild>
            <Link href={lab.href}>{t("home.lab.cta")}</Link>
          </Button>
        </Frame>
      ) : null}

      {/* ------------------------------------------------------------- end */}
      <Frame className="pt-10 pb-24 sm:pt-14 sm:pb-32">
        <div className="flex flex-col gap-4 border-t pt-10 sm:flex-row sm:items-baseline sm:justify-between sm:gap-10">
          <div>
            <Eyebrow>{t("home.hi.label")}</Eyebrow>
            <a
              href={`mailto:${facts.email}`}
              className="mt-4 block text-2xl font-semibold tracking-[-0.03em] break-all transition-opacity hover:opacity-60 sm:text-4xl"
            >
              {facts.email}
            </a>
          </div>
          <p className="text-muted-foreground max-w-xs text-sm text-pretty">{t("home.hi.text")}</p>
        </div>
      </Frame>
    </>
  );
}

/** A faint grid behind the hero, fading out downwards. Decoration only. */
function Grid() {
  return (
    <div
      aria-hidden
      className="pointer-events-none absolute inset-x-0 top-0 -z-10 h-[36rem]"
      style={{
        maskImage: "linear-gradient(to bottom, black 10%, transparent)",
        WebkitMaskImage: "linear-gradient(to bottom, black 10%, transparent)",
      }}
    >
      <div
        className="absolute inset-0 opacity-[0.06] dark:opacity-[0.09]"
        style={{
          backgroundImage:
            "linear-gradient(to right, currentColor 1px, transparent 1px), linear-gradient(to bottom, currentColor 1px, transparent 1px)",
          backgroundSize: "72px 72px",
        }}
      />
    </div>
  );
}
