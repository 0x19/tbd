import type { MetadataRoute } from "next";

import { indexable, url } from "@/data/site";

// A static export has no request to vary on: build it once, at build time.
export const dynamic = "force-static";

/** Generated at build time into `out/robots.txt`. */
export default function robots(): MetadataRoute.Robots {
  return {
    rules: indexable ? { userAgent: "*", allow: "/" } : { userAgent: "*", disallow: "/" },
    sitemap: `${url}/sitemap.xml`,
  };
}
