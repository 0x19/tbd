"use client";

// The fretboard trainer. Two ways round: it names a note and a string and
// the microphone says whether you played it; or it marks a spot on the
// neck and you name it. Misses are counted per note, so the notes you keep
// getting wrong surface on their own. Nothing is stored or sent.

import { useCallback, useEffect, useRef, useState } from "react";

import { Eyebrow, Tag } from "@/components/kit";
import { Fretboard } from "@/components/playgrounds/fretboard";
import { MicPanel, MicStatus } from "@/components/playgrounds/mic-panel";
import { useMic } from "@/components/playgrounds/use-mic";
import { Button } from "@/components/ui/button";
import { detectPitch, noteOf } from "@/lib/music/pitch";
import { name, noteAt, octave, pc, SHARP } from "@/lib/music/theory";
import { cn } from "@/lib/utils";

type Mode = "play" | "name";
type Question = { string: number; fret: number };
type Verdict = { ok: boolean; heard?: string } | null;

const STRING_NAMES = ["E", "A", "D", "G", "B", "e"];
const ORDINAL = ["6th", "5th", "4th", "3rd", "2nd", "1st"];
/** Consecutive frames a note must hold to count: a real note, not a passing one. */
const HOLD = 4;

function ask(strings: number[], maxFret: number, last: Question | null): Question {
  for (let i = 0; i < 20; i += 1) {
    const string = strings[Math.floor(Math.random() * strings.length)]!;
    const fret = Math.floor(Math.random() * (maxFret + 1));
    if (!last || last.string !== string || last.fret !== fret) return { string, fret };
  }
  return { string: strings[0]!, fret: 0 };
}

export function Trainer() {
  const [mode, setMode] = useState<Mode>("play");
  const [strings, setStrings] = useState<number[]>([0, 1, 2, 3, 4, 5]);
  const [maxFret, setMaxFret] = useState(12);
  const [flats, setFlats] = useState(false);
  const [q, setQ] = useState<Question>(() => ask([0, 1, 2, 3, 4, 5], 12, null));
  const [verdict, setVerdict] = useState<Verdict>(null);
  const [score, setScore] = useState({ right: 0, wrong: 0, streak: 0, best: 0 });
  const [misses, setMisses] = useState<Record<string, number>>({});
  const held = useRef<{ midi: number; frames: number } | null>(null);
  const settled = useRef(false);
  const live = useRef({ q, mode });
  useEffect(() => {
    live.current = { q, mode };
  }, [q, mode]);

  const target = noteAt(q.string, q.fret);

  const next = useCallback(() => {
    setQ((last) => ask(strings, maxFret, last));
    setVerdict(null);
    held.current = null;
    settled.current = false;
  }, [strings, maxFret]);

  // A new setting makes a new question, so the old one cannot be off the board.
  useEffect(() => {
    setQ((last) =>
      strings.includes(last.string) && last.fret <= maxFret ? last : ask(strings, maxFret, null),
    );
  }, [strings, maxFret]);

  const judge = useCallback(
    (ok: boolean, heard?: string) => {
      settled.current = true;
      setVerdict({ ok, heard });
      setScore((s) => ({
        right: s.right + Number(ok),
        wrong: s.wrong + Number(!ok),
        streak: ok ? s.streak + 1 : 0,
        best: ok ? Math.max(s.best, s.streak + 1) : s.best,
      }));
      if (!ok) {
        const key = name(noteAt(live.current.q.string, live.current.q.fret), flats);
        setMisses((m) => ({ ...m, [key]: (m[key] ?? 0) + 1 }));
      }
    },
    [flats],
  );

  // After a verdict, move on by itself; a miss lingers long enough to read.
  useEffect(() => {
    if (!verdict) return;
    const t = setTimeout(next, verdict.ok ? 900 : 2200);
    return () => clearTimeout(t);
  }, [verdict, next]);

  const onFrame = useCallback(
    (buf: Float32Array, sampleRate: number) => {
      if (live.current.mode !== "play" || settled.current) return;
      const pitch = detectPitch(buf, sampleRate);
      if (!pitch || pitch.clarity < 0.75) {
        held.current = null;
        return;
      }
      const n = noteOf(pitch.freq);
      if (Math.abs(n.cents) > 40) return;
      if (held.current?.midi === n.midi) held.current.frames += 1;
      else held.current = { midi: n.midi, frames: 1 };
      if (held.current.frames < HOLD) return;
      const want = noteAt(live.current.q.string, live.current.q.fret);
      if (n.midi === want) judge(true);
      else if (held.current.frames >= HOLD * 3) judge(false, `${name(n.midi, flats)}${octave(n.midi)}`);
    },
    [judge, flats],
  );
  const mic = useMic(onFrame);

  const toggleString = (s: number) =>
    setStrings((cur) => {
      const nextSet = cur.includes(s) ? cur.filter((x) => x !== s) : [...cur, s].sort();
      return nextSet.length ? nextSet : cur;
    });

  const worst = Object.entries(misses)
    .sort((a, b) => b[1] - a[1])
    .slice(0, 5);

  return (
    <div className="bg-border/70 grid gap-px border-y sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      {/* ---------------------------------------------------- the question */}
      <div className="bg-background p-6 sm:p-8">
        <div className="flex items-baseline justify-between gap-4">
          <Eyebrow>{mode === "play" ? "Play this" : "Name this"}</Eyebrow>
          {mode === "play" ? (
            <MicStatus mic={mic} hearing={verdict ? (verdict.ok ? "yes" : "no") : null} />
          ) : null}
        </div>

        {mode === "play" ? (
          <div className="mt-8">
            <div
              className={cn(
                "font-semibold tracking-[-0.04em]",
                verdict?.ok && "text-emerald-600 dark:text-emerald-400",
                verdict && !verdict.ok && "text-destructive",
              )}
              style={{ fontSize: "clamp(3rem, 12vw, 6rem)", lineHeight: 1 }}
            >
              {name(target, flats)}
              <span className="text-muted-foreground ml-1 text-2xl font-normal">{octave(target)}</span>
            </div>
            <p className="text-muted-foreground mt-3 text-lg">
              on the <span className="text-foreground">{ORDINAL[q.string]}</span> string
              <span className="text-muted-foreground/70"> · the {STRING_NAMES[q.string]} string</span>
            </p>
            <p className="mt-2 h-6 text-sm">
              {verdict?.ok ? (
                <span className="text-emerald-600 dark:text-emerald-400">That is it.</span>
              ) : verdict ? (
                <span className="text-destructive">
                  That was {verdict.heard}. {name(target, flats)} is fret {q.fret} on that string.
                </span>
              ) : mic.status === "listening" ? (
                <span className="text-muted-foreground">Play it and let it ring.</span>
              ) : (
                <span className="text-muted-foreground">Start listening, then play the note.</span>
              )}
            </p>
            <div className="mt-6">
              <Fretboard
                frets={maxFret}
                dots={
                  verdict
                    ? [
                        {
                          string: q.string,
                          fret: q.fret,
                          label: name(target, flats),
                          tone: verdict.ok ? "good" : "bad",
                        },
                      ]
                    : []
                }
              />
            </div>
            <MicPanel mic={mic}>
              <Button variant="ghost" onClick={next}>
                Skip
              </Button>
            </MicPanel>
          </div>
        ) : (
          <div className="mt-6">
            <Fretboard
              frets={maxFret}
              dots={[
                {
                  string: q.string,
                  fret: q.fret,
                  label: verdict ? name(target, flats) : "?",
                  tone: verdict ? (verdict.ok ? "good" : "bad") : "aim",
                },
              ]}
            />
            <p className="text-muted-foreground mt-4 text-sm">
              {verdict?.ok ? (
                <span className="text-emerald-600 dark:text-emerald-400">Yes, {name(target, flats)}.</span>
              ) : verdict ? (
                <span className="text-destructive">No: that is {name(target, flats)}.</span>
              ) : (
                `Which note is the dot? ${ORDINAL[q.string]} string, fret ${q.fret}.`
              )}
            </p>
            <div className="mt-4 grid grid-cols-6 gap-2 sm:grid-cols-12">
              {SHARP.map((_, i) => (
                <Button
                  key={i}
                  variant={verdict && pc(target) === i ? "default" : "outline"}
                  size="sm"
                  disabled={verdict !== null}
                  onClick={() => judge(pc(target) === i)}
                >
                  {name(i, flats)}
                </Button>
              ))}
            </div>
          </div>
        )}
      </div>

      {/* ---------------------------------------------------- the settings */}
      <div className="bg-background p-6 sm:p-8">
        <Eyebrow>Score</Eyebrow>
        <div className="mt-4 grid grid-cols-3 gap-4">
          {[
            ["right", score.right],
            ["streak", score.streak],
            ["best", score.best],
          ].map(([k, v]) => (
            <div key={k}>
              <div className="text-3xl font-semibold tracking-tight tabular-nums">{v}</div>
              <div className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
                {k}
              </div>
            </div>
          ))}
        </div>
        {worst.length ? (
          <div className="mt-5">
            <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
              Keep missing
            </p>
            <div className="mt-2 flex flex-wrap gap-2">
              {worst.map(([n, c]) => (
                <Tag key={n}>
                  {n} ×{c}
                </Tag>
              ))}
            </div>
          </div>
        ) : null}
        {score.right + score.wrong > 0 ? (
          <Button
            variant="ghost"
            size="sm"
            className="mt-3"
            onClick={() => {
              setScore({ right: 0, wrong: 0, streak: 0, best: 0 });
              setMisses({});
            }}
          >
            Reset
          </Button>
        ) : null}

        <div className="mt-8">
          <Eyebrow>Mode</Eyebrow>
          <div className="mt-3 flex gap-2">
            {(
              [
                ["play", "Play the note"],
                ["name", "Name the note"],
              ] as const
            ).map(([id, label]) => (
              <button
                key={id}
                type="button"
                onClick={() => {
                  setMode(id);
                  next();
                }}
                className={cn(
                  "rounded-full border px-3 py-1 text-xs transition-colors",
                  mode === id ? "bg-foreground text-background border-foreground" : "hover:bg-muted/40",
                )}
              >
                {label}
              </button>
            ))}
          </div>
        </div>

        <div className="mt-6">
          <Eyebrow>Strings</Eyebrow>
          <div className="mt-3 flex gap-2">
            {STRING_NAMES.map((n, s) => (
              <button
                key={s}
                type="button"
                onClick={() => toggleString(s)}
                className={cn(
                  "size-9 rounded-full border font-mono text-sm transition-colors",
                  strings.includes(s)
                    ? "bg-foreground text-background border-foreground"
                    : "text-muted-foreground hover:bg-muted/40",
                )}
                title={`${ORDINAL[s]} string`}
              >
                {n}
              </button>
            ))}
          </div>
        </div>

        <div className="mt-6 flex items-center justify-between gap-4">
          <div>
            <Eyebrow>Frets</Eyebrow>
            <p className="text-muted-foreground mt-1 text-sm">
              up to <span className="text-foreground font-mono tabular-nums">{maxFret}</span>
            </p>
          </div>
          <div className="flex items-center gap-1">
            {[5, 12, 15].map((f) => (
              <Button
                key={f}
                variant={maxFret === f ? "default" : "outline"}
                size="sm"
                onClick={() => setMaxFret(f)}
              >
                {f}
              </Button>
            ))}
          </div>
        </div>

        <div className="mt-6 flex items-center justify-between gap-4">
          <Eyebrow>Names</Eyebrow>
          <div className="flex items-center gap-1">
            <Button variant={flats ? "outline" : "default"} size="sm" onClick={() => setFlats(false)}>
              ♯
            </Button>
            <Button variant={flats ? "default" : "outline"} size="sm" onClick={() => setFlats(true)}>
              ♭
            </Button>
          </div>
        </div>
      </div>
    </div>
  );
}
