import type { Metadata } from "next";

import { LabHomeContent } from "@/components/pages/lab-home";
import { lab } from "@/data/site";

export const metadata: Metadata = {
  title: "A model platform on one workstation",
  description:
    "Two open-weight models served from one machine behind one service, reached over REST, server-sent events, a WebSocket and MCP, and measured under load.",
  alternates: { canonical: "/lab/llm/" },
  ...(lab.public ? {} : { robots: { index: false, follow: false } }),
};

/** The model lab's page. A lab is a folder here, one per id in `src/data/labs.ts`. */
export default function LlmLabPage() {
  return <LabHomeContent id="llm" />;
}
