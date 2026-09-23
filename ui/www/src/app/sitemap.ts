import type { MetadataRoute } from "next";

import { lab, nav, navVisible, playgrounds, url } from "@/data/site";
import { rfcs, studies } from "@/generated/lab/index";

// A static export has no request to vary on: build it once, at build time.
export const dynamic = "force-static";

/** Generated at build time into `out/sitemap.xml`: every page the nav names
 *  (the lab only once it is public), the legal pages, every playground that is
 *  open, and every public RFC and study. Redirect pages are never listed. */
export default function sitemap(): MetadataRoute.Sitemap {
  const open = playgrounds.flatMap((p) =>
    p.href && (lab.public || !p.href.startsWith("/lab/")) ? [p.href] : [],
  );
  const labPages = lab.public
    ? ["/lab/demo/", ...rfcs.map((r) => r.href), ...studies.map((s) => s.href)]
    : [];
  const paths = [
    ...nav.filter((i) => navVisible(i, false)).map((i) => i.href),
    "/legal/",
    "/terms/",
    ...open,
    ...labPages,
  ];
  return paths.map((path) => ({
    url: `${url}${path}`,
    changeFrequency: "monthly",
    priority: path === "/" ? 1 : 0.5,
  }));
}
