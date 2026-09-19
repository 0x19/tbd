import type { MetadataRoute } from "next";

import { nav, url } from "@/data/site";

// A static export has no request to vary on: build it once, at build time.
export const dynamic = "force-static";

/** Generated at build time into `out/sitemap.xml`; every page the nav names. */
export default function sitemap(): MetadataRoute.Sitemap {
  const paths = [...nav.map((i) => i.href), "/legal/", "/terms/"];
  return paths.map((path) => ({
    url: `${url}${path}`,
    changeFrequency: "monthly",
    priority: path === "/" ? 1 : 0.5,
  }));
}
