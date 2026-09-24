"use client";

// A metronome with a memory. Clicks are scheduled on the audio clock a
// tenth of a second ahead, so they land exactly where they should whatever
// the page is doing; the lights follow the same clock. Sessions of half a
// minute or more are logged, in this browser only.

import { useCallback, useEffect, useRef, useState } from "react";

import { Eyebrow } from "@/components/kit";
import { Button } from "@/components/ui/button";
import { cn } from "@/lib/utils";

const MIN = 30;
const MAX = 250;
const LOOKAHEAD = 0.1;
const TICK_MS = 25;
const LOG_KEY = "playground.metronome.log";

type Session = { at: string; bpm: number; seconds: number };

function readLog(): Session[] {
  try {
    const raw = localStorage.getItem(LOG_KEY);
    const parsed: unknown = raw ? JSON.parse(raw) : [];
    return Array.isArray(parsed) ? (parsed as Session[]).slice(0, 20) : [];
  } catch {
    return [];
  }
}
function writeLog(log: Session[]) {
  try {
    localStorage.setItem(LOG_KEY, JSON.stringify(log.slice(0, 20)));
  } catch {
    // Storage blocked: the log is simply not kept.
  }
}

/** One click: a short sine with a sharp edge; higher and louder on the accent. */
function click(ctx: AudioContext, at: number, kind: "accent" | "beat" | "sub") {
  const osc = ctx.createOscillator();
  const gain = ctx.createGain();
  osc.type = "sine";
  osc.frequency.value = kind === "accent" ? 1600 : kind === "beat" ? 1100 : 800;
  const peak = kind === "accent" ? 0.9 : kind === "beat" ? 0.6 : 0.25;
  gain.gain.setValueAtTime(0.0001, at);
  gain.gain.exponentialRampToValueAtTime(peak, at + 0.002);
  gain.gain.exponentialRampToValueAtTime(0.0001, at + (kind === "sub" ? 0.03 : 0.05));
  osc.connect(gain).connect(ctx.destination);
  osc.start(at);
  osc.stop(at + 0.06);
}

export function Metronome() {
  const [bpm, setBpm] = useState(100);
  const [beats, setBeats] = useState(4);
  const [sub, setSub] = useState(1);
  const [running, setRunning] = useState(false);
  const [beat, setBeat] = useState(-1);
  const [log, setLog] = useState<Session[]>([]);
  const ctx = useRef<AudioContext | null>(null);
  const timer = useRef<number | null>(null);
  const nextTime = useRef(0);
  const nextIndex = useRef(0);
  const startedAt = useRef(0);
  const taps = useRef<number[]>([]);
  const live = useRef({ bpm, beats, sub });
  useEffect(() => {
    live.current = { bpm, beats, sub };
  }, [bpm, beats, sub]);
  useEffect(() => setLog(readLog()), []);

  const schedule = useCallback(() => {
    const audio = ctx.current;
    if (!audio) return;
    const { bpm: tempo, beats: perBar, sub: div } = live.current;
    const step = 60 / tempo / div;
    while (nextTime.current < audio.currentTime + LOOKAHEAD) {
      const i = nextIndex.current;
      const onBeat = i % div === 0;
      const beatInBar = Math.floor(i / div) % perBar;
      click(audio, nextTime.current, onBeat ? (beatInBar === 0 ? "accent" : "beat") : "sub");
      if (onBeat) {
        const delay = Math.max(0, (nextTime.current - audio.currentTime) * 1000);
        window.setTimeout(() => setBeat(beatInBar), delay);
      }
      nextTime.current += step;
      nextIndex.current += 1;
    }
  }, []);

  const stop = useCallback(() => {
    if (timer.current !== null) window.clearInterval(timer.current);
    timer.current = null;
    setRunning(false);
    setBeat(-1);
    const seconds = Math.round((performance.now() - startedAt.current) / 1000);
    if (seconds >= 30) {
      const next = [{ at: new Date().toISOString(), bpm: live.current.bpm, seconds }, ...readLog()];
      writeLog(next);
      setLog(next.slice(0, 20));
    }
  }, []);

  const start = () => {
    if (!ctx.current) ctx.current = new AudioContext();
    void ctx.current.resume();
    nextTime.current = ctx.current.currentTime + 0.05;
    nextIndex.current = 0;
    startedAt.current = performance.now();
    schedule();
    timer.current = window.setInterval(schedule, TICK_MS);
    setRunning(true);
  };

  useEffect(
    () => () => {
      if (timer.current !== null) window.clearInterval(timer.current);
      void ctx.current?.close();
    },
    [],
  );

  const tap = () => {
    const now = performance.now();
    taps.current = [...taps.current.filter((t) => now - t < 3000), now];
    if (taps.current.length >= 2) {
      const gaps = taps.current.slice(1).map((t, i) => t - taps.current[i]!);
      const avg = gaps.reduce((a, b) => a + b, 0) / gaps.length;
      setBpm(Math.max(MIN, Math.min(MAX, Math.round(60000 / avg))));
    }
  };

  const nudge = (d: number) => setBpm((v) => Math.max(MIN, Math.min(MAX, v + d)));
  const total = log.reduce((s, x) => s + x.seconds, 0);

  return (
    <div className="bg-border/70 grid gap-px border-y sm:grid-cols-[minmax(0,3fr)_minmax(0,2fr)]">
      {/* ------------------------------------------------------ the tempo */}
      <div className="bg-background p-6 sm:p-8">
        <div className="flex items-baseline justify-between gap-4">
          <Eyebrow>Tempo</Eyebrow>
          <span className="text-muted-foreground flex items-center gap-2 font-mono text-[11px] uppercase">
            <span
              className={cn("size-2 rounded-full", running ? "bg-emerald-500" : "bg-muted-foreground/40")}
            />
            {running ? "running" : "stopped"}
          </span>
        </div>
        <div className="mt-6 flex items-end gap-4">
          <div
            className="font-semibold tracking-[-0.04em] tabular-nums"
            style={{ fontSize: "clamp(4rem, 14vw, 7rem)", lineHeight: 1 }}
          >
            {bpm}
          </div>
          <div className="text-muted-foreground mb-3 font-mono text-[11px] tracking-[0.18em] uppercase">
            bpm
          </div>
        </div>
        <input
          type="range"
          min={MIN}
          max={MAX}
          value={bpm}
          onChange={(e) => setBpm(Number(e.target.value))}
          className="accent-foreground mt-4 w-full"
          aria-label="Tempo"
        />
        <div className="mt-3 flex flex-wrap gap-2">
          {[-10, -5, -1, 1, 5, 10].map((d) => (
            <Button key={d} variant="outline" size="sm" onClick={() => nudge(d)}>
              {d > 0 ? `+${d}` : d}
            </Button>
          ))}
          <Button variant="outline" size="sm" onClick={tap}>
            Tap tempo
          </Button>
        </div>

        {/* the lights */}
        <div className="mt-8 flex items-center gap-3">
          {Array.from({ length: beats }, (_, i) => (
            <span
              key={i}
              className={cn(
                "size-5 rounded-full border transition-colors duration-75",
                beat === i
                  ? i === 0
                    ? "bg-foreground border-foreground"
                    : "border-emerald-500 bg-emerald-500"
                  : "bg-transparent",
              )}
            />
          ))}
        </div>

        <div className="mt-8 flex flex-wrap items-center gap-3">
          {running ? (
            <Button variant="outline" onClick={stop}>
              Stop
            </Button>
          ) : (
            <Button onClick={start}>Start</Button>
          )}
        </div>
      </div>

      {/* --------------------------------------------- the meter and the log */}
      <div className="bg-background p-6 sm:p-8">
        <Eyebrow>Beats per bar</Eyebrow>
        <div className="mt-3 flex flex-wrap gap-2">
          {[2, 3, 4, 5, 6, 7].map((n) => (
            <Button
              key={n}
              variant={beats === n ? "default" : "outline"}
              size="sm"
              onClick={() => setBeats(n)}
            >
              {n}
            </Button>
          ))}
        </div>
        <Eyebrow className="mt-6">Subdivision</Eyebrow>
        <div className="mt-3 flex flex-wrap gap-2">
          {(
            [
              [1, "quarters"],
              [2, "eighths"],
              [3, "triplets"],
              [4, "sixteenths"],
            ] as const
          ).map(([n, label]) => (
            <Button key={n} variant={sub === n ? "default" : "outline"} size="sm" onClick={() => setSub(n)}>
              {label}
            </Button>
          ))}
        </div>

        <div className="mt-8 flex items-baseline justify-between gap-4">
          <Eyebrow>Practice log</Eyebrow>
          {log.length ? (
            <span className="text-muted-foreground font-mono text-[11px] uppercase">
              {Math.round(total / 60)} min in {log.length} sessions
            </span>
          ) : null}
        </div>
        {log.length ? (
          <ol className="mt-3 divide-y border-y text-sm">
            {log.slice(0, 8).map((s) => (
              <li key={s.at} className="flex items-center justify-between py-2">
                <span className="text-muted-foreground tabular-nums">
                  {new Date(s.at).toLocaleDateString(undefined, { day: "numeric", month: "short" })}{" "}
                  {new Date(s.at).toLocaleTimeString(undefined, { hour: "2-digit", minute: "2-digit" })}
                </span>
                <span className="font-mono tabular-nums">
                  {s.bpm} bpm · {Math.floor(s.seconds / 60)}:{String(s.seconds % 60).padStart(2, "0")}
                </span>
              </li>
            ))}
          </ol>
        ) : (
          <p className="text-muted-foreground mt-3 text-sm">
            A session of half a minute or more is noted here, in this browser only.
          </p>
        )}
        {log.length ? (
          <Button
            variant="ghost"
            size="sm"
            className="mt-3"
            onClick={() => {
              writeLog([]);
              setLog([]);
            }}
          >
            Clear the log
          </Button>
        ) : null}
      </div>
    </div>
  );
}
