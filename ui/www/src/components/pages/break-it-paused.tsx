"use client";

import Link from "next/link";

import { Frame } from "@/components/kit";
import { useT } from "@/lib/i18n";

/** Break it while it is paused: what it was, that it is coming back, and nothing live. */
export function BreakItPausedContent() {
  const t = useT();
  return (
    <Frame className="pt-20 pb-24 sm:pt-28 sm:pb-32">
      <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
        {t("playgrounds.paused")}
      </p>
      <h1 className="mt-4 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-5xl">
        {t("playgrounds.breakit.paused.title")}
      </h1>
      <p className="text-muted-foreground mt-6 max-w-xl text-lg text-pretty">
        {t("playgrounds.breakit.paused.lead")}
      </p>
      <p className="mt-10">
        <Link
          href="/playgrounds/"
          className="text-muted-foreground hover:text-foreground font-mono text-[11px] tracking-[0.14em] uppercase"
        >
          ← {t("playgrounds.back")}
        </Link>
      </p>
    </Frame>
  );
}
