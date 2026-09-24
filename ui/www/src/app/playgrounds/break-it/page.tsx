import type { Metadata } from "next";

import { BreakItPausedContent } from "@/components/pages/break-it-paused";

export const metadata: Metadata = {
  title: "Break it",
  description: "Paused while it is rebuilt.",
  alternates: { canonical: "/playgrounds/break-it/" },
  robots: { index: false, follow: true },
};

/**
 * Break it is paused: listed under Play as paused, connecting to nothing. Its
 * code is kept whole in `src/components/playgrounds/break-it/` for the next
 * version; the service behind it keeps running.
 */
export default function BreakItPage() {
  return <BreakItPausedContent />;
}
