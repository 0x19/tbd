import type { MetadataRoute } from "next";

import { nav, playgrounds, url } from "@/data/site";

// A static export has no request to vary on: build it once, at build time.
export const dynamic = "force-static";

/** Generated at build time into `out/sitemap.xml`: every page the nav names,
 *  the legal pages, and every playground that is open. */
export default function sitemap(): MetadataRoute.Sitemap {
  const open = playgrounds.flatMap((p) => (p.href ? [p.href] : []));
  const paths = [...nav.map((i) => i.href), "/legal/", "/terms/", ...open];
  return paths.map((path) => ({
    url: `${url}${path}`,
    changeFrequency: "monthly",
    priority: path === "/" ? 1 : 0.5,
  }));
}
