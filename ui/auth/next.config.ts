import type { NextConfig } from "next";

// Runs as a small Node server (consent and logout talk to Hydra's admin API),
// packaged standalone into a distroless image (devops/docker/Dockerfile.auth-ui).
const nextConfig: NextConfig = {
  output: "standalone",
  poweredByHeader: false,
};

export default nextConfig;
