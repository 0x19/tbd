// Writes cv/cv.json from src/data/site.ts, so the PDF (cv/cv.typ) and the /cv/
// page draw on the same facts. Run through `mise run www:cv`; Node strips the
// types itself, no bundler needed.
import { mkdirSync, writeFileSync } from "node:fs";
import { dirname, join } from "node:path";
import { fileURLToPath } from "node:url";

import {
  about,
  achievements,
  company,
  cv,
  earlier,
  experience,
  languages,
  projects,
  url,
} from "../src/data/site.ts";

const here = dirname(fileURLToPath(import.meta.url));
const out = join(here, "..", "cv", "cv.json");
mkdirSync(dirname(out), { recursive: true });
writeFileSync(
  out,
  JSON.stringify(
    {
      person: company.person,
      title: company.title,
      city: company.city,
      email: company.email,
      site: url.replace(/^https?:\/\//, ""),
      github: company.github.replace(/^https?:\/\//, ""),
      linkedin: company.linkedin.replace(/^https?:\/\//, ""),
      availability: company.availability,
      summary: company.summary,
      about,
      achievements,
      // The about page's compressed "Earlier" entry is expanded by `earlier` below.
      experience: experience
        .filter((e) => e.company !== "Earlier")
        .map((e) => ({
          company: e.company,
          role: e.role,
          when: e.when.replace(/ — /g, "–"),
          where: e.where,
          body: e.body,
          highlights: e.highlights,
          tags: e.tags,
        })),
      earlier: earlier.map((e) => ({ ...e, when: e.when.replace(/ — /g, "–") })),
      projects: projects.map((p) => ({
        name: p.name,
        year: p.year,
        what: p.what,
        href: p.href.replace(/^https?:\/\//, ""),
      })),
      languages,
      education: cv.education,
    },
    null,
    2,
  ) + "\n",
);
console.log(`wrote ${out}`);
