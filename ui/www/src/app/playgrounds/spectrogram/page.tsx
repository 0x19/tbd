import type { Metadata } from "next";

import { Eyebrow, Frame } from "@/components/kit";

import { Spectrogram } from "./spectrogram";

export const metadata: Metadata = {
  title: "See your voice",
  description:
    "A live spectrogram of whatever the microphone hears, with the note it finds named as it goes.",
  alternates: { canonical: "/playgrounds/spectrogram/" },
};

export default function SpectrogramPage() {
  return (
    <>
      <Frame className="pt-20 pb-10 sm:pt-28">
        <Eyebrow>Playground · Spectrogram</Eyebrow>
        <h1 className="mt-6 max-w-3xl text-4xl font-semibold tracking-[-0.035em] text-balance sm:text-6xl">
          See your voice. Or the kettle, or a guitar, or the room.
        </h1>
        <p className="text-muted-foreground mt-6 max-w-2xl text-pretty">
          Press start and the page draws what the microphone hears: time runs left to right, pitch runs up the
          side, brightness is loudness. Hum and a stack of lines appears, the fundamental and its harmonics,
          and the note is named at the edge. Whistle and there is one line. Say &ldquo;sss&rdquo; and it is a
          cloud. Freeze it to look closely.
        </p>
      </Frame>

      <Frame className="pb-10">
        <Spectrogram />
      </Frame>

      <Frame className="pb-10">
        <Eyebrow>How it draws</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          Every frame takes a 4,096-point spectrum of the last few milliseconds and paints it as one column,
          on a logarithmic frequency axis so that every octave is the same height, which is why harmonics
          crowd together higher up. The white dot is the fundamental the tuner&rsquo;s pitch detector finds,
          the fainter ones are where its harmonics should be, and the label is that pitch as a note. A window
          of the last sixteen seconds or so is kept on screen; nothing before that is kept anywhere.
        </p>
      </Frame>

      <Frame className="pb-24 sm:pb-32">
        <Eyebrow>What this page sends</Eyebrow>
        <p className="text-muted-foreground mt-3 max-w-2xl text-sm text-pretty">
          Nothing. The microphone is read in your browser and drawn straight onto the page; no sound is
          recorded, uploaded or kept, and no request is made to any server while it runs. No cookies, no
          analytics, nothing from anyone else.
        </p>
      </Frame>
    </>
  );
}
