"use client";

import Link from "next/link";

import { Eyebrow, Frame, Reveal } from "@/components/kit";
import { PlaygroundTabs } from "@/components/playground-tabs";
import { Button } from "@/components/ui/button";
import { useT } from "@/lib/i18n";
import { useSite } from "@/lib/i18n/site";

/**
 * The playgrounds index in the reader's language, as tabs by category with the
 * systems piece featured (`PlaygroundTabs`); `app/playgrounds/page.tsx`
 * carries the metadata.
 */
export function PlaygroundsContent() {
  const t = useT();
  const { playgrounds } = useSite();
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>{t("playgrounds.eyebrow")}</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          {t("playgrounds.title")}
        </h1>
        <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">{t("playgrounds.lead")}</p>
        <p className="text-muted-foreground mt-4 max-w-xl text-pretty">{t("playgrounds.since")}</p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        {playgrounds.length ? (
          <PlaygroundTabs items={playgrounds} feature hash columns="lg:grid-cols-3" />
        ) : (
          <Reveal>
            <div className="border-y py-20 text-center">
              <p className="text-muted-foreground/60 font-mono text-[11px] tracking-[0.18em] uppercase">
                {t("playgrounds.status")}
              </p>
              <p className="mx-auto mt-6 max-w-2xl text-2xl font-semibold tracking-[-0.02em] text-balance sm:text-3xl">
                {t("playgrounds.none.title")}
              </p>
              <p className="text-muted-foreground mx-auto mt-5 max-w-md text-sm text-pretty">
                {t("playgrounds.none.lead")}
              </p>
              <Button variant="outline" className="mt-8" asChild>
                <Link href="/contact/">{t("playgrounds.none.cta")}</Link>
              </Button>
            </div>
          </Reveal>
        )}
      </Frame>
    </>
  );
}
