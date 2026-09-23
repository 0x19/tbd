import type { NextConfig } from "next";

// Static export: the built `out/` is served at the root of the cv host by
// Caddy (devops/docker/Dockerfile.cv-ui); Envoy's `cv.*` virtual host puts
// the browser sign-in in front of it and routes /v1/ to the protocol on the
// same origin. Everything is a client component; no Node.js runtime anywhere.
const nextConfig: NextConfig = {
  output: "export",
  trailingSlash: true,
  images: { unoptimized: true },
};

export default nextConfig;
