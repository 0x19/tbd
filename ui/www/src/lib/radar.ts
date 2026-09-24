// The Radar's data shapes, as the radar service sends them (the protocol
// gateway keeps the proto's field names), shared by the page and by
// `tool/radar-data.ts`, which renders the published issues into the gitignored
// `src/generated/radar/` at build time so the HTML carries them.

export type ImpactKey = "IMPACT_BREAKING" | "IMPACT_WORTH_KNOWING" | "IMPACT_NICE_TO_KNOW";

export type RadarChange = {
  title: string;
  impact: ImpactKey | "IMPACT_UNSPECIFIED";
  area: string;
  url: string;
  what: string;
  production_impact: string;
  try_it: string;
};

export type RadarDigest = {
  id: string;
  week: string;
  language: "go" | "rust";
  lang: "en" | "hr";
  summary?: string;
  changes?: RadarChange[];
  changed: string;
  why: string;
  drill: string;
  script: string;
  item_count: number;
  model: string;
  stub: boolean;
  status?: string;
  published_at?: string;
  created_at?: string;
  /** Always true: a digest is written by a language model. */
  ai_written?: boolean;
  /** "live" (the weekly run, reviewed) or "archive" (the backfill, not individually reviewed). */
  origin?: string;
};

/** One published week: its number in the series, its slug and its digests. */
export type RadarIssue = {
  /** 1 for the first reviewed live week; null for an archive week. */
  number: number | null;
  /** "2026-W39" */
  week: string;
  /** "2026-w39", the URL segment. */
  slug: string;
  digests: RadarDigest[];
};

/** Whether a digest came from the archive backfill. */
export const isArchive = (d: { origin?: string }) => d.origin === "archive";

/** "#001" */
export const issueLabel = (n: number) => `#${String(n).padStart(3, "0")}`;
