import { type Bend, CELL, type Cell, px, route, segments, trace } from "@/lib/mesh";
import { cn } from "@/lib/utils";

/**
 * The network the site's story begins and ends in: the hero's old mesh grown
 * to the whole width, two hand-laid tiles of nodes alternating across it and
 * joined at their seams, so it reads as one fabric and never as a pattern.
 * It hangs from an edge, the top of the home page or the bottom of every
 * page, densest at that edge, and the box around it fades it out towards
 * the content. Same 72px grid as the spine, drawn as a pattern from the
 * spine and the anchored edge, so every node sits on an intersection at any
 * width. Kept faint on purpose, and alive in three quiet ways: packets run
 * away from the spine, rings ripple out from the hub on it every few
 * seconds, and the halos on the points of presence breathe (`.mesh-packet`,
 * `.mesh-ripple`, `.mesh-halo`, `site.css`; all stop under
 * prefers-reduced-motion). Decoration only: `aria-hidden`, no pointer events.
 */

const ROWS = 8;
const H = ROWS * CELL;
const TILE = 10;
// Tiles either side of the spine: enough for a 2560px screen with the spine
// near its middle.
const TILES = 4;

// Rows count from the anchored edge, so the densest row hugs it: a cell is
// (col, rowFromEdge), placed by `at` for the anchor in hand.
type Anchor = "top" | "bottom";
const place = (anchor: Anchor, [c, r]: Cell): Cell => [c, anchor === "top" ? r : ROWS - r];

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

/** Everything drawn, laid out once per anchor. */
function layout(anchor: Anchor) {
  const at = (c: Cell) => place(anchor, c);
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
  const hub = px(cell(0, A, "a"));
  return { solid, dashed, nodes, packets, hub };
}

const layouts = { top: layout("top"), bottom: layout("bottom") };

/** The fabric itself, hung from the top or the bottom edge of its box. */
export function MeshFabric({ anchor }: { anchor: Anchor }) {
  const { solid, dashed, nodes, packets, hub } = layouts[anchor];
  const grid = `mesh-grid-${anchor}`;
  return (
    <svg
      aria-hidden
      className={cn(
        "text-foreground absolute left-3 overflow-visible opacity-[0.4] sm:left-4 dark:opacity-[0.3]",
        anchor === "top" ? "top-0" : "bottom-0",
      )}
      width={1}
      height={H}
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
    >
      <defs>
        <pattern id={grid} width={CELL} height={CELL} patternUnits="userSpaceOnUse">
          <path d={`M${CELL} 0 H0 V${CELL}`} strokeWidth="1" shapeRendering="crispEdges" />
        </pattern>
      </defs>
      {/* the grid, from the spine and the anchored edge outwards */}
      <rect
        x={-(TILES + 1) * TILE * CELL}
        y={anchor === "top" ? 0 : -4 * CELL}
        width={2 * (TILES + 1) * TILE * CELL}
        height={H + 4 * CELL}
        fill={`url(#${grid})`}
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
          .map((n, i) => {
            const [x, y] = px(n.cell);
            return (
              <circle
                key={`halo-${x}-${y}`}
                cx={x}
                cy={y}
                r={13}
                strokeDasharray="2 3.5"
                strokeWidth="0.9"
                className="mesh-halo [stroke-opacity:0.35]"
                style={{ animationDelay: `${(i * 1.7) % 7}s` }}
              />
            );
          })}
        {nodes.map((n) => {
          const [x, y] = px(n.cell);
          return <circle key={`${x}-${y}`} cx={x} cy={y} r={n.pop ? 5 : 4} fill="var(--background)" />;
        })}
        {/* the reply going out: rings that ripple from the hub every few seconds */}
        {[0, 1, 2].map((i) => (
          <circle
            key={`ripple-${i}`}
            cx={hub[0]}
            cy={hub[1]}
            r={1}
            vectorEffect="non-scaling-stroke"
            strokeWidth="1"
            className="mesh-ripple"
            style={{ animationDelay: `${i * 3}s` }}
          />
        ))}
        {/* the hub on the spine: a dot in the ring, like the hero's */}
        <circle cx={hub[0]} cy={hub[1]} r={1.6} fill="currentColor" stroke="none" />
      </g>
    </svg>
  );
}
