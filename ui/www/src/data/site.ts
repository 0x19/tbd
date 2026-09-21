/**
 * Everything on the site that is a fact rather than layout. One file to edit;
 * the pages read from here.
 *
 * TODO markers are details only the company can supply (the registration
 * numbers and the registered address). They render as "—" until they are
 * filled in, so an empty field never looks like a real one.
 */

/** Where this build is published. The canonical URL and the sitemap use it. */
export const url = process.env.NEXT_PUBLIC_SITE_URL ?? "https://inorbit.hr";

/** Only the real domain is indexed; a build on any other host asks robots to stay away. */
export const indexable = new URL(url).hostname.endsWith("inorbit.hr");

export const company = {
  name: "InOrbit",
  legalName: "InOrbit d.o.o.",
  person: "Nevio Vesic",
  /** The title as it is actually held, in Tenderly's own levelling. */
  role: "L5 software engineer",
  /** The professional title, independent of any one employer. */
  title: "Senior software / protocol engineer",
  /** Where the day job is; it is why the blockchain work is not theoretical. */
  employer: { name: "Tenderly", href: "https://tenderly.co" },
  /** Short, for the browser tab and the footer. */
  tagline: "Backend and blockchain systems.",
  /** The hero. Long enough to say something only I could say. */
  headline: "Systems that stay up when it matters, and experiments that do not have to.",
  summary:
    "Twenty years in the unglamorous half of the stack: voice servers and SMS gateways first, then anycast networks carrying real-time traffic at sixty gigabits, then rollups, bridges and indexers. Ten of those years in Go, and as much Rust lately. Whatever I build for the fun of it ends up here too.",
  email: "nevio@inorbit.hr",
  city: "Rijeka and Zagreb, Croatia",
  github: "https://github.com/0x19",
  x: "https://x.com/vesicnevio",
  linkedin: "https://www.linkedin.com/in/neviovesic/",
  /** TODO: the registered address, as it appears in the court register. */
  address: "",
  /** OIB, the Croatian tax number. */
  oib: "38846238650",
  /** TODO: MBS (court register number) and the registering commercial court. */
  registration: "",
  /** Registered in June 2018 as the vehicle for the B2B work. */
  founded: 2018 as number | null,
} as const;

/** The ticker under the hero: what the work is actually made of. */
export const stack = [
  "Go",
  "Rust",
  "Elixir",
  "C",
  "eBPF",
  "WASM",
  "Solidity",
  "EVM",
  "RLPx",
  "libp2p",
  "WebRTC",
  "SIP",
  "gRPC",
  "JSON-RPC",
  "Postgres",
  "ClickHouse",
  "MDBX",
  "DuckDB",
  "Spanner",
  "CockroachDB",
  "Cassandra",
  "Mongo",
  "Kafka",
  "Kubernetes",
  "Envoy",
  "OpenTelemetry",
] as const;

/**
 * The working record: companies, in the order they happened. Dates come from
 * the CV; where one is genuinely open (when Eiger ended and Tenderly began) it
 * says so in words rather than guessing a month.
 */
export const experience = [
  {
    company: "Tenderly",
    role: "L5 software engineer",
    when: "Now",
    where: "Remote",
    href: "https://tenderly.co",
    body: "Developer infrastructure for Ethereum — simulation, debugging and the systems behind them, at production volume.",
    tags: ["Go", "Rust", "EVM"],
  },
  {
    company: "Eiger",
    role: "Senior software / protocol engineer",
    when: "2022 — until Tenderly",
    where: "Remote",
    href: "https://www.eiger.co",
    body: "Protocol work across several chains, from research to deployment: a proprietary EVM-compatible optimistic rollup taken from inception to production, one of the first WASM ports of a Layer 2 node in Go, and a cross-chain liquidity bridge between Ethereum and Bitcoin built on multi-party computation and threshold ECDSA. Led teams of up to five, ran the research, and wrote the grant proposals that funded some of it.",
    tags: ["Go", "Rust", "WASM", "EVM", "P2P", "RLPx"],
  },
  {
    company: "InOrbit",
    role: "Owner",
    when: "2018 — present",
    where: "Croatia",
    href: null,
    body: "The company the B2B work runs through; Eiger and Subspace were both engaged this way.",
    tags: [],
  },
  {
    company: "Subspace",
    role: "Senior software engineer",
    when: "2018 — 2022",
    where: "Remote · Los Angeles",
    href: null,
    body: "A network built for traffic that cannot wait. I wrote the user-space services sitting between the kernel and the control plane, and some of the kernel side itself — IP filtering, packet rewriting, network-card caching, map management. Co-built the first version of the TURN anycast network, live in over 150 points of presence, and the first SIP anycast network on Kamailio and FreeSWITCH; also the Elixir control plane that provisioned tunnels and billed usage from a geo-aware distributed database.",
    tags: ["Go", "Elixir", "C", "eBPF", "Kubernetes", "Kafka"],
  },
  {
    company: "Avaya",
    role: "Senior engineer, then software engineer on CPaaS",
    when: "2016 — 2018",
    where: "Croatia",
    href: null,
    body: "The TelAPI platform after its acquisition, as Zang Cloud and then Avaya CPaaS. Go microservices on the platform, then architecting the next generation of its front end and running the team that built it, including the security and compliance side — code scanning, HIPAA, GDPR, SOC.",
    tags: ["Go", "React", "Node.js", "Kubernetes", "GCP", "AWS"],
  },
  {
    company: "TelTech Systems · TelAPI",
    role: "Senior software engineer",
    when: "2014 — 2016",
    where: "Remote · New York",
    href: null,
    body: "Telecom at the protocol level: voice servers on FreeSWITCH and Kamailio, an SMS stack over SMPP with its SMSC and SMSE sides, phone-number and carrier services, and the full rewrite of those services from Python to Go. Consumer products on the same plumbing, spoofcard.com and tapeacall.com among them.",
    tags: ["Go", "Python", "C", "FreeSWITCH", "Kamailio", "SMPP"],
  },
  {
    company: "TelAPI Adriatica",
    role: "Director",
    when: "2013 — 2015",
    where: "Croatia",
    href: null,
    body: "The Croatian branch, and two engineers in it. Closed when Avaya acquired the parent.",
    tags: [],
  },
  {
    company: "Earlier",
    role: "Web development and server administration",
    when: "2007 — 2014",
    where: "Rijeka · New Jersey",
    href: null,
    body: "TelTech Systems, CLKCLK, Adria24, Web Factory, In-tech, Design Strategist and Skin29 — where the twenty years start, and where I learned that somebody has to run the server too.",
    tags: [],
  },
] as const;

/**
 * The things worth pulling out of the record. Each one is a specific claim from
 * the CV, not a summary of a role.
 */
export const achievements = [
  "Early work on a Layer 1 to Layer 7 network with a distributed team, carrying over 60 Gbps, which secured a $3M+ annual contract and opened the following investment round.",
  "One of the first WASM ports of a Layer 2 blockchain node in Go.",
  "An optimistic EVM rollup written entirely in Go, from inception to deployment.",
  "The first Go Solidity AST and IR parser, control-flow graph construction included.",
  "A cross-chain liquidity bridge between Ethereum and Bitcoin on multi-party computation and threshold ECDSA.",
  "A cross-chain EVM indexer that streams an entire blockchain dataset in under ten hours.",
] as const;

/**
 * Things I have written and left in the open. Every line is a real repository;
 * the descriptions say what it does, not what it promises.
 */
export const projects = [
  {
    name: "solgo",
    year: "2023",
    what: "A Solidity parser in Go that turns contract source into a structured form you can analyse — the base for detectors, ABI work and standards discovery.",
    language: "Go",
    href: "https://github.com/unpackdev/solgo",
  },
  {
    name: "sourcify-go",
    year: "2023",
    what: "A Go client for the Sourcify API: verify a contract, fetch its metadata and sources, check what a chain already knows about an address.",
    language: "Go",
    href: "https://github.com/unpackdev/sourcify-go",
  },
  {
    name: "fdb",
    year: "2024",
    what: "A high-performance transport layer in front of embedded key-value databases such as MDBX, for the reads a node or an indexer cannot wait on.",
    language: "Go",
    href: "https://github.com/unpackdev/fdb",
  },
  {
    name: "solc-switch",
    year: "2023",
    what: "Manages every Solidity compiler version at once and compiles with the right one, concurrently, instead of juggling toolchains by hand.",
    language: "Go",
    href: "https://github.com/0x19/solc-switch",
  },
  {
    name: "go-clickhouse-orm",
    year: "2023",
    what: "Model and migration support for ClickHouse in Go, so an analytics schema is versioned like any other part of a service.",
    language: "Go",
    href: "https://github.com/0x19/go-clickhouse-orm",
  },
  {
    name: "gotostruct",
    year: "2015",
    what: "Turns a JSON object into a Go struct, as a library and as the public tool that ran at jsonstruct.com. Written because I was tired of doing it by hand.",
    language: "Go",
    href: "https://github.com/0x19/gotostruct",
  },
  {
    name: "disposable",
    year: "2016",
    what: "A JSON and gRPC API that answers one question — is this a throwaway email address? Small, public, and quietly used by strangers ever since.",
    language: "Go",
    href: "https://github.com/0x19/disposable",
  },
  {
    name: "goesl",
    year: "2015",
    what: "A FreeSWITCH Event Socket library for Go. Written in 2015 and still forked and shipped by other people — telephony was the first system I had to keep up.",
    language: "Go",
    href: "https://github.com/0x19/goesl",
  },
] as const;

/**
 * The short version of how I got here. Three paragraphs, no career timeline.
 */
export const about = [
  "Twenty years of building software, most of it infrastructure: distributed systems, storage, protocols and telecommunications. Ten of those years in Go, and as much Rust lately. Along the way I have led teams of up to five, and done the part of that job that is talking to management and clients rather than to a compiler.",
  "The arc runs telecom, then real-time networks, then blockchain protocols. Voice servers, SMS and carrier services to begin with; then anycast TURN and SIP networks and the kernel-side packet work underneath them; then an optimistic EVM rollup, a cross-chain bridge and indexers that keep up with a chain. Now developer infrastructure for Ethereum at Tenderly.",
  "Away from the screen: a guitar, more philosophy and psychology than is strictly useful, the dogs, and my girlfriend.",
] as const;

/**
 * Where the work happened before now. Companies only, newest first; the current
 * one is in `clients`.
 */
export const previously = ["Eiger", "Subspace", "Avaya", "TelAPI"] as const;

/** Languages I can hold a conversation in. */
export const languages = ["English", "Croatian", "Bosnian", "Serbian", "Slovenian"] as const;

/**
 * Who I work with. Clients only — my own projects live in `projects` and the
 * bio. The figures are Tenderly's own published numbers, not mine to verify,
 * and they are attributed as such on the page.
 */
export const clients = [
  {
    name: "Tenderly",
    href: "https://tenderly.co",
    role: "L5 software engineer",
    tagline: "Model every onchain move.",
    what: "Tenderly is the simulation layer for onchain operations: try a transaction against live production state before any capital moves, debug what a contract actually did rather than guessing, watch production and get told the moment it misbehaves, and run all of it on RPC infrastructure spanning more than a hundred networks.",
    mine: "I work on the backend systems underneath that — the unglamorous half, in Go and Rust, where correctness at volume is the whole job.",
    facts: [
      { value: "4B+", label: "transactions simulated against live state" },
      { value: "10M+", label: "transactions debugged" },
      { value: "50%", label: "of the top-100 DeFi protocols by value" },
      { value: "$50B+", label: "in onchain value on systems it serves" },
    ],
  },
] as const;

/**
 * The column most of the work has happened in: a packet on the way in, a
 * protocol on the way through, a service that decides something, and state at
 * the bottom. Every layer here is one I have actually had to debug.
 */
export const pipeline = [
  {
    stage: "Wire",
    name: "packets, kernel",
    rows: [
      { k: "work", v: "eBPF, filtering, rewriting" },
      { k: "measured in", v: "microseconds" },
      { k: "at the edge", v: "anycast, 150+ PoPs" },
    ],
  },
  {
    stage: "Protocol",
    name: "SIP · WebRTC · RLPx · gRPC",
    rows: [
      { k: "spoken", v: "to the letter" },
      { k: "checked", v: "against the capture" },
      { k: "carried", v: "voice, video and blocks" },
    ],
  },
  {
    stage: "Service",
    name: "Go · Rust",
    rows: [
      { k: "does", v: "one thing, restartable" },
      { k: "ships with", v: "health · metrics · traces" },
      { k: "written in", v: "Go · Rust · Elixir · C" },
    ],
  },
  {
    stage: "State",
    name: "chosen per shape",
    rows: [
      { k: "embedded", v: "MDBX · DuckDB" },
      { k: "relational", v: "Postgres · Spanner · CockroachDB" },
      { k: "at scale", v: "Cassandra · ClickHouse · Mongo" },
    ],
  },
] as const;

/** The two rails that run under every layer above. */
export const rails = [
  {
    label: "Every layer",
    value: "a trace id into logs, traces, metrics and continuous profiles — and a capture when it lies",
  },
  {
    label: "Before production",
    value: "load, injected latency, dropped packets, storage faults, property-based runs",
  },
] as const;

/** How I like to build. Short, and I do actually mean them. */
export const principles = [
  {
    title: "The spec, then the wire",
    body: "An RFC says what should happen; a capture says what does. When they disagree the wire wins, and the interesting bugs live in that gap.",
  },
  {
    title: "Observable by default",
    body: "Every request carries a trace id into logs, traces, metrics and continuous profiles. A failure gets read, not guessed at.",
  },
  {
    title: "Tested against faults",
    body: "The same tooling that runs in development injects latency, errors, dropped packets and storage failures in CI, so the first outage is not the first test.",
  },
  {
    title: "Out in the open",
    body: "Open source where it can be. The good parts are worth more read by other people than kept in a drawer.",
  },
] as const;

/**
 * Public things to try. Empty until the first one is live — a playground that
 * does not exist yet is not listed as though it did.
 */
export const playgrounds: {
  name: string;
  what: string;
  /** Where it lives. `null` while it is still being built. */
  href: string | null;
  tag: string;
}[] = [
  {
    name: "Break it",
    what: "Four real services under live traffic with an objective to hold, and a shared budget of faults to spend trying to break it. Everyone pokes the same sandbox; it heals itself. Watch it over a WebSocket, server-sent events or plain polling — the same call, three ways.",
    href: "/playgrounds/break-it/",
    tag: "Chaos",
  },
  {
    name: "Guitar tuner",
    what: "Pluck a string and the meter names it and shows how far off you are, to the cent. Five tunings, a reference tone per string, an adjustable A4. It listens through the microphone and nothing leaves the page.",
    href: "/playgrounds/tuner/",
    tag: "Audio",
  },
];

export const nav = [
  { href: "/", label: "Home" },
  { href: "/playgrounds/", label: "Playgrounds" },
  { href: "/projects/", label: "Projects" },
  { href: "/about/", label: "About" },
  { href: "/contact/", label: "Contact" },
] as const;

export const site = {
  title: company.name,
  description: company.summary,
  /** The mark is drawn in `components/logo.tsx`; these are the files for everything else. */
  markSrc: "/brand/mark.svg",
  lockupSrc: "/brand/lockup.svg",
} as const;

/** A TODO field renders as an em dash rather than an empty line. */
export function orDash(value: string): string {
  return value.trim() === "" ? "—" : value;
}
