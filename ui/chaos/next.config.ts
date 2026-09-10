import type { NextConfig } from "next";

// Static export: the built `out/` is served by `chaos serve` at /chaos and by
// Envoy in the cluster. Everything is a client component talking to the API,
// so no Node.js runtime is needed anywhere.
const nextConfig: NextConfig = {
  output: "export",
  basePath: "/chaos",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
