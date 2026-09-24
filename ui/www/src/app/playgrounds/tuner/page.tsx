import type { Metadata } from "next";

import { Eyebrow, Frame } from "@/components/kit";

import { Tuner } from "./tuner";

export const metadata: Metadata = {
  title: "Guitar tuner",
  description:
    "A guitar tuner in the browser. It listens through the microphone and keeps what it hears to itself.",
  alternates: { canonical: "/playgrounds/tuner/" },
};

export default function TunerPage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playground · Tuner</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          A guitar tuner that listens, and keeps what it hears to itself.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-pretty">
          Press start, allow the microphone, pluck a string. The meter finds the note, names the string it is
          nearest to and shows how many cents you are off. Tap a string to hear its reference tone and to aim
          the meter at it; change the tuning or the reference pitch if your guitar lives somewhere other than
          440. Nothing is recorded and nothing leaves the page.
        </p>
      </Frame>

      <Frame className="pb-10">
        <Tuner />
      </Frame>

      <Frame className="pb-10">
        <Eyebrow>How it hears</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          The microphone is read in windows of 4,096 samples. Each window is compared against itself at every
          lag (McLeod&rsquo;s normalised square difference), and the first strong peak is the period of the
          string, refined between samples with a parabola. That is what keeps a low E from being mistaken for
          its octave, which plain autocorrelation does on a phone. The frequency becomes a note with
          1200&nbsp;·&nbsp;log₂(f&nbsp;/&nbsp;f₀) cents, the last five readings are averaged by their median
          so the needle does not shake, and the needle rests when the note dies away. Within five cents counts
          as in tune, which is closer than most ears.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow>What this page sends</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          Nothing. The microphone is opened only when you press start, it is read by the page in your browser,
          and it is closed when you press stop or leave. No sound is recorded, uploaded or kept, and no
          request is made to any server while you tune. No cookies, no analytics, nothing from anyone else.
        </p>
      </Frame>
    </>
  );
}
