"use client";

// The facts of the site in the reader's language. `src/data/site.ts` is the
// source and is English; `src/data/site.hr.ts` carries the Croatian for the
// strings a person reads, keyed by what identifies each entry (a company, a
// project name, a playground path, a stage), so a new entry there without a
// Croatian twin simply reads in English until it gets one.
import * as en from "@/data/site";
import { hr } from "@/data/site.hr";

import { useLang } from "./index";

type Company = Omit<typeof en.company, "tagline" | "headline" | "summary" | "availability" | "now"> & {
  tagline: string;
  headline: string;
  summary: string;
  availability: string;
  now: string;
};

export type Site = {
  company: Company;
  experience: {
    company: string;
    role: string;
    when: string;
    where: string;
    href: string | null;
    body: string;
    tags: readonly string[];
    highlights: readonly string[];
  }[];
  earlier: { when: string; company: string; role: string }[];
  achievements: string[];
  about: string[];
  chapters: { when: string; title: string; body: string; where: string }[];
  projects: { name: string; year: string; what: string; language: string; href: string }[];
  playgrounds: {
    name: string;
    what: string;
    href: string | null;
    tag: string;
    category: "systems" | "music";
    summary: string;
    specs: string[];
  }[];
  principles: { title: string; body: string }[];
  pipeline: { stage: string; name: string; rows: { k: string; v: string }[] }[];
  rails: { label: string; value: string }[];
  languages: string[];
  education: string;
};

function english(): Site {
  return {
    company: en.company,
    experience: en.experience.map((e) => ({ ...e })),
    earlier: en.earlier.map((e) => ({ ...e })),
    achievements: [...en.achievements],
    about: [...en.about],
    chapters: en.chapters.map((c) => ({ ...c })),
    projects: en.projects.map((p) => ({ ...p })),
    playgrounds: en.playgrounds.map((p) => ({ ...p, specs: [...p.specs] })),
    principles: en.principles.map((p) => ({ ...p })),
    pipeline: en.pipeline.map((s) => ({ ...s, rows: s.rows.map((r) => ({ ...r })) })),
    rails: en.rails.map((r) => ({ ...r })),
    languages: [...en.languages],
    education: en.cv.education,
  };
}

function croatian(): Site {
  const s = english();
  s.company = { ...s.company, ...hr.company };
  s.experience = s.experience.map((e) => ({ ...e, ...(hr.experience[e.company] ?? {}) }));
  s.earlier = s.earlier.map((e) => ({ ...e, ...(hr.earlier[e.company] ?? {}) }));
  s.achievements = hr.achievements.length === s.achievements.length ? [...hr.achievements] : s.achievements;
  s.about = hr.about.length === s.about.length ? [...hr.about] : s.about;
  s.chapters = s.chapters.map((c, i) => ({ ...c, ...(hr.chapters[i] ?? {}) }));
  s.projects = s.projects.map((p) => ({ ...p, ...(hr.projects[p.name] ?? {}) }));
  s.playgrounds = s.playgrounds.map((p) => ({ ...p, ...(p.href ? (hr.playgrounds[p.href] ?? {}) : {}) }));
  s.principles = s.principles.map((p, i) => ({ ...p, ...(hr.principles[i] ?? {}) }));
  s.pipeline = s.pipeline.map((st) => {
    const o = hr.pipeline[st.stage];
    if (!o) return st;
    return {
      ...st,
      stage: o.stage ?? st.stage,
      name: o.name ?? st.name,
      rows: st.rows.map((r, i) => ({ ...r, ...(o.rows?.[i] ?? {}) })),
    };
  });
  s.rails = s.rails.map((r, i) => ({ ...r, ...(hr.rails[i] ?? {}) }));
  s.languages = hr.languages.length === s.languages.length ? [...hr.languages] : s.languages;
  s.education = hr.education || s.education;
  return s;
}

/** The site's facts in the current language. */
export function useSite(): Site {
  const { lang } = useLang();
  return lang === "hr" ? croatian() : english();
}
