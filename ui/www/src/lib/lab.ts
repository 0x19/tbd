// The lab's data shapes: what `tool/lab-data.ts` renders from `docs/rfcs/`
// and `docs/studies/` into the gitignored `src/generated/lab/`. Hand-written
// and committed so the pages type-check before the generator has run.

export type LabKind = "rfc" | "study";

export type LabStatus = "open" | "decided" | "superseded" | "running" | "measured" | "published";

/** One entry of the index: everything but the prose. */
export type LabEntry = {
  kind: LabKind;
  /** "0001" */
  number: string;
  /** The file stem, e.g. "0001-the-llm-service"; the URL never changes. */
  slug: string;
  /** "/lab/rfc/0001-the-llm-service/" */
  href: string;
  title: string;
  status: LabStatus;
  /** "2026-09-23" */
  date: string;
  summary: string;
  /** RFC: the slug this one replaces. */
  supersedes?: string;
  /** RFC: the slug that replaced this one (derived). */
  supersededBy?: string;
  /** Study: the RFC slug it measures. */
  rfc?: string;
  /** Study: the one number, e.g. "72 tok/s". */
  headline?: string;
  /** Study: what that number is. */
  headlineNote?: string;
};

/** A rendered page: the entry plus its prose as HTML and its headings. */
export type LabDoc = LabEntry & {
  html: string;
  toc: { level: number; id: string; title: string }[];
};
