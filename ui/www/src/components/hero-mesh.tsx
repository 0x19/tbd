/**
 * One drawing behind the hero, upper right, on the same 72px grid as the
 * hairlines under it: a small mesh of points of presence, routed the way a
 * board is routed. Every node sits on a grid intersection, every trace runs
 * along the grid or at 45°, a segment two traces share is drawn once, and a
 * node knocks the trace and the grid out behind it, so the whole thing reads
 * as one drawn system and not a doodle over a page. Three packets crawl
 * along it (`.mesh-packet`, `site.css`) so it is alive without moving; they
 * stop under prefers-reduced-motion. Decoration only: hidden from readers, no
 * pointer events, and only from `xl` up, where the headline leaves room for
 * it; the left fade moves with the viewport so no node sits under the text.
 */

import { type Bend, CELL, type Cell, px, route, segments, trace } from "@/lib/mesh";

const COLS = 12;
const ROWS = 7;

// The nodes, by cell. `pops` are the points of presence: one address,
// announced from three places, drawn with a dashed halo.
const nodes = {
  a: [5, 3],
  b: [7, 2],
  c: [7, 4],
  d: [9, 1],
  e: [9, 3],
  f: [10, 5],
  g: [11, 2],
  h: [11, 4],
  i: [6, 1],
} as const satisfies Record<string, Cell>;
type Node = keyof typeof nodes;

const pops: Node[] = ["e", "g", "f"];

const T = (x: Node, y: Node, diagonal: Bend = "last") => trace(nodes[x], nodes[y], diagonal);
const back = (x: Node, y: Node, diagonal: Bend = "last") => T(x, y, diagonal).reverse();

// Who talks to whom, and which way each trace bends.
const links: [Node, Node, Bend][] = [
  ["a", "b", "last"],
  ["a", "c", "last"],
  ["i", "b", "last"],
  ["b", "d", "last"],
  ["b", "e", "last"],
  ["c", "e", "first"],
  ["c", "f", "first"],
  ["d", "e", "last"],
  ["e", "f", "last"],
  ["d", "g", "first"],
  ["e", "g", "last"],
  ["e", "h", "last"],
  ["f", "h", "first"],
  ["g", "h", "last"],
];
const solid = segments(links.map(([x, y, d]) => T(x, y, d)));

// The long way round, dashed, up and over the top; and where the traffic
// comes from and goes on: stubs off both edges.
const enter: Cell = [2, 3];
const exits: Cell[] = [
  [COLS, 2],
  [COLS, 4],
];
const dashed = segments([
  T("a", "i"),
  T("i", "d"),
  trace(enter, nodes.a),
  trace(nodes.g, exits[0]!),
  trace(nodes.h, exits[1]!),
]);

// Three packets, each on its own route and clock, so no two are in step.
const packets: { d: string; duration: number; delay: number }[] = [
  {
    d: route(
      trace(enter, nodes.a),
      T("a", "b"),
      T("b", "d"),
      T("d", "g", "first"),
      trace(nodes.g, exits[0]!),
    ),
    duration: 16,
    delay: 0,
  },
  {
    d: route(
      trace(enter, nodes.a),
      T("a", "c"),
      T("c", "f", "first"),
      T("f", "h", "first"),
      trace(nodes.h, exits[1]!),
    ),
    duration: 19,
    delay: 6,
  },
  {
    d: route(
      trace(exits[0]!, nodes.g),
      back("e", "g"),
      back("c", "e", "first"),
      back("a", "c"),
      trace(nodes.a, enter),
    ),
    duration: 17,
    delay: 11,
  },
];

// The left fade follows the viewport: at 1440 it clears the headline's last
// word, and every 100px less hides 75px more, so the ingress goes first.
const fade =
  "linear-gradient(to right, transparent max(0px, calc(1320px - 75vw)), black max(120px, calc(1440px - 75vw)))";
const mask = `${fade}, linear-gradient(to bottom, black 60%, transparent 100%)`;

export function HeroMesh() {
  return (
    <svg
      aria-hidden
      className="text-foreground pointer-events-none absolute top-0 right-0 -z-10 hidden xl:block"
      width={COLS * CELL}
      height={ROWS * CELL}
      viewBox={`0 0 ${COLS * CELL} ${ROWS * CELL}`}
      fill="none"
      stroke="currentColor"
      strokeLinecap="round"
      strokeLinejoin="round"
      style={{
        maskImage: mask,
        WebkitMaskImage: mask,
        maskComposite: "intersect",
        WebkitMaskComposite: "source-in",
      }}
    >
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
        {pops.map((k) => {
          const [x, y] = px(nodes[k]);
          return (
            <circle
              key={`halo-${k}`}
              cx={x}
              cy={y}
              r={13}
              strokeDasharray="2 3.5"
              strokeWidth="0.9"
              className="[stroke-opacity:0.35]"
            />
          );
        })}
        {(Object.keys(nodes) as Node[]).map((k) => {
          const [x, y] = px(nodes[k]);
          return <circle key={k} cx={x} cy={y} r={pops.includes(k) ? 5 : 4} fill="var(--background)" />;
        })}
        {/* the hub: a dot in the ring */}
        <circle cx={px(nodes.e)[0]} cy={px(nodes.e)[1]} r={1.6} fill="currentColor" stroke="none" />
      </g>
    </svg>
  );
}
