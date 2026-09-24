"use client";

// The chord namer, both ways round. Tap frets on the neck and it names what
// you are holding, with the alternatives a jazz player would offer. Or
// type a chord and it lays out ways to play it, each one tappable onto the
// neck and strummed. Pure theory plus the synthesised strings; no audio in.

import { useMemo, useRef, useState } from "react";

import { Eyebrow, Tag } from "@/components/kit";
import { ChordBox, type Dot, Fretboard } from "@/components/playgrounds/fretboard";
import { Button } from "@/components/ui/button";
import { strum } from "@/lib/music/play";
import { name, nameChord, noteAt, octave, parseChord, STANDARD, voicings } from "@/lib/music/theory";
import { cn } from "@/lib/utils";

const OPEN_C: (number | null)[] = [null, 3, 2, 0, 1, 0];

export function Chords() {
  const [frets, setFrets] = useState<(number | null)[]>(OPEN_C);
  const [typed, setTyped] = useState("Am7");
  const [flats, setFlats] = useState(false);
  const ctx = useRef<AudioContext | null>(null);
  const audio = () => {
    if (!ctx.current) ctx.current = new AudioContext();
    void ctx.current.resume();
    return ctx.current;
  };

  const sounding = frets
    .map((f, s) => (f === null ? null : noteAt(s, f)))
    .filter((n): n is number => n !== null);
  const names = useMemo(() => nameChord(sounding, flats), [sounding, flats]);
  const parsed = useMemo(() => parseChord(typed), [typed]);
  const shapes = useMemo(() => (parsed ? voicings(parsed.root, parsed.chord) : []), [parsed]);
  const typedFlats = /[b♭]/.test(typed.slice(1, 3));

  const tap = (string: number, fret: number) =>
    setFrets((cur) => {
      const nextFrets = [...cur];
      nextFrets[string] = cur[string] === fret ? null : fret;
      return nextFrets;
    });
  const dots: Dot[] = frets.map((f, s) =>
    f === null
      ? { string: s, fret: 0, tone: "muted" }
      : { string: s, fret: f, label: name(noteAt(s, f), flats), tone: f === 0 ? "plain" : "aim" },
  );
  const play = () => strum(audio(), sounding);

  return (
    <div className="bg-border/70 grid gap-px border-y sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      {/* ------------------------------------------------------- the neck */}
      <div className="bg-background p-6 sm:p-8">
        <div className="flex items-baseline justify-between gap-4">
          <Eyebrow>Tap the neck</Eyebrow>
          <div className="flex items-center gap-1">
            <Button variant={flats ? "outline" : "default"} size="sm" onClick={() => setFlats(false)}>
              ♯
            </Button>
            <Button variant={flats ? "default" : "outline"} size="sm" onClick={() => setFlats(true)}>
              ♭
            </Button>
          </div>
        </div>
        <div className="mt-5">
          <Fretboard frets={12} dots={dots} onTap={tap} />
        </div>
        <p className="text-muted-foreground mt-2 text-xs">
          Tap a fret to hold it, tap it again to mute the string, tap left of the nut for an open string.
        </p>

        <div className="mt-6 flex flex-wrap items-end justify-between gap-4">
          <div>
            <div className="text-5xl font-semibold tracking-[-0.03em] sm:text-6xl">
              {names[0]?.symbol ?? (sounding.length ? "?" : "—")}
            </div>
            <p className="text-muted-foreground mt-2 text-sm">
              {names[0]
                ? `${name(names[0].root, flats)} ${names[0].chord.name}${names[0].bass !== undefined ? `, ${name(names[0].bass, flats)} in the bass` : ""}`
                : sounding.length
                  ? "not a chord I have a name for"
                  : "nothing held"}
            </p>
            <p className="text-muted-foreground mt-1 font-mono text-xs">
              {frets
                .map((f, s) => (f === null ? "×" : `${name(noteAt(s, f), flats)}${octave(noteAt(s, f))}`))
                .join("  ")}
            </p>
          </div>
          <div className="flex items-center gap-2">
            <Button onClick={play} disabled={sounding.length === 0}>
              Strum
            </Button>
            <Button variant="ghost" onClick={() => setFrets([null, null, null, null, null, null])}>
              Clear
            </Button>
          </div>
        </div>
        {names.length > 1 ? (
          <div className="mt-4 flex flex-wrap items-center gap-2">
            <span className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
              also
            </span>
            {names.slice(1, 6).map((n) => (
              <Tag key={n.symbol}>{n.symbol}</Tag>
            ))}
          </div>
        ) : null}
      </div>

      {/* ----------------------------------------------------- the shapes */}
      <div className="bg-background p-6 sm:p-8">
        <Eyebrow>Type a chord</Eyebrow>
        <input
          value={typed}
          onChange={(e) => setTyped(e.target.value)}
          placeholder="Am7, F#m, Bbmaj7, G/B…"
          className="bg-background mt-3 w-full rounded-md border px-3 py-2 font-mono text-lg"
          spellCheck={false}
        />
        <p className="text-muted-foreground mt-2 text-sm">
          {parsed
            ? `${name(parsed.root, typedFlats)} ${parsed.chord.name} · ${parsed.chord.tones.map((t) => name(parsed.root + t, typedFlats)).join(" ")}`
            : typed.trim()
              ? "not a chord I can read"
              : "a root, an accidental if any, and a quality"}
        </p>
        {shapes.length ? (
          <div className="mt-5 grid grid-cols-3 gap-3 sm:grid-cols-4">
            {shapes.map((v) => {
              const key = v.frets.map((f) => (f === null ? "x" : f)).join(",");
              const held = key === frets.map((f) => (f === null ? "x" : f)).join(",");
              return (
                <button
                  key={key}
                  type="button"
                  onClick={() => {
                    setFrets(v.frets);
                    strum(
                      audio(),
                      v.frets
                        .map((f, s) => (f === null ? null : noteAt(s, f)))
                        .filter((n): n is number => n !== null),
                    );
                  }}
                  className={cn(
                    "hover:bg-muted/40 rounded-md border p-2 transition-colors",
                    held && "border-foreground",
                  )}
                  title="Put this shape on the neck and strum it"
                >
                  <ChordBox
                    frets={v.frets}
                    label={v.frets.map((f) => (f === null ? "x" : f)).join("")}
                    className="w-full"
                  />
                </button>
              );
            })}
          </div>
        ) : parsed ? (
          <p className="text-muted-foreground mt-5 text-sm">
            No shape within twelve frets with the root in the bass.
          </p>
        ) : null}
        <p className="text-muted-foreground mt-4 text-xs">
          Standard tuning, {STANDARD.length} strings. Shapes keep the root in the bass and the fingers within
          three frets; the open ones come first.
        </p>
      </div>
    </div>
  );
}
