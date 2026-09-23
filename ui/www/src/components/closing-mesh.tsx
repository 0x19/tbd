import { CELL, type Cell, path, px, route, segments, trace } from "@/lib/mesh";

/**
 * The last drawing, under the strip the reply leaves by: the hero's mesh
 * turned outward. The reply drops out of the strip's corner onto a bus that
 * runs the whole width of the page, and from the bus the traces fan out to
 * points of presence on both sides and off the bottom, with packets running
 * away from the spine. Same 72px grid as the hero, drawn here as a pattern
 * whose origin is the spine, so every node sits on an intersection whatever
 * the viewport; the band clips it and fades it into the footer. Decoration
 * only: `aria-hidden`, no pointer events, packets stop under
 * prefers-reduced-motion (`.mesh-packet`, `site.css`).
 */

const ROWS = 4;
// Far enough either way for a 2560px screen with the spine near its middle.
const REACH = 40;

// The bus along row 1, and where it branches. `pop` marks a point of
// presence with a halo; `down` a branch that leaves off the bottom.
const branches: { at: number; to: Cell; pop?: boolean; down?: Cell }[] = [
  { at: -7, to: [-7, 3] },
  { at: -3, to: [-5, 3], pop: true, down: [-6, ROWS] },
  { at: 3, to: [5, 3], pop: true, down: [6, ROWS] },
  { at: 7, to: [7, 3], down: [7, ROWS] },
  { at: 11, to: [13, 3], pop: true, down: [14, ROWS] },
  { at: 15, to: [15, 3] },
  { at: 19, to: [21, 3], pop: true, down: [22, ROWS] },
  { at: 24, to: [24, 3], down: [24, ROWS] },
  { at: 28, to: [30, 3], pop: true },
  { at: 33, to: [33, 3], down: [33, ROWS] },
];

const drop = trace([0, 0], [0, 1]);
const bus = trace([-REACH, 1], [REACH, 1]);
const solid = segments([drop, bus, ...branches.map((b) => trace([b.at, 1], b.to))]);
// The long haul between the points of presence, dashed, and the branches
// that leave off the bottom.
const haul = trace([-REACH, 3], [REACH, 3]);
const dashed = segments([haul, ...branches.flatMap((b) => (b.down ? [trace(b.to, b.down)] : []))]);

const out = (b: (typeof branches)[number]) =>
  route(drop, trace([0, 1], [b.at, 1]), trace([b.at, 1], b.to), ...(b.down ? [trace(b.to, b.down)] : []));

// Packets, each leaving by a different branch on its own clock.
const packets = [
  { d: out(branches[4]!), duration: 11, delay: 0 },
  { d: out(branches[1]!), duration: 8, delay: 3 },
  { d: out(branches[6]!), duration: 15, delay: 6 },
  { d: out(branches[3]!), duration: 8, delay: 10 },
];

export function ClosingMesh() {
  return (
    <svg
      aria-hidden
      className="text-foreground pointer-events-none absolute top-0 left-3 overflow-visible sm:left-4"
      width={1}
      height={ROWS * CELL}
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
      {/* the grid, from the spine outwards */}
      <rect
        x={-REACH * CELL}
        y={0}
        width={2 * REACH * CELL}
        height={ROWS * CELL}
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
        {branches
          .filter((b) => b.pop)
          .map((b) => {
            const [x, y] = px(b.to);
            return (
              <circle
                key={`halo-${b.at}`}
                cx={x}
                cy={y}
                r={13}
                strokeDasharray="2 3.5"
                strokeWidth="0.9"
                className="[stroke-opacity:0.35]"
              />
            );
          })}
        {branches.map((b) => {
          const [x, y] = px(b.to);
          return <circle key={b.at} cx={x} cy={y} r={b.pop ? 5 : 4} fill="var(--background)" />;
        })}
        {/* the junctions on the bus, the spine's own first */}
        {[{ at: 0 }, ...branches].map((b) => {
          const [x, y] = px([b.at, 1]);
          return <circle key={`bus-${b.at}`} cx={x} cy={y} r={1.6} fill="currentColor" stroke="none" />;
        })}
        {/* where the reply leaves the strip: the same dot as the hero's origin */}
        <circle cx={0} cy={0} r={2.5} fill="var(--foreground)" fillOpacity={0.6} stroke="none" />
        <path d={path(drop)} strokeWidth="1" className="[stroke-opacity:0.26]" />
      </g>
    </svg>
  );
}
