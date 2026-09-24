// The labs: each one a subject being built in the open, with its own page, its
// RFCs and studies (the `lab:` key in their front matter names it), and, when
// it runs, a live view. Plain data with no imports, because `tool/lab-data.ts`
// reads it too, to refuse a document that names no lab or one that does not
// exist. Both languages sit side by side here rather than in `site.hr.ts`: the
// generator has no language, and the pages pick with `useLang()`.

type Text = { readonly en: string; readonly hr: string };

export type Lab = {
  /** The `lab:` value in a document's front matter, and the URL segment. */
  readonly id: string;
  readonly href: string;
  readonly name: Text;
  /** One plain paragraph: what it is, not why. */
  readonly what: Text;
  /** Where it can be tried, when there is somewhere. */
  readonly workbench?: string;
  /** Whether the page shows the platform's live feed (the arena). */
  readonly live: boolean;
  /** The ways in, as they are named on the page. */
  readonly surfaces: readonly string[];
};

export const labs: readonly Lab[] = [
  {
    id: "llm",
    href: "/lab/llm/",
    name: {
      en: "A model platform on one workstation",
      hr: "Platforma za modele na jednoj radnoj stanici",
    },
    what: {
      en: "Two open-weight models served from one machine: a fast one on the graphics card and a large one from memory, behind one service that owns the contract, the admission queue, each caller's budget and the record of every generation. Reached through the same gateway as the rest of the platform, over REST, server-sent events, a WebSocket and MCP, and measured under load and fault by the chaos tool.",
      hr: "Dva modela otvorenih težina posluživana s jednog računala: brzi na grafičkoj kartici i veliki iz memorije, iza jednog servisa koji drži ugovor, red za prijem, proračun svakog pozivatelja i zapis svake generacije. Dostupni kroz isti gateway kao ostatak platforme, preko REST-a, server-sent eventa, WebSocketa i MCP-a, i mjereni pod opterećenjem i kvarovima alatom za kaos.",
    },
    workbench: "/lab/llm/workbench/",
    live: true,
    surfaces: ["REST", "SSE", "WebSocket", "MCP", "gRPC"],
  },
];

export const labIds: readonly string[] = labs.map((l) => l.id);
