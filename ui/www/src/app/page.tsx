import type { Metadata } from "next";
import Link from "next/link";

import { Eyebrow, Frame, IndexRow, Marquee, Reveal, SectionHead } from "@/components/kit";
import { Pipeline } from "@/components/pipeline";
import { Button } from "@/components/ui/button";
import { clients, company, playgrounds, principles, projects, stack } from "@/data/site";

export const metadata: Metadata = {
  alternates: { canonical: "/" },
};

export default function HomePage() {
  return (
    <>
      {/* ---------------------------------------------------------- hero */}
      <section className="relative overflow-hidden">
        <Grid />
        <Frame className="relative pt-20 pb-14 sm:pt-32 sm:pb-20">
          <Eyebrow className="flex flex-wrap items-center gap-x-3 gap-y-1">
            <span className="text-foreground">{company.person}</span>
            <span className="bg-border h-px w-6" />
            <span>{company.city}</span>
          </Eyebrow>

          <h1 className="mt-8 max-w-4xl text-[2.75rem] leading-[1.02] font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
            {company.headline}
          </h1>

          <p className="text-muted-foreground mt-8 max-w-2xl text-lg text-pretty">{company.summary}</p>

          <p className="mt-6 max-w-2xl border-l-2 pl-4 text-pretty">{company.availability}</p>

          <div className="mt-9 flex flex-wrap gap-2">
            <Button size="lg" asChild>
              <Link href="/about/">Read the CV</Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="/playgrounds/">See the playgrounds</Link>
            </Button>
            <Button size="lg" variant="outline" asChild>
              <Link href="/projects/">What I have built</Link>
            </Button>
          </div>
        </Frame>

        <div className="border-y">
          <Frame>
            <Marquee items={stack} />
          </Frame>
        </div>
      </section>

      {/* --------------------------------------------------- playgrounds */}
      <Frame className="py-14 sm:py-20">
        <SectionHead n="01" label="Playgrounds" />
        <Reveal className="mt-10">
          <Link
            href="/playgrounds/"
            className="group hover:bg-muted/30 flex flex-col gap-8 border-y p-8 transition-colors sm:flex-row sm:items-end sm:justify-between sm:p-10"
          >
            <div>
              <h2 className="max-w-2xl text-2xl font-semibold tracking-[-0.02em] text-balance sm:text-4xl">
                Small things I build for fun, put up so you can poke at them.
              </h2>
              <p className="text-muted-foreground mt-5 max-w-xl text-pretty">
                Every so often a prototype is interesting enough to leave running. Each one is the real thing
                — you press the button and something actually happens.{" "}
                {playgrounds.length
                  ? "Have a look at what is up."
                  : "None are up yet; the first will appear here when it is ready."}
              </p>
            </div>
            <span className="text-muted-foreground group-hover:text-foreground shrink-0 font-mono text-[11px] tracking-[0.18em] uppercase transition-colors">
              {playgrounds.length ? `${playgrounds.length} up →` : "What is coming →"}
            </span>
          </Link>
        </Reveal>
      </Frame>

      {/* ------------------------------------------------------- projects */}
      <Frame className="py-10 sm:py-14">
        <SectionHead
          n="02"
          label="Things I have built"
          title="Libraries I wrote and other people use."
          lead="Left in the open because they are more useful read than kept in a drawer."
        />
        <Reveal className="mt-12">
          <div>
            {projects.slice(0, 4).map((p, i) => (
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
          <Button variant="ghost" className="mt-6 -ml-4" asChild>
            <Link href="/projects/">All of them →</Link>
          </Button>
        </Reveal>
      </Frame>

      {/* --------------------------------------------------------- clients */}
      {clients.length > 0 && (
        <Frame className="py-10 sm:py-14">
          <SectionHead n="03" label="Working with" />
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
                Figures published by {c.name}
              </p>
            </Reveal>
          ))}
        </Frame>
      )}

      {/* ---------------------------------------------------------- method */}
      <Frame className="py-10 sm:py-14">
        <SectionHead
          n={clients.length > 0 ? "04" : "03"}
          label="How I like to build"
          title="Layer 1 to Layer 7, and the rails that cross them."
          lead="Most of the work has been somewhere in this column — a packet on the way in, a chain on the way through, a query on the way out. The rails are the part people skip, and the part that decides whether a bad night is an incident or a shrug."
        />
        <Reveal className="mt-10">
          <Pipeline />
        </Reveal>
        <div className="bg-border/70 mt-px grid gap-px border-b sm:grid-cols-4">
          {principles.map((p, i) => (
            <Reveal key={p.title} delay={i * 60} className="h-full">
              <div className="bg-background h-full px-6 py-6">
                <span className="text-muted-foreground/50 font-mono text-[11px] tabular-nums">
                  {String(i + 1).padStart(2, "0")}
                </span>
                <h3 className="mt-3 font-medium tracking-tight">{p.title}</h3>
                <p className="text-muted-foreground mt-2 text-sm text-pretty">{p.body}</p>
              </div>
            </Reveal>
          ))}
        </div>
      </Frame>

      {/* ------------------------------------------------------------- end */}
      <Frame className="pt-10 pb-24 sm:pt-16 sm:pb-32">
        <Reveal>
          <div className="border-t pt-12">
            <Eyebrow>Say hi</Eyebrow>
            <a
              href={`mailto:${company.email}`}
              className="mt-6 block text-3xl font-semibold tracking-[-0.03em] break-all transition-opacity hover:opacity-60 sm:text-6xl"
            >
              {company.email}
            </a>
            <p className="text-muted-foreground mt-6 max-w-xl text-pretty">
              Questions about any of this, something you are stuck on, or an idea worth a prototype — all
              welcome. There is no form and no funnel, just me reading the mail.{" "}
              <Link href="/about/" className="text-foreground underline-offset-4 hover:underline">
                More about me
              </Link>
              .
            </p>
          </div>
        </Reveal>
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
