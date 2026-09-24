import type { Metadata } from "next";

import { RadarContent } from "@/components/pages/radar";

export const metadata: Metadata = {
  title: "Radar",
  description: "What changed in Go and Rust this week, from the official sources, with a ten-minute drill.",
  alternates: { canonical: "/radar/" },
  // Until the first issue has been read and published (RFC 0007).
  robots: { index: false, follow: false },
};

export default function RadarPage() {
  return <RadarContent />;
}
