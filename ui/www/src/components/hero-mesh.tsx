/**
 * Two drawings behind the hero, under the grid: a chain of blocks along the
 * bottom and a mesh of points of presence upper right, in the page's own
 * colour at a whisper of opacity. Hand-drawn on purpose: the lines are bent
 * by hand and roughened by a turbulence filter, the way a diagram on a
 * whiteboard is, so it reads as a sketch and not a chart. Decoration only:
 * hidden from readers, no pointer events, and the one slow drift on the
 * dashed links stops under prefers-reduced-motion (`site.css`).
 */
export function HeroMesh() {
  // The points of presence, roughly a map with no map under it.
  const pops: [number, number][] = [
    [790, 96],
    [900, 62],
    [1030, 88],
    [1140, 140],
    [850, 190],
    [960, 170],
    [1080, 230],
    [800, 300],
    [930, 290],
    [1050, 340],
    [1150, 300],
  ];
  // Who talks to whom; a few of them long-haul, drawn dashed.
  const links: [number, number, boolean?][] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [0, 4],
    [1, 5],
    [4, 5],
    [5, 2],
    [5, 6],
    [6, 3],
    [4, 7],
    [7, 8],
    [8, 5],
    [8, 9],
    [9, 6],
    [9, 10],
    [10, 3],
    [0, 8, true],
    [2, 9, true],
    [7, 10, true],
  ];
  // A slight bend on every edge, so none of them is ruler-straight.
  const bend = (a: [number, number], b: [number, number], i: number) => {
    const mx = (a[0] + b[0]) / 2;
    const my = (a[1] + b[1]) / 2;
    const k = ((i % 3) - 1) * 9;
    return `M${a[0]} ${a[1]} Q${mx + k} ${my - k} ${b[0]} ${b[1]}`;
  };
  // The chain: blocks along the bottom, each pointing at the one before it.
  const blocks = [110, 260, 410, 560, 710];

  return (
    <svg
      aria-hidden
      className="text-foreground pointer-events-none absolute inset-0 -z-10 h-full w-full opacity-[0.11] dark:opacity-[0.15]"
      viewBox="0 0 1200 640"
      preserveAspectRatio="xMidYMid slice"
      fill="none"
      stroke="currentColor"
      strokeWidth="1.1"
      strokeLinecap="round"
      strokeLinejoin="round"
      style={{
        maskImage: "linear-gradient(to bottom, black 0%, black 55%, transparent 100%)",
        WebkitMaskImage: "linear-gradient(to bottom, black 0%, black 55%, transparent 100%)",
      }}
    >
      <defs>
        <filter id="hand" x="-5%" y="-5%" width="110%" height="110%">
          <feTurbulence type="fractalNoise" baseFrequency="0.018" numOctaves="2" seed="7" result="n" />
          <feDisplacementMap
            in="SourceGraphic"
            in2="n"
            scale="3.2"
            xChannelSelector="R"
            yChannelSelector="G"
          />
        </filter>
      </defs>

      <g filter="url(#hand)">
        {/* ---------------------------------------------------- the mesh */}
        {links.map(([a, b, far], i) => (
          <path
            key={`${a}-${b}`}
            d={bend(pops[a]!, pops[b]!, i)}
            className={far ? "mesh-link" : undefined}
            strokeDasharray={far ? "5 7" : undefined}
            strokeWidth={far ? 0.9 : 1.1}
          />
        ))}
        {pops.map(([x, y], i) => (
          <g key={`${x}-${y}`}>
            <circle cx={x} cy={y} r={i % 4 === 0 ? 5 : 3.5} />
            {/* the second pass of a pen going round twice */}
            <circle cx={x + 0.8} cy={y - 0.6} r={i % 4 === 0 ? 5.4 : 3.9} strokeWidth="0.6" />
          </g>
        ))}
        {/* anycast: one address, announced from three places */}
        <circle cx={960} cy={170} r={16} strokeDasharray="2 4" strokeWidth="0.8" />
        <circle cx={960} cy={170} r={26} strokeDasharray="2 5" strokeWidth="0.6" />
        <circle cx={850} cy={190} r={13} strokeDasharray="2 4" strokeWidth="0.7" />
        <circle cx={1050} cy={340} r={13} strokeDasharray="2 4" strokeWidth="0.7" />

        {/* --------------------------------------------------- the chain */}
        {blocks.map((x, i) => (
          <g key={x}>
            <rect x={x} y={470} width={78} height={52} rx={4} />
            <rect x={x + 1.5} y={471.5} width={78} height={52} rx={4} strokeWidth="0.5" />
            {/* the header, then three transactions */}
            <path d={`M${x + 8} ${482} L${x + 40} ${481.5}`} strokeWidth="1.4" />
            <path d={`M${x + 8} ${496} L${x + 66} ${495.5}`} strokeWidth="0.8" />
            <path d={`M${x + 8} ${504} L${x + 58} ${504.5}`} strokeWidth="0.8" />
            <path d={`M${x + 8} ${512} L${x + 62} ${511.5}`} strokeWidth="0.8" />
            {/* the hash of the one before */}
            {i > 0 ? (
              <>
                <path d={`M${x} ${496} C${x - 24} ${494}, ${x - 48} ${498}, ${blocks[i - 1]! + 78} ${496}`} />
                <path d={`M${x - 8} ${491} L${x - 1} ${496} L${x - 8} ${501}`} strokeWidth="0.9" />
              </>
            ) : null}
          </g>
        ))}
        {/* a merkle tree above the third block, roots down */}
        <g strokeWidth="0.9">
          <circle cx={449} cy={392} r={3.5} />
          <circle cx={425} cy={416} r={3} />
          <circle cx={473} cy={416} r={3} />
          <circle cx={413} cy={440} r={2.5} />
          <circle cx={437} cy={440} r={2.5} />
          <circle cx={461} cy={440} r={2.5} />
          <circle cx={485} cy={440} r={2.5} />
          <path d="M446 395 Q436 404 427 413" />
          <path d="M452 395 Q462 404 471 413" />
          <path d="M423 419 Q418 429 414 437" />
          <path d="M427 419 Q432 429 436 437" />
          <path d="M471 419 Q466 429 462 437" />
          <path d="M475 419 Q480 429 484 437" />
          <path d="M449 396 Q449 440 449 468" strokeDasharray="2 4" strokeWidth="0.7" />
        </g>
        {/* the next block, still being made */}
        <rect x={860} y={470} width={78} height={52} rx={4} strokeDasharray="4 5" strokeWidth="0.9" />
        <path d="M860 496 C836 494, 812 498, 788 496" strokeDasharray="4 5" strokeWidth="0.9" />
      </g>
    </svg>
  );
}
