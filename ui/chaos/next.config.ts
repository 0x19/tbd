import type { NextConfig } from "next";

// Static export: the built `out/` is served at the root of its host by
// `chaos serve` (http://localhost:7700/) and, in a cluster, by Envoy's
// `chaos.*` / `chaosadmin.*` virtual host (http://chaos.localhost:18080/).
// Everything is a client component talking to the API at /api/chaos/v1 on the
// same origin, so no Node.js runtime is needed anywhere.
const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
