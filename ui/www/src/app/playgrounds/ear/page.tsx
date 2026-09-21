import type { Metadata } from "next";

import { Eyebrow, Frame } from "@/components/kit";

import { Ear } from "./ear";

export const metadata: Metadata = {
  title: "Ear trainer",
  description: "It plays two notes or a chord on a synthesised guitar; you name the interval or the chord.",
  alternates: { canonical: "/playgrounds/ear/" },
};

export default function EarPage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playground · Ear</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          It plays. You name it. The ones that fool you make a list.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-pretty">
          Intervals rising, falling or sounded together, and the common chords strummed, all on the same
          synthesised strings as the tuner. Answer, hear whether you were right, and watch the score gather
          per interval, so a minor sixth that keeps passing for a fifth stops getting away with it.
        </p>
      </Frame>

      <Frame className="pb-10">
        <Ear />
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow>What this page sends</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          Nothing. The sounds are made in your browser, the score lives in the page until you leave it, and no
          request is made to any server. No microphone either. No cookies, no analytics, nothing from anyone
          else.
        </p>
      </Frame>
    </>
  );
}
