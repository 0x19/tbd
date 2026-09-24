"use client";

// The tuner: the shared microphone, the pitch arithmetic, a needle. The
// reference tones are synthesised strings played through the same context.

import { useCallback, useEffect, useRef, useState } from "react";

import { Eyebrow, Tag } from "@/components/kit";
import { MicPanel, MicStatus } from "@/components/playgrounds/mic-panel";
import { useMic } from "@/components/playgrounds/use-mic";
import { Button } from "@/components/ui/button";
import { centsOff, detectPitch, freqOfMidi, median, nearestString, noteOf, TUNINGS } from "@/lib/music/pitch";
import { pluck } from "@/lib/music/pluck";
import { cn } from "@/lib/utils";

const A4_MIN = 432;
const A4_MAX = 446;
const SMOOTH = 5;
/** A note this periodic counts; below it is noise, a chord or the room. */
const CLARITY = 0.72;

export function Tuner() {
  const [tuningId, setTuningId] = useState(TUNINGS[0]!.id);
  const [a4, setA4] = useState(440);
  /** The string the meter aims at; null follows whatever is nearest. */
  const [locked, setLocked] = useState<number | null>(null);
  const [reading, setReading] = useState<{ freq: number; string: number; cents: number } | null>(null);
  const [quiet, setQuiet] = useState(true);
  const recent = useRef<number[]>([]);
  const lastHeard = useRef(0);
  // The frame callback reads these without re-subscribing on every change.
  const settings = useRef({ tuningId, a4, locked });
  useEffect(() => {
    settings.current = { tuningId, a4, locked };
  }, [tuningId, a4, locked]);

  const tuning = TUNINGS.find((t) => t.id === tuningId) ?? TUNINGS[0]!;

  const onFrame = useCallback((buf: Float32Array, sampleRate: number) => {
    const pitch = detectPitch(buf, sampleRate);
    const now = performance.now();
    if (pitch && pitch.clarity > CLARITY) {
      lastHeard.current = now;
      recent.current.push(pitch.freq);
      if (recent.current.length > SMOOTH) recent.current.shift();
      const freq = median(recent.current);
      const { tuningId: id, a4: ref, locked: lock } = settings.current;
      const t = TUNINGS.find((x) => x.id === id) ?? TUNINGS[0]!;
      const string = lock ?? nearestString(freq, t, ref);
      setReading({ freq, string, cents: centsOff(freq, freqOfMidi(t.strings[string]!, ref)) });
      setQuiet(false);
    } else if (now - lastHeard.current > 700) {
      // The note has died away: let the needle rest rather than hold a ghost.
      recent.current = [];
      setQuiet(true);
    }
  }, []);
  const mic = useMic(onFrame);

  /** A reference tone for a string: a synthesised pluck, not a beep. */
  const play = (index: number) => {
    const audio = mic.context();
    void audio.resume();
    const samples = pluck(audio.sampleRate, freqOfMidi(tuning.strings[index]!, a4));
    const buffer = audio.createBuffer(1, samples.length, audio.sampleRate);
    buffer.copyToChannel(samples, 0);
    const source = audio.createBufferSource();
    source.buffer = buffer;
    source.connect(audio.destination);
    source.start();
    setLocked(index);
  };

  const listening = mic.status === "listening";
  const cents = reading?.cents ?? 0;
  const shown = listening && reading !== null && !quiet;
  const aimed = shown ? reading.string : locked;
  const inTune = shown && Math.abs(cents) <= 5;
  const close = shown && Math.abs(cents) <= 15;
  const needle = Math.max(-50, Math.min(50, cents));
  const tone = inTune
    ? "text-emerald-600 dark:text-emerald-400"
    : close
      ? "text-amber-600 dark:text-amber-400"
      : "";
  const note = shown ? noteOf(reading.freq, a4) : null;

  return (
    <div className="bg-border/70 grid gap-px border-y sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      {/* ------------------------------------------------------ the meter */}
      <div className="bg-background p-6 sm:p-8">
        <div className="flex items-baseline justify-between gap-4">
          <Eyebrow>Meter</Eyebrow>
          <MicStatus mic={mic} hearing={shown ? "hearing a note" : null} />
        </div>

        <div className="mt-8 flex items-end justify-between gap-6">
          <div>
            <div
              className={cn("font-semibold tracking-[-0.04em] tabular-nums", tone)}
              style={{ fontSize: "clamp(4rem, 14vw, 7rem)", lineHeight: 1 }}
            >
              {aimed !== null ? noteOf(freqOfMidi(tuning.strings[aimed]!, a4), a4).name : "—"}
              <span className="text-muted-foreground ml-1 text-2xl font-normal">
                {aimed !== null ? noteOf(freqOfMidi(tuning.strings[aimed]!, a4), a4).octave : ""}
              </span>
            </div>
            <p className="text-muted-foreground mt-3 font-mono text-sm tabular-nums">
              {shown
                ? `${reading.freq.toFixed(1)} Hz · ${note!.name}${note!.octave} ${cents > 0 ? "+" : ""}${cents} ¢`
                : aimed !== null
                  ? `aiming at ${freqOfMidi(tuning.strings[aimed]!, a4).toFixed(1)} Hz`
                  : "pluck a string"}
            </p>
          </div>
          <div className={cn("text-right font-mono text-[11px] tracking-[0.18em] uppercase", tone)}>
            {shown ? (inTune ? "in tune" : cents < 0 ? "tune up" : "tune down") : ""}
          </div>
        </div>

        {/* The needle: −50 to +50 cents, the middle fifth is green. */}
        <div className="relative mt-8 h-16">
          <div className="bg-border absolute inset-x-0 top-8 h-px" />
          <div className="absolute top-6 left-[45%] h-5 w-[10%] rounded-sm bg-emerald-500/15" />
          {[-50, -25, 0, 25, 50].map((c) => (
            <span
              key={c}
              className="text-muted-foreground/70 absolute top-11 -translate-x-1/2 font-mono text-[10px] tabular-nums"
              style={{ left: `${50 + c}%` }}
            >
              {c > 0 ? `+${c}` : c}
            </span>
          ))}
          <div
            className={cn(
              "absolute top-2 h-12 w-0.5 -translate-x-1/2 rounded-full transition-[left,opacity] duration-100",
              inTune ? "bg-emerald-500" : close ? "bg-amber-500" : "bg-foreground",
              shown ? "opacity-100" : "opacity-20",
            )}
            style={{ left: `${50 + needle}%` }}
          />
        </div>

        <MicPanel mic={mic}>
          {locked !== null ? (
            <Button variant="ghost" onClick={() => setLocked(null)}>
              Follow the nearest string
            </Button>
          ) : null}
        </MicPanel>
      </div>

      {/* ---------------------------------------------------- the strings */}
      <div className="bg-background p-6 sm:p-8">
        <Eyebrow>Strings</Eyebrow>
        <div className="mt-5 flex flex-wrap gap-2">
          {TUNINGS.map((t) => (
            <button
              key={t.id}
              type="button"
              onClick={() => {
                setTuningId(t.id);
                setLocked(null);
              }}
              className={cn(
                "rounded-full border px-3 py-1 text-xs transition-colors",
                t.id === tuningId ? "bg-foreground text-background border-foreground" : "hover:bg-muted/40",
              )}
              title={t.note}
            >
              {t.name}
            </button>
          ))}
        </div>
        <p className="text-muted-foreground mt-2 font-mono text-[11px] tracking-[0.18em] uppercase">
          {tuning.note}
        </p>

        <ol className="mt-6 divide-y border-y">
          {tuning.strings.map((midi, i) => {
            const n = noteOf(freqOfMidi(midi, a4), a4);
            const isAim = aimed === i;
            return (
              <li key={`${tuning.id}-${i}`}>
                <button
                  type="button"
                  onClick={() => play(i)}
                  className={cn(
                    "flex w-full items-center gap-4 py-3 text-left transition-colors",
                    isAim ? "text-foreground" : "text-muted-foreground hover:text-foreground",
                  )}
                  title="Play the reference tone and aim the meter at this string"
                >
                  <span className="text-muted-foreground/60 w-5 font-mono text-[11px]">{6 - i}</span>
                  <span className={cn("w-12 text-xl font-medium tracking-tight", isAim && shown && tone)}>
                    {n.name}
                    <span className="text-muted-foreground ml-0.5 text-xs font-normal">{n.octave}</span>
                  </span>
                  <span className="text-muted-foreground font-mono text-sm tabular-nums">
                    {freqOfMidi(midi, a4).toFixed(2)} Hz
                  </span>
                  <span className="ml-auto flex items-center gap-2">
                    {isAim && shown ? (
                      <Tag>{inTune ? "in tune" : `${cents > 0 ? "+" : ""}${cents} ¢`}</Tag>
                    ) : null}
                    <span className="text-muted-foreground/60 font-mono text-[11px] tracking-[0.18em] uppercase">
                      play
                    </span>
                  </span>
                </button>
              </li>
            );
          })}
        </ol>

        <div className="mt-6 flex items-center justify-between gap-4">
          <div>
            <Eyebrow>Reference</Eyebrow>
            <p className="text-muted-foreground mt-1 text-sm">
              A4 = <span className="text-foreground font-mono tabular-nums">{a4}</span> Hz
            </p>
          </div>
          <div className="flex items-center gap-1">
            <Button
              variant="outline"
              size="sm"
              onClick={() => setA4((v) => Math.max(A4_MIN, v - 1))}
              disabled={a4 <= A4_MIN}
            >
              −
            </Button>
            <Button variant="outline" size="sm" onClick={() => setA4(440)} disabled={a4 === 440}>
              440
            </Button>
            <Button
              variant="outline"
              size="sm"
              onClick={() => setA4((v) => Math.min(A4_MAX, v + 1))}
              disabled={a4 >= A4_MAX}
            >
              +
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
