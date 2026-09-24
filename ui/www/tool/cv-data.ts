// Writes crates/cv/assets/cv.json from src/data/site.ts, so the public PDF
// (crates/cv/assets/cv.typ, set by `mise run www:cv`), the /cv/ page and the cv
// service's full render all draw on the same facts. The JSON is committed and
// CI checks it is current. Node strips the types itself, no bundler needed.
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
const out = join(here, "..", "..", "..", "crates", "cv", "assets", "cv.json");
mkdirSync(dirname(out), { recursive: true });
writeFileSync(
  out,
  JSON.stringify(
    {
      person: company.person,
      title: company.title,
      focus: company.focus,
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
