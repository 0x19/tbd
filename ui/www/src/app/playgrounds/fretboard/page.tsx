import type { Metadata } from "next";

import { Eyebrow, Frame } from "@/components/kit";

import { Trainer } from "./trainer";

export const metadata: Metadata = {
  title: "Fretboard trainer",
  description:
    "Learn the notes on the neck. It asks for a note, you play it, and the microphone says whether you did.",
  alternates: { canonical: "/playgrounds/fretboard/" },
};

export default function FretboardPage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playground · Fretboard</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          Learn the neck with something that can hear whether you got it.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-pretty">
          It names a note and a string; you find it and play it, and the same ear as the tuner says yes or no.
          Or turn it round: it marks a spot on the neck and you name it. It counts what you get right, how
          long the streak runs, and which notes you keep missing, so the practice aims itself. Pick the
          strings and the stretch of frets you are working on.
        </p>
      </Frame>

      <Frame className="pb-10">
        <Trainer />
      </Frame>

      <Frame className="pb-10">
        <Eyebrow>How it judges</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          A note counts once it has held for a few frames within forty cents of a pitch, so a passing scrape
          does not. The right pitch on the right string is exact: the E at the 12th fret of the low string and
          the open E above it are different questions with different answers. A wrong note that holds is a
          miss and the answer is shown; a quiet room is just a wait.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow>What this page sends</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          Nothing. The microphone is opened when you press start and read in your browser, and the score lives
          only in the page until you leave it. No recording, no upload, no cookies, no analytics, nothing from
          anyone else.
        </p>
      </Frame>
    </>
  );
}
