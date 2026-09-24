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
  /** The professional title, independent of any one employer. */
  title: "Staff Software / Protocol Engineer",
  /** The line under the title: the scope first, the tools after it. */
  focus: "Distributed Systems · Infrastructure · Rust · Go",
  /** What I am doing now, in one line: the about page's "Now". */
  now: "Independent, through InOrbit",
  /**
   * The one sentence under the hero that says whether I can be hired, for what,
   * and from when. Keep it true: it is the first thing a reader acts on.
   */
  availability:
    "Available from October 2026 for protocol, infrastructure and distributed-systems work in Rust and Go, on a B2B contract through my own company, InOrbit d.o.o.",
  /** Short, for the browser tab and the footer. */
  tagline: "Distributed systems and infrastructure.",
  /** The hero. Long enough to say something only I could say. */
  headline: "Systems built to stay up.",
  summary:
    "Twenty years in the unglamorous half of the stack: voice servers and SMS gateways first, then anycast networks carrying real-time traffic at sixty gigabits, then rollups, bridges and indexers, and most recently the RPC infrastructure in front of a hundred-odd blockchain networks. Ten of those years in Go, and the last one in Rust. Whatever I build for the fun of it ends up here too.",
  email: "nevio@inorbit.hr",
  city: "Rijeka and Zagreb, Croatia",
  github: "https://github.com/0x19",
  x: "https://x.com/vesicnevio",
  linkedin: "https://www.linkedin.com/in/neviovesic/",
  /** The registered seat, as it appears in the court register (read 2026-09-21). */
  address: "Benčani 15A, Saršoni, 51216 Viškovo, Croatia",
  /** OIB, the Croatian tax number. */
  oib: "38846238650",
  /** MBS (court register number) and the registering commercial court. */
  registration: "081116183, Commercial Court in Rijeka",
  /** Registered in June 2018 as the vehicle for the B2B work. */
  founded: 2018 as number | null,
} as const;

/**
 * The working record: companies, in the order they happened, newest first. Dates
 * come from the CV. `body` is the one paragraph the about page shows; `highlights`
 * are the lines the CV page and the PDF add under it. Nothing here restates an
 * employer's confidential figures: what a company publishes about itself is
 * theirs to publish.
 */
export const experience = [
  {
    company: "Tenderly",
    role: "Software engineer, network infrastructure",
    when: "2024 — 2026",
    where: "Remote",
    href: "https://tenderly.co",
    body: "Developer infrastructure for Ethereum — simulation, debugging and the systems behind them, at production volume. Backend work in Go and Rust on the systems that carry it. The contract ended in September 2026.",
    tags: ["Go", "Rust", "EVM"],
    highlights: [],
  },
  {
    company: "(Un)Pack",
    role: "Founder",
    when: "2023 — 2024",
    where: "Remote",
    href: "https://github.com/unpackdev",
    body: "My own product: a platform that pulls Ethereum contracts apart at scale — source, AST and IR, bytecode, control-flow graphs — with a discovery service over GraphQL that used language models to say what a contract does and whether it looks like a rug pull. Switched off when the Tenderly work began, because running it cost more than it earned. The libraries stay public.",
    tags: ["Go", "Rust", "Python", "ClickHouse", "LLM"],
    highlights: [
      "Designed, built and ran it alone: crawler, storage, analysis, API and the bill.",
      "solgo — the first Solidity AST and IR parser in Go, with control-flow graph construction; used by others since.",
      "A crawler that peaked above 30k requests a second against the chain, feeding one to one and a half terabytes a day into Postgres and ClickHouse.",
      "Token pricing from pool reserves directly, without Chainlink or a third-party API.",
    ],
  },
  {
    company: "Eiger",
    role: "Senior software / protocol engineer",
    when: "2022 — 2024",
    where: "Remote",
    href: "https://www.eiger.co",
    body: "Protocol work across several chains, from research to deployment: a proprietary EVM-compatible optimistic rollup taken from inception to production, one of the first WASM ports of a Layer 2 node in Go, and a cross-chain liquidity bridge between Ethereum and Bitcoin built on multi-party computation and threshold ECDSA. Led teams of up to five, ran the research, and wrote the grant proposals that funded some of it.",
    tags: ["Go", "Rust", "WASM", "EVM", "P2P", "RLPx"],
    // The body already says it all; the CV adds nothing under this one.
    highlights: [],
  },
  {
    company: "InOrbit",
    role: "Owner",
    when: "2018 — present",
    where: "Croatia",
    href: null,
    body: "The company the B2B work runs through; Subspace, Eiger and Tenderly were all engaged this way.",
    tags: [],
    highlights: [],
  },
  {
    company: "Subspace",
    role: "Senior software engineer",
    when: "2018 — 2022",
    where: "Remote · Los Angeles",
    href: null,
    body: "A network built for traffic that cannot wait. I wrote the user-space services sitting between the kernel and the control plane, and some of the kernel side itself — IP filtering, packet rewriting, network-card caching, map management. Co-built the first version of the TURN anycast network, live in over 150 points of presence, and the first SIP anycast network on Kamailio and FreeSWITCH; also the Elixir control plane that provisioned tunnels and billed usage from a geo-aware distributed database.",
    tags: ["Go", "Elixir", "C", "eBPF", "Kubernetes", "Kafka"],
    highlights: [
      "One of the first three engineers. A Layer 1 to Layer 7 network built in six months for the MENA region, carrying over 60 Gbps from the start; it secured a contract above three million dollars a year and the next funding round.",
      "Co-conceived and co-built the first global TURN anycast network, over 150 points of presence, and the SIP anycast network on Kamailio.",
      "User-space services between the kernel and the control plane, and the kernel side itself in eBPF: IP filtering, packet rewriting, NIC caching, map management. A patent-pending contribution on the eBPF design.",
      "The Elixir control plane that took customer API requests and turned them into IPv4 tunnels, with usage billing from a geo-aware distributed database.",
    ],
  },
  {
    company: "Avaya",
    role: "Senior engineer, then software engineer on CPaaS",
    when: "2016 — 2018",
    where: "Croatia",
    href: null,
    body: "The TelAPI platform after its acquisition, as Zang Cloud and then Avaya CPaaS. Go microservices on the platform, then architecting the next generation of its front end and running the team that built it, including the security and compliance side — code scanning, HIPAA, GDPR, SOC.",
    tags: ["Go", "React", "Node.js", "Kubernetes", "GCP", "AWS"],
    highlights: [
      "Led the front-end services team: planning, unblocking, delivery.",
      "Architected the next generation of the CPaaS front end (React, Node.js, Go, Kubernetes, GCP, AWS).",
      "Owned the security side of it: code scanning and HIPAA, GDPR and SOC compliance work.",
    ],
  },
  {
    company: "TelTech Systems · TelAPI",
    role: "Senior software engineer",
    when: "2014 — 2016",
    where: "Remote · New York",
    href: null,
    body: "Telecom at the protocol level: voice servers on FreeSWITCH and Kamailio, an SMS stack over SMPP with its SMSC and SMSE sides, phone-number and carrier services, and the full rewrite of those services from Python to Go. Consumer products on the same plumbing, spoofcard.com and tapeacall.com among them.",
    tags: ["Go", "Python", "C", "FreeSWITCH", "Kamailio", "SMPP"],
    highlights: [
      "Voice servers on FreeSWITCH and Kamailio; an SMS stack over SMPP with its SMSC and SMSE sides; phone-number and carrier services.",
      "The full rewrite of the services from Python to Go.",
    ],
  },
  {
    company: "TelAPI Adriatica",
    role: "Director",
    when: "2013 — 2015",
    where: "Croatia",
    href: null,
    body: "The Croatian branch, and two engineers in it. Closed when Avaya acquired the parent.",
    tags: [],
    highlights: [],
  },
  {
    company: "Earlier",
    role: "Web development and server administration",
    when: "2007 — 2014",
    where: "Rijeka · New Jersey",
    href: null,
    body: "TelTech Systems, CLKCLK, Adria24, Web Factory, In-tech, Design Strategist and Skin29 — where the twenty years start, and where I learned that somebody has to run the server too.",
    tags: [],
    highlights: [],
  },
] as const;

/**
 * The things worth pulling out of the record. Each one is a specific claim from
 * the CV, not a summary of a role.
 */
export const achievements = [
  "One of the first three engineers at Subspace: a Layer 1-7 anycast network built in six months, 60+ Gbps from day one, 150+ points of presence; it won a $3M+/year contract and the next funding round. Patent-pending eBPF work.",
  "An optimistic EVM rollup in Go, from inception to production; one of the first WASM ports of a Layer 2 node in Go.",
  "A cross-chain liquidity bridge between Ethereum and Bitcoin on multi-party computation and threshold ECDSA.",
  "A cross-chain EVM indexer that streams an entire chain in under ten hours; a crawler at 30k req/s feeding 1-1.5 TB a day.",
  "solgo, the first Solidity AST/IR parser in Go with control-flow graphs -- open source, used by others since.",
] as const;

/**
 * Things I have written and left in the open. Every line is a real repository;
 * the descriptions say what it does, not what it promises.
 */
export const projects = [
  {
    name: "solgo",
    domain: "blockchain",
    year: "2023",
    what: "A Solidity parser in Go that turns contract source into a structured form you can analyse: the base for detectors, ABI work and standards discovery.",
    language: "Go",
    href: "https://github.com/unpackdev/solgo",
  },
  {
    name: "sourcify-go",
    domain: "blockchain",
    year: "2023",
    what: "A Go client for the Sourcify API: verify a contract, fetch its metadata and sources, check what a chain already knows about an address.",
    language: "Go",
    href: "https://github.com/unpackdev/sourcify-go",
  },
  {
    name: "fdb",
    domain: "data",
    year: "2024",
    what: "A high-performance transport layer in front of embedded key-value databases such as MDBX, for the reads a node or an indexer cannot wait on.",
    language: "Go",
    href: "https://github.com/unpackdev/fdb",
  },
  {
    name: "solc-switch",
    domain: "blockchain",
    year: "2023",
    what: "Manages every Solidity compiler version at once and compiles with the right one, concurrently, instead of juggling toolchains by hand.",
    language: "Go",
    href: "https://github.com/0x19/solc-switch",
  },
  {
    name: "go-clickhouse-orm",
    domain: "data",
    year: "2023",
    what: "Model and migration support for ClickHouse in Go, so an analytics schema is versioned like any other part of a service.",
    language: "Go",
    href: "https://github.com/0x19/go-clickhouse-orm",
  },
  {
    name: "gotostruct",
    domain: "tools",
    year: "2015",
    what: "Turns a JSON object into a Go struct, as a library and as the public tool that ran at jsonstruct.com. Written because I was tired of doing it by hand.",
    language: "Go",
    href: "https://github.com/0x19/gotostruct",
  },
  {
    name: "disposable",
    domain: "tools",
    year: "2016",
    what: "A JSON and gRPC API that answers one question: is this a throwaway email address? Small, public, and in quiet use ever since.",
    language: "Go",
    href: "https://github.com/0x19/disposable",
  },
  {
    name: "goesl",
    domain: "tools",
    year: "2015",
    what: "A FreeSWITCH Event Socket library for Go. Written in 2015 and still forked and shipped by others. Telephony was the first system I had to keep up.",
    language: "Go",
    href: "https://github.com/0x19/goesl",
  },
] as const;

/**
 * The short version of how I got here. Three paragraphs, no career timeline.
 */
export const about = [
  "Twenty years of building software, most of it infrastructure: distributed systems, storage, protocols and telecommunications. Ten of those years in Go, and the last one in Rust. Along the way I have led teams of up to five, and done the part of that job that is talking to management and clients rather than to a compiler.",
  "The arc runs telecom, then real-time networks, then blockchain protocols. Voice servers, SMS and carrier services to begin with; then anycast TURN and SIP networks and the kernel-side packet work underneath them; then an optimistic EVM rollup, a cross-chain bridge and indexers that keep up with a chain; then two years of developer infrastructure for Ethereum at Tenderly, the RPC layer in front of a hundred-odd networks.",
  "Away from the screen: a guitar, more philosophy and psychology than is strictly useful, the dogs, and my girlfriend.",
] as const;

/**
 * Where the work happened before now. Companies only, newest first; the current
 * one is in `clients`.
 */
export const previously = ["Tenderly", "Eiger", "Subspace", "Avaya", "TelAPI"] as const;

/**
 * The path in chapters, for the about page: the eras, not the positions. The
 * positions, with their dates and the lines under each, are `experience` and
 * the PDF; a chapter says what the years were about and names where. Nothing
 * here restates an employer's confidential figures.
 */
export const chapters = [
  {
    when: "2007 — 2014",
    title: "Web, then the server under it",
    body: "Agencies and product shops in Rijeka and New Jersey, PHP and JavaScript: a CMS, a booking system, an e-commerce platform, the front end of a CPaaS. Where I learned that someone has to keep the server up, and that it might as well be me.",
    where: "Skin29 · Design Strategist · In-tech · WebFactory · Adria24 · ClkClk · TelTech Systems",
  },
  {
    when: "2013 — 2018",
    title: "Telecom at the protocol level",
    body: "Voice servers on FreeSWITCH and Kamailio, an SMS stack over SMPP, carrier and number services, and a rewrite of all of it from Python into Go; then, after the acquisition, the architecture of the CPaaS front end and the team that built it, with the security and compliance side thrown in.",
    where: "TelAPI · TelAPI Adriatica · Avaya",
  },
  {
    when: "2018 — 2022",
    title: "Networks for traffic that cannot wait",
    body: "One of the first three engineers at Subspace: an anycast network from Layer 1 to Layer 7, built in six months for a region and running above sixty gigabits from the start; anycast TURN and SIP on 150-plus points of presence; the packet work in eBPF underneath, and a patent filing on it.",
    where: "Subspace, through InOrbit",
  },
  {
    when: "2022 — 2024",
    title: "Blockchain protocols",
    body: "An optimistic EVM rollup from idea to production, one of the first WASM ports of a layer-two node in Go, a liquidity bridge between Ethereum and Bitcoin on multi-party computation; on the side, (Un)Pack, a contract-analysis product of my own, and the solgo parser it left behind in the open.",
    where: "Eiger · (Un)Pack",
  },
  {
    when: "2024 — 2026",
    title: "Ethereum infrastructure",
    body: "Two years of developer infrastructure at Tenderly: the RPC layer in front of a hundred-odd networks, in Go and Rust, at production scale. The contract ended in September 2026.",
    where: "Tenderly, through InOrbit",
  },
  {
    when: "2026 —",
    title: "Independent, through InOrbit",
    body: "This platform, the playgrounds on it, and the next contract: protocols, infrastructure and distributed systems in Rust and Go, B2B through my own company.",
    where: "InOrbit d.o.o.",
  },
] as const;

/**
 * The years before the record above, compressed: the CV page lists them in one
 * block. Web work in PHP and JavaScript, mostly in Croatia.
 */
export const earlier = [
  {
    when: "2011 — 2014",
    company: "TelTech Systems",
    role: "Web application developer — the CPaaS front end in Zend, its documentation and API explorer",
  },
  {
    when: "2011",
    company: "ClkClk",
    role: "Web developer — the company's SaaS, then its internal administration from scratch",
  },
  {
    when: "2010 — 2011",
    company: "Adria24",
    role: "Web developer — the internal booking system of a tourist agency, front and back",
  },
  {
    when: "2009 — 2010",
    company: "In-tech, WebFactory",
    role: "Lead web developer — a book e-commerce platform; WordPress",
  },
  {
    when: "2007 — 2008",
    company: "Skin29, Design Strategist",
    role: "Web developer — a CMS later used by several large Croatian companies",
  },
] as const;

/**
 * The bar above the header while the site is being rebuilt, and the one honest
 * sentence about storage beside it. `version` is what a dismissal remembers:
 * change it and everyone sees the notice once more; empty `text` removes the bar.
 */
export const notice = {
  version: "2026-09",
  text: "This site is being rebuilt and some pages are out of date. For current details, write to nevio@inorbit.hr. This notice goes away when the rebuild is done.",
  privacy: "No cookies, no analytics: the only thing kept in your browser is that you closed this notice.",
} as const;

/** The CV page and the PDF it links: one source for both. */
export const cv = {
  pdf: "/cv/nevio-vesic.pdf",
  /**
   * Where the full version lives: phone, address and references, behind a
   * sign-in and my approval, rendered for each reader (docs/cv/README.md).
   */
  fullUrl: "https://cv.inorbit.hr/",
  education: "Secondary school, 2000 — 2003. Everything since, self-taught on the job.",
} as const;

/** Languages I can hold a conversation in. */
export const languages = ["English", "Croatian", "Bosnian", "Serbian", "Slovenian"] as const;

/**
 * Who I work with. Clients only — my own projects live in `projects` and the
 * bio. Empty since September 2026, and the home page then shows no section at
 * all: an out-of-date "now" is worse than none. A client's figures, when there
 * is one again, are quoted as theirs and labelled as published by them.
 */
export const clients: readonly {
  name: string;
  href: string;
  role: string;
  tagline: string;
  what: string;
  mine: string;
  facts: readonly { label: string; value: string; note?: string }[];
}[] = [];

/**
 * The column most of the work has happened in: a packet on the way in, a
 * protocol on the way through, a service that decides something, and state at
 * the bottom. Every layer here is one I have actually had to debug.
 */
export const pipeline = [
  {
    stage: "Network",
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
  /** The tab it sits under on the home page and `/playgrounds/`. */
  category: "systems" | "music";
  /** One line for the card; `what` is the longer description. */
  summary: string;
  /** Two or three facts for the card's foot. */
  specs: string[];
}[] = [
  {
    name: "Break it",
    what: "Four real services under live traffic with an objective to hold, and a shared budget of faults to spend trying to break it. Everyone pokes the same sandbox; it heals itself. Watch it over a WebSocket, server-sent events or plain polling — the same call, three ways.",
    href: "/lab/break-it/",
    tag: "Chaos",
    category: "systems",
    summary: "Four live services, one objective, and a shared budget of faults to break them with.",
    specs: ["Live traffic", "WebSocket, SSE, polling", "Self-healing"],
  },
  {
    name: "Guitar tuner",
    what: "Pluck a string and the meter names it and shows how far off you are, to the cent. Five tunings, a reference tone per string, an adjustable A4. It listens through the microphone and nothing leaves the page.",
    href: "/playgrounds/tuner/",
    tag: "Audio",
    category: "music",
    summary: "Names the string you pluck and how far off it is, to the cent.",
    specs: ["Microphone", "5 tunings", "On-device"],
  },
  {
    name: "Fretboard trainer",
    what: "It names a note and a string, you play it, and the same ear as the tuner says whether you did. Or it marks a spot on the neck and you name it. Streaks, and a list of the notes you keep missing.",
    href: "/playgrounds/fretboard/",
    tag: "Audio",
    category: "music",
    summary: "Calls a note, hears whether you played it, and remembers the ones you miss.",
    specs: ["Microphone", "Streaks", "On-device"],
  },
  {
    name: "See your voice",
    what: "A live spectrogram of whatever the microphone hears: time left to right, pitch up the side, brightness for loudness, the fundamental found and named as it goes. Hum, whistle, hiss.",
    href: "/playgrounds/spectrogram/",
    tag: "Audio",
    category: "music",
    summary: "A live spectrogram of whatever the microphone hears, with the pitch named as it goes.",
    specs: ["Microphone", "Real time", "On-device"],
  },
  {
    name: "Chord namer",
    what: "Tap the frets you are holding and it names the chord, with the other names it could go by. Type a chord and it lays out shapes up the neck, each one strummed on tap. No microphone needed.",
    href: "/playgrounds/chords/",
    tag: "Theory",
    category: "music",
    summary: "Names the chord under your fingers, or lays out every shape for a chord you type.",
    specs: ["No microphone", "Playable shapes"],
  },
  {
    name: "Metronome",
    what: "Clicks on the audio clock so they never drift, tap tempo, beats per bar and subdivisions, and a practice log that lives in your browser only.",
    href: "/playgrounds/metronome/",
    tag: "Audio",
    category: "music",
    summary: "A click that never drifts, with tap tempo, subdivisions and a private practice log.",
    specs: ["Audio clock", "Tap tempo", "Local log"],
  },
  {
    name: "Ear trainer",
    what: "It plays two notes or a chord on the synthesised strings and you name the interval or the chord. The score gathers per answer, so the ones that fool you surface.",
    href: "/playgrounds/ear/",
    tag: "Theory",
    category: "music",
    summary: "Hear an interval or a chord and name it; the ones that fool you come back.",
    specs: ["Intervals", "Chords", "Adaptive"],
  },
];

/**
 * The lab (`/lab/`: RFCs, studies and demos in progress) is admins-only until the
 * first concrete thing is published. This flag is one of the two places the flip
 * touches; the other is the `/lab/` route in `devops/envoy/envoy.yaml`, which is
 * the gate itself (a static site cannot keep anyone out). While `public` is false a
 * `gated` nav item renders only for a signed-in admin (`src/lib/me.ts`), the lab
 * pages ask not to be indexed, and the sitemap leaves them out.
 */
export const lab = { public: false, href: "/lab/" } as const;

type NavItem = {
  readonly href: string;
  readonly label: string;
  readonly key: string;
  /** Shown only to admins while `lab.public` is false. */
  readonly gated?: boolean;
};

/** The pages in the header and footer; `key` names the label in `common.<key>`. */
export const nav = [
  { href: "/", label: "Home", key: "home" },
  // Present, past, play: what is being built (with numbers), what shipped, what to try.
  { href: "/lab/", label: "Lab", key: "lab", gated: true },
  { href: "/open-source/", label: "Open source", key: "oss" },
  { href: "/playgrounds/", label: "Play", key: "play" },
  { href: "/about/", label: "About", key: "about" },
  { href: "/contact/", label: "Contact", key: "contact" },
] as const satisfies readonly NavItem[];

/** Whether a nav item is for this visitor: everything, unless it is gated and the lab is not public yet. */
export function navVisible(item: NavItem, admin: boolean): boolean {
  return !item.gated || lab.public || admin;
}

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
