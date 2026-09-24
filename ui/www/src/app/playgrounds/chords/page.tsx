import type { Metadata } from "next";

import { Eyebrow, Frame } from "@/components/kit";

import { Chords } from "./chords";

export const metadata: Metadata = {
  title: "Chord namer",
  description: "Tap frets on a neck and it names the chord; type a chord and it shows you ways to play it.",
  alternates: { canonical: "/playgrounds/chords/" },
};

export default function ChordsPage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playground · Chords</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          What is this chord I am holding? And how else could I hold it?
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-pretty">
          Tap the frets you have your fingers on and the page names the chord, with the other names it could
          go by. Or type one, Am7 or F♯m or B♭maj7, and it lays out shapes up the neck; tap a shape to put it
          on the neck and hear it strummed. No microphone: this one is pure theory, so it works on the bus.
        </p>
      </Frame>

      <Frame className="pb-10">
        <Chords />
      </Frame>

      <Frame className="pb-10">
        <Eyebrow>How it names</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          The notes you hold are reduced to their pitch classes and compared with nineteen chord formulas from
          every possible root. A match with its root in the bass comes first; a match with another note in the
          bass is an inversion and is written with a slash. Shapes are found by search: every string open,
          muted or fretted within a three-fret span, keeping every chord tone, the root lowest and no muted
          string above a played one. The strum is the same synthesised string as the tuner&rsquo;s.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow>What this page sends</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          Nothing. What you tap and type stays in the page, and no request is made to any server. No cookies,
          no analytics, nothing from anyone else.
        </p>
      </Frame>
    </>
  );
}
