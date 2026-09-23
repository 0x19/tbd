"use client";

import { usePathname } from "next/navigation";

import { Frame } from "@/components/kit";
import { type Bend, CELL, type Cell, px, route, segments, trace } from "@/lib/mesh";

/**
 * The end of the home page's story, behind the footer: the network the reply
 * goes back out into, the hero's mesh grown to the whole width. It is
 * anchored to the bottom of the page and strongest there, fading upward to
 * nothing just under the strip the reply left by. Two hand-laid tiles of
 * nodes alternate across the width, joined at their seams, so it reads as one
 * fabric and never as a pattern; a few dashed stubs leave off the bottom and
 * into the fade above, and three packets run away from the spine. Same 72px
 * grid as the hero, drawn as a pattern from the spine and the page's bottom
 * edge, so every node sits on an intersection at any width.
 *
 * `Closing` wraps the site footer in the layout: on the home page it adds
 * the room under the last strip and this drawing behind both; elsewhere it
 * is the footer alone. Decoration only: `aria-hidden`, no pointer events,
 * packets stop under prefers-reduced-motion (`.mesh-packet`, `site.css`).
 */
export function Closing({ children }: { children: React.ReactNode }) {
  const home = usePathname() === "/";
  if (!home) return <>{children}</>;
  return (
    <div className="relative">
      <div aria-hidden className="h-40 sm:h-56" />
      <div
        aria-hidden
        className="pointer-events-none absolute inset-0 -z-10 overflow-hidden"
        style={{
          maskImage: "linear-gradient(to top, black 0%, black 30%, rgba(0, 0, 0, 0.55) 62%, transparent 97%)",
          WebkitMaskImage:
            "linear-gradient(to top, black 0%, black 30%, rgba(0, 0, 0, 0.55) 62%, transparent 97%)",
        }}
      >
        <Frame className="relative h-full">
          <ClosingMesh />
        </Frame>
      </div>
      {children}
    </div>
  );
}

const ROWS = 8;
const H = ROWS * CELL;
const TILE = 10;
// Tiles either side of the spine: enough for a 2560px screen with the spine
// near its middle.
const TILES = 4;

// Rows count from the page's bottom edge here, so the densest row is the
// lowest: a cell is (col, rowFromBottom).
const at = ([c, r]: Cell): Cell => [c, ROWS - r];

type Tile = {
  nodes: Record<string, Cell>;
  links: [string, string, Bend?][];
  pops: string[];
  stubs: [string, Cell][];
};

// Two tiles of ten columns, laid by hand like the hero's mesh.
const A: Tile = {
  nodes: {
    a: [0, 2],
    b: [2, 4],
    c: [2, 1],
    d: [4, 3],
    e: [5, 5],
    f: [6, 1],
    g: [7, 3],
    h: [9, 4],
    i: [9, 2],
    j: [5, 7],
  },
  links: [
    ["a", "b"],
    ["a", "c"],
    ["b", "d"],
    ["c", "d", "first"],
    ["d", "e"],
    ["d", "g"],
    ["c", "f"],
    ["f", "g"],
    ["g", "h"],
    ["g", "i"],
    ["e", "h"],
    ["e", "j"],
  ],
  pops: ["d", "h"],
  stubs: [
    ["f", [6, 0]],
    ["j", [5, 8]],
    ["h", [9, 6]],
  ],
};
const B: Tile = {
  nodes: {
    a: [0, 3],
    b: [1, 1],
    c: [3, 2],
    d: [3, 5],
    e: [5, 4],
    f: [5, 1],
    g: [7, 2],
    h: [8, 5],
    i: [9, 3],
    j: [9, 1],
    k: [8, 7],
  },
  links: [
    ["a", "b"],
    ["a", "c"],
    ["c", "d"],
    ["c", "e"],
    ["c", "f"],
    ["d", "e"],
    ["e", "g"],
    ["f", "g"],
    ["e", "h"],
    ["g", "i"],
    ["g", "j"],
    ["h", "i", "first"],
    ["h", "k"],
  ],
  pops: ["e", "i"],
  stubs: [
    ["b", [1, 0]],
    ["k", [8, 8]],
    ["d", [3, 7]],
  ],
};
// The seams: which node of one tile reaches which node of the next.
const seams: { from: [Tile, string]; to: [Tile, string]; bend?: Bend }[] = [
  { from: [A, "i"], to: [B, "a"] },
  { from: [A, "h"], to: [B, "a"] },
  { from: [B, "i"], to: [A, "a"] },
  { from: [B, "j"], to: [A, "c"] },
];

const variant = (k: number) => (k % 2 === 0 ? A : B);
const cell = (k: number, t: Tile, n: string): Cell => {
  const [c, r] = t.nodes[n]!;
  return at([c + k * TILE, r]);
};

const solidTraces: ReturnType<typeof trace>[] = [];
const dashedTraces: ReturnType<typeof trace>[] = [];
const nodes: { cell: Cell; pop: boolean }[] = [];
for (let k = -TILES; k <= TILES; k++) {
  const t = variant(k);
  for (const [x, y, bend] of t.links) solidTraces.push(trace(cell(k, t, x), cell(k, t, y), bend));
  for (const [x, to] of t.stubs) dashedTraces.push(trace(cell(k, t, x), at([to[0] + k * TILE, to[1]])));
  for (const n of Object.keys(t.nodes)) nodes.push({ cell: cell(k, t, n), pop: t.pops.includes(n) });
  if (k < TILES) {
    const next = variant(k + 1);
    for (const s of seams) {
      if (s.from[0] !== t || s.to[0] !== next) continue;
      solidTraces.push(trace(cell(k, t, s.from[1]), cell(k + 1, next, s.to[1]), s.bend));
    }
  }
}
const solid = segments(solidTraces);
const dashed = segments(dashedTraces);

// Three packets: right along the fabric, left along it, and one off the bottom.
const T = (k: number, n: string, k2: number, n2: string, bend?: Bend) =>
  trace(cell(k, variant(k), n), cell(k2, variant(k2), n2), bend);
const packets = [
  {
    d: route(
      T(0, "a", 0, "b"),
      T(0, "b", 0, "d"),
      T(0, "d", 0, "g"),
      T(0, "g", 0, "i"),
      T(0, "i", 1, "a"),
      T(1, "a", 1, "c"),
      T(1, "c", 1, "e"),
      T(1, "e", 1, "g"),
      T(1, "g", 1, "i"),
      T(1, "i", 2, "a"),
    ),
    duration: 18,
    delay: 0,
  },
  {
    d: route(
      T(0, "a", -1, "i"),
      T(-1, "i", -1, "g"),
      T(-1, "g", -1, "f"),
      T(-1, "f", -1, "c"),
      T(-1, "c", -1, "a"),
      T(-1, "a", -2, "i"),
      T(-2, "i", -2, "g"),
      T(-2, "g", -2, "d"),
    ),
    duration: 15,
    delay: 5,
  },
  {
    d: route(T(0, "a", 0, "c"), T(0, "c", 0, "f"), trace(cell(0, A, "f"), at([6, 0]))),
    duration: 7,
    delay: 11,
  },
];

function ClosingMesh() {
  return (
    <svg
      aria-hidden
      className="text-foreground absolute bottom-0 left-3 overflow-visible sm:left-4"
      width={1}
      height={H}
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <defs>
        <pattern id="closing-grid" width={CELL} height={CELL} patternUnits="userSpaceOnUse">
          <path d={`M${CELL} 0 H0 V${CELL}`} strokeWidth="1" shapeRendering="crispEdges" />
        </pattern>
      </defs>
      {/* the grid, from the spine and the bottom edge outwards */}
      <rect
        x={-(TILES + 1) * TILE * CELL}
        y={-4 * CELL}
        width={2 * (TILES + 1) * TILE * CELL}
        height={H + 4 * CELL}
        fill="url(#closing-grid)"
        stroke="none"
        className="opacity-[0.06] dark:opacity-[0.09]"
      />

      {/* ------------------------------------------------------- traces */}
      <g strokeWidth="1" className="[stroke-opacity:0.26] dark:[stroke-opacity:0.3]">
        {solid.map((d) => (
          <path key={d} d={d} />
        ))}
      </g>
      <g strokeWidth="1" strokeDasharray="3 5" className="[stroke-opacity:0.22] dark:[stroke-opacity:0.26]">
        {dashed.map((d) => (
          <path key={d} d={d} />
        ))}
      </g>

      {/* ------------------------------------------------------ packets */}
      <g fill="currentColor" stroke="none">
        {packets.map((p) => (
          <circle
            key={p.d}
            r={2.6}
            className="mesh-packet"
            style={{
              offsetPath: `path("${p.d}")`,
              animationDuration: `${p.duration}s`,
              animationDelay: `${p.delay}s`,
            }}
          />
        ))}
      </g>

      {/* -------------------------------------------------------- nodes */}
      <g strokeWidth="1.2" className="[stroke-opacity:0.6] dark:[stroke-opacity:0.65]">
        {nodes
          .filter((n) => n.pop)
          .map((n) => {
            const [x, y] = px(n.cell);
            return (
              <circle
                key={`halo-${x}-${y}`}
                cx={x}
                cy={y}
                r={13}
                strokeDasharray="2 3.5"
                strokeWidth="0.9"
                className="[stroke-opacity:0.35]"
              />
            );
          })}
        {nodes.map((n) => {
          const [x, y] = px(n.cell);
          return <circle key={`${x}-${y}`} cx={x} cy={y} r={n.pop ? 5 : 4} fill="var(--background)" />;
        })}
        {/* the hub on the spine: a dot in the ring, like the hero's */}
        <circle
          cx={px(cell(0, A, "a"))[0]}
          cy={px(cell(0, A, "a"))[1]}
          r={1.6}
          fill="currentColor"
          stroke="none"
        />
      </g>
    </svg>
  );
}
