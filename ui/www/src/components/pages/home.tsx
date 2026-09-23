"use client";

import Link from "next/link";
import { useEffect, useRef, useState } from "react";

import { HeroMesh } from "@/components/hero-mesh";
import { Eyebrow, Frame, Reveal, SectionHead, Tag } from "@/components/kit";
import { Pipeline } from "@/components/pipeline";
import { Button } from "@/components/ui/button";
import { clients, company as facts, lab } from "@/data/site";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";
import { cn } from "@/lib/utils";

/**
 * The front door: every section shows something no other page shows and
 * links out instead of restating it. Who and whether I can be hired, three
 * facts about now, the one drawing of how I build, the playgrounds to click,
 * the lab once it is public, and the email, all hung on one line down the
 * page as boxes on the request's path. The story is `/about/`, the
 * libraries `/projects/`. `app/page.tsx` carries the metadata.
 */
export function HomeContent() {
  const t = useT();
  const { company, playgrounds, principles } = useSite();
  const open = playgrounds.filter((p) => p.href);
  const spineRef = useRef<HTMLDivElement>(null);
  const endRef = useRef<HTMLDivElement>(null);

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
          <Frame className="relative">
            {/* where the spine leaves the hero */}
            <span
              aria-hidden
              className="bg-foreground/60 absolute bottom-0 left-3 size-[5px] -translate-x-1/2 translate-y-1/2 rounded-full sm:left-4"
            />
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

      {/* Everything below hangs on one line: the request's path down the page.
          Each section is a box on it, with a port where its rule meets the
          spine, and one packet crawls the whole way (`Spine`). */}
      <div ref={spineRef} className="relative">
        <Spine root={spineRef} end={endRef} />

        {/* --------------------------------------------------------- method */}
        <Box
          n="01"
          label={t("home.method.label")}
          title={t("home.method.title")}
          lead={t("home.method.lead")}
        >
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
        </Box>

        {/* ---------------------------------------------------- playgrounds */}
        <Box
          n="02"
          label={t("home.playgrounds.label")}
          title={t("home.playgrounds.title")}
          lead={t("home.playgrounds.lead")}
        >
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
        </Box>

        {/* -------------------------------------------------------- clients */}
        {clients.length > 0 && (
          <Box n="03" label={t("home.clients.label")}>
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
          </Box>
        )}

        {/* ------------------------------------------------------------ lab */}
        {lab.public ? (
          <Box
            n={clients.length > 0 ? "04" : "03"}
            label={t("home.lab.label")}
            title={t("home.lab.title")}
            lead={t("home.lab.lead")}
          >
            <Button variant="ghost" className="mt-6 -ml-4" asChild>
              <Link href={lab.href}>{t("home.lab.cta")}</Link>
            </Button>
          </Box>
        ) : null}

        {/* ------------------------------------------------------------ end */}
        <Frame className="relative pt-10 pb-24 sm:pt-14 sm:pb-32">
          <span aria-hidden className="bg-border absolute top-0 left-3 h-10 w-px sm:left-4 sm:h-14" />
          <div ref={endRef} className="relative -ml-3 border-t pl-3 sm:-ml-4 sm:pl-4">
            <Port filled />
            <div className="flex flex-col gap-4 pt-10 sm:flex-row sm:items-baseline sm:justify-between sm:gap-10">
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
          </div>
        </Frame>
      </div>
    </>
  );
}

/**
 * A section as a box on the spine: the line runs down its left gutter, its
 * rule starts at the line with a port on the corner, and the head and the
 * content sit inside as before. `-ml-3 sm:-ml-4` is half the Frame's gutter,
 * which is where the spine is; `Pipeline` uses the same offset.
 */
function Box({
  n,
  label,
  title,
  lead,
  children,
}: {
  n: string;
  label: string;
  title?: string;
  lead?: string;
  children: React.ReactNode;
}) {
  return (
    <Frame className="relative py-10 sm:py-14">
      <span aria-hidden className="bg-border absolute inset-y-0 left-3 w-px sm:left-4" />
      <div className="relative -ml-3 border-t pl-3 sm:-ml-4 sm:pl-4">
        <Port />
        <SectionHead n={n} label={label} title={title} lead={lead} className="border-t-0" />
      </div>
      {children}
    </Frame>
  );
}

/** The node where a section's rule meets the spine; filled where the line ends. */
function Port({ filled }: { filled?: boolean }) {
  return (
    <span
      aria-hidden
      className={cn(
        "absolute -top-[4.5px] -left-[4.5px] size-[9px] rounded-full border",
        filled ? "border-foreground/60 bg-foreground/60" : "border-foreground/40 bg-background",
      )}
    />
  );
}

/**
 * The one packet that crawls the spine from the hero to the email. The line
 * itself is drawn by each box, so it needs no script; the packet needs the
 * distance to the last port, measured here and kept fresh on resize. Hidden
 * until measured, and gone under prefers-reduced-motion (`site.css`).
 */
function Spine({
  root,
  end,
}: {
  root: React.RefObject<HTMLDivElement | null>;
  end: React.RefObject<HTMLDivElement | null>;
}) {
  const [length, setLength] = useState<number | null>(null);
  useEffect(() => {
    const el = root.current;
    const target = end.current;
    if (!el || !target) return;
    const measure = () => setLength(target.getBoundingClientRect().top - el.getBoundingClientRect().top);
    measure();
    if (typeof ResizeObserver === "undefined") return;
    const ro = new ResizeObserver(measure);
    ro.observe(el);
    return () => ro.disconnect();
  }, [root, end]);
  if (length === null) return null;
  return (
    <span
      aria-hidden
      className="spine-packet bg-foreground pointer-events-none absolute top-0 left-[calc(max(0px,50%-36rem)+0.75rem)] size-[5px] rounded-full sm:left-[calc(max(0px,50%-36rem)+1rem)]"
      style={{ "--spine-end": `${length}px` } as React.CSSProperties}
    />
  );
}

/** A faint grid behind the hero, fading out downwards, anchored to the right edge
 * so the mesh drawing sits on its intersections. Decoration only. */
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
          backgroundPosition: "right top",
        }}
      />
    </div>
  );
}
