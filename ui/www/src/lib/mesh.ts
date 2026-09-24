/**
 * The arithmetic behind the mesh fabric (`mesh-fabric.tsx`, behind the hero
 * and the footer): a 72px cell, traces that run along the grid or at 45°,
 * and the segments of a set of traces with each drawn once.
 */
export const CELL = 72;

export type Cell = readonly [number, number];
export type Pt = readonly [number, number];
export type Bend = "first" | "last";

export const px = ([c, r]: Cell): Pt => [c * CELL, r * CELL];

/**
 * The points of a trace from one cell to another using only horizontal,
 * vertical and 45° segments, the diagonal taken first or last.
 */
export function trace(a: Cell, b: Cell, diagonal: Bend = "last"): Pt[] {
  const [ax, ay] = px(a);
  const [bx, by] = px(b);
  const dx = bx - ax;
  const dy = by - ay;
  const d = Math.min(Math.abs(dx), Math.abs(dy));
  if (d === 0 || Math.abs(dx) === Math.abs(dy))
    return [
      [ax, ay],
      [bx, by],
    ];
  const sx = Math.sign(dx);
  const sy = Math.sign(dy);
  const straight: Pt =
    Math.abs(dx) > Math.abs(dy) ? [sx * (Math.abs(dx) - d), 0] : [0, sy * (Math.abs(dy) - d)];
  const diag: Pt = [sx * d, sy * d];
  const [first] = diagonal === "first" ? [diag] : [straight];
  return [
    [ax, ay],
    [ax + first[0], ay + first[1]],
    [bx, by],
  ];
}

export const path = (pts: Pt[]) => pts.map(([x, y], i) => `${i === 0 ? "M" : "L"}${x} ${y}`).join(" ");

/** Several traces joined end to end, for a packet to travel along. */
export const route = (...legs: Pt[][]): string => path(legs.flatMap((l, i) => (i === 0 ? l : l.slice(1))));

/** The distinct segments of a set of traces, each drawn once. */
export function segments(traces: Pt[][]): string[] {
  const seen = new Map<string, string>();
  for (const t of traces) {
    for (let i = 1; i < t.length; i++) {
      const [p, q] = [t[i - 1]!, t[i]!].sort((u, v) => u[0] - v[0] || u[1] - v[1]);
      seen.set(`${p}|${q}`, path([p!, q!]));
    }
  }
  return [...seen.values()];
}
