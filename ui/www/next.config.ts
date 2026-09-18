import type { NextConfig } from "next";

// Static export: the built `out/` is copied into the www image and served by
// Caddy. Nothing here runs on a server, so the site needs no Node.js runtime
// and no per-environment configuration beyond NEXT_PUBLIC_SITE_URL, which is a
// build argument.
const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
