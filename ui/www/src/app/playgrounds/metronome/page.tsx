import type { Metadata } from "next";

import { Eyebrow, Frame } from "@/components/kit";

import { Metronome } from "./metronome";

export const metadata: Metadata = {
  title: "Metronome",
  description:
    "A metronome that keeps time on the audio clock and remembers your practice, in your browser only.",
  alternates: { canonical: "/playgrounds/metronome/" },
};

export default function MetronomePage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playground · Metronome</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          A metronome that keeps time, and keeps count of your practice.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-pretty">
          Set a tempo, or tap it in. Choose the beats in a bar and how they divide, and the first beat rings
          higher. It runs on the audio clock rather than the page&rsquo;s, so it does not drift when the
          browser is busy. Every session of half a minute or more goes into a small log that lives in this
          browser and nowhere else.
        </p>
      </Frame>

      <Frame className="pb-10">
        <Metronome />
      </Frame>

      <Frame className="pb-10">
        <Eyebrow>How it keeps time</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          JavaScript timers are allowed to be late, and are, so a metronome built on them limps. This one asks
          the audio engine to play each click at an exact time on its own clock, scheduling a tenth of a
          second ahead every twenty-five milliseconds. The clicks land where they should; only the lights
          follow the page&rsquo;s timer. Tap tempo averages the gaps between your last few taps.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow>What this page sends</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          Nothing. The practice log is kept in your browser&rsquo;s local storage, on this device, for you; it
          is never sent anywhere and you can clear it with one button. No cookies, no analytics, nothing from
          anyone else.
        </p>
      </Frame>
    </>
  );
}
