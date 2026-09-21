"use client";

// A neck, drawn: six strings, the frets, the inlays, and dots where the
// playground says. Hairlines and text, no library. `onTap` makes it an
// input: the chord namer sets a fret per string by tapping.

import { cn } from "@/lib/utils";

export type Dot = {
  string: number;
  fret: number;
  label?: string;
  tone?: "aim" | "good" | "bad" | "muted" | "plain";
};

const TONE: Record<NonNullable<Dot["tone"]>, string> = {
  aim: "fill-foreground",
  good: "fill-emerald-500",
  bad: "fill-destructive",
  muted: "fill-muted-foreground/40",
  plain: "fill-muted-foreground",
};

/** Strings are 0 = low E at the bottom, as on a neck seen from above. */
export function Fretboard({
  frets = 12,
  dots = [],
  onTap,
  className,
}: {
  frets?: number;
  dots?: Dot[];
  onTap?: (string: number, fret: number) => void;
  className?: string;
}) {
  const w = 900;
  const h = 200;
  const nut = 36;
  const right = 12;
  const top = 24;
  const bottom = 24;
  const gap = (h - top - bottom) / 5;
  // Frets narrow up the neck, as they do.
  const x = (fret: number) => nut + ((w - nut - right) * (1 - 2 ** (-fret / 12))) / (1 - 2 ** (-frets / 12));
  const y = (string: number) => h - bottom - string * gap;
  const mid = (fret: number) => (fret === 0 ? nut / 2 : (x(fret - 1) + x(fret)) / 2);
  return (
    <svg
      viewBox={`0 0 ${w} ${h}`}
      className={cn("w-full select-none", className)}
      role="img"
      aria-label="Fretboard"
    >
      {/* the inlays */}
      {[3, 5, 7, 9, 15, 17, 19, 21]
        .filter((f) => f <= frets)
        .map((f) => (
          <circle key={f} cx={mid(f)} cy={h / 2} r={5} className="fill-muted-foreground/15" />
        ))}
      {frets >= 12 ? (
        <>
          <circle cx={mid(12)} cy={y(1) + gap / 2} r={5} className="fill-muted-foreground/15" />
          <circle cx={mid(12)} cy={y(3) + gap / 2} r={5} className="fill-muted-foreground/15" />
        </>
      ) : null}
      {/* the nut and the frets */}
      <rect x={nut - 3} y={top - 4} width={4} height={h - top - bottom + 8} className="fill-foreground/70" />
      {Array.from({ length: frets }, (_, i) => i + 1).map((f) => (
        <line
          key={f}
          x1={x(f)}
          x2={x(f)}
          y1={top - 4}
          y2={h - bottom + 4}
          className="stroke-border"
          strokeWidth={1.5}
        />
      ))}
      {/* the strings, thicker low */}
      {[0, 1, 2, 3, 4, 5].map((s) => (
        <line
          key={s}
          x1={nut - 3}
          x2={w - right}
          y1={y(s)}
          y2={y(s)}
          className="stroke-foreground/60"
          strokeWidth={2.2 - s * 0.3}
        />
      ))}
      {/* fret numbers */}
      {Array.from({ length: frets }, (_, i) => i + 1).map((f) => (
        <text
          key={f}
          x={mid(f)}
          y={h - 6}
          textAnchor="middle"
          className="fill-muted-foreground/60 font-mono text-[10px]"
        >
          {f}
        </text>
      ))}
      {/* tap targets */}
      {onTap
        ? [0, 1, 2, 3, 4, 5].flatMap((s) =>
            Array.from({ length: frets + 1 }, (_, f) => (
              <rect
                key={`${s}-${f}`}
                x={f === 0 ? 0 : x(f - 1)}
                y={y(s) - gap / 2}
                width={f === 0 ? nut : x(f) - x(f - 1)}
                height={gap}
                className="hover:fill-foreground/5 cursor-pointer fill-transparent"
                onClick={() => onTap(s, f)}
              />
            )),
          )
        : null}
      {/* the dots */}
      {dots.map((d, i) => (
        <g key={i} className="pointer-events-none">
          {d.tone === "muted" ? (
            <text
              x={mid(0)}
              y={y(d.string) + 5}
              textAnchor="middle"
              className="fill-muted-foreground font-mono text-[14px]"
            >
              ×
            </text>
          ) : (
            <>
              <circle cx={mid(d.fret)} cy={y(d.string)} r={11} className={TONE[d.tone ?? "plain"]} />
              {d.label ? (
                <text
                  x={mid(d.fret)}
                  y={y(d.string) + 4}
                  textAnchor="middle"
                  className="fill-background font-mono text-[11px] font-medium"
                >
                  {d.label}
                </text>
              ) : null}
            </>
          )}
        </g>
      ))}
    </svg>
  );
}

/** A chord box: the first few frets of a shape, upright, as in a songbook. */
export function ChordBox({
  frets,
  label,
  className,
}: {
  frets: (number | null)[];
  label?: string;
  className?: string;
}) {
  const fretted = frets.filter((f): f is number => f !== null && f > 0);
  const base = fretted.length && Math.min(...fretted) > 1 ? Math.min(...fretted) : 1;
  const rows = 4;
  const w = 80;
  const h = 100;
  const left = 12;
  const top = 22;
  const cw = (w - 2 * left) / 5;
  const rh = (h - top - 10) / rows;
  const x = (s: number) => left + s * cw;
  const y = (row: number) => top + row * rh;
  return (
    <svg viewBox={`0 0 ${w} ${h}`} className={cn("w-20", className)} role="img" aria-label={label ?? "Chord"}>
      {label ? (
        <text
          x={w / 2}
          y={11}
          textAnchor="middle"
          className="fill-foreground font-mono text-[10px] font-medium"
        >
          {label}
        </text>
      ) : null}
      {base === 1 ? (
        <rect x={left - 1} y={top - 3} width={w - 2 * left + 2} height={3} className="fill-foreground" />
      ) : null}
      {base > 1 ? (
        <text x={2} y={y(0) + rh / 2 + 3} className="fill-muted-foreground font-mono text-[8px]">
          {base}
        </text>
      ) : null}
      {Array.from({ length: rows + 1 }, (_, r) => (
        <line key={r} x1={left} x2={w - left} y1={y(r)} y2={y(r)} className="stroke-border" strokeWidth={1} />
      ))}
      {[0, 1, 2, 3, 4, 5].map((s) => (
        <line
          key={s}
          x1={x(s)}
          x2={x(s)}
          y1={y(0)}
          y2={y(rows)}
          className="stroke-foreground/60"
          strokeWidth={1}
        />
      ))}
      {frets.map((f, s) =>
        f === null ? (
          <text
            key={s}
            x={x(s)}
            y={top - 6}
            textAnchor="middle"
            className="fill-muted-foreground font-mono text-[9px]"
          >
            ×
          </text>
        ) : f === 0 ? (
          <circle
            key={s}
            cx={x(s)}
            cy={top - 8}
            r={2.6}
            className="stroke-foreground fill-none"
            strokeWidth={1}
          />
        ) : (
          <circle key={s} cx={x(s)} cy={y(f - base) + rh / 2} r={4.2} className="fill-foreground" />
        ),
      )}
    </svg>
  );
}
