"use client";

// A live spectrogram: what the microphone hears, drawn as it goes. Each
// frame is one column, frequency up the side on a log scale so an octave
// is the same height everywhere, brightness for loudness. The fundamental
// the pitch detector finds is marked and named, with its harmonics above.

import { useCallback, useEffect, useRef, useState } from "react";

import { Eyebrow } from "@/components/kit";
import { MicPanel, MicStatus } from "@/components/playgrounds/mic-panel";
import { useMic } from "@/components/playgrounds/use-mic";
import { Button } from "@/components/ui/button";
import { detectPitch, noteOf } from "@/lib/music/pitch";
import { cn } from "@/lib/utils";

const RANGES = {
  voice: { lo: 70, hi: 1500, label: "Voice and guitar" },
  wide: { lo: 40, hi: 8000, label: "Everything" },
} as const;
type Range = keyof typeof RANGES;

const WIDTH = 960;
const HEIGHT = 360;

/** A colour ramp from near-black through violet and amber to white. */
function colour(v: number): [number, number, number] {
  const t = Math.max(0, Math.min(1, v));
  if (t < 0.35) {
    const k = t / 0.35;
    return [12 + 70 * k, 10 + 10 * k, 24 + 120 * k];
  }
  if (t < 0.7) {
    const k = (t - 0.35) / 0.35;
    return [82 + 160 * k, 20 + 110 * k, 144 - 100 * k];
  }
  const k = (t - 0.7) / 0.3;
  return [242 + 13 * k, 130 + 125 * k, 44 + 211 * k];
}
const LUT = Array.from({ length: 256 }, (_, i) => colour(i / 255));

export function Spectrogram() {
  const canvas = useRef<HTMLCanvasElement>(null);
  const [range, setRange] = useState<Range>("voice");
  const [frozen, setFrozen] = useState(false);
  const [note, setNote] = useState<{ name: string; octave: number; freq: number; cents: number } | null>(
    null,
  );
  const live = useRef({ range, frozen });
  useEffect(() => {
    live.current = { range, frozen };
  }, [range, frozen]);
  const bins = useRef<Uint8Array<ArrayBuffer> | null>(null);
  const lastNote = useRef(0);

  const onFrame = useCallback((buf: Float32Array, sampleRate: number, analyser: AnalyserNode) => {
    const el = canvas.current;
    if (!el || live.current.frozen) return;
    const g = el.getContext("2d");
    if (!g) return;
    if (!bins.current || bins.current.length !== analyser.frequencyBinCount) {
      bins.current = new Uint8Array(new ArrayBuffer(analyser.frequencyBinCount));
    }
    analyser.getByteFrequencyData(bins.current);
    const { lo, hi } = RANGES[live.current.range];
    const hzPerBin = sampleRate / analyser.fftSize;

    // Scroll by one column, then draw the new one at the right edge.
    g.drawImage(el, -1, 0);
    const column = g.createImageData(1, HEIGHT);
    for (let y = 0; y < HEIGHT; y += 1) {
      const f = lo * (hi / lo) ** (1 - y / HEIGHT);
      const bin = f / hzPerBin;
      const i = Math.floor(bin);
      const frac = bin - i;
      const a = bins.current[i] ?? 0;
      const b = bins.current[i + 1] ?? a;
      const v = (a + (b - a) * frac) / 255;
      // A little gamma so quiet detail is visible without the loud parts blowing out.
      const [r, gg, bb] = LUT[Math.floor(v ** 0.7 * 255)]!;
      const o = y * 4;
      column.data[o] = r;
      column.data[o + 1] = gg;
      column.data[o + 2] = bb;
      column.data[o + 3] = 255;
    }
    g.putImageData(column, WIDTH - 1, 0);

    const pitch = detectPitch(buf, sampleRate, 0.0015, 50, 2000);
    const yOf = (f: number) => HEIGHT * (1 - Math.log(f / lo) / Math.log(hi / lo));
    if (pitch && pitch.clarity > 0.8) {
      for (let h = 1; h <= 6; h += 1) {
        const f = pitch.freq * h;
        if (f < lo || f > hi) continue;
        g.fillStyle = h === 1 ? "rgba(255,255,255,0.95)" : "rgba(255,255,255,0.35)";
        g.fillRect(WIDTH - 2, Math.round(yOf(f)) - 1, 2, 2);
      }
      const now = performance.now();
      if (now - lastNote.current > 80) {
        lastNote.current = now;
        const n = noteOf(pitch.freq);
        setNote({ name: n.name, octave: n.octave, freq: pitch.freq, cents: n.cents });
      }
    } else if (performance.now() - lastNote.current > 600) {
      setNote(null);
    }
  }, []);
  const mic = useMic(onFrame);

  // A clean dark field to start from, whatever the theme.
  useEffect(() => {
    const g = canvas.current?.getContext("2d");
    if (!g) return;
    g.fillStyle = "rgb(12,10,24)";
    g.fillRect(0, 0, WIDTH, HEIGHT);
  }, [range]);

  const { lo, hi } = RANGES[range];
  const ticks = [50, 100, 200, 500, 1000, 2000, 5000].filter((f) => f >= lo && f <= hi);

  return (
    <div className="bg-border/70 grid gap-px border-y">
      <div className="bg-background p-6 sm:p-8">
        <div className="flex items-baseline justify-between gap-4">
          <Eyebrow>Spectrogram</Eyebrow>
          <MicStatus
            mic={mic}
            hearing={note ? `${note.name}${note.octave} · ${note.freq.toFixed(0)} Hz` : null}
          />
        </div>

        <div
          className="relative mt-6 overflow-hidden rounded-md border"
          style={{ background: "rgb(12,10,24)" }}
        >
          <canvas
            ref={canvas}
            width={WIDTH}
            height={HEIGHT}
            className="block h-auto w-full"
            style={{ imageRendering: "pixelated" }}
          />
          {ticks.map((f) => (
            <span
              key={f}
              className="pointer-events-none absolute left-2 -translate-y-1/2 font-mono text-[10px] text-white/50"
              style={{ top: `${(1 - Math.log(f / lo) / Math.log(hi / lo)) * 100}%` }}
            >
              {f >= 1000 ? `${f / 1000}k` : f}
            </span>
          ))}
          {note ? (
            <span
              className="pointer-events-none absolute right-2 -translate-y-1/2 rounded bg-white/90 px-1.5 py-0.5 font-mono text-[11px] text-black"
              style={{ top: `${(1 - Math.log(note.freq / lo) / Math.log(hi / lo)) * 100}%` }}
            >
              {note.name}
              {note.octave} {note.cents > 0 ? "+" : ""}
              {note.cents}¢
            </span>
          ) : null}
        </div>

        <MicPanel mic={mic}>
          <div className="flex gap-2">
            {(Object.keys(RANGES) as Range[]).map((r) => (
              <button
                key={r}
                type="button"
                onClick={() => setRange(r)}
                className={cn(
                  "rounded-full border px-3 py-1 text-xs transition-colors",
                  range === r ? "bg-foreground text-background border-foreground" : "hover:bg-muted/40",
                )}
              >
                {RANGES[r].label}
              </button>
            ))}
          </div>
          {mic.status === "listening" ? (
            <Button variant="ghost" onClick={() => setFrozen((f) => !f)}>
              {frozen ? "Resume" : "Freeze"}
            </Button>
          ) : null}
        </MicPanel>
      </div>
    </div>
  );
}
