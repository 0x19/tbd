import type { Metadata } from "next";
import Link from "next/link";

import { Eyebrow, Frame, Reveal, Tag } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { playgrounds } from "@/data/site";

export const metadata: Metadata = {
  title: "Playgrounds",
  description: "Small things I build for fun, put up so anyone can try them.",
  alternates: { canonical: "/playgrounds/" },
};

export default function PlaygroundsPage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playgrounds</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          Things to try, not screenshots of things to try.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">
          Every so often I build a small thing for the fun of it — a parser you can paste into, a simulator
          you can push, a piece of a bigger system pulled out and left running. When one is worth keeping up,
          it lands here. No sign-up, nothing to install.
        </p>
        <p className="text-muted-foreground mt-4 max-w-xl text-pretty">
          This is not a new habit: a JSON-to-Go-struct converter and a disposable-email API have been sitting
          in the open since 2015 and 2016 for exactly the same reason.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        {playgrounds.length ? (
          <div className="bg-border/70 grid gap-px border-y sm:grid-cols-2">
            {playgrounds.map((p) => {
              const inner = (
                <>
                  <div className="flex items-center gap-3">
                    <h2 className="text-xl font-medium tracking-tight">{p.name}</h2>
                    <Tag>{p.tag}</Tag>
                    {p.href ? (
                      <span className="text-muted-foreground ml-auto transition-transform group-hover:translate-x-1">
                        →
                      </span>
                    ) : (
                      <span className="text-muted-foreground/70 ml-auto font-mono text-[11px] tracking-[0.18em] uppercase">
                        Building
                      </span>
                    )}
                  </div>
                  <p className="text-muted-foreground mt-3 text-sm text-pretty">{p.what}</p>
                </>
              );
              return p.href ? (
                <a
                  key={p.name}
                  href={p.href}
                  className="group bg-background hover:bg-muted/30 p-8 transition-colors"
                >
                  {inner}
                </a>
              ) : (
                <div key={p.name} className="bg-background p-8">
                  {inner}
                </div>
              );
            })}
          </div>
        ) : (
          <Reveal>
            <div className="border-y py-20 text-center">
              <p className="text-muted-foreground/60 font-mono text-[11px] tracking-[0.18em] uppercase">
                Status
              </p>
              <p className="mx-auto mt-6 max-w-2xl text-2xl font-semibold tracking-[-0.02em] text-balance sm:text-3xl">
                Nothing is up yet. The first one appears here when it is ready, not before.
              </p>
              <p className="text-muted-foreground mx-auto mt-5 max-w-md text-sm text-pretty">
                If you want a shout when it does, send me a line. It is one person reading the mail.
              </p>
              <Button variant="outline" className="mt-8" asChild>
                <Link href="/contact/">Send me a line</Link>
              </Button>
            </div>
          </Reveal>
        )}
      </Frame>
    </>
  );
}
