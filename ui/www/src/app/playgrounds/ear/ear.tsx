"use client";

// The ear trainer: it plays, you name. Intervals one after the other, one
// under the other or together; chords strummed. The score is kept per
// answer, so the intervals that fool you show up as a list. Session only.

import { useCallback, useEffect, useRef, useState } from "react";

import { Eyebrow, Tag } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { pair, strum } from "@/lib/music/play";
import { CHORDS, type ChordType, INTERVALS, name } from "@/lib/music/theory";
import { cn } from "@/lib/utils";

type Mode = "intervals" | "chords";
type Direction = "up" | "down" | "together";

const INTERVAL_POOL = INTERVALS.filter((i) => i.semis > 0);
const CHORD_POOL = CHORDS.filter((c) =>
  ["maj", "min", "dim", "aug", "7", "maj7", "m7", "sus4"].includes(c.id),
);

type Round =
  | { kind: "interval"; root: number; semis: number; direction: Direction }
  | { kind: "chord"; root: number; chord: ChordType };

function randomRoot(lo = 45, hi = 64): number {
  return lo + Math.floor(Math.random() * (hi - lo + 1));
}

export function Ear() {
  const [mode, setMode] = useState<Mode>("intervals");
  const [direction, setDirection] = useState<Direction>("up");
  const [round, setRound] = useState<Round | null>(null);
  const [answered, setAnswered] = useState<string | null>(null);
  const [score, setScore] = useState<Record<string, { right: number; wrong: number }>>({});
  const ctx = useRef<AudioContext | null>(null);
  const audio = () => {
    if (!ctx.current) ctx.current = new AudioContext();
    void ctx.current.resume();
    return ctx.current;
  };
  useEffect(() => () => void ctx.current?.close(), []);

  const play = useCallback((r: Round) => {
    const a = audio();
    if (r.kind === "interval") {
      const other = r.direction === "down" ? r.root - r.semis : r.root + r.semis;
      pair(a, r.root, other, r.direction === "together");
    } else {
      strum(
        a,
        r.chord.tones.map((t) => r.root + t),
      );
    }
  }, []);

  const next = useCallback(() => {
    const r: Round =
      mode === "intervals"
        ? {
            kind: "interval",
            root: randomRoot(direction === "down" ? 52 : 45, direction === "down" ? 69 : 60),
            semis: INTERVAL_POOL[Math.floor(Math.random() * INTERVAL_POOL.length)]!.semis,
            direction,
          }
        : {
            kind: "chord",
            root: randomRoot(45, 57),
            chord: CHORD_POOL[Math.floor(Math.random() * CHORD_POOL.length)]!,
          };
    setRound(r);
    setAnswered(null);
    play(r);
  }, [mode, direction, play]);

  const answer = (id: string) => {
    if (!round || answered) return;
    const truth = round.kind === "interval" ? String(round.semis) : round.chord.id;
    const label =
      round.kind === "interval" ? INTERVALS.find((i) => i.semis === round.semis)!.short : round.chord.id;
    setAnswered(id);
    setScore((s) => {
      const cur = s[label] ?? { right: 0, wrong: 0 };
      return {
        ...s,
        [label]: id === truth ? { ...cur, right: cur.right + 1 } : { ...cur, wrong: cur.wrong + 1 },
      };
    });
  };

  const truth = round ? (round.kind === "interval" ? String(round.semis) : round.chord.id) : null;
  const totals = Object.values(score).reduce(
    (a, s) => ({ right: a.right + s.right, wrong: a.wrong + s.wrong }),
    { right: 0, wrong: 0 },
  );
  const weak = Object.entries(score)
    .filter(([, s]) => s.wrong > 0)
    .sort((a, b) => b[1].wrong - a[1].wrong)
    .slice(0, 5);

  return (
    <div className="bg-border/70 grid gap-px border-y sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      {/* ------------------------------------------------------ the round */}
      <div className="bg-background p-6 sm:p-8">
        <div className="flex items-baseline justify-between gap-4">
          <Eyebrow>{mode === "intervals" ? "Which interval?" : "Which chord?"}</Eyebrow>
          {round ? (
            <span className="text-muted-foreground font-mono text-[11px] uppercase">
              {answered ? (answered === truth ? "right" : "wrong") : "listening to you"}
            </span>
          ) : null}
        </div>

        <div className="mt-6 flex flex-wrap items-center gap-3">
          <Button onClick={next}>{round ? "Next" : "Play one"}</Button>
          {round ? (
            <Button variant="outline" onClick={() => play(round)}>
              Play again
            </Button>
          ) : null}
        </div>

        <div className="mt-6 min-h-8 text-sm">
          {round && answered ? (
            answered === truth ? (
              <span className="text-emerald-600 dark:text-emerald-400">
                Yes:{" "}
                {round.kind === "interval"
                  ? `${INTERVALS.find((i) => i.semis === round.semis)!.name}, ${name(round.root)} to ${name(round.direction === "down" ? round.root - round.semis : round.root + round.semis)}`
                  : `${name(round.root)} ${round.chord.name}`}
                .
              </span>
            ) : (
              <span className="text-destructive">
                No, that was{" "}
                {round.kind === "interval"
                  ? `a ${INTERVALS.find((i) => i.semis === round.semis)!.name}, ${name(round.root)} to ${name(round.direction === "down" ? round.root - round.semis : round.root + round.semis)}`
                  : `${name(round.root)} ${round.chord.name}`}
                .
              </span>
            )
          ) : round ? (
            <span className="text-muted-foreground">Name what you heard.</span>
          ) : (
            <span className="text-muted-foreground">
              Press play. Two notes, or a chord, and you say which.
            </span>
          )}
        </div>

        <div
          className={cn(
            "mt-4 grid gap-2",
            mode === "intervals" ? "grid-cols-3 sm:grid-cols-4" : "grid-cols-2 sm:grid-cols-4",
          )}
        >
          {mode === "intervals"
            ? INTERVAL_POOL.map((i) => (
                <Button
                  key={i.semis}
                  variant={answered && String(i.semis) === truth ? "default" : "outline"}
                  size="sm"
                  disabled={!round || answered !== null}
                  onClick={() => answer(String(i.semis))}
                  className={cn(
                    answered === String(i.semis) &&
                      answered !== truth &&
                      "border-destructive text-destructive",
                  )}
                  title={i.name}
                >
                  <span className="font-mono">{i.short}</span>
                  <span className="text-muted-foreground ml-1 hidden text-xs sm:inline">{i.name}</span>
                </Button>
              ))
            : CHORD_POOL.map((c) => (
                <Button
                  key={c.id}
                  variant={answered && c.id === truth ? "default" : "outline"}
                  size="sm"
                  disabled={!round || answered !== null}
                  onClick={() => answer(c.id)}
                  className={cn(
                    answered === c.id && answered !== truth && "border-destructive text-destructive",
                  )}
                >
                  {c.name}
                </Button>
              ))}
        </div>
      </div>

      {/* --------------------------------------------- the settings and score */}
      <div className="bg-background p-6 sm:p-8">
        <Eyebrow>Score</Eyebrow>
        <div className="mt-4 grid grid-cols-3 gap-4">
          {[
            ["right", totals.right],
            ["wrong", totals.wrong],
            [
              "hit rate",
              totals.right + totals.wrong
                ? `${Math.round((100 * totals.right) / (totals.right + totals.wrong))}%`
                : "—",
            ],
          ].map(([k, v]) => (
            <div key={k}>
              <div className="text-3xl font-semibold tracking-tight tabular-nums">{v}</div>
              <div className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
                {k}
              </div>
            </div>
          ))}
        </div>
        {weak.length ? (
          <div className="mt-5">
            <p className="text-muted-foreground font-mono text-[11px] tracking-[0.18em] uppercase">
              Fool you most
            </p>
            <div className="mt-2 flex flex-wrap gap-2">
              {weak.map(([k, s]) => (
                <Tag key={k}>
                  {k} ×{s.wrong}
                </Tag>
              ))}
            </div>
          </div>
        ) : null}
        {totals.right + totals.wrong > 0 ? (
          <Button variant="ghost" size="sm" className="mt-3" onClick={() => setScore({})}>
            Reset
          </Button>
        ) : null}

        <div className="mt-8">
          <Eyebrow>Mode</Eyebrow>
          <div className="mt-3 flex gap-2">
            {(
              [
                ["intervals", "Intervals"],
                ["chords", "Chords"],
              ] as const
            ).map(([id, label]) => (
              <button
                key={id}
                type="button"
                onClick={() => {
                  setMode(id);
                  setRound(null);
                  setAnswered(null);
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
        {mode === "intervals" ? (
          <div className="mt-6">
            <Eyebrow>Played</Eyebrow>
            <div className="mt-3 flex gap-2">
              {(
                [
                  ["up", "Rising"],
                  ["down", "Falling"],
                  ["together", "Together"],
                ] as const
              ).map(([id, label]) => (
                <button
                  key={id}
                  type="button"
                  onClick={() => setDirection(id)}
                  className={cn(
                    "rounded-full border px-3 py-1 text-xs transition-colors",
                    direction === id
                      ? "bg-foreground text-background border-foreground"
                      : "hover:bg-muted/40",
                  )}
                >
                  {label}
                </button>
              ))}
            </div>
          </div>
        ) : null}
        <p className="text-muted-foreground mt-6 text-xs">
          The notes are the tuner&rsquo;s synthesised strings, somewhere between A2 and E4, so this is a
          guitar ear you are training.
        </p>
      </div>
    </div>
  );
}
